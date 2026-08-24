//! Accessibility evidence from the live document, not a crawler score.

use weavatrix_seo_model::{
    ExtractedPage, Finding, FindingFamily, ImageRef, Indexability, Inventory, Locator, Severity,
};

pub fn audit(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page
        .html_lang
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        emit(
            findings,
            page,
            Issue {
                number: 1,
                subject: page.url.to_string(),
                summary: format!("{} is missing html lang", page.url),
                path: "html[lang]",
                why: "Assistive tech and hreflang consumers use html lang.",
                action: "Set html lang on the document template.",
                verification: "html[lang] is a BCP 47 tag.",
            },
        );
    }
    let missing_alt = page.images.iter().filter(|image| needs_alt(image)).count();
    if missing_alt > 0 {
        emit(
            findings,
            page,
            Issue {
                number: 2,
                subject: page.url.to_string(),
                summary: format!(
                    "{missing_alt} content images on {} have no alt attribute",
                    page.url
                ),
                path: "img",
                why: "Content images without an alt attribute are unlabelled.",
                action: "Add alt text. Empty alt is for decorative images only.",
                verification: "Every non-decorative img has an alt attribute.",
            },
        );
    }
    if !page.has_main {
        emit(
            findings,
            page,
            Issue {
                number: 4,
                subject: page.url.to_string(),
                summary: format!("{} has no main landmark", page.url),
                path: "main",
                why: "Search and AT use main to find the primary content.",
                action: "Wrap primary content in <main>.",
                verification: "A main landmark exists.",
            },
        );
    }
}

/// Shared chrome is one origin fact, not a per-URL dump.
pub fn audit_controls(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let indexable: Vec<_> = inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.indexability == Indexability::Indexable)
        .collect();
    let unlabeled: Vec<_> = indexable
        .iter()
        .copied()
        .filter(|page| page.unlabeled_controls > 0)
        .collect();
    if unlabeled.is_empty() {
        return;
    }
    let shared = unlabeled.len() >= 3 && unlabeled.len() * 2 >= indexable.len();
    if shared {
        let sample = unlabeled[0];
        let total: usize = unlabeled.iter().map(|page| page.unlabeled_controls).sum();
        emit(
            findings,
            sample,
            Issue {
                number: 5,
                subject: sample.url.host().to_owned(),
                summary: format!(
                    "{} indexable pages share unlabelled controls ({} total)",
                    unlabeled.len(),
                    total
                ),
                path: "input,select,textarea,button",
                why: "A template control without an accessible name repeats across the search surface.",
                action: "Name the shared control in the owning component.",
                verification: "The chrome control has a label or aria-label.",
            },
        );
        return;
    }
    for page in unlabeled {
        emit(
            findings,
            page,
            Issue {
                number: 5,
                subject: page.url.to_string(),
                summary: format!(
                    "{} has {} controls without an accessible name",
                    page.url, page.unlabeled_controls
                ),
                path: "input,select,textarea,button",
                why: "Unlabelled controls fail forms and search UI.",
                action: "Associate a label, aria-label, or wrapping label.",
                verification: "Every control has an accessible name.",
            },
        );
    }
}

fn needs_alt(image: &ImageRef) -> bool {
    if image.hidden || image.src.is_empty() || image.src.starts_with("data:") {
        return false;
    }
    image.alt.is_none()
}

struct Issue<'a> {
    number: u16,
    subject: String,
    summary: String,
    path: &'a str,
    why: &'a str,
    action: &'a str,
    verification: &'a str,
}

fn emit(findings: &mut Vec<Finding>, page: &ExtractedPage, issue: Issue<'_>) {
    findings.push(
        Finding::new(
            FindingFamily::A11y,
            issue.number,
            Severity::Warn,
            &issue.subject,
            issue.summary,
            Locator::dom(&page.url, issue.path),
            page.evidence.clone(),
        )
        .explained(issue.why, issue.action, issue.verification),
    );
}
