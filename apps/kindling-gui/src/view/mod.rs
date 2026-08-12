//! Explorer shell layout (GUI M1).
//!
//! Three-pane layout: left section sidebar + centre cover grid + right
//! preview pane, with a status bar across the bottom.

mod grid;
mod preview;
mod sidebar;
pub mod theme;

use iced::widget::{Column, Row, container, row, space, text};
use iced::{Element, Length};

use crate::model::{AppState, Message};

use self::grid::book_grid;
use self::preview::preview;
use self::sidebar::sidebar;
use self::theme::status_bar_style;

/// Shared layout constants.
pub const SIDEBAR_WIDTH: f32 = 210.0;
pub const PREVIEW_WIDTH: f32 = 280.0;
pub const COVER_WIDTH: f32 = 128.0;
pub const COVER_HEIGHT: f32 = 168.0;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let body = Row::new()
        .push(sidebar(state))
        .push(book_grid(state))
        .push(preview(state));

    let status_bar = container(
        row![
            text(format!(
                "{} · {} books · {} selected",
                state.section.title(),
                state.visible_books().len(),
                if state.selected.is_some() { 1 } else { 0 },
            )),
            space::horizontal(),
            text("Kindling · GUI M1 · mock data"),
        ]
        .padding(6),
    )
    .width(Length::Fill)
    .style(status_bar_style);

    Column::new().push(body).push(status_bar).into()
}
