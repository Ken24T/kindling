//! Merge the device inventory and local library into UI entries.

use std::collections::HashSet;

use kindred::{Book, BookStatus, KindleInventory, LocalLibrary};

use crate::model::BookEntry;

/// The stable library key for a book (ASIN, or `dict:<title>`).
pub fn book_key(book: &Book) -> String {
    book.asin
        .clone()
        .unwrap_or_else(|| format!("dict:{}", book.title))
}

/// Merge device books and local records into one UI catalogue.
pub fn build_catalogue(
    inventory: Option<&KindleInventory>,
    library: &LocalLibrary,
) -> Vec<BookEntry> {
    let mut entries: Vec<BookEntry> = Vec::new();
    let mut matched: HashSet<String> = HashSet::new();

    if let Some(inventory) = inventory {
        device_entries(inventory, library, &mut matched, &mut entries);
    }
    local_only_entries(library, &matched, &mut entries);

    entries
}

/// One entry per device book; `Both` when a local copy exists.
fn device_entries(
    inventory: &KindleInventory,
    library: &LocalLibrary,
    matched: &mut HashSet<String>,
    entries: &mut Vec<BookEntry>,
) {
    for book in &inventory.books {
        let key = book_key(book);
        let local = library.lookup(&key);
        if local.is_some() {
            matched.insert(key);
        }
        let has_local = local
            .and_then(|record| record.local_path.as_deref())
            .is_some();
        entries.push(BookEntry {
            title: book.title.clone(),
            format: book.format,
            size_bytes: book.size_bytes,
            status: if has_local {
                BookStatus::Both
            } else {
                BookStatus::OnDevice
            },
            device: Some(book.clone()),
            local: local.cloned(),
        });
    }
}

/// Local records that hold a file and are not on the device.
fn local_only_entries(
    library: &LocalLibrary,
    matched: &HashSet<String>,
    entries: &mut Vec<BookEntry>,
) {
    for record in library
        .records
        .iter()
        .filter(|record| record.local_path.is_some())
    {
        if matched.contains(&record.key) {
            continue;
        }
        entries.push(BookEntry {
            title: record.title.clone(),
            format: record.format,
            size_bytes: record.size_bytes,
            status: BookStatus::LocalOnly,
            device: None,
            local: Some(record.clone()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kindred::{BookFormat, LibraryRecord};

    fn book(title: &str, asin: Option<&str>) -> Book {
        Book {
            title: title.to_owned(),
            asin: asin.map(str::to_owned),
            format: BookFormat::Kfx,
            size_bytes: 1_000,
            content_handle: 1,
            sidecar_handle: None,
            metadata_handles: Vec::new(),
        }
    }

    fn record(key: &str, title: &str, local: Option<&str>) -> LibraryRecord {
        LibraryRecord {
            key: key.to_owned(),
            title: title.to_owned(),
            format: BookFormat::Kfx,
            size_bytes: 1_000,
            local_path: local.map(str::to_owned),
            on_device: false,
            last_seen_device: None,
        }
    }

    fn inventory(books: Vec<Book>) -> KindleInventory {
        KindleInventory {
            storage_description: "Internal Storage".to_owned(),
            books,
        }
    }

    #[test]
    fn book_key_uses_asin_or_dict_prefix() {
        assert_eq!(
            book_key(&book(
                "The Way of Kings",
                Some("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
            )),
            "JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2"
        );
        assert_eq!(
            book_key(&book("Oxford Dictionary", None)),
            "dict:Oxford Dictionary"
        );
    }

    #[test]
    fn device_only_books_are_on_device() {
        let inventory = inventory(vec![
            book("The Way of Kings", Some("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")),
            book("Rhythm of War", Some("HXEBCTKSYKRSHFCDOGPTPR3UOKBPKXQ3")),
        ]);
        let library = LocalLibrary::default();

        let entries = build_catalogue(Some(&inventory), &library);

        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .all(|entry| entry.status == BookStatus::OnDevice)
        );
        assert!(entries.iter().all(|entry| entry.device.is_some()));
    }

    #[test]
    fn local_copy_of_device_book_is_both() {
        let inventory = inventory(vec![book(
            "The Way of Kings",
            Some("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2"),
        )]);
        let mut library = LocalLibrary::default();
        library.upsert(record(
            "JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2",
            "The Way of Kings",
            Some("/books/The Way of Kings.kfx"),
        ));

        let entries = build_catalogue(Some(&inventory), &library);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].status, BookStatus::Both);
        assert!(entries[0].device.is_some());
        assert!(entries[0].local.is_some());
    }

    #[test]
    fn local_only_record_appears_without_device() {
        let mut library = LocalLibrary::default();
        library.upsert(record(
            "B0048EL62A",
            "Nice Girls Don't",
            Some("/books/Nice Girls Don't.kfx"),
        ));

        let entries = build_catalogue(None, &library);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].status, BookStatus::LocalOnly);
        assert!(entries[0].device.is_none());
        assert!(entries[0].local.is_some());
    }

    #[test]
    fn records_without_local_copy_are_skipped() {
        let mut library = LocalLibrary::default();
        library.upsert(record("B0048EL62A", "Nice Girls Don't", None));

        let entries = build_catalogue(None, &library);

        assert!(entries.is_empty());
    }
}
