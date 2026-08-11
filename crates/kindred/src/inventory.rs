//! Kindle-aware inventory built above the raw MTP object layer.
//!
//! This module turns raw MTP object listings into user-facing `Book`
//! values. Identity comes from the `Title_ASIN` filename convention
//! observed on the physical Paperwhite (see PLAN.md and the project
//! instructions §9).

use crate::error::KindredError;
use crate::mtp::MtpObjectSummary;

use mtp_rs::mtp::{MtpDevice, Storage};
use serde::{Deserialize, Serialize};

/// Book content formats recognised from device evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookFormat {
    #[serde(rename = "KFX")]
    Kfx,
    #[serde(rename = "AZW")]
    Azw,
}

impl BookFormat {
    /// Map a filename extension to a recognised format.
    pub fn from_extension(ext: &str) -> Option<BookFormat> {
        match ext.to_ascii_lowercase().as_str() {
            "kfx" => Some(BookFormat::Kfx),
            "azw" => Some(BookFormat::Azw),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            BookFormat::Kfx => "KFX",
            BookFormat::Azw => "AZW",
        }
    }
}

/// A single book on the device: user-facing identity plus the storage
/// objects that make it up.
#[derive(Debug, Clone)]
pub struct Book {
    pub title: String,
    /// Stable per-book key from the `Title_ASIN` filename suffix. Absent for
    /// items without an identifier (e.g. dictionaries).
    pub asin: Option<String>,
    pub format: BookFormat,
    pub size_bytes: u64,
    /// MTP handle of the content file.
    pub content_handle: u64,
    /// MTP handle of the `.sdr` sidecar folder, when one exists.
    pub sidecar_handle: Option<u64>,
    /// MTP handles of associated metadata objects (`.mf`/`.yjf`).
    pub metadata_handles: Vec<u64>,
}

/// The full inventory of one attached Kindle.
#[derive(Debug, Clone)]
pub struct KindleInventory {
    pub storage_description: String,
    pub books: Vec<Book>,
}

/// Build the device inventory from the physical Kindle.
///
/// Books are read from `documents/` (root sideloads), `documents/Downloads/Items01/`
/// and `documents/dictionaries/`. Read-only; never modifies the device.
///
/// One MTP session is opened for the whole walk. For every book the contents
/// of its `.sdr` sidecar folder are inspected so the real metadata handles
/// (`.mf`/`.yjf`/`.meta`) are associated with the book.
pub async fn inventory_device() -> Result<Option<KindleInventory>, KindredError> {
    let device = MtpDevice::open_first().await?;
    let mut storages = device.storages().await?;
    let Some(storage) = storages.pop() else {
        return Ok(None);
    };

    let root = to_summaries(storage.list_objects(None).await?);
    let Some(documents) = find_folder(&root, "documents") else {
        return Ok(None);
    };
    let documents_children = to_summaries(
        storage
            .list_objects(Some(mtp_rs::mtp::ObjectHandle(documents.handle)))
            .await?,
    );

    let mut books = Vec::new();

    // Books sideloaded directly in the documents/ root.
    let root_books = books_from_folder(&documents_children);
    books.extend(enrich_sidecar_metadata(&storage, root_books).await);

    // Book content area: documents/Downloads/Items01/
    if let Some(downloads) = find_folder(&documents_children, "Downloads")
        && let items = list_children(&storage, downloads.handle).await?
        && let Some(items01) = find_folder(&items, "Items01")
    {
        let content = list_children(&storage, items01.handle).await?;
        let content_books = books_from_folder(&content);
        books.extend(enrich_sidecar_metadata(&storage, content_books).await);
    }

    // Dictionaries: documents/dictionaries/ (`.azw` files)
    if let Some(dictionaries) = find_folder(&documents_children, "dictionaries") {
        let content = list_children(&storage, dictionaries.handle).await?;
        for object in content.iter().filter(|object| !object.is_folder) {
            let Some(format) = extension(&object.filename).and_then(BookFormat::from_extension)
            else {
                continue;
            };

            books.push(Book {
                title: strip_extension(&object.filename).to_owned(),
                asin: None,
                format,
                size_bytes: object.size_bytes,
                content_handle: object.handle,
                sidecar_handle: None,
                metadata_handles: Vec::new(),
            });
        }
    }

    Ok(Some(KindleInventory {
        storage_description: storage.info().description.clone(),
        books,
    }))
}

