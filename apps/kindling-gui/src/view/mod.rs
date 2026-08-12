//! Explorer layout (GUI M2): sidebar + Local pane + Kindle pane + preview.
//!
//! Both library panes are always visible side-by-side so a book can be
//! dragged from one to the other (device ↔ library).

mod grid;
mod list;
mod preview;
mod sidebar;
pub mod theme;

use iced::widget::{Column, Row, button, column, container, mouse_area, row, space, text};
use iced::{Element, Length};

use crate::model::{AppState, Message, Pane, ViewMode};

use self::grid::book_grid;
use self::list::{book_details, book_list};
use self::preview::preview;
use self::sidebar::sidebar;
use self::theme::{pane_style, status_bar_style};

/// Shared layout constants.
pub const SIDEBAR_WIDTH: f32 = 200.0;
pub const PREVIEW_WIDTH: f32 = 280.0;
pub const COVER_WIDTH: f32 = 118.0;
pub const COVER_HEIGHT: f32 = 156.0;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let body = Row::new()
        .push(sidebar(state))
        .push(book_pane(state, Pane::Local))
        .push(book_pane(state, Pane::Kindle))
        .push(preview(state));

    let status_bar = container(
        row![
            text(status_text(state)),
            space::horizontal(),
            button("Refresh").on_press(Message::Refresh),
        ]
        .padding(6),
    )
    .width(Length::Fill)
    .style(status_bar_style);

    // The outer mouse-area cancels a drag that ends outside any book pane.
    mouse_area(Column::new().push(body).push(status_bar))
        .on_release(Message::DragCancelled)
        .into()
}

fn status_text(state: &AppState) -> String {
    if let Some(message) = &state.status_message {
        return message.clone();
    }
    if state.loading {
        return "Loading…".to_owned();
    }
    match &state.device {
        Some(device) => format!("{} · {} books", device.friendly_name, state.catalogue.len()),
        None => "No Kindle attached · local library only".to_owned(),
    }
}

/// One draggable/droppable library pane.
fn book_pane(state: &AppState, pane: Pane) -> Element<'_, Message> {
    let indices = state.pane_books(pane);
    let header = row![
        text(pane.title()).size(15),
        space::horizontal(),
        text(indices.len().to_string()).size(12),
    ]
    .padding(6);

    let content: Element<'_, Message> = match state.view_mode {
        ViewMode::Covers => book_grid(state, pane, &indices),
        ViewMode::List => book_list(state, pane, &indices),
        ViewMode::Details => book_details(state, pane, &indices),
    };

    let highlight = state.drag.is_some() && state.drop_target == Some(pane);

    mouse_area(
        container(column![header, content])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |theme| pane_style(theme, highlight)),
    )
    .on_enter(Message::DragOver(pane))
    .on_exit(Message::DragExited)
    .on_release(Message::DropOn(pane))
    .into()
}

/// Human-readable byte size.
pub fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.0} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}
