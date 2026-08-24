//! H1 evidence.

use weavatrix_seo_model::{ExtractedPage, Finding, FindingFamily, Locator, Severity};

pub fn audit(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    let h1: Vec<_> = page
        .headings
        .iter()
        .filter(|heading| heading.level == 1)
        .collect();
    if h1.is_empty() {
        findings.push(
            Finding::new(
                FindingFamily::Content,
                1,
                Severity::Error,
                &page.url.to_string(),
                format!("{} is missing an H1", page.url),
                Locator::dom(&page.url, "h1"),
                page.evidence.clone(),
            )
            .explained(
                "Indexable pages need one clear H1.",
                "Emit a single H1 from the page template.",
                "Exactly one H1 is present.",
            ),
        );
    } else if h1.len() > 1 {
        findings.push(
            Finding::new(
                FindingFamily::Content,
                2,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} has {} H1 headings", page.url, h1.len()),
                Locator::dom(&page.url, "h1"),
                page.evidence.clone(),
            )
            .explained(
                "Multiple H1s dilute the primary heading.",
                "Keep one H1 and demote the rest.",
                "The document has a single H1.",
            ),
        );
    }
}
