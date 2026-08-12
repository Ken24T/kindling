//! Left sidebar: device identity, view-mode selection, per-pane counts.

use iced::widget::{button, column, container, space, text};
use iced::{Element, Length};

use crate::model::{AppState, Message, Pane, ViewMode};

use super::SIDEBAR_WIDTH;
use super::theme::{active_section_style, panel_style};

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    container(
        column![
            text("Kindling").size(20),
            text("Library").size(13),
            device_block(state),
            space::vertical(),
            text("View").size(13),
            view_button(state, ViewMode::Covers),
            view_button(state, ViewMode::List),
            view_button(state, ViewMode::Details),
            space::vertical(),
            text(format!("Local: {}", state.pane_books(Pane::Local).len())).size(11),
            text(format!("Kindle: {}", state.pane_books(Pane::Kindle).len())).size(11),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .padding(10)
    .style(panel_style)
    .into()
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
