//! UI update logic (GUI M2): message handling and iced tasks.
//!
//! The pure types live in `model.rs`; this module owns the `Message` enum,
//! the iced `update`/`boot` functions, and the drag-drop transfer tasks.

use iced::task::Task;

use crate::data::{self, LoadResult, TransferOutcome};
use crate::model::{AppState, DropAction, Pane, Sort, SortKey, ViewMode};
use crate::tasks::{
    collection_add_task, collection_changed_task, copy_to_library_task, send_to_kindle_task,
};

/// Messages the UI can process.
#[derive(Debug, Clone)]
pub enum Message {
    ViewModeSelected(ViewMode),
    SortSelected(SortKey),
    DragStarted { pane: Pane, index: usize },
    DragOver(Pane),
    DragExited,
    DropOn(Pane),
    DropOnCollection(usize),
    DragCancelled,
    CopyToLibrary(usize),
    SendToKindle(usize),
    Refresh,
    Loaded(Box<LoadResult>),
    TransferFinished(Box<Result<TransferOutcome, String>>),
    CollectionSelected(Option<usize>),
    CollectionCreate,
    CollectionNameChanged(String),
    ShowNewCollection,
    CollectionDelete(usize),
    CollectionRemoveBook { index: usize },
    CollectionChanged(Box<Result<String, String>>),
}

/// iced boot: default state plus an initial data load.
pub fn boot() -> (AppState, Task<Message>) {
    (
        AppState::default(),
        Task::perform(data::load_all(), |result| Message::Loaded(Box::new(result))),
    )
}

/// Update the state from a message, returning any async work (iced `UpdateFn`).
pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::ViewModeSelected(mode) => {
            state.view_mode = mode;
            Task::none()
        }
        Message::SortSelected(key) => {
            if state.sort.key == key {
                state.sort.ascending = !state.sort.ascending;
            } else {
                state.sort = Sort {
                    key,
                    ascending: true,
                };
            }
            Task::none()
        }
        Message::DragStarted { pane, index } => {
            state.selected = Some(index);
            state.drag = Some(crate::model::Drag { pane, index });
            Task::none()
        }
        Message::DragOver(pane) => {
            state.drop_target = Some(pane);
            Task::none()
        }
        Message::DragExited => {
            state.drop_target = None;
            Task::none()
        }
        Message::DropOn(target) => {
            state.drag = None;
            state.drop_target = None;
            match drop_action(state, target) {
                Some(DropAction::CopyFromKindle { index }) => copy_to_library_task(state, index),
                Some(DropAction::AddToKindle { index }) => send_to_kindle_task(state, index),
                None => Task::none(),
            }
        }
        Message::DropOnCollection(collection_index) => {
            let task = collection_add_task(state, collection_index);
            state.drag = None;
            state.drop_target = None;
            task
        }
        Message::DragCancelled => {
            state.drag = None;
            state.drop_target = None;
            Task::none()
        }
        Message::CopyToLibrary(index) => copy_to_library_task(state, index),
        Message::SendToKindle(index) => send_to_kindle_task(state, index),
        Message::CollectionSelected(selection) => {
            state.selected_collection = selection;
            Task::none()
        }
        Message::CollectionCreate => {
            let name = state.new_collection_name.trim().to_owned();
            if name.is_empty() {
                return Task::none();
            }
            state.new_collection_name = String::new();
            state.show_new_collection = false;
            collection_changed_task(data::create_collection(name))
        }
        Message::CollectionNameChanged(name) => {
            state.new_collection_name = name;
            Task::none()
        }
        Message::ShowNewCollection => {
            state.show_new_collection = !state.show_new_collection;
            Task::none()
        }
        Message::CollectionDelete(index) => {
            let Some(collection) = state.collections.get(index) else {
                return Task::none();
            };
            let name = collection.name.clone();
            if state.selected_collection == Some(index) {
                state.selected_collection = None;
            }
            collection_changed_task(data::delete_collection(name))
        }
        Message::CollectionRemoveBook { index } => {
            let Some(collection_index) = state.selected_collection else {
                return Task::none();
            };
            let Some(entry) = state.catalogue.get(index) else {
                return Task::none();
            };
            let Some(collection) = state.collections.get(collection_index) else {
                return Task::none();
            };
            collection_changed_task(data::remove_book_from_collection(
                collection.name.clone(),
                entry.key(),
            ))
        }
        Message::CollectionChanged(result) => match *result {
            Ok(summary) => {
                state.status_message = Some(summary);
                state.loading = true;
                Task::perform(data::load_all(), |result| Message::Loaded(Box::new(result)))
            }
            Err(error) => {
                state.status_message = Some(format!("Collection change failed: {error}"));
                Task::none()
            }
        },
        Message::Refresh => {
            state.loading = true;
            Task::perform(data::load_all(), |result| Message::Loaded(Box::new(result)))
        }
        Message::Loaded(result) => {
            apply_loaded(state, *result);
            Task::none()
        }
        Message::TransferFinished(result) => match *result {
            Ok(outcome) => {
                state.status_message = Some(outcome.summary());
                state.loading = true;
                Task::perform(data::load_all(), |result| Message::Loaded(Box::new(result)))
            }
            Err(error) => {
                state.status_message = Some(format!("Transfer failed: {error}"));
                Task::none()
            }
        },
    }
}

