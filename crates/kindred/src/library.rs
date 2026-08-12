//! Local JSON book library (per-user index + offline view).
//!
//! The library is a device-independent index of books keyed by ASIN (with a
//! `dict:<title>` key for non-ASIN items such as dictionaries). The attached
//! device remains the source of truth for what is physically on it; the
//! library is reconciled against a fresh inventory on connect.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::collections::LocalCollection;
use crate::inventory::{Book, BookFormat, KindleInventory};

/// Whether a book lives on the attached device, in the local library, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookStatus {
    /// Present on the device; no local copy in the library.
    OnDevice,
    /// In the local library only (offline copy, not currently on the device).
    LocalOnly,
    /// Present in both.
    Both,
}

/// One book record in the local library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryRecord {
    /// Stable record key: the ASIN, or `dict:<title>` for non-ASIN items.
    pub key: String,
    pub title: String,
    pub format: BookFormat,
    pub size_bytes: u64,
    /// Local path of the book file when a local copy exists.
    pub local_path: Option<String>,
    /// Last known presence on an attached device.
    pub on_device: bool,
    /// Serial of the device last seen holding this book.
    pub last_seen_device: Option<String>,
}

/// The JSON-backed local library.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalLibrary {
    pub version: u32,
    pub records: Vec<LibraryRecord>,
    /// User-authored local collections (see `collections` module).
    /// `default` keeps older library files (without the field) loadable.
    #[serde(default)]
    pub collections: Vec<LocalCollection>,
}

impl LocalLibrary {
    pub const CURRENT_VERSION: u32 = 1;

    /// Load the library from `path`, or an empty library when absent.
    pub fn load(path: &Path) -> Result<LocalLibrary, LibraryError> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LocalLibrary {
                version: Self::CURRENT_VERSION,
                records: Vec::new(),
                collections: Vec::new(),
            }),
            Err(error) => Err(LibraryError::Io(error)),
        }
    }

    /// Persist the library to `path`, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), LibraryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn lookup(&self, key: &str) -> Option<&LibraryRecord> {
        self.records.iter().find(|record| record.key == key)
    }

    /// Insert or replace the record for a key.
    pub fn upsert(&mut self, record: LibraryRecord) {
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|existing| existing.key == record.key)
        {
            *existing = record;
        } else {
            self.records.push(record);
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.records.retain(|record| record.key != key);
    }

    /// Derive the status of a record from its presence and local copy.
    pub fn status(&self, key: &str) -> Option<BookStatus> {
        self.lookup(key).map(record_status)
    }

    /// Reconcile records against a fresh device inventory.
    ///
    /// Device books are added as records (or refreshed), records absent from
    /// the device are marked off-device, and `device_serial` is recorded on
    /// books present.
    pub fn reconcile(&mut self, inventory: &KindleInventory, device_serial: Option<&str>) {
        let present: Vec<String> = inventory.books.iter().filter_map(book_key).collect();

        for book in &inventory.books {
            let Some(key) = book_key(book) else {
                continue;
            };
            if self.lookup(&key).is_none() {
                self.upsert(record_from_book(book, key, device_serial));
            }
        }

        for record in &mut self.records {
            let on_device = present.iter().any(|key| key == &record.key);
            record.on_device = on_device;
            if on_device {
                record.last_seen_device = device_serial.map(str::to_owned);
            }
        }
    }
}

fn record_status(record: &LibraryRecord) -> BookStatus {
    match (record.on_device, record.local_path.is_some()) {
        (true, true) => BookStatus::Both,
        (true, false) => BookStatus::OnDevice,
        (false, _) => BookStatus::LocalOnly,
    }
}

/// The stable library key for a book (ASIN, or `dict:<title>`).
fn book_key(book: &Book) -> Option<String> {
    match &book.asin {
        Some(asin) => Some(asin.clone()),
        None => Some(format!("dict:{}", book.title)),
    }
}

fn record_from_book(book: &Book, key: String, device_serial: Option<&str>) -> LibraryRecord {
    LibraryRecord {
        key,
        title: book.title.clone(),
        format: book.format,
        size_bytes: book.size_bytes,
        local_path: None,
        on_device: true,
        last_seen_device: device_serial.map(str::to_owned),
    }
}

#[derive(Debug)]
pub enum LibraryError {
    Io(std::io::Error),
    Json(serde_json::Error),
    EmptyCollectionName,
    CollectionExists(String),
    CollectionNotFound(String),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Io(error) => write!(f, "local library I/O error: {error}"),
            LibraryError::Json(error) => write!(f, "local library JSON error: {error}"),
            LibraryError::EmptyCollectionName => write!(f, "collection name must not be empty"),
            LibraryError::CollectionExists(name) => {
                write!(f, "collection '{name}' already exists")
            }
            LibraryError::CollectionNotFound(name) => {
                write!(f, "collection '{name}' not found")
            }
        }
    }
}