/// List a folder's children as Kindred-owned summaries.
async fn list_children(
    storage: &Storage,
    parent: u64,
) -> Result<Vec<MtpObjectSummary>, KindredError> {
    storage
        .list_objects(Some(mtp_rs::mtp::ObjectHandle(parent)))
        .await
        .map(to_summaries)
        .map_err(KindredError::from)
}

/// Convert raw MTP object info into Kindred-owned summaries.
fn to_summaries(objects: Vec<mtp_rs::mtp::ObjectInfo>) -> Vec<MtpObjectSummary> {
    objects.into_iter().map(MtpObjectSummary::from).collect()
}

/// Fill each book's `metadata_handles` from its `.sdr` sidecar folder.
///
/// The `.mf`/`.yjf`/`.meta` files observed on the physical device live inside
/// the sidecar folder rather than beside the content file, so the handles are
/// collected by descending into each sidecar once.
async fn enrich_sidecar_metadata(storage: &Storage, mut books: Vec<Book>) -> Vec<Book> {
    for book in &mut books {
        let Some(sidecar_handle) = book.sidecar_handle else {
            continue;
        };

        let Ok(children) = storage
            .list_objects(Some(mtp_rs::mtp::ObjectHandle(sidecar_handle)))
            .await
        else {
            continue;
        };

        book.metadata_handles = children
            .iter()
            .filter(|object| !object.is_folder() && is_metadata_filename(&object.filename))
            .map(|object| object.handle.0)
            .collect();
    }
    books
}

/// True for the per-book metadata extensions observed on the device.
fn is_metadata_filename(filename: &str) -> bool {
    extension(filename)
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "mf" | "yjf" | "meta"))
        .unwrap_or(false)
}

fn find_folder<'a>(objects: &'a [MtpObjectSummary], name: &str) -> Option<&'a MtpObjectSummary> {
    objects
        .iter()
        .find(|object| object.is_folder && object.filename == name)
}

/// Group the objects of a book-content folder into `Book` values.
///
/// On the observed device each book is a `Title_ASIN.kfx` content file paired
/// with a `Title_ASIN.sdr` sidecar folder and `<id>.mf` / `Title_ASIN….yjf`
/// metadata objects. Folder-level `.meta` files are not per-book.
fn books_from_folder(objects: &[MtpObjectSummary]) -> Vec<Book> {
    let mut content: Vec<&MtpObjectSummary> = Vec::new();
    let mut sidecars: Vec<(&str, u64)> = Vec::new();
    let mut metadata: Vec<(&str, u64)> = Vec::new();

    for object in objects {
        if object.is_folder {
            if let Some(stem) = object.filename.strip_suffix(".sdr") {
                sidecars.push((stem, object.handle));
            }
        } else if let Some(ext) = extension(&object.filename) {
            match ext.to_ascii_lowercase().as_str() {
                "kfx" | "azw" => content.push(object),
                "mf" | "yjf" | "meta" => {
                    metadata.push((strip_extension(&object.filename), object.handle));
                }
                _ => {}
            }
        }
    }

    let mut books = Vec::new();

    for object in content {
        let stem = strip_extension(&object.filename);
        let (title, asin) = split_identity(stem);

        let sidecar_handle = sidecars
            .iter()
            .find(|(sidecar_stem, _)| *sidecar_stem == stem)
            .map(|(_, handle)| *handle);

        let mut metadata_handles = Vec::new();
        for (meta_stem, handle) in &metadata {
            if meta_stem.starts_with(stem) || asin.as_deref() == Some(*meta_stem) {
                metadata_handles.push(*handle);
            }
        }

        let Some(format) = extension(&object.filename).and_then(BookFormat::from_extension) else {
            continue;
        };

        books.push(Book {
            title,
            asin,
            format,
            size_bytes: object.size_bytes,
            content_handle: object.handle,
            sidecar_handle,
            metadata_handles,
        });
    }

    books
}

/// Split a `Title_ASIN` stem into (title, identifier).
///
/// The identifier is the trailing underscore-separated token when it looks
/// like a device id (10-char ASIN or 32-char hex id). Items without one
/// (e.g. dictionaries) keep the whole stem as the title.
fn split_identity(stem: &str) -> (String, Option<String>) {
    match stem.rsplit_once('_') {
        Some((title, id)) if is_identifier(id) => (title.to_owned(), Some(id.to_owned())),
        _ => (stem.to_owned(), None),
    }
}

