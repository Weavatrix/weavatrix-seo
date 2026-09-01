//! Compact crawl history and revision-bound `seo_diff`.

#![forbid(unsafe_code)]

mod diff;
mod snapshot;
mod store;

pub use diff::{DiffRef, SearchDiff, diff, diff_paths};
pub use snapshot::{StoredFinding, StoredPage, StoredSnapshot};
pub use store::{HistoryIndexRow, load, load_index, save};
