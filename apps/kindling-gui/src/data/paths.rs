//! Local data paths (mirror the CLI): state files and the book store.

use std::path::PathBuf;

/// Directory holding `library.json`, `profiles.json` and the book store.
pub fn state_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("KINDLING_STATE_DIR") {
        return PathBuf::from(path);
    }
    directories::ProjectDirs::from("", "", "kindling")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn library_path() -> PathBuf {
    state_dir().join("library.json")
}

pub fn profile_path() -> PathBuf {
    state_dir().join("profiles.json")
}

/// Directory where books copied from the Kindle are stored.
pub fn library_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("KINDLING_LIBRARY_DIR") {
        return PathBuf::from(path);
    }
    state_dir().join("library")
}

/// Directory holding user-supplied cover images (`<key>.<ext>`).
pub fn covers_dir() -> PathBuf {
    library_dir().join("covers")
}
