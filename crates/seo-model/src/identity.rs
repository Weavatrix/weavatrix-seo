//! Run, snapshot, revision, and policy identities.

use crate::ContentHash;
use std::time::{SystemTime, UNIX_EPOCH};

/// Policy identifier shipped with this crate. Bump with the crate version
/// whenever finding semantics change.
pub const POLICY_VERSION: &str = "0.1.5";

/// Origin identity for one site (`scheme://host[:port]`).
#[must_use]
pub fn site_identity(origin: &str) -> String {
    origin.trim().to_owned()
}

/// Unique analysis-run identity. Distinct even when the seed URL repeats.
#[must_use]
pub fn new_run_id(seed: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    ContentHash::of_str(&format!("{seed}:{nanos}:{}", std::process::id())).hex()
}

/// Digest of a measured crawl surface. Includes the run id so two crawls of
/// the same origin never collapse into one snapshot.
#[must_use]
pub fn snapshot_digest(run_id: &str, seed: &str, measured: &str) -> String {
    ContentHash::of_str(&format!("{run_id}\n{seed}\n{POLICY_VERSION}\n{measured}")).hex()
}

/// Config digest used by the CI gate to decide comparability.
#[must_use]
pub fn config_digest(parts: &[&str]) -> String {
    ContentHash::of_str(&parts.join("\n")).hex()
}
