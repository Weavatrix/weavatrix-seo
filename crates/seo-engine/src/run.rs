//! Site-only audit orchestration.

use crate::plan_from;
use serde::{Deserialize, Serialize};
use weavatrix_seo_architecture::analyze as analyze_architecture;
use weavatrix_seo_claims::unmeasured as claims_unmeasured;
use weavatrix_seo_competitor::{CompareRequest, compare};
use weavatrix_seo_content::exact_duplicates;
use weavatrix_seo_crawl::{Crawl, CrawlBudget, CrawlConfig, CrawlError};
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, AuditReport, AxisScore, FindingFamily, Inventory, SeoError, Severity,
};
use weavatrix_seo_observation::unmeasured as observations_unmeasured;
use weavatrix_seo_opportunity::opportunities;
use weavatrix_seo_render::unmeasured as render_unmeasured;
use weavatrix_seo_rules::audit as rule_audit;
use weavatrix_seo_source::unmeasured as source_unmeasured;

/// Invocation for one engine run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRequest {
    /// Analysis mode.
    pub mode: AnalysisMode,
    /// Site origin or seed URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Repository path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Public competitor origins.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub competitors: Vec<String>,
    /// Page cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<usize>,
}

impl AuditRequest {
    /// Site-only request.
    #[must_use]
    pub fn site(url: impl Into<String>) -> Self {
        Self {
            mode: AnalysisMode::Site,
            site: Some(url.into()),
            repo: None,
            competitors: Vec::new(),
            max_pages: None,
        }
    }
}

/// Engine-level error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    /// Missing required input.
    Usage(String),
    /// URL parse error.
    Url(SeoError),
    /// Crawl error.
    Crawl(CrawlError),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) | Self::Crawl(CrawlError::Transport(message)) => {
                write!(formatter, "{message}")
            }
            Self::Url(error) => write!(formatter, "{error}"),
            Self::Crawl(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for EngineError {}

/// Runs inventory + audit + opportunities for a request.
///
/// # Errors
///
/// Returns [`EngineError`] when the site URL is missing or the crawl fails.
pub fn run_audit(request: &AuditRequest) -> Result<AuditReport, EngineError> {
    let Some(site) = request.site.as_deref() else {
        return Err(EngineError::Usage(
            "site-only mode requires --site URL".into(),
        ));
    };
    let seed = AbsoluteUrl::parse(site).map_err(EngineError::Url)?;
    let mut budget = CrawlBudget::default();
    if let Some(max_pages) = request.max_pages {
        budget = budget.with_max_pages(max_pages);
    }
    let inventory = Crawl::new(CrawlConfig { seed, budget })
        .inventory()
        .map_err(EngineError::Crawl)?;
    Ok(assemble(request, inventory))
}

fn assemble(request: &AuditRequest, inventory: Inventory) -> AuditReport {
    let mut findings = rule_audit(&inventory);
    let (architecture, architecture_findings) = analyze_architecture(&inventory);
    findings.extend(architecture_findings);
    findings.extend(exact_duplicates(&inventory));
    let mut items = opportunities(&inventory, &architecture);
    if request.mode == AnalysisMode::Compare {
        items.extend(compare(&CompareRequest {
            site: request.site.clone().unwrap_or_default(),
            competitors: request.competitors.clone(),
        }));
    }
    if request.repo.is_some() {
        let _ = source_unmeasured(request.repo.as_deref().unwrap_or("."));
        let _ = weavatrix_seo_nextjs::predict(request.repo.as_deref().unwrap_or("."));
    }
    let _ = render_unmeasured();
    let _ = claims_unmeasured();
    let _ = observations_unmeasured();
    let _ = plan_from(&items);
    let axes = axes(&findings);
    AuditReport {
        inventory,
        findings,
        axes,
        opportunities: items,
    }
}

fn axes(findings: &[weavatrix_seo_model::Finding]) -> Vec<AxisScore> {
    let named = [
        ("technical_discoverability", FindingFamily::Crawl),
        ("indexability", FindingFamily::Idx),
        ("canonical_integrity", FindingFamily::Canon),
        ("architecture", FindingFamily::Link),
        ("content_coverage", FindingFamily::Content),
        ("claim_integrity", FindingFamily::Claim),
        ("international", FindingFamily::I18n),
        ("programmatic_safety", FindingFamily::Prog),
        ("observed_search", FindingFamily::Obs),
        ("ai_search", FindingFamily::Ai),
    ];
    named
        .into_iter()
        .map(|(axis, family)| {
            let subset: Vec<_> = findings
                .iter()
                .filter(|item| item.family == family)
                .collect();
            let unmeasured = matches!(
                family,
                FindingFamily::Claim | FindingFamily::Obs | FindingFamily::Ai | FindingFamily::Prog
            ) && subset.is_empty();
            AxisScore {
                axis: axis.into(),
                errors: subset
                    .iter()
                    .filter(|item| item.severity == Severity::Error)
                    .count(),
                warnings: subset
                    .iter()
                    .filter(|item| item.severity == Severity::Warn)
                    .count(),
                infos: subset
                    .iter()
                    .filter(|item| item.severity == Severity::Info)
                    .count(),
                unmeasured,
            }
        })
        .chain([AxisScore {
            axis: "render_reconciliation".into(),
            errors: 0,
            warnings: 0,
            infos: 0,
            unmeasured: true,
        }])
        .collect()
}

/// Inventory-only convenience.
///
/// # Errors
///
/// Propagates [`run_audit`].
pub fn run_inventory(request: &AuditRequest) -> Result<Inventory, EngineError> {
    Ok(run_audit(request)?.inventory)
}

/// Explains one finding.
#[must_use]
pub fn explain<'a>(report: &'a AuditReport, id: &str) -> Option<&'a weavatrix_seo_model::Finding> {
    report.finding(id)
}
