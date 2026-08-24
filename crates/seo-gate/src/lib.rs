//! Evidence CI: fail on error findings, regress against a fingerprint baseline.

#![forbid(unsafe_code)]

mod baseline;
mod verdict;

pub use baseline::load_fingerprints;
pub use verdict::{GateVerdict, evaluate};