impl std::error::Error for LibraryError {}

impl From<std::io::Error> for LibraryError {
    fn from(error: std::io::Error) -> Self {
        LibraryError::Io(error)
    }
}

impl From<serde_json::Error> for LibraryError {
    fn from(error: serde_json::Error) -> Self {
        LibraryError::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(key: &str, on_device: bool, local: bool) -> LibraryRecord {
        LibraryRecord {
            key: key.to_owned(),
            title: key.to_owned(),
            format: BookFormat::Kfx,
            size_bytes: 100,
            local_path: local.then(|| format!("/books/{key}.kfx")),
            on_device,
            last_seen_device: None,
        }
    }

    #[test]
    fn load_missing_file_returns_empty_library() {
        let path = std::env::temp_dir().join("kindling_test_missing_library.json");
        let _ = std::fs::remove_file(&path);
        let library = LocalLibrary::load(&path).unwrap();
        assert_eq!(library.version, LocalLibrary::CURRENT_VERSION);
        assert!(library.records.is_empty());
    }

    #[test]
    fn save_and_load_round_trips() {
        let path = std::env::temp_dir().join("kindling_test_library.json");
        let _ = std::fs::remove_file(&path);

        let mut library = LocalLibrary::default();
        library.upsert(record("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2", true, true));
        library.save(&path).unwrap();

        let loaded = LocalLibrary::load(&path).unwrap();
        assert_eq!(loaded, library);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn status_reflects_presence_and_local_copy() {
        let mut library = LocalLibrary::default();
        library.upsert(record("A", true, true));
        library.upsert(record("B", true, false));
        library.upsert(record("C", false, true));

        assert_eq!(library.status("A"), Some(BookStatus::Both));
        assert_eq!(library.status("B"), Some(BookStatus::OnDevice));
        assert_eq!(library.status("C"), Some(BookStatus::LocalOnly));
        assert_eq!(library.status("missing"), None);
    }

    #[test]
    fn upsert_and_remove() {
        let mut library = LocalLibrary::default();
        library.upsert(record("A", false, false));
        library.upsert(record("A", true, false));
        assert_eq!(library.records.len(), 1);
        assert!(library.lookup("A").is_some());

        library.remove("A");
        assert!(library.lookup("A").is_none());
    }

    #[test]
    fn create_collection_rejects_blank_and_duplicates() {
        let mut library = LocalLibrary::default();
        assert!(matches!(
            library.create_collection("   "),
            Err(LibraryError::EmptyCollectionName)
        ));
        library.create_collection("Favourites").unwrap();
        assert!(matches!(
            library.create_collection("Favourites"),
            Err(LibraryError::CollectionExists(_))
        ));
        assert_eq!(library.collections.len(), 1);
    }

    #[test]
    fn collection_add_and_remove_book_dedupe() {
        let mut library = LocalLibrary::default();
        library.create_collection("Favourites").unwrap();

        library
            .add_book_to_collection("Favourites", "JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
            .unwrap();
        library
            .add_book_to_collection("Favourites", "JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
            .unwrap();

        assert_eq!(library.collection("Favourites").unwrap().book_keys.len(), 1);

        library
            .remove_book_from_collection("Favourites", "JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
            .unwrap();
        assert!(
            library
                .collection("Favourites")
                .unwrap()
                .book_keys
                .is_empty()
        );
    }

    #[test]
    fn collection_ops_require_existing_collection() {
        let mut library = LocalLibrary::default();
        assert!(matches!(
            library.add_book_to_collection("Missing", "KEY"),
            Err(LibraryError::CollectionNotFound(_))
        ));
        assert!(matches!(
            library.remove_book_from_collection("Missing", "KEY"),
            Err(LibraryError::CollectionNotFound(_))
        ));
    }

    #[test]
    fn rename_and_delete_collection() {
        let mut library = LocalLibrary::default();
        library.create_collection("Old Name").unwrap();
        library.rename_collection("Old Name", "New Name").unwrap();
        assert!(library.collection("New Name").is_some());
        assert!(library.collection("Old Name").is_none());

        library.delete_collection("New Name");
        assert!(library.collections.is_empty());
    }

    #[test]
    fn old_library_json_without_collections_still_loads() {
        // A pre-collections library file must deserialize with an empty list.
        let json = r#"{"version":1,"records":[{"key":"A","title":"A","format":"KFX","size_bytes":1,"local_path":null,"on_device":false,"last_seen_device":null}]}"#;
        let library: LocalLibrary = serde_json::from_str(json).unwrap();
        assert!(library.collections.is_empty());
        assert_eq!(library.records.len(), 1);
    }
}
