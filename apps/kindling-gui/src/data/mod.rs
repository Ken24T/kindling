//! Real data wiring for the GUI (GUI M2).
//!
//! Small cohesive modules: `paths` (local data locations), `load` (refresh),
//! `transfer` (pane-to-pane moves), `merge` (inventory + library → UI entries).

mod load;
mod merge;
mod paths;
mod transfer;

pub use load::{LoadResult, load_all};
pub use merge::build_catalogue;
pub use transfer::{TransferOutcome, add_to_kindle, copy_from_kindle};
