//! Audit orchestration for site, repo, hybrid, and compare modes.

use crate::plan_from;
use serde::{Deserialize, Serialize};
use weavatrix_seo_architecture::analyze as analyze_architecture;
use weavatrix_seo_claims::unmeasured as claims_unmeasured;
use weavatrix_seo_competitor::compare_inventories;
use weavatrix_seo_content::exact_duplicates;
use weavatrix_seo_crawl::{Crawl, CrawlBudget, CrawlConfig, CrawlError};
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, AuditReport, AxisScore, ContentHash, Evidence, EvidenceKind,
    EvidenceSource, Finding, FindingFamily, Inventory, InventoryCounts, Locator, SeoError,
    Severity,
};
use weavatrix_seo_nextjs::{predict, route_matches};
use weavatrix_seo_observation::unmeasured as observations_unmeasured;
use weavatrix_seo_opportunity::opportunities;
use weavatrix_seo_render::unmeasured as render_unmeasured;
use weavatrix_seo_rules::audit as rule_audit;
use weavatrix_seo_source::SourceSurface;

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
/// Returns [`EngineError`] when both site and repo are missing, or a crawl fails.
pub fn run_audit(request: &AuditRequest) -> Result<AuditReport, EngineError> {
    if request.site.is_none() && request.repo.is_none() {
        return Err(EngineError::Usage(
            "provide --site URL and/or --repo PATH".into(),
        ));
    }
    let budget = budget(request);
    let surface = request.repo.as_deref().map(predict);
    let mut inventory = if let Some(site) = request.site.as_deref() {
        crawl_site(site, &budget)?
    } else {
        empty_repo_inventory(request)
    };
    inventory.mode = request.mode;
    inventory.repo.clone_from(&request.repo);
    if let Some(surface) = &surface {
        inventory.predicted_routes = surface.patterns();
    }
    let mut competitor_inventories = Vec::new();
    if request.mode == AnalysisMode::Compare {
        for origin in &request.competitors {
            competitor_inventories.push((origin.clone(), crawl_site(origin, &budget)?));
        }
    }
    Ok(assemble(
        request,
        inventory,
        surface.as_ref(),
        &competitor_inventories,
    ))
}

fn budget(request: &AuditRequest) -> CrawlBudget {
    let mut budget = CrawlBudget::default();
    if let Some(max_pages) = request.max_pages {
        budget = budget.with_max_pages(max_pages);
    }
    budget
}

fn crawl_site(site: &str, budget: &CrawlBudget) -> Result<Inventory, EngineError> {
    let seed = AbsoluteUrl::parse(site).map_err(EngineError::Url)?;
    Crawl::new(CrawlConfig {
        seed,
        budget: budget.clone(),
    })
    .inventory()
    .map_err(EngineError::Crawl)
}

fn empty_repo_inventory(request: &AuditRequest) -> Inventory {
    Inventory {
        mode: AnalysisMode::Repo,
        snapshot_id: ContentHash::of_str(request.repo.as_deref().unwrap_or("repo")).hex(),
        site: None,
        repo: request.repo.clone(),
        hosts: Vec::new(),
        pages: Vec::new(),
        edges: Vec::new(),
        predicted_routes: Vec::new(),
        sitemap_discovered: 0,
        counts: InventoryCounts {
            crawled: 0,
            fetched: 0,
            redirected: 0,
            errors: 0,
            sitemap_urls: 0,
            indexable: 0,
        },
    }
}

fn assemble(
    request: &AuditRequest,
    inventory: Inventory,
    surface: Option<&SourceSurface>,
    competitors: &[(String, Inventory)],
) -> AuditReport {
    let mut findings = rule_audit(&inventory);
    let (architecture, architecture_findings) = analyze_architecture(&inventory);
    findings.extend(architecture_findings);
    findings.extend(exact_duplicates(&inventory));
    if let Some(surface) = &surface {
        findings.extend(source_findings(&inventory, surface));
        findings.extend(programmatic_findings(surface));
    }
    let mut items = opportunities(&inventory, &architecture);
    if request.mode == AnalysisMode::Compare {
        items.extend(compare_inventories(&inventory, competitors));
    }
    let _ = render_unmeasured();
    let _ = claims_unmeasured();
    let _ = observations_unmeasured();
    let _ = plan_from(&items);
    let axes = axes(&findings, surface.is_some());
    AuditReport {
        inventory,
        findings,
        axes,
        opportunities: items,
    }
}

