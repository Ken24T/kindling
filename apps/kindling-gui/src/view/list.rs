//! List and details (sortable table) rows for one library pane (GUI M2).
//!
//! List mode is compact rows; Details mode adds a sortable column header.
//! Rows are `mouse_area`s that start a drag, matching the cover cards.

use iced::widget::{Column, button, container, mouse_area, row, scrollable, text};
use iced::{Element, Length, mouse};

use crate::model::{AppState, Message, Pane, SortKey, status_label};

use super::format_size;
use super::theme::{header_button_style, row_style};

pub fn book_list<'a>(state: &'a AppState, pane: Pane, indices: &[usize]) -> Element<'a, Message> {
    if indices.is_empty() {
        return empty_pane();
    }

    let mut column = Column::new().spacing(2).padding(4);
    for &index in indices {
        column = column.push(list_row(state, pane, index));
    }

    scrollable(column)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

pub fn book_details<'a>(
    state: &'a AppState,
    pane: Pane,
    indices: &[usize],
) -> Element<'a, Message> {
    if indices.is_empty() {
        return empty_pane();
    }

    let header = row![
        sort_button(state, SortKey::Title, "Title"),
        sort_button(state, SortKey::Format, "Format"),
        sort_button(state, SortKey::Size, "Size"),
        sort_button(state, SortKey::Status, "Status"),
    ]
    .padding(4);

    let mut column = Column::new().spacing(2).padding(4).push(header);
    for &index in indices {
        column = column.push(list_row(state, pane, index));
    }

    scrollable(column)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn empty_pane<'a>() -> Element<'a, Message> {
    container(text("No books").size(13))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn sort_button<'a>(state: &'a AppState, key: SortKey, label: &'static str) -> Element<'a, Message> {
    let active = state.sort.key == key;
    let arrow = if active {
        if state.sort.ascending { " ↑" } else { " ↓" }
    } else {
        ""
    };

    let mut button = button(text(format!("{label}{arrow}")).size(12))
        .on_press(Message::SortSelected(key))
        .width(Length::Fill);

    if active {
        button = button.style(header_button_style);
    }

    button.into()
}

fn list_row<'a>(state: &'a AppState, pane: Pane, index: usize) -> Element<'a, Message> {
    let entry = &state.catalogue[index];
    let selected = state.selected == Some(index);

    let row_content = container(
        row![
            text(&entry.title).size(12).width(Length::FillPortion(3)),
            text(entry.format.label())
                .size(11)
                .width(Length::FillPortion(1)),
            text(format_size(entry.size_bytes))
                .size(11)
                .width(Length::FillPortion(1)),
            text(status_label(entry.status))
                .size(11)
                .width(Length::FillPortion(1)),
        ]
        .spacing(6)
        .padding(4),
    )
    .width(Length::Fill)
    .style(move |theme| row_style(theme, selected));

    mouse_area(row_content)
        .on_press(Message::DragStarted { pane, index })
        .interaction(mouse::Interaction::Pointer)
        .into()
}