/// Parse a `Title_ASIN.ext` filename into (title, asin, format).
///
/// Returns `None` when the extension is not a recognised book format.
pub fn parse_book_filename(filename: &str) -> Option<(String, Option<String>, BookFormat)> {
    let format = BookFormat::from_extension(extension(filename)?)?;
    let (title, asin) = split_identity(strip_extension(filename));
    Some((title, asin, format))
}

/// True when `token` looks like a device identifier: uppercase letters and
/// digits, at least 10 characters long.
fn is_identifier(token: &str) -> bool {
    token.len() >= 10
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn extension(filename: &str) -> Option<&str> {
    filename.rsplit_once('.').map(|(_, ext)| ext)
}

fn strip_extension(filename: &str) -> &str {
    filename
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(name: &str, is_folder: bool) -> MtpObjectSummary {
        MtpObjectSummary {
            filename: name.to_owned(),
            is_folder,
            handle: 1,
            size_bytes: if is_folder { 0 } else { 1_234_567 },
        }
    }

    #[test]
    fn split_identity_extracts_32_char_identifier() {
        let (title, asin) = split_identity("The Way of Kings_JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2");
        assert_eq!(title, "The Way of Kings");
        assert_eq!(asin.as_deref(), Some("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2"));
    }

    #[test]
    fn split_identity_extracts_classic_asin() {
        let (title, asin) = split_identity("Nice Girls Don't (BDSM Erotica)_B0048EL62A");
        assert_eq!(title, "Nice Girls Don't (BDSM Erotica)");
        assert_eq!(asin.as_deref(), Some("B0048EL62A"));
    }

    #[test]
    fn split_identity_keeps_underscored_title_without_identifier() {
        let (title, asin) = split_identity("Oxford_Dictionary_of_English");
        assert_eq!(title, "Oxford_Dictionary_of_English");
        assert_eq!(asin, None);
    }

    #[test]
    fn format_from_extension_is_case_insensitive() {
        assert_eq!(BookFormat::from_extension("kfx"), Some(BookFormat::Kfx));
        assert_eq!(BookFormat::from_extension("AZW"), Some(BookFormat::Azw));
        assert_eq!(BookFormat::from_extension("mobi"), None);
    }

    #[test]
    fn groups_content_sidecar_and_metadata_into_one_book() {
        let objects = [
            object(
                "The Way of Kings_JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.kfx",
                false,
            ),
            object(
                "The Way of Kings_JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.sdr",
                true,
            ),
            object("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.mf", false),
            object(
                "The Way of Kings_JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2b82730da213c86376c92859560e37ca7.yjf",
                false,
            ),
            object("AssetDownloadMetadata.meta", false),
        ];

        let books = books_from_folder(&objects);

        assert_eq!(books.len(), 1);
        let book = &books[0];
        assert_eq!(book.title, "The Way of Kings");
        assert_eq!(
            book.asin.as_deref(),
            Some("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
        );
        assert_eq!(book.format, BookFormat::Kfx);
        assert_eq!(book.size_bytes, 1_234_567);
        assert!(book.sidecar_handle.is_some());
        assert_eq!(book.metadata_handles.len(), 2);
    }

    #[test]
    fn folder_level_meta_is_not_attached_to_a_book() {
        let objects = [
            object(
                "Homemade_Pizza_Dough_Kindle_7in.pdf_GOGASE4ZUWFC6PFG2OLAVTAMCEFHMKFY.kfx",
                false,
            ),
            object("AssetDownloadMetadata.meta", false),
        ];

        let books = books_from_folder(&objects);

        assert_eq!(books.len(), 1);
        assert!(books[0].metadata_handles.is_empty());
    }

    #[test]
    fn documents_root_sideload_is_found_alongside_folders() {
        let objects = [
            object(
                "Moresome 3_ Key West_QLHYMITS2CQGJKXWY4E4VZAKG2H3XP2Q.kfx",
                false,
            ),
            object(".cache", true),
            object("My Clippings.sdr", true),
            object("dictionaries", true),
            object("Downloads", true),
        ];

        let books = books_from_folder(&objects);

        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "Moresome 3_ Key West");
        assert!(books[0].sidecar_handle.is_none());
    }
}
