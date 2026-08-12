//! Pane-to-pane transfers (device ↔ local library).

use std::path::Path;

use kindred::{Book, LibraryRecord, LocalLibrary, add_book_to_kindle, copy_book_from_kindle};

use crate::data::merge::book_key;
use crate::data::paths::{library_dir, library_path};

/// Outcome of a pane-to-pane transfer.
#[derive(Debug, Clone)]
pub enum TransferOutcome {
    CopiedFromKindle { title: String },
    AddedToKindle { title: String },
}

impl TransferOutcome {
    pub fn summary(&self) -> String {
        match self {
            TransferOutcome::CopiedFromKindle { title } => {
                format!("Copied '{title}' to the local library.")
            }
            TransferOutcome::AddedToKindle { title } => {
                format!("Sent '{title}' to the Kindle.")
            }
        }
    }
}

/// Copy a device book into the local library directory, then record it.
pub async fn copy_from_kindle(book: Book) -> Result<TransferOutcome, String> {
    let dest = copy_book_from_kindle(&book, &library_dir())
        .await
        .map_err(|error| error.to_string())?;
    record_copied_book(&book, &dest).await?;
    Ok(TransferOutcome::CopiedFromKindle { title: book.title })
}

/// Upload a local book file onto the Kindle (`documents/` sideload).
pub async fn add_to_kindle(local_path: String, title: String) -> Result<TransferOutcome, String> {
    add_book_to_kindle(Path::new(&local_path))
        .await
        .map_err(|error| error.to_string())?;
    Ok(TransferOutcome::AddedToKindle { title })
}

/// Upsert the local library record for a freshly copied book.
async fn record_copied_book(book: &Book, dest: &Path) -> Result<(), String> {
    let mut library = LocalLibrary::load(&library_path()).map_err(|error| error.to_string())?;
    library.upsert(LibraryRecord {
        key: book_key(book),
        title: book.title.clone(),
        format: book.format,
        size_bytes: book.size_bytes,
        local_path: Some(dest.to_string_lossy().into_owned()),
        cover_path: None,
        on_device: true,
        last_seen_device: None,
    });
    library
        .save(&library_path())
        .map_err(|error| error.to_string())
}
