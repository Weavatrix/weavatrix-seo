//! Compact crawl history and revision-bound `seo_diff`.

#![forbid(unsafe_code)]

mod diff;
mod snapshot;
mod sqlite;
mod store;

pub use diff::{DiffRef, SearchDiff, diff, diff_paths};
pub use snapshot::{StoredFinding, StoredPage, StoredSnapshot};
pub use sqlite::{ingest, ingest_at, query_maps};
pub use store::{HistoryIndexRow, load, load_index, save, save_at};
