//! Additive search-intelligence artifacts: profiles, chunks, outcomes, matrices.
//!
//! These sit beside findings. They never collapse into one SEO score.

use crate::EvidenceSemantics;
use serde::{Deserialize, Serialize};

/// Qualitative band. Prefer this over a fake percentage of "AI writing".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalLevel {
    /// Below the diagnostic band.
    Low,
    /// In the middle of the diagnostic band.
    Medium,
    /// Above the diagnostic band.
    High,
    /// The axis was not measured.
    Unmeasured,
}

impl Default for SignalLevel {
    fn default() -> Self {
        Self::Unmeasured
    }
}

/// One measured outcome. Not a finding; not an error/warn/info count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeMetric {
    /// Metric name, for example `citation_rate`.
    pub name: String,
    /// Value when measured. `None` is unmeasured, never zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Numerator when the metric is a rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numerator: Option<u64>,
    /// Denominator when the metric is a rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denominator: Option<u64>,
    /// Observation window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
    /// Provider or source label.
    pub source: String,
    /// Confidence label: exact, high, medium, low, unmeasured.
    pub confidence: String,
}

impl OutcomeMetric {
    /// Explicitly unmeasured outcome.
    #[must_use]
    pub fn unmeasured(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            numerator: None,
            denominator: None,
            window: None,
            source: source.into(),
            confidence: "unmeasured".into(),
        }
    }
}

/// Per-page content profile. Every metric is optional; missing stays missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentProfile {
    /// Page URL.
    pub url: String,
    /// Lexical diversity (MATTR 0–100) when text exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mattr: Option<u16>,
    /// MTLD-style diversity when text exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtld: Option<u16>,
    /// Term entropy 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub term_entropy: Option<u16>,
    /// Repeated-token share 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repetition: Option<u16>,
    /// Entity-like token density 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_density: Option<u16>,
    /// Numeric token density 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_density: Option<u16>,
    /// Fact-bearing token density 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact_density: Option<u16>,
    /// Generic/function-word share 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genericity: Option<u16>,
    /// Specific (rare/numeric/long) token share 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specificity: Option<u16>,
    /// Sentence redundancy band.
    #[serde(default)]
    pub sentence_redundancy: SignalLevel,
    /// Topic cohesion band.
    #[serde(default)]
    pub topic_cohesion: SignalLevel,
    /// Function-word ratio 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_word_ratio: Option<u16>,
    /// Filler-phrase ratio 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filler_phrase_ratio: Option<u16>,
    /// Average sentence length in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_sentence_length: Option<u16>,
    /// Share of sentences longer than 30 words, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_sentence_share: Option<u16>,
    /// Synthetic-style signals. Authorship stays unmeasured.
    #[serde(default)]
    pub synthetic: SyntheticStyle,
    /// Witness span (first distinctive phrase) when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
}

/// Diagnostic synthetic-style signals. Never "84% written by AI".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticStyle {
    /// Semantic redundancy band.
    #[serde(default)]
    pub semantic_redundancy: SignalLevel,
    /// Sentence-length variance band.
    #[serde(default)]
    pub sentence_variance: SignalLevel,
    /// Genericity band.
    #[serde(default)]
    pub genericity: SignalLevel,
    /// Factual specificity band.
    #[serde(default)]
    pub factual_specificity: SignalLevel,
    /// Template reuse band.
    #[serde(default)]
    pub template_reuse: SignalLevel,
    /// Authorship attribution. Always `UNMEASURED` in this engine.
    pub authorship: String,
}

impl Default for SyntheticStyle {
    fn default() -> Self {
        Self {
            semantic_redundancy: SignalLevel::Unmeasured,
            sentence_variance: SignalLevel::Unmeasured,
            genericity: SignalLevel::Unmeasured,
            factual_specificity: SignalLevel::Unmeasured,
            template_reuse: SignalLevel::Unmeasured,
            authorship: "UNMEASURED".into(),
        }
    }
}

