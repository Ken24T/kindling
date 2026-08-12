//! Mock library data for the M1 shell (invented titles, deterministic ids).

use kindred::{Book, BookFormat, BookStatus};

use crate::model::BookEntry;

/// Build the 60-entry mock catalogue used by the M1 shell.
pub fn mock_catalogue() -> Vec<BookEntry> {
    MOCK_TITLES
        .iter()
        .enumerate()
        .map(|(index, title)| {
            // Deterministic status mix: every 3rd local-only, every 5th on-device.
            let status = if index % 3 == 0 {
                BookStatus::LocalOnly
            } else if index % 5 == 0 {
                BookStatus::OnDevice
            } else {
                BookStatus::Both
            };

            BookEntry {
                book: Book {
                    title: (*title).to_owned(),
                    asin: Some(book_asin(index)),
                    format: BookFormat::Kfx,
                    size_bytes: 1_500_000 + (index as u64) * 97_000,
                    content_handle: 0,
                    sidecar_handle: None,
                    metadata_handles: Vec::new(),
                },
                status,
            }
        })
        .collect()
}

/// Deterministic 32-char uppercase hex id, shaped like the device ids.
fn book_asin(index: usize) -> String {
    format!("{:032X}", index.wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// Invented placeholder titles for the M1 mock library (not real user books).
const MOCK_TITLES: &[&str] = &[
    "The Amber Compass",
    "Salt and Starlight",
    "The Cartographer's Daughter",
    "Beneath the Iron Sky",
    "The Last Lighthouse",
    "A Winter in Venice",
    "The Glass Orchard",
    "Embers of the North",
    "The Midnight Taxidermist",
    "Rivers of Amber",
    "The Clockmaker's Apprentice",
    "Paper Lanterns",
    "The Silent Meridian",
    "A Song for the Tide",
    "The Gilded Serpent",
    "Maps of Forgotten Places",
    "The Heretic's Almanac",
    "Cinder and Bone",
    "The Observatory at Night",
    "Woven",
    "The Bellwether",
    "A Field Guide to Ghosts",
    "The Copper Dagger",
    "Notes from the Undertow",
    "The Pale Frontier",
    "Saltwater Saints",
    "The Inventory of Shadows",
    "Kingdoms of Ash",
    "The Navigator's Vow",
    "A House of Many Doors",
    "The Winter Orchard",
    "Gravity's Garden",
    "The Antique Dealer",
    "Lanterns in the Fog",
    "The River Between Worlds",
    "A Calendar of Small Joys",
    "The Stone Scribe",
    "Harbor Lights",
    "The Exile's Atlas",
    "Moonwater",
    "The Printer's Devil",
    "Between Two Rivers",
    "Silver and Salt",
    "The Quiet Revolution",
    "A Map of the Heart",
    "The Night Ferry",
    "Orchards of the Sun",
    "The Bookseller's Daughter",
    "Ink and Ember",
    "The Far Lighthouse",
    "A Pocket History of Clouds",
    "The Weaver's War",
    "Glasshouse",
    "The Return of the Swallows",
    "Marrow",
    "The Lighthouse Keeper's Wife",
    "Storm Glass",
    "The Cartographer's Guild",
    "Ash and Amber",
];
