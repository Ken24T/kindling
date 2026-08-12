//! Centre cover grid: cards for the visible books of the current section.

use iced::widget::{Grid, button, column, container, scrollable, text};
use iced::{Color, Element, Length};

use kindred::BookStatus;

use crate::model::{AppState, BookEntry, Message};

use super::theme::{cover_color, selected_card_style};
use super::{COVER_HEIGHT, COVER_WIDTH};

pub fn book_grid(state: &AppState) -> Element<'_, Message> {
    let mut grid = Grid::new().fluid(COVER_WIDTH + 24.0).spacing(14);
    for index in state.visible_books() {
        let entry = &state.catalogue[index];
        grid = grid.push(cover_card(state, index, entry));
    }

    scrollable(grid)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn cover_card<'a>(state: &AppState, index: usize, entry: &'a BookEntry) -> Element<'a, Message> {
    let selected = state.selected == Some(index);

    let cover = container(
        text(first_letter(&entry.book.title))
            .size(30)
            .color(Color::WHITE),
    )
    .width(Length::Fixed(COVER_WIDTH))
    .height(Length::Fixed(COVER_HEIGHT))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_| container::Style::default().background(cover_color(&entry.book.title)));

    let mut card = button(
        column![
            cover,
            text(&entry.book.title).size(12),
            status_badge(entry.status),
        ]
        .spacing(4),
    )
    .on_press(Message::BookSelected(index))
    .width(Length::Fixed(COVER_WIDTH + 8.0));

    if selected {
        card = card.style(selected_card_style);
    }

    card.into()
}

fn status_badge(status: BookStatus) -> Element<'static, Message> {
    let (label, color) = match status {
        BookStatus::Both => ("on device + local", Color::from_rgb(0.2, 0.6, 0.35)),
        BookStatus::OnDevice => ("on device", Color::from_rgb(0.2, 0.45, 0.75)),
        BookStatus::LocalOnly => ("local only", Color::from_rgb(0.8, 0.55, 0.15)),
    };
    text(label).size(11).color(color).into()
}

fn first_letter(title: &str) -> String {
    title
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_owned())
}