/// Decide what (if anything) a drop on `target` should do.
pub fn drop_action(state: &AppState, target: Pane) -> Option<DropAction> {
    let drag = state.drag?;
    match (drag.pane, target) {
        (Pane::Kindle, Pane::Local) => {
            let entry = state.catalogue.get(drag.index)?;
            entry
                .device
                .is_some()
                .then_some(DropAction::CopyFromKindle { index: drag.index })
        }
        (Pane::Local, Pane::Kindle) => {
            let entry = state.catalogue.get(drag.index)?;
            entry
                .has_local_copy()
                .then_some(DropAction::AddToKindle { index: drag.index })
        }
        _ => None,
    }
}

/// Apply a fresh `LoadResult`: rebuild the catalogue, device identity, and
/// collections.
fn apply_loaded(state: &mut AppState, result: LoadResult) {
    state.loading = false;
    state.device = result.device.clone();
    state.catalogue = data::build_catalogue(result.inventory.as_ref(), &result.library);
    state.collections = result.library.collections.clone();

    if let Some(selected) = state.selected
        && selected >= state.catalogue.len()
    {
        state.selected = None;
    }
    if let Some(selected) = state.selected_collection
        && selected >= state.collections.len()
    {
        state.selected_collection = None;
    }
    if !result.errors.is_empty() {
        state.status_message = Some(result.errors.join("; "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::test_support::state;
    use kindred::LocalLibrary;

    /// Apply a message in tests, discarding the returned task.
    fn apply(state: &mut AppState, message: Message) {
        let _ = update(state, message);
    }

    #[test]
    fn view_mode_switch() {
        let mut state = state();
        apply(&mut state, Message::ViewModeSelected(ViewMode::Details));
        assert_eq!(state.view_mode, ViewMode::Details);
    }

    #[test]
    fn sort_toggles_direction_on_same_key() {
        let mut state = state();
        apply(&mut state, Message::SortSelected(SortKey::Size));
        assert_eq!(state.sort.key, SortKey::Size);
        assert!(state.sort.ascending);

        apply(&mut state, Message::SortSelected(SortKey::Size));
        assert_eq!(state.sort.key, SortKey::Size);
        assert!(!state.sort.ascending);

        apply(&mut state, Message::SortSelected(SortKey::Title));
        assert_eq!(state.sort.key, SortKey::Title);
        assert!(state.sort.ascending);
    }

    #[test]
    fn drag_started_selects_and_records_drag() {
        let mut state = state();
        apply(
            &mut state,
            Message::DragStarted {
                pane: Pane::Kindle,
                index: 1,
            },
        );
        assert_eq!(state.selected, Some(1));
        assert_eq!(
            state.drag,
            Some(crate::model::Drag {
                pane: Pane::Kindle,
                index: 1
            })
        );
    }

    #[test]
    fn drop_on_same_pane_is_a_no_op() {
        let mut state = state();
        apply(
            &mut state,
            Message::DragStarted {
                pane: Pane::Kindle,
                index: 1,
            },
        );
        assert!(drop_action(&state, Pane::Kindle).is_none());
    }

    #[test]
    fn drop_kindle_to_local_copies_device_book() {
        let mut state = state();
        apply(
            &mut state,
            Message::DragStarted {
                pane: Pane::Kindle,
                index: 1,
            },
        );
        assert_eq!(
            drop_action(&state, Pane::Local),
            Some(DropAction::CopyFromKindle { index: 1 })
        );
    }

    #[test]
    fn drop_local_to_kindle_sends_local_copy() {
        let mut state = state();
        apply(
            &mut state,
            Message::DragStarted {
                pane: Pane::Local,
                index: 2,
            },
        );
        assert_eq!(
            drop_action(&state, Pane::Kindle),
            Some(DropAction::AddToKindle { index: 2 })
        );
    }

    #[test]
    fn dropping_a_device_only_book_from_local_pane_is_none() {
        let mut state = state();
        apply(
            &mut state,
            Message::DragStarted {
                pane: Pane::Local,
                index: 1,
            },
        );
        assert!(drop_action(&state, Pane::Kindle).is_none());
    }

    #[test]
    fn loaded_message_rebuilds_catalogue() {
        let mut state = state();
        let result = LoadResult {
            device: None,
            inventory: None,
            library: LocalLibrary::default(),
            errors: Vec::new(),
        };
        apply(&mut state, Message::Loaded(Box::new(result)));
        assert!(!state.loading);
        assert!(state.catalogue.is_empty());
        assert!(state.device.is_none());
    }

    #[test]
    fn collection_selection_sets_and_clears() {
        let mut state = state();
        state.collections = vec![kindred::LocalCollection {
            name: "Favourites".to_owned(),
            book_keys: vec![],
        }];

        apply(&mut state, Message::CollectionSelected(Some(0)));
        assert_eq!(state.selected_collection, Some(0));

        apply(&mut state, Message::CollectionSelected(None));
        assert_eq!(state.selected_collection, None);
    }

    #[test]
    fn collection_create_with_blank_name_is_a_no_op() {
        let mut state = state();
        state.show_new_collection = true;
        state.new_collection_name = "   ".to_owned();
        apply(&mut state, Message::CollectionCreate);
        assert!(state.show_new_collection);
        assert_eq!(state.new_collection_name, "   ");
    }

    #[test]
    fn collection_delete_clears_matching_selection() {
        let mut state = state();
        state.collections = vec![kindred::LocalCollection {
            name: "Favourites".to_owned(),
            book_keys: vec![],
        }];
        state.selected_collection = Some(0);

        apply(&mut state, Message::CollectionDelete(0));
        assert_eq!(state.selected_collection, None);
    }
}
