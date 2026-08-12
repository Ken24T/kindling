//! Local collection operations for the GUI: create/delete and book
//! add/remove, persisted to the local library JSON.
//!
//! Collections are a local, USB-independent grouping. The device's own
//! Amazon Collections are not readable over MTP (see docs/device/README.md),
//! so these are the app's own categories keyed by stable book key.

use kindred::LocalLibrary;

use crate::data::paths::library_path;

/// Create a collection; returns a user-facing status summary.
pub async fn create_collection(name: String) -> Result<String, String> {
    let mut library = LocalLibrary::load(&library_path()).map_err(|error| error.to_string())?;
    library
        .create_collection(&name)
        .map_err(|error| error.to_string())?;
    library
        .save(&library_path())
        .map_err(|error| error.to_string())?;
    Ok(format!("Created collection '{name}'."))
}

/// Rename a collection; returns a user-facing status summary.
pub async fn rename_collection(old: String, new: String) -> Result<String, String> {
    let mut library = LocalLibrary::load(&library_path()).map_err(|error| error.to_string())?;
    library
        .rename_collection(&old, &new)
        .map_err(|error| error.to_string())?;
    library
        .save(&library_path())
        .map_err(|error| error.to_string())?;
    Ok(format!("Renamed collection '{old}' to '{new}'."))
}

/// Delete a collection (its books stay in the library); returns a summary.
pub async fn delete_collection(name: String) -> Result<String, String> {
    let mut library = LocalLibrary::load(&library_path()).map_err(|error| error.to_string())?;
    library.delete_collection(&name);
    library
        .save(&library_path())
        .map_err(|error| error.to_string())?;
    Ok(format!("Deleted collection '{name}'."))
}

/// Add a book key to a collection; returns a user-facing status summary.
pub async fn add_book_to_collection(name: String, key: String) -> Result<String, String> {
    let mut library = LocalLibrary::load(&library_path()).map_err(|error| error.to_string())?;
    library
        .add_book_to_collection(&name, &key)
        .map_err(|error| error.to_string())?;
    library
        .save(&library_path())
        .map_err(|error| error.to_string())?;
    Ok(format!("Added to '{name}'."))
}

/// Remove a book key from a collection; returns a user-facing status summary.
pub async fn remove_book_from_collection(name: String, key: String) -> Result<String, String> {
    let mut library = LocalLibrary::load(&library_path()).map_err(|error| error.to_string())?;
    library
        .remove_book_from_collection(&name, &key)
        .map_err(|error| error.to_string())?;
    library
        .save(&library_path())
        .map_err(|error| error.to_string())?;
    Ok(format!("Removed from '{name}'."))
}
