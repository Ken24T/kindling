//! Local collections: user-named groupings of book keys.
//!
//! Device-side Amazon Collections are not readable over MTP (see
//! docs/device/README.md), so collections here are a local, USB-independent
//! grouping stored inside the local library JSON.

use serde::{Deserialize, Serialize};

use crate::library::{LibraryError, LocalLibrary};

/// A user-named local collection of book keys.
///
/// Keys are the stable library keys: ASIN, or `dict:<title>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCollection {
    pub name: String,
    /// Stable library keys of the books in this collection.
    pub book_keys: Vec<String>,
}

impl LocalLibrary {
    /// Look up a named collection.
    pub fn collection(&self, name: &str) -> Option<&LocalCollection> {
        self.collections
            .iter()
            .find(|collection| collection.name == name)
    }

    /// Create a new empty collection; errors when the name exists or is blank.
    pub fn create_collection(&mut self, name: &str) -> Result<(), LibraryError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(LibraryError::EmptyCollectionName);
        }
        if self.collection(name).is_some() {
            return Err(LibraryError::CollectionExists(name.to_owned()));
        }
        self.collections.push(LocalCollection {
            name: name.to_owned(),
            book_keys: Vec::new(),
        });
        Ok(())
    }

    /// Rename a collection; errors when the new name exists or is blank.
    pub fn rename_collection(&mut self, old: &str, new: &str) -> Result<(), LibraryError> {
        let new = new.trim();
        if new.is_empty() {
            return Err(LibraryError::EmptyCollectionName);
        }
        if self.collection(new).is_some() && new != old {
            return Err(LibraryError::CollectionExists(new.to_owned()));
        }
        let collection = self
            .collections
            .iter_mut()
            .find(|collection| collection.name == old)
            .ok_or_else(|| LibraryError::CollectionNotFound(old.to_owned()))?;
        collection.name = new.to_owned();
        Ok(())
    }

    /// Delete a collection (its books are not removed from the library).
    pub fn delete_collection(&mut self, name: &str) {
        self.collections
            .retain(|collection| collection.name != name);
    }

    /// Add a book key to a collection (no-op when already present).
    pub fn add_book_to_collection(&mut self, name: &str, key: &str) -> Result<(), LibraryError> {
        let collection = self
            .collections
            .iter_mut()
            .find(|collection| collection.name == name)
            .ok_or_else(|| LibraryError::CollectionNotFound(name.to_owned()))?;
        if !collection.book_keys.iter().any(|existing| existing == key) {
            collection.book_keys.push(key.to_owned());
        }
        Ok(())
    }

    /// Remove a book key from a collection (no-op when absent).
    pub fn remove_book_from_collection(
        &mut self,
        name: &str,
        key: &str,
    ) -> Result<(), LibraryError> {
        let collection = self
            .collections
            .iter_mut()
            .find(|collection| collection.name == name)
            .ok_or_else(|| LibraryError::CollectionNotFound(name.to_owned()))?;
        collection.book_keys.retain(|existing| existing != key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