fn source_findings(inventory: &Inventory, surface: &SourceSurface) -> Vec<Finding> {
    let mut findings = Vec::new();
    let evidence = Evidence {
        kind: EvidenceKind::Deterministic,
        source: EvidenceSource::Repo,
        confidence: weavatrix_seo_model::Confidence::High,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    };
    if !inventory.pages.is_empty() {
        for family in &surface.families {
            if is_private(&family.pattern) {
                continue;
            }
            let matched = inventory
                .pages
                .iter()
                .any(|page| route_matches(&family.pattern, page.url.path()));
            if !matched {
                findings.push(
                    Finding::new(
                        FindingFamily::Render,
                        1,
                        Severity::Info,
                        &family.pattern,
                        format!("route {} is SOURCE_ONLY in this crawl budget", family.pattern),
                        Locator::Source {
                            path: family.owner.clone().unwrap_or_default(),
                            start_line: None,
                        },
                        evidence.clone(),
                    )
                    .explained(
                        "The App Router defines this family, but no crawled URL matched it.",
                        "Raise the crawl budget or add internal links to a live instance of the family.",
                        "A live URL matches the pattern or the family is intentionally non-indexable.",
                    ),
                );
            }
        }
        for page in inventory
            .pages
            .iter()
            .filter(|page| page.status == 200 && !is_private(page.url.path()))
        {
            let matched = surface
                .families
                .iter()
                .any(|family| route_matches(&family.pattern, page.url.path()));
            if !matched && !surface.families.is_empty() {
                findings.push(
                    Finding::new(
                        FindingFamily::Render,
                        2,
                        Severity::Info,
                        &page.url.to_string(),
                        format!("{} is RESPONSE_ONLY against the route model", page.url),
                        Locator::url(&page.url),
                        Evidence::http(),
                    )
                    .explained(
                        "The live URL did not match a predicted App Router family.",
                        "Add the route, or stop emitting the URL.",
                        "The URL maps to a source route family.",
                    ),
                );
            }
        }
    }
    if surface.sitemaps.is_empty() {
        findings.push(
            Finding::new(
                FindingFamily::Sitemap,
                3,
                Severity::Warn,
                "sitemap.ts",
                "no App Router sitemap.ts was found",
                Locator::Source {
                    path: String::new(),
                    start_line: None,
                },
                evidence,
            )
            .explained(
                "Repo-only analysis found no sitemap generator.",
                "Add app/sitemap.ts or a sitemap index route.",
                "A sitemap module exists in the App Router tree.",
            ),
        );
    }
    findings
}

fn programmatic_findings(surface: &SourceSurface) -> Vec<Finding> {
    let mut findings = Vec::new();
    for family in surface
        .families
        .iter()
        .filter(|family| family.pattern.contains(":city") || family.pattern.contains('*'))
    {
        let severity = if family.has_static_params {
            Severity::Info
        } else {
            Severity::Warn
        };
        findings.push(
            Finding::new(
                FindingFamily::Prog,
                1,
                severity,
                &family.pattern,
                format!("programmatic family {}", family.pattern),
                Locator::Source {
                    path: family.owner.clone().unwrap_or_default(),
                    start_line: None,
                },
                Evidence {
                    kind: EvidenceKind::Deterministic,
                    source: EvidenceSource::Repo,
                    confidence: weavatrix_seo_model::Confidence::High,
                    snapshot_id: None,
                    revision: None,
                    policy_version: None,
                },
            )
            .explained(
                "A dynamic route family can explode in cardinality.",
                "Require unique facts per combination before generating the matrix.",
                "generateStaticParams and unique data are both present, or the family is noindexed.",
            ),
        );
    }
    findings
}

fn is_private(pattern: &str) -> bool {
    [
        "/admin",
        "/dashboard",
        "/auth",
        "/chats",
        "/settings",
        "/pro/",
        "/tasks/",
        "/profile",
    ]
    .iter()
    .any(|token| pattern.contains(token))
}

fn axes(findings: &[Finding], has_source: bool) -> Vec<AxisScore> {
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
            let unmeasured = (matches!(
                family,
                FindingFamily::Claim | FindingFamily::Obs | FindingFamily::Ai
            ) && subset.is_empty())
                || (family == FindingFamily::Prog && !has_source && subset.is_empty());
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
            errors: findings
                .iter()
                .filter(|item| {
                    item.family == FindingFamily::Render && item.severity == Severity::Error
                })
                .count(),
            warnings: findings
                .iter()
                .filter(|item| {
                    item.family == FindingFamily::Render && item.severity == Severity::Warn
                })
                .count(),
            infos: findings
                .iter()
                .filter(|item| {
                    item.family == FindingFamily::Render && item.severity == Severity::Info
                })
                .count(),
            unmeasured: !has_source,
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
pub fn explain<'a>(report: &'a AuditReport, id: &str) -> Option<&'a Finding> {
    report.finding(id)
}
