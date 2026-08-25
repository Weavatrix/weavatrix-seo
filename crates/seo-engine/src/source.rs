//! Repo/live classification findings.

use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Locator, Severity,
};
use weavatrix_seo_nextjs::route_matches;
use weavatrix_seo_source::SourceSurface;

pub fn source_findings(
    inventory: &weavatrix_seo_model::Inventory,
    surface: &SourceSurface,
) -> Vec<Finding> {
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
            if !weavatrix_seo_source::allows_family(inventory.policy.as_ref(), &family.pattern) {
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
                        Locator::source_span(family.owner.clone().unwrap_or_default(), None, None),
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
        for page in inventory.pages.iter().filter(|page| {
            page.status == 200
                && weavatrix_seo_source::allows_family(inventory.policy.as_ref(), page.url.path())
        }) {
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
                Locator::source_span(String::new(), None, None),
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

pub fn programmatic_findings(surface: &SourceSurface) -> Vec<Finding> {
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
                Locator::source_span(family.owner.clone().unwrap_or_default(), None, None),
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
