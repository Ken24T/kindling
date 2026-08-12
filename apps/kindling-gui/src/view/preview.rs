//! Right preview pane: details and action buttons for the selected book.

use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::model::{AppState, BookEntry, Message, status_label};

use super::PREVIEW_WIDTH;
use super::format_size;
use super::theme::panel_style;

pub fn preview(state: &AppState) -> Element<'_, Message> {
    let details: Element<'_, Message> = match state
        .selected
        .and_then(|index| state.catalogue.get(index).map(|entry| (index, entry)))
    {
        Some((index, entry)) => preview_details(state, index, entry),
        None => text("Select a book to see details.").size(13).into(),
    };

    container(column![text("Details").size(16), details].spacing(8))
        .width(Length::Fixed(PREVIEW_WIDTH))
        .height(Length::Fill)
        .padding(10)
        .style(panel_style)
        .into()
}

fn preview_details<'a>(
    state: &'a AppState,
    index: usize,
    entry: &'a BookEntry,
) -> Element<'a, Message> {
    let mut details = column![
        text(&entry.title).size(16),
        text(format!("Format: {}", entry.format.label())).size(13),
        text(format!("Size: {}", format_size(entry.size_bytes))).size(13),
        text(format!("Status: {}", status_label(entry.status))).size(13),
    ]
    .spacing(6);

    if let Some(asin) = entry.asin() {
        details = details.push(text(format!("ASIN: {asin}")).size(13));
    }

    if let Some(record) = &entry.local {
        let location = record.local_path.as_deref().unwrap_or("device only");
        details = details.push(text(format!("Local: {location}")).size(11));
    }
    if let Some(book) = &entry.device {
        details = details.push(
            text(format!(
                "On device: yes · {} metadata objects",
                book.metadata_handles.len()
            ))
            .size(11),
        );
    }

    // Click equivalents of the drag-drop transfers (PLAN: "context-menu/button
    // equivalents").
    let mut actions: Vec<Element<'a, Message>> = Vec::new();
    if entry.device.is_some() {
        actions.push(
            button("Copy to Library")
                .on_press(Message::CopyToLibrary(index))
                .width(Length::Fill)
                .into(),
        );
    }
    if entry.has_local_copy() {
        actions.push(
            button("Send to Kindle")
                .on_press(Message::SendToKindle(index))
                .width(Length::Fill)
                .into(),
        );
    }
    if state
        .selected_collection
        .and_then(|collection_index| state.collections.get(collection_index))
        .is_some_and(|collection| collection.book_keys.contains(&entry.key()))
    {
        actions.push(
            button("Remove from Collection")
                .on_press(Message::CollectionRemoveBook { index })
                .width(Length::Fill)
                .into(),
        );
    }
    if !actions.is_empty() {
        details = details.push(column(actions).spacing(4).padding(4));
    }

    details.into()
}
