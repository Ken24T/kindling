//! UI state types (GUI M2).
//!
//! Pure types only; update logic lives in `update.rs`, data wiring in
//! `data/`. Views import `Message` through this module.

use kindred::{Book, BookFormat, BookStatus, LibraryRecord};

pub use crate::update::Message;

/// The two library panes; also the drag-drop source/targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Local,
    Kindle,
}

impl Pane {
    pub fn title(&self) -> &'static str {
        match self {
            Pane::Local => "Local Library",
            Pane::Kindle => "Kindle Library",
        }
    }
}

/// Explorer-style view modes, applied to both panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Covers,
    List,
    Details,
}

impl ViewMode {
    pub fn title(&self) -> &'static str {
        match self {
            ViewMode::Covers => "Covers",
            ViewMode::List => "List",
            ViewMode::Details => "Details",
        }
    }
}

/// Sortable columns in Details mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Title,
    Format,
    Size,
    Status,
}

/// Current sort: key plus direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub key: SortKey,
    pub ascending: bool,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            key: SortKey::Title,
            ascending: true,
        }
    }
}

/// Friendly device info shown in the UI (never the raw serial).
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub friendly_name: String,
    pub model: String,
}

/// A book as shown in the UI, merged from the device inventory and the
/// local library.
#[derive(Debug, Clone)]
pub struct BookEntry {
    pub title: String,
    pub format: BookFormat,
    pub size_bytes: u64,
    pub status: BookStatus,
    /// Device-side data (with MTP handles) when present on the device.
    pub device: Option<Book>,
    /// Local library record when one exists.
    pub local: Option<LibraryRecord>,
}

impl BookEntry {
    /// Whether a local file copy exists for this book.
    pub fn has_local_copy(&self) -> bool {
        self.local
            .as_ref()
            .and_then(|record| record.local_path.as_deref())
            .is_some()
    }

    /// The book's ASIN for display (device ASIN, or a non-dict record key).
    pub fn asin(&self) -> Option<&str> {
        if let Some(book) = &self.device
            && let Some(asin) = book.asin.as_deref()
        {
            return Some(asin);
        }
        if let Some(record) = &self.local
            && !record.key.starts_with("dict:")
        {
            return Some(record.key.as_str());
        }
        None
    }
}

/// An in-flight drag of a book from one pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drag {
    pub pane: Pane,
    pub index: usize,
}

/// What a drop on a pane resolves to (if anything).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropAction {
    CopyFromKindle { index: usize },
    AddToKindle { index: usize },
}

/// Full UI state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub view_mode: ViewMode,
    pub sort: Sort,
    /// Merged entries (device + local library).
    pub catalogue: Vec<BookEntry>,
    /// Catalogue index of the selected book.
    pub selected: Option<usize>,
    /// Friendly identity of the attached device, when present.
    pub device: Option<DeviceInfo>,
    pub loading: bool,
    /// Book currently being dragged.
    pub drag: Option<Drag>,
    /// Pane currently hovered by the drag.
    pub drop_target: Option<Pane>,
    /// Last user-facing status (transfer results, load errors).
    pub status_message: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Covers,
            sort: Sort::default(),
            catalogue: Vec::new(),
            selected: None,
            device: None,
            loading: true,
            drag: None,
            drop_target: None,
            status_message: None,
        }
    }
}

impl AppState {
    /// Catalogue indices visible in a pane, sorted by the current sort.
    pub fn pane_books(&self, pane: Pane) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .catalogue
            .iter()
            .enumerate()
            .filter(|(_, entry)| match pane {
                Pane::Local => entry.has_local_copy(),
                Pane::Kindle => entry.device.is_some(),
            })
            .map(|(index, _)| index)
            .collect();

        indices.sort_by(|&a, &b| {
            compare_entries(&self.catalogue[a], &self.catalogue[b], self.sort.key)
        });
        if !self.sort.ascending {
            indices.reverse();
        }
        indices
    }
}

/// Short status label shared by the views.
pub fn status_label(status: BookStatus) -> &'static str {
    match status {
        BookStatus::Both => "On device + local",
        BookStatus::OnDevice => "On device",
        BookStatus::LocalOnly => "Local only",
    }
}

/// Ascending comparator for a sort key; the caller reverses for descending.
fn compare_entries(a: &BookEntry, b: &BookEntry, key: SortKey) -> std::cmp::Ordering {
    match key {
        SortKey::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        SortKey::Format => a.format.label().cmp(b.format.label()),
        SortKey::Size => a.size_bytes.cmp(&b.size_bytes),
        SortKey::Status => status_rank(a.status).cmp(&status_rank(b.status)),
    }
}

fn status_rank(status: BookStatus) -> u8 {
    match status {
        BookStatus::LocalOnly => 0,
        BookStatus::OnDevice => 1,
        BookStatus::Both => 2,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Shared fixtures for the model and update test modules.
    use super::*;

    pub fn entry(title: &str, status: BookStatus) -> BookEntry {
        let device = (status != BookStatus::LocalOnly).then(|| Book {
            title: title.to_owned(),
            asin: Some(format!("ASIN{title}")),
            format: BookFormat::Kfx,
            size_bytes: 1_000,
            content_handle: 1,
            sidecar_handle: None,
            metadata_handles: Vec::new(),
        });
        let local = (status != BookStatus::OnDevice).then(|| LibraryRecord {
            key: format!("ASIN{title}"),
            title: title.to_owned(),
            format: BookFormat::Kfx,
            size_bytes: 1_000,
            local_path: Some(format!("/books/{title}.kfx")),
            on_device: status == BookStatus::Both,
            last_seen_device: None,
        });
        BookEntry {
            title: title.to_owned(),
            format: BookFormat::Kfx,
            size_bytes: 1_000,
            status,
            device,
            local,
        }
    }

    pub fn state() -> AppState {
        AppState {
            catalogue: vec![
                entry("Alpha", BookStatus::Both),
                entry("Beta", BookStatus::OnDevice),
                entry("Gamma", BookStatus::LocalOnly),
            ],
            ..AppState::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_covers_with_no_device() {
        let state = AppState::default();
        assert_eq!(state.view_mode, ViewMode::Covers);
        assert!(state.device.is_none());
        assert!(state.catalogue.is_empty());
        assert!(state.loading);
    }

    #[test]
    fn pane_books_filters_and_sorts() {
        let state = test_support::state();
        let local = state.pane_books(Pane::Local);
        let kindle = state.pane_books(Pane::Kindle);

        // Local: Alpha (Both) + Gamma (LocalOnly) — sorted by title.
        assert_eq!(local, vec![0, 2]);
        // Kindle: Alpha (Both) + Beta (OnDevice).
        assert_eq!(kindle, vec![0, 1]);
    }

    #[test]
    fn pane_books_sort_by_size_descending() {
        let mut state = test_support::state();
        state.sort = Sort {
            key: SortKey::Size,
            ascending: false,
        };
        let kindle = state.pane_books(Pane::Kindle);
        assert_eq!(kindle, vec![1, 0]);
    }

    #[test]
    fn asin_falls_back_to_non_dict_record_key() {
        let mut state = test_support::state();
        state.catalogue[2].local.as_mut().unwrap().key = "B0048EL62A".to_owned();
        assert_eq!(state.catalogue[2].asin(), Some("B0048EL62A"));
    }
}
