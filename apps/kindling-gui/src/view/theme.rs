//! Theme/style helpers: panel chrome, active/selected states, cover colours.

use iced::widget::{button, container};
use iced::{Background, Border, Color, Theme};

/// Panel chrome shared by the sidebar and preview panes.
pub fn panel_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    container::Style::default()
        .background(palette.background)
        .border(Border::default().color(border_color(theme)).width(1))
}

pub fn status_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    container::Style::default().background(palette.background)
}

/// Highlight the currently selected section button.
pub fn active_section_style(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = theme.palette();
    button::Style {
        background: Some(Background::Color(palette.primary)),
        text_color: palette.background,
        ..button::Style::default()
    }
}

/// Outline the selected book card with the theme's primary colour.
pub fn selected_card_style(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = theme.palette();
    button::Style {
        border: Border::default().width(2).color(palette.primary),
        ..button::Style::default()
    }
}

/// Border colour derived from the theme text colour (alpha-tinted).
fn border_color(theme: &Theme) -> Color {
    Color {
        a: 0.35,
        ..theme.palette().text
    }
}

/// A stable pastel cover colour derived from the title.
pub fn cover_color(title: &str) -> Color {
    let mut hash: u32 = 2166136261;
    for byte in title.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16777619);
    }
    let hue = (hash % 360) as f32 / 360.0;
    hsl_to_rgb(hue, 0.45, 0.55)
}

fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> Color {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let x = chroma * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
    let m = lightness - chroma / 2.0;

    let (red, green, blue) = match (hue * 6.0) as u32 % 6 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };

    Color::from_rgb(red + m, green + m, blue + m)
}
