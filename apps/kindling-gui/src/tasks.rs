//! Async task builders for transfers and local collections (GUI M2).
//!
//! Thin wrappers that turn Kindred/data operations into iced `Task`s that
//! publish `Message`s back to the update loop. Kept separate from the
//! `update` dispatcher so each file stays small and single-purpose.

use iced::task::Task;

use crate::data;
use crate::model::AppState;
use crate::update::Message;

/// Task for copying a device book into the local library.
pub fn copy_to_library_task(state: &AppState, index: usize) -> Task<Message> {
    let Some(book) = state
        .catalogue
        .get(index)
        .and_then(|entry| entry.device.clone())
    else {
        return Task::none();
    };
    Task::perform(data::copy_from_kindle(book), |result| {
        Message::TransferFinished(Box::new(result))
    })
}

/// Task for sending a local book file to the Kindle.
pub fn send_to_kindle_task(state: &AppState, index: usize) -> Task<Message> {
    let Some(entry) = state.catalogue.get(index) else {
        return Task::none();
    };
    let Some(local_path) = entry
        .local
        .as_ref()
        .and_then(|record| record.local_path.clone())
    else {
        return Task::none();
    };
    let title = entry.title.clone();
    Task::perform(data::add_to_kindle(local_path, title), |result| {
        Message::TransferFinished(Box::new(result))
    })
}

/// Task for dropping the dragged book into a collection.
pub fn collection_add_task(state: &AppState, collection_index: usize) -> Task<Message> {
    match collection_add_target(state, collection_index) {
        Some((name, key)) => collection_changed_task(data::add_book_to_collection(name, key)),
        None => Task::none(),
    }
}

/// The (collection name, book key) a drop on a collection would add, or
/// `None` when there is no active drag or the collection index is invalid.
pub fn collection_add_target(
    state: &AppState,
    collection_index: usize,
) -> Option<(String, String)> {
    let drag = state.drag?;
    let entry = state.catalogue.get(drag.index)?;
    let collection = state.collections.get(collection_index)?;
    Some((collection.name.clone(), entry.key()))
}

/// Wrap a collection-mutation future in a `CollectionChanged` task.
pub fn collection_changed_task(
    future: impl std::future::Future<Output = Result<String, String>> + Send + 'static,
) -> Task<Message> {
    Task::perform(future, |result| {
        Message::CollectionChanged(Box::new(result))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::test_support::state;
    use kindred::LocalCollection;

    #[test]
    fn collection_drop_target_needs_drag_and_valid_collection() {
        let mut state = state();
        state.collections = vec![LocalCollection {
            name: "Favourites".to_owned(),
            book_keys: vec![],
        }];

        // No drag in flight → None.
        assert!(collection_add_target(&state, 0).is_none());

        // Drag a local book; valid collection → (name, key).
        let _ = crate::update::update(
            &mut state,
            crate::update::Message::DragStarted {
                pane: crate::model::Pane::Local,
                index: 2,
            },
        );
        assert_eq!(
            collection_add_target(&state, 0),
            Some(("Favourites".to_owned(), "ASINGamma".to_owned()))
        );

        // Out-of-range collection index → None.
        assert!(collection_add_target(&state, 5).is_none());
    }
}
