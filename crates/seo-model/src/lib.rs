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
mod inventory;
mod locator;
mod page;
mod report;
mod url;

pub use edge::{GraphEdge, Relation};
pub use error::{Result, SeoError};
pub use evidence::{Confidence, Evidence, EvidenceKind, EvidenceSource, LayerState};
pub use finding::{Finding, FindingFamily, Severity};
pub use hash::ContentHash;
pub use inventory::{AnalysisMode, Inventory, InventoryCounts};
pub use locator::Locator;
pub use page::{Alternate, ExtractedPage, Heading, ImageRef, Indexability, JsonLd, RedirectHop};
pub use report::{AuditReport, AxisScore, Opportunity};
pub use url::{AbsoluteUrl, Scheme};
