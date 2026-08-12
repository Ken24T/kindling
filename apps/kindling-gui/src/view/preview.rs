//! Right preview pane: details of the selected book.

use iced::widget::{column, container, text};
use iced::{Element, Length};

use kindred::{BookFormat, BookStatus};

use crate::model::{AppState, Message};

use super::PREVIEW_WIDTH;
use super::theme::panel_style;

pub fn preview(state: &AppState) -> Element<'_, Message> {
    let details: Element<'_, Message> = match state.selected {
        Some(index) => {
            let entry = &state.catalogue[index];
            let book = &entry.book;
            column![
                text(&book.title).size(16),
                text(format!("Format: {}", format_name(book.format))).size(13),
                text(format!(
                    "Size: {:.2} MB",
                    book.size_bytes as f64 / 1_000_000.0
                ))
                .size(13),
                match &book.asin {
                    Some(asin) => text(format!("ASIN: {asin}")).size(13),
                    None => text("ASIN: -").size(13),
                },
                text(format!("Status: {}", status_name(entry.status))).size(13),
            ]
            .spacing(6)
            .into()
        }
        None => text("Select a book to see details.").size(13).into(),
    };

    container(column![text("Details").size(16), details].spacing(8))
        .width(Length::Fixed(PREVIEW_WIDTH))
        .height(Length::Fill)
        .padding(10)
        .style(panel_style)
        .into()
}

fn format_name(format: BookFormat) -> &'static str {
    match format {
        BookFormat::Kfx => "KFX",
        BookFormat::Azw => "AZW",
    }
}

fn status_name(status: BookStatus) -> &'static str {
    match status {
        BookStatus::Both => "On device + local",
        BookStatus::OnDevice => "On device",
        BookStatus::LocalOnly => "Local only",
    }
}