/// Family-level content decomposition for programmatic SEO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyContent {
    /// Family identity, for example `category/electrician`.
    pub family: String,
    /// URLs measured in this family.
    pub measured_urls: u64,
    /// Shared template copy, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_shared_ratio: Option<u16>,
    /// Parameter (city/service) substitution, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_substitution_ratio: Option<u16>,
    /// Unique factual content, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_fact_ratio: Option<u16>,
    /// Other unique content, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_semantic_ratio: Option<u16>,
    /// Local fact coverage, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_fact_coverage: Option<u16>,
    /// Schema fact coverage, 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_fact_coverage: Option<u16>,
    /// Primary producer when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_producer: Option<String>,
}

/// Enriched programmatic matrix row. Verdict labels stay the existing enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyMatrix {
    /// Family identity.
    pub family: String,
    /// Measured URLs.
    pub measured_urls: u64,
    /// `SAFE_TO_GENERATE` and friends.
    pub verdict: String,
    /// Dimensions detected in the pattern.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dimensions: Vec<String>,
    /// Estimated cardinality when generators were read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cardinality: Option<u64>,
    /// Fact coverage 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact_coverage: Option<u16>,
    /// Unique-fact ratio 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_fact_ratio: Option<u16>,
    /// Template boilerplate ratio 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_boilerplate_ratio: Option<u16>,
    /// Semantic distinctness 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_distinctness: Option<u16>,
    /// Requirements still unmet for a safe generate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmet_requirements: Vec<String>,
}

/// First-class chunk of a page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk id `chunk:{url}#{n}`.
    pub id: String,
    /// Parent URL.
    pub url: String,
    /// Heading or leading sentence.
    pub heading: String,
    /// Chunk text.
    pub text: String,
    /// Topical cohesion 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cohesion: Option<u16>,
    /// Self-contained meaning 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_contained: Option<u16>,
    /// Answer density 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_density: Option<u16>,
    /// Specificity 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specificity: Option<u16>,
    /// Citation suitability 0–100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation_suitability: Option<u16>,
    /// Witness span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
}

/// Intent / question fanout coverage for one topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentCoverage {
    /// Intent label.
    pub intent: String,
    /// Fan-out questions.
    pub questions: Vec<String>,
    /// Questions answered by a chunk.
    pub answered: Vec<String>,
    /// Questions still missing.
    pub missing: Vec<String>,
    /// Answered/total as `3/8`. Unmeasured when total is 0.
    pub coverage: String,
}

/// Near-duplicate cluster that is not byte-identical.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearDuplicateGroup {
    /// URLs in the cluster.
    pub urls: Vec<String>,
    /// Approximate Jaccard of `MinHash` signatures, 0–100.
    pub similarity: u16,
    /// Matching witness shingle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
    /// Shared overlapping shingles that explain the cluster.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<String>,
}

/// Candidate page for retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidatePage {
    /// URL.
    pub url: String,
    /// Lexical relevance 0–100.
    pub lexical: u16,
    /// Semantic relevance when an embedding exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic: Option<u16>,
    /// Overlapping entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<String>,
    /// Page language when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Impressions when an observation was imported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impressions: Option<u32>,
    /// Why this page was selected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub why: Vec<String>,
}

/// Additive intelligence bundle attached to an audit report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchIntelligence {
    /// Evidence semantics identity for this run.
    pub semantics: EvidenceSemantics,
    /// Per-page content profiles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<ContentProfile>,
    /// Family decompositions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub families: Vec<FamilyContent>,
    /// Programmatic matrices.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matrices: Vec<FamilyMatrix>,
    /// Chunks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chunks: Vec<Chunk>,
    /// Intent coverage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<IntentCoverage>,
    /// Outcome metrics (citation rate, mention rate, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<OutcomeMetric>,
    /// Near-duplicate groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub near_duplicates: Vec<NearDuplicateGroup>,
}

/// Chunk node id.
#[must_use]
pub fn chunk_id(url: &str, index: usize) -> String {
    format!("chunk:{url}#{index}")
}

/// Intent node id.
#[must_use]
pub fn intent_id(label: &str) -> String {
    format!("intent:{label}")
}
