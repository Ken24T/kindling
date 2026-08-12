//! Kindling GUI — iced Explorer shell (GUI M1).
//!
//! A thin UI shell over the `kindred` boundary. M1 is mock-data driven; real
//! device/library wiring lands in GUI M2 (per PLAN.md).

mod mock;
mod model;
mod view;

fn main() -> iced::Result {
    iced::application(model::AppState::default, model::update, view::view)
        .title("Kindling")
        .window_size(iced::Size::new(1200.0, 800.0))
        .run()
}
