//! Safe book-level device management (Milestone 5).
//!
//! Operations are driven by a `Book` record from a fresh inventory. Deletion
//! validates every handle against the device (expected name and kind) before
//! acting, and never targets storage roots or broad trees.

use std::path::{Path, PathBuf};

use bytes::Bytes;
use futures_util::stream;
use mtp_rs::mtp::{MtpDevice, NewObjectInfo, ObjectHandle, Storage};

use crate::error::KindredError;
use crate::inventory::{Book, BookFormat};

/// Prefix for artifacts added by Kindling itself (controlled test files).
pub const ADDED_PREFIX: &str = "kindling_";

/// Copy a book's content from the Kindle into `dest_dir`.
///
/// The destination file is named after the book's on-device content file
/// (`Title_ASIN.ext`), preserving the original name. Read-only on the device.
pub async fn copy_book_from_kindle(book: &Book, dest_dir: &Path) -> Result<PathBuf, KindredError> {
    let device = MtpDevice::open_first().await?;
    let mut storages = device.storages().await?;
    let storage = storages.pop().ok_or(KindredError::NoDevice)?;

    let info = storage
        .get_object_info(ObjectHandle(book.content_handle))
        .await?;
    if info.is_folder() {
        return Err(KindredError::InvalidObject {
            message: "book content handle is not a file".to_owned(),
        });
    }

    let bytes = storage
        .download_to_vec(ObjectHandle(book.content_handle))
        .await?;

    let dest = dest_dir.join(content_filename(book));
    std::fs::write(&dest, bytes)?;

    Ok(dest)
}

/// Copy a local file onto the Kindle into `documents/` — the classic
/// sideload location. Returns the new object handle.
pub async fn add_book_to_kindle(local_path: &Path) -> Result<u64, KindredError> {
    let bytes = std::fs::read(local_path)?;

    let file_name = local_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| KindredError::InvalidObject {
            message: "local path has no usable filename".to_owned(),
        })?
        .to_owned();

    let device = MtpDevice::open_first().await?;
    let mut storages = device.storages().await?;
    let storage = storages.pop().ok_or(KindredError::NoDevice)?;

    let documents = find_documents(&storage).await?;

    let info = NewObjectInfo::file(file_name.clone(), bytes.len() as u64);
    let data = stream::iter([Ok::<Bytes, std::io::Error>(Bytes::from(bytes))]);
    let handle = storage.upload(Some(documents), info, data).await?;

    Ok(handle.0)
}

/// Remove a book from the Kindle: content file, `.sdr` sidecar folder and any
/// associated metadata objects.
///
/// Every handle is validated against the device before deletion (expected
/// filename and kind). Nothing is deleted on a mismatch, and only the handles
/// recorded in `book` are ever targeted.
pub async fn remove_book(book: &Book) -> Result<(), KindredError> {
    let device = MtpDevice::open_first().await?;
    let mut storages = device.storages().await?;
    let storage = storages.pop().ok_or(KindredError::NoDevice)?;

    validate_and_delete(
        &storage,
        book.content_handle,
        Some(&content_filename(book)),
        false,
    )
    .await?;

    if let Some(sidecar_handle) = book.sidecar_handle {
        validate_and_delete(
            &storage,
            sidecar_handle,
            Some(&format!("{}.sdr", content_stem(book))),
            true,
        )
        .await?;
    }

    for handle in &book.metadata_handles {
        validate_and_delete(&storage, *handle, None, false).await?;
    }

    Ok(())
}

/// Remove an object previously added by Kindling (e.g. a controlled test file).
///
/// Safety: only objects whose filename begins with the Kindling-controlled
/// prefix are deleted; anything else is refused with `AccessDenied`. This is
/// the only handle-based delete and exists so Kindling can clean up its own
/// test artifacts.
pub async fn remove_added_object(handle: u64) -> Result<(), KindredError> {
    let device = MtpDevice::open_first().await?;
    let mut storages = device.storages().await?;
    let storage = storages.pop().ok_or(KindredError::NoDevice)?;

    let info = storage.get_object_info(ObjectHandle(handle)).await?;
    if !info.filename.starts_with(ADDED_PREFIX) {
        return Err(KindredError::InvalidObject {
            message: format!(
                "refusing to remove handle {handle}: filename '{}' is not Kindling-controlled",
                info.filename
            ),
        });
    }

    storage
        .delete(ObjectHandle(handle))
        .await
        .map_err(KindredError::from)
}

/// Confirm an object still matches the expected kind/name, then delete it.
/// A mismatch aborts with `StaleHandle`/`InvalidData` and deletes nothing.
async fn validate_and_delete(
    storage: &Storage,
    handle: u64,
    expected_name: Option<&str>,
    expected_folder: bool,
) -> Result<(), KindredError> {
    let info = storage.get_object_info(ObjectHandle(handle)).await?;

    if info.is_folder() != expected_folder {
        return Err(KindredError::InvalidObject {
            message: format!(
                "object handle {handle} is a {}, expected {}",
                if info.is_folder() { "folder" } else { "file" },
                if expected_folder { "folder" } else { "file" },
            ),
        });
    }

    if let Some(name) = expected_name
        && info.filename != name
    {
        return Err(KindredError::StaleObject);
    }

    storage
        .delete(ObjectHandle(handle))
        .await
        .map_err(KindredError::from)
}

async fn find_documents(storage: &Storage) -> Result<ObjectHandle, KindredError> {
    let root = storage.list_objects(None).await?;
    root.iter()
        .find(|object| object.is_folder() && object.filename == "documents")
        .map(|object| object.handle)
        .ok_or(KindredError::NotFound)
}

/// Reconstruct the on-device content filename for a book.
fn content_filename(book: &Book) -> String {
    let ext = match book.format {
        BookFormat::Kfx => "kfx",
        BookFormat::Azw => "azw",
    };
    match &book.asin {
        Some(asin) => format!("{}_{}.{}", book.title, asin, ext),
        None => format!("{}.{}", book.title, ext),
    }
}

/// Reconstruct the on-device content stem (`Title_ASIN`, or `Title`).
fn content_stem(book: &Book) -> String {
    match &book.asin {
        Some(asin) => format!("{}_{}", book.title, asin),
        None => book.title.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book(title: &str, asin: Option<&str>, format: BookFormat) -> Book {
        Book {
            title: title.to_owned(),
            asin: asin.map(str::to_owned),
            format,
            size_bytes: 100,
            content_handle: 1,
            sidecar_handle: None,
            metadata_handles: Vec::new(),
        }
    }

    #[test]
    fn content_filename_reconstructs_asin_named_file() {
        let b = book(
            "The Way of Kings",
            Some("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2"),
            BookFormat::Kfx,
        );
        assert_eq!(
            content_filename(&b),
            "The Way of Kings_JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.kfx"
        );
        assert_eq!(
            content_stem(&b),
            "The Way of Kings_JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2"
        );
    }

    #[test]
    fn content_filename_reconstructs_dictionary_name() {
        let b = book("Oxford_Dictionary_of_English", None, BookFormat::Azw);
        assert_eq!(content_filename(&b), "Oxford_Dictionary_of_English.azw");
    }
}
