//! Search Evidence Graph contracts for Weavatrix SEO.
//!
//! This crate owns typed identities, evidence, findings, extracted pages, and
//! inventory snapshots. It does not crawl, parse HTML, or rank opportunities.

#![forbid(unsafe_code)]

mod ai_agent;
mod authority;
mod discovery;
mod edge;
mod error;
mod evidence;
mod finding;
mod hash;
mod identity;
mod intelligence;
mod inventory;
mod link;
mod locator;
mod media;
mod node;
mod observation;
mod page;
mod policy;
mod producer;
mod registry;
mod report;
mod schema_feature;
mod scope;
mod semantics;
mod text;
mod url;
mod url_parse;

pub use ai_agent::{AiAgentDefinition, AiAgentRole, all as ai_agents, lookup as ai_agent};
pub use authority::RuleAuthority;
pub use discovery::DiscoverySource;
pub use edge::{GraphEdge, Relation};
pub use error::{Result, SeoError};
pub use evidence::{Confidence, Evidence, EvidenceKind, EvidenceSource, LayerState};
pub use finding::{Finding, FindingFamily, Severity};
pub use hash::ContentHash;
pub use identity::{POLICY_VERSION, config_digest, new_run_id, site_identity, snapshot_digest};
pub use intelligence::{
    CandidatePage, Chunk, ContentProfile, FamilyContent, FamilyMatrix, IntentCoverage,
    NearDuplicateGroup, OutcomeMetric, SearchIntelligence, SignalLevel, SyntheticStyle, chunk_id,
    intent_id,
};
pub use inventory::{AiSurface, AnalysisMode, Inventory, InventoryCounts};
pub use link::{LinkLocation, LinkRef};
pub use locator::Locator;
pub use media::MediaKind;
pub use node::{
    FactEdge, SearchNode, SearchNodeKind, chunk_node_id, intent_node_id, route_id, symbol_id,
    url_id,
};
pub use observation::{FetchObservation, FetchOutcome};
pub use page::{
    Alternate, ExtractedPage, Heading, ImageRef, Indexability, JsonLd, JsonLdNode, RedirectHop,
};
pub use policy::{IndexabilityPolicy, InternationalPolicy, SearchPolicy, glob_match};
pub use producer::ProducerFact;
pub use registry::{RuleDefinition, all as rules, authority as rule_authority, lookup as rule};
pub use report::{AuditReport, AxisScore, Opportunity, OpportunityAxes};
pub use schema_feature::{
    FeatureStatus, Requirement, SchemaFeatureProfile, SchemaProvider, missing as schema_missing,
    profiles as schema_features, satisfied as schema_satisfied,
};
pub use scope::EvidenceScope;
pub use semantics::{
    ARTIFACT_SCHEMA_VERSION, ENGINE_VERSION, EvidenceSemantics, LEGACY_UNIQUE_SAMPLE_FLOOR,
    MAX_RISK, MIN_CONFIDENCE, policy_pack_digest, rule_semantics_digest,
};
pub use text::strip_bom;
pub use url::{AbsoluteUrl, Scheme};
