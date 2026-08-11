//! Controlled write proof (Milestone 4).
//!
//! This module deliberately exposes exactly ONE write operation: a tightly
//! scoped integration test that uploads, verifies, and removes a harmless
//! test artifact inside a dedicated folder under `documents/`. It is the
//! only write path in Kindred and never touches book content.

use bytes::Bytes;
use futures_util::stream;
use mtp_rs::mtp::{MtpDevice, NewObjectInfo, ObjectHandle, Storage};

/// Name prefix of the dedicated test folder created under `documents/`.
pub const TEST_FOLDER_PREFIX: &str = "kindling_m4_test";

/// Outcome of a controlled transfer test run.
#[derive(Debug, Clone)]
pub struct TransferTestResult {
    pub folder_name: String,
    pub uploaded_file: String,
    pub uploaded_bytes: usize,
    pub verified_readback: bool,
    pub cleaned_up: bool,
}

/// Upload a harmless artifact, verify it appears and reads back byte-for-byte,
/// then remove the artifact and its test folder.
///
/// The test folder is created under `documents/` with a unique name and is
/// removed on success. On failure the folder is removed if possible so no
/// partial artifact is ever left behind.
pub async fn run_controlled_transfer_test() -> Result<TransferTestResult, mtp_rs::Error> {
    let device = MtpDevice::open_first().await?;
    let mut storages = device.storages().await?;
    let storage = storages.pop().ok_or(mtp_rs::Error::NoDevice)?;

    let documents = find_documents(&storage).await?;
    let folder_name = format!("{TEST_FOLDER_PREFIX}_{}", timestamp_suffix());
    let folder = storage.create_folder(Some(documents), &folder_name).await?;

    let result = run_test_body(&storage, folder, &folder_name).await;

    if result.is_err() {
        // Never leave a partial test artifact behind.
        let _ = storage.delete(folder).await;
    }

    result
}

async fn run_test_body(
    storage: &Storage,
    folder: ObjectHandle,
    folder_name: &str,
) -> Result<TransferTestResult, mtp_rs::Error> {
    let file_name = "kindling_controlled_test.txt".to_owned();
    let payload = b"kindling controlled transfer test\n".to_vec();

    // Upload the controlled artifact into the test folder.
    let info = NewObjectInfo::file(file_name.clone(), payload.len() as u64);
    let data = stream::iter([Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(
        &payload,
    ))]);
    let handle = storage.upload(Some(folder), info, data).await?;

    // Verify the artifact appears in the folder listing.
    let children = storage.list_objects(Some(folder)).await?;
    let found = children
        .iter()
        .any(|object| object.filename == file_name && !object.is_folder());
    if !found {
        return Err(mtp_rs::Error::InvalidData {
            message: "uploaded artifact not found in test folder".to_owned(),
        });
    }

    // Read it back and verify it is byte-for-byte identical.
    let read_back = storage.download_to_vec(handle).await?;
    let verified_readback = read_back == payload;

    // Remove the controlled artifact and its test folder.
    storage.delete(handle).await?;
    storage.delete(folder).await?;

    Ok(TransferTestResult {
        folder_name: folder_name.to_owned(),
        uploaded_file: file_name,
        uploaded_bytes: payload.len(),
        verified_readback,
        cleaned_up: true,
    })
}

async fn find_documents(storage: &Storage) -> Result<ObjectHandle, mtp_rs::Error> {
    let root = storage.list_objects(None).await?;
    root.iter()
        .find(|object| object.is_folder() && object.filename == "documents")
        .map(|object| object.handle)
        .ok_or(mtp_rs::Error::NotFound)
}

fn timestamp_suffix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
