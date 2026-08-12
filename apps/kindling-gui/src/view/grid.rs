//! Cover grid for one library pane (GUI M2): draggable cover cards.
//!
//! Each card is a `mouse_area` whose press starts a drag (`DragStarted`);
//! the surrounding pane handles the release/drop (see `view::book_pane`).

use iced::widget::image::{Handle, Image};
use iced::widget::{Grid, column, container, mouse_area, scrollable, text};
use iced::{Color, ContentFit, Element, Length, mouse};

use kindred::BookStatus;

use crate::model::{AppState, BookEntry, Message, Pane};

use super::theme::{card_style, cover_color};
use super::{COVER_HEIGHT, COVER_WIDTH};

pub fn book_grid<'a>(state: &'a AppState, pane: Pane, indices: &[usize]) -> Element<'a, Message> {
    if indices.is_empty() {
        return empty_pane();
    }

    let mut grid = Grid::new().fluid(COVER_WIDTH + 24.0).spacing(12);
    for &index in indices {
        grid = grid.push(cover_card(state, pane, index));
    }

    scrollable(grid)
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

fn cover_card<'a>(state: &'a AppState, pane: Pane, index: usize) -> Element<'a, Message> {
    let entry = &state.catalogue[index];
    let selected = state.selected == Some(index);
    let dragging = state.drag.is_some_and(|drag| drag.index == index);

    let card = container(
        column![
            cover(entry),
            text(&entry.title).size(11),
            status_badge(entry.status),
        ]
        .spacing(4),
    )
    .width(Length::Fixed(COVER_WIDTH + 8.0))
    .style(move |theme| card_style(theme, selected, dragging));

    mouse_area(card)
        .on_press(Message::DragStarted { pane, index })
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// The cover artwork: the user-supplied image when one exists, else a
/// pastel letter placeholder.
fn cover(entry: &BookEntry) -> Element<'static, Message> {
    let path = entry
        .local
        .as_ref()
        .and_then(|record| record.cover_path.as_deref());

    match path {
        Some(path) if std::path::Path::new(path).is_file() => Image::new(Handle::from_path(path))
            .width(Length::Fixed(COVER_WIDTH))
            .height(Length::Fixed(COVER_HEIGHT))
            .content_fit(ContentFit::Cover)
            .into(),
        _ => {
            let color = cover_color(&entry.title);
            container(
                text(first_letter(&entry.title))
                    .size(28)
                    .color(Color::WHITE),
            )
            .width(Length::Fixed(COVER_WIDTH))
            .height(Length::Fixed(COVER_HEIGHT))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| container::Style::default().background(color))
            .into()
        }
    }
}

fn status_badge(status: BookStatus) -> Element<'static, Message> {
    let (label, color) = match status {
        BookStatus::Both => ("on device + local", Color::from_rgb(0.2, 0.6, 0.35)),
        BookStatus::OnDevice => ("on device", Color::from_rgb(0.2, 0.45, 0.75)),
        BookStatus::LocalOnly => ("local only", Color::from_rgb(0.8, 0.55, 0.15)),
    };
    text(label).size(10).color(color).into()
}

fn first_letter(title: &str) -> String {
    title
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_owned())
}
