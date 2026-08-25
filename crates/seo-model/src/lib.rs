//! Search Evidence Graph contracts for Weavatrix SEO.
//!
//! This crate owns typed identities, evidence, findings, extracted pages, and
//! inventory snapshots. It does not crawl, parse HTML, or rank opportunities.

#![forbid(unsafe_code)]

mod edge;
mod error;
mod evidence;
mod finding;
mod hash;
mod identity;
mod inventory;
mod link;
mod locator;
mod media;
mod observation;
mod page;
mod report;
mod url;
mod url_parse;

pub use edge::{GraphEdge, Relation};
pub use error::{Result, SeoError};
pub use evidence::{Confidence, Evidence, EvidenceKind, EvidenceSource, LayerState};
pub use finding::{Finding, FindingFamily, Severity};
pub use hash::ContentHash;
pub use identity::{POLICY_VERSION, config_digest, new_run_id, site_identity, snapshot_digest};
pub use inventory::{AnalysisMode, Inventory, InventoryCounts};
pub use link::{LinkLocation, LinkRef};
pub use locator::Locator;
pub use media::MediaKind;
pub use observation::{FetchObservation, FetchOutcome};
pub use page::{Alternate, ExtractedPage, Heading, ImageRef, Indexability, JsonLd, RedirectHop};
pub use report::{AuditReport, AxisScore, Opportunity};
pub use url::{AbsoluteUrl, Scheme};
