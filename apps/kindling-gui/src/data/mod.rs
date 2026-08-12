//! Real data wiring for the GUI (GUI M2).
//!
//! Small cohesive modules: `paths` (local data locations), `load` (refresh),
//! `transfer` (pane-to-pane moves), `merge` (inventory + library → UI
//! entries), `collections` (local collection CRUD).

mod collections;
mod load;
mod merge;
mod paths;
mod transfer;

pub use collections::{
    add_book_to_collection, create_collection, delete_collection, remove_book_from_collection,
};
pub use load::{LoadResult, load_all};
pub use merge::build_catalogue;
pub use transfer::{TransferOutcome, add_to_kindle, copy_from_kindle};
