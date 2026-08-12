//! User-supplied cover images for the local library.
//!
//! Covers are local images the user drops into a `covers/` folder, named
//! `<key>.<ext>` where the key is the record's stable key (ASIN, or
//! `dict:<title>`) with non-alphanumerics replaced by `_`. The device's own
//! cover caches are classic-ASIN-keyed and do not cover this library's
//! 32-char-id books (see docs/device/README.md), so covers are a local,
//! offline feature rather than a device sync.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::library::{LibraryError, LocalLibrary};

impl LocalLibrary {
    /// Match user-supplied cover images to records and record their paths.
    ///
    /// Covers live in `covers_dir` as `<sanitized-key>.<ext>` where the key
    /// (ASIN, or `dict:<title>`) has non-alphanumeric characters replaced with
    /// `_` (e.g. `JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.jpg`). Stale paths are
    /// cleared when a cover is removed. Returns the number of records changed.
    pub fn scan_covers(&mut self, covers_dir: &Path) -> Result<usize, LibraryError> {
        if !covers_dir.is_dir() {
            return Ok(0);
        }

        let mut covers: HashMap<String, PathBuf> = HashMap::new();
        for entry in fs::read_dir(covers_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                continue;
            };
            covers.insert(stem.to_owned(), path);
        }

        let mut changed = 0;
        for record in &mut self.records {
            let matched = covers.get(&sanitize_key(&record.key));
            let path_str = matched.map(|path| path.to_string_lossy().into_owned());
            if record.cover_path != path_str {
                record.cover_path = path_str;
                changed += 1;
            }
        }
        Ok(changed)
    }
}

/// Replace non-alphanumeric characters so a key can be a filename.
fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::BookFormat;
    use crate::library::LibraryRecord;

    fn record(key: &str) -> LibraryRecord {
        LibraryRecord {
            key: key.to_owned(),
            title: key.to_owned(),
            format: BookFormat::Kfx,
            size_bytes: 100,
            local_path: None,
            cover_path: None,
            on_device: false,
            last_seen_device: None,
        }
    }

    #[test]
    fn scan_covers_matches_files_and_clears_stale() {
        let dir = std::env::temp_dir().join("kindling_test_covers");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.jpg"), b"x").unwrap();
        std::fs::write(dir.join("dict_Oxford_Dictionary.png"), b"x").unwrap();

        let mut library = LocalLibrary::default();
        library.upsert(record("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2"));
        library.upsert(record("dict:Oxford Dictionary"));
        library.upsert(record("B0048EL62A"));

        let changed = library.scan_covers(&dir).unwrap();

        assert_eq!(changed, 2);
        assert!(
            library
                .lookup("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
                .unwrap()
                .cover_path
                .as_deref()
                .is_some_and(|path| path.ends_with("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.jpg"))
        );
        assert!(
            library
                .lookup("dict:Oxford Dictionary")
                .unwrap()
                .cover_path
                .is_some()
        );
        assert!(library.lookup("B0048EL62A").unwrap().cover_path.is_none());

        // Re-scanning is stable (no changes), and removing the file clears it.
        assert_eq!(library.scan_covers(&dir).unwrap(), 0);
        std::fs::remove_file(dir.join("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2.jpg")).unwrap();
        assert_eq!(library.scan_covers(&dir).unwrap(), 1);
        assert!(
            library
                .lookup("JX7VYKM3IPBSUZSDGADVUXVLCVHECJD2")
                .unwrap()
                .cover_path
                .is_none()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
