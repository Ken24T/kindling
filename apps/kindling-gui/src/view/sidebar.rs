//! Left sidebar: device identity, local collections, view modes, counts.
//!
//! Collections are the app's own local groupings; each row is also a
//! drag-drop target for adding books to that collection.

use iced::widget::{button, column, container, mouse_area, row, space, text, text_input};
use iced::{Element, Length, mouse};

use crate::model::{AppState, Message, Pane, ViewMode};

use super::SIDEBAR_WIDTH;
use super::theme::{active_section_style, panel_style};

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = vec![
        text("Kindling").size(20).into(),
        text("Library").size(13).into(),
        device_block(state),
        space::vertical().into(),
        text("Local Collections").size(13).into(),
        collection_row(state, None),
    ];
    for index in 0..state.collections.len() {
        items.push(collection_row(state, Some(index)));
    }
    items.push(new_collection_block(state));
    items.push(space::vertical().into());
    items.push(text("View").size(13).into());
    for mode in [ViewMode::Covers, ViewMode::List, ViewMode::Details] {
        items.push(view_button(state, mode));
    }
    items.push(space::vertical().into());
    let local_total = state
        .catalogue
        .iter()
        .filter(|entry| entry.has_local_copy())
        .count();
    items.push(text(format!("Local: {local_total}")).size(11).into());
    items.push(
        text(format!("Kindle: {}", state.pane_books(Pane::Kindle).len()))
            .size(11)
            .into(),
    );

    container(column(items).spacing(6))
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .padding(10)
        .style(panel_style)
        .into()
}

/// A collection row: click to view, drop a book onto it to add, delete via ✕.
/// `None` renders the "All Local Books" row that clears the selection.
fn collection_row(state: &AppState, selection: Option<usize>) -> Element<'_, Message> {
    let (label, active) = match selection {
        None => (
            "All Local Books".to_owned(),
            state.selected_collection.is_none(),
        ),
        Some(index) => {
            let name = state
                .collections
                .get(index)
                .map(|collection| collection.name.as_str())
                .unwrap_or("?");
            (
                format!("{name} ({})", state.collection_count(index)),
                state.selected_collection == Some(index),
            )
        }
    };

    match selection {
        Some(index) => {
            let renaming = state.renaming_collection == Some(index);
            let mut select = button(text(label).size(12))
                .on_press(Message::CollectionSelected(Some(index)))
                .width(Length::Fill);
            if active {
                select = select.style(active_section_style);
            }

            let main: Element<'_, Message> = if renaming {
                row![
                    text_input(&state.rename_input, "Name")
                        .on_input(Message::CollectionRenameNameChanged)
                        .on_submit(Message::CollectionRenameSave),
                    button(text("✓").size(11)).on_press(Message::CollectionRenameSave),
                ]
                .spacing(2)
                .into()
            } else {
                select.into()
            };

            // Collections are drag-drop targets, so the whole row is a
            // mouse_area; the ✎/✕ buttons only act on a direct press.
            mouse_area(
                row![
                    main,
                    if renaming {
                        button(text("Cancel").size(10)).on_press(Message::CollectionRenameCancel)
                    } else {
                        button(text("✎").size(11)).on_press(Message::CollectionRenameStart(index))
                    },
                    button(text("✕").size(11)).on_press(Message::CollectionDelete(index)),
                ]
                .spacing(2),
            )
            .on_release(Message::DropOnCollection(index))
            .interaction(mouse::Interaction::Pointer)
            .into()
        }
        None => {
            let mut button = button(text(label).size(12))
                .on_press(Message::CollectionSelected(None))
                .width(Length::Fill);
            if active {
                button = button.style(active_section_style);
            }
            button.into()
        }
    }
}

/// "+ New" toggle with an inline name input and Create button.
fn new_collection_block(state: &AppState) -> Element<'_, Message> {
    let mut block = column![
        button(if state.show_new_collection {
            "− New"
        } else {
            "+ New"
        })
        .on_press(Message::ShowNewCollection)
        .width(Length::Fill),
    ];

    if state.show_new_collection {
        block = block
            .push(
                text_input(&state.new_collection_name, "Collection name")
                    .on_input(Message::CollectionNameChanged)
                    .on_submit(Message::CollectionCreate),
            )
            .push(
                button("Create")
                    .on_press(Message::CollectionCreate)
                    .width(Length::Fill),
            );
    }

    block.into()
}

fn device_block(state: &AppState) -> Element<'_, Message> {
    match &state.device {
        Some(device) => column![
            text(&device.friendly_name).size(13),
            text(&device.model).size(11),
        ]
        .spacing(2)
        .into(),
        None => text("No Kindle attached").size(12).into(),
    }
}

fn view_button(state: &AppState, mode: ViewMode) -> Element<'_, Message> {
    let active = state.view_mode == mode;

    let mut button = button(text(mode.title()))
        .on_press(Message::ViewModeSelected(mode))
        .width(Length::Fill);

    if active {
        button = button.style(active_section_style);
    }

    button.into()
}
