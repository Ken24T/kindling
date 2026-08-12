//! GUI application state (GUI M1).
//!
//! A thin view model over Kindred's shared data types (`Book`, `BookStatus`).
//! No device logic lives here; the shell is mock-data driven until real
//! `inventory_device` / `LocalLibrary` wiring lands (GUI M2, per PLAN.md).

use kindred::{Book, BookStatus};

use crate::mock::mock_catalogue;

/// Logical sections in the left pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Locally-held books (LocalOnly + Both).
    LocalLibrary,
    /// Books on the attached Kindle (OnDevice + Both).
    KindleLibrary,
}

impl Section {
    pub fn title(&self) -> &'static str {
        match self {
            Section::LocalLibrary => "Local Library",
            Section::KindleLibrary => "Kindle Library",
        }
    }
}

/// Messages the UI can process.
#[derive(Debug, Clone)]
pub enum Message {
    SectionSelected(Section),
    BookSelected(usize),
}

/// A book as shown in the UI: the shared Kindred data plus a status badge.
#[derive(Debug, Clone)]
pub struct BookEntry {
    pub book: Book,
    pub status: BookStatus,
}

/// Full UI state (implements `Default` for the iced boot function).
#[derive(Debug, Clone)]
pub struct AppState {
    pub section: Section,
    /// The full mock catalogue.
    pub catalogue: Vec<BookEntry>,
    /// Catalogue index of the selected book.
    pub selected: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            section: Section::LocalLibrary,
            catalogue: mock_catalogue(),
            selected: None,
        }
    }
}

impl AppState {
    /// Catalogue indices visible for the current section.
    pub fn visible_books(&self) -> Vec<usize> {
        self.catalogue
            .iter()
            .enumerate()
            .filter(|(_, entry)| match self.section {
                Section::LocalLibrary => entry.status != BookStatus::OnDevice,
                Section::KindleLibrary => entry.status != BookStatus::LocalOnly,
            })
            .map(|(index, _)| index)
            .collect()
    }
}

/// Update the state from a message (iced `UpdateFn`).
pub fn update(state: &mut AppState, message: Message) {
    match message {
        Message::SectionSelected(section) => {
            state.section = section;
            state.selected = None;
        }
        Message::BookSelected(index) => state.selected = Some(index),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> AppState {
        AppState::default()
    }

    #[test]
    fn default_section_is_local_library() {
        assert_eq!(state().section, Section::LocalLibrary);
        assert!(state().selected.is_none());
    }

    #[test]
    fn selecting_book_sets_selection() {
        let mut state = state();
        update(&mut state, Message::BookSelected(7));
        assert_eq!(state.selected, Some(7));
    }

    #[test]
    fn switching_section_clears_selection() {
        let mut state = state();
        update(&mut state, Message::BookSelected(7));
        update(&mut state, Message::SectionSelected(Section::KindleLibrary));
        assert_eq!(state.section, Section::KindleLibrary);
        assert!(state.selected.is_none());
    }

    #[test]
    fn visible_books_filters_by_section() {
        use std::collections::HashSet;

        let state = state();
        let local: HashSet<usize> = state.visible_books().into_iter().collect();
        let kindle: HashSet<usize> = AppState {
            section: Section::KindleLibrary,
            ..state.clone()
        }
        .visible_books()
        .into_iter()
        .collect();

        // Every book is visible in at least one section; some appear in both.
        assert!(!local.is_empty());
        assert!(!kindle.is_empty());
        assert_eq!(local.union(&kindle).count(), state.catalogue.len());
    }
}
