//! Left section sidebar: app title, library sections, footer.

use iced::widget::{button, column, container, row, space, text};
use iced::{Element, Length};

use kindred::BookStatus;

use crate::model::{AppState, Message, Section};

use super::SIDEBAR_WIDTH;
use super::theme::{active_section_style, panel_style};

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    container(
        column![
            text("Kindling").size(20),
            text("Library").size(13),
            section_button(state, Section::LocalLibrary),
            section_button(state, Section::KindleLibrary),
            space::vertical(),
            text("GUI M1").size(11),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .padding(10)
    .style(panel_style)
    .into()
}

fn section_button(state: &AppState, section: Section) -> Element<'_, Message> {
    let count = state
        .catalogue
        .iter()
        .filter(|entry| match section {
            Section::LocalLibrary => entry.status != BookStatus::OnDevice,
            Section::KindleLibrary => entry.status != BookStatus::LocalOnly,
        })
        .count();

    let label = row![
        text(section.title()),
        space::horizontal(),
        text(count.to_string()),
    ]
    .padding(6)
    .width(Length::Fill);

    let active = state.section == section;
    let mut button = button(label).on_press(Message::SectionSelected(section));
    if active {
        button = button.style(active_section_style);
    }

    button.into()
}
