//! Kindling GUI — iced Explorer (GUI M2).
//!
//! A thin UI shell over the `kindred` boundary. Real device inventory and
//! local-library data are wired through `data::load_all` iced tasks; the
//! mock catalogue from M1 has been removed.

mod data;
mod model;
mod update;
mod view;

fn main() -> iced::Result {
    iced::application(update::boot, update::update, view::view)
        .title("Kindling")
        .window_size(iced::Size::new(1280.0, 800.0))
        .run()
}
