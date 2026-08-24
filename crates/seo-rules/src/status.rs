//! HTTP status and redirect chains.

use weavatrix_seo_model::{Finding, FindingFamily, Inventory, Locator, Severity};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in &inventory.pages {
        let url = page.url.to_string();
        if (400..500).contains(&page.status) {
            findings.push(
                Finding::new(
                    FindingFamily::Crawl,
                    1,
                    Severity::Error,
                    &url,
                    format!("{} returned {}", page.url, page.status),
                    Locator::url(&page.url),
                    page.evidence.clone(),
                )
                .explained(
                    "Search engines cannot index a client-error URL.",
                    "Restore the route or update internal links and sitemaps.",
                    "HTTP status is 200 and the URL is linked or listed as intended.",
                ),
            );
        }
        if page.status >= 500 {
            findings.push(
                Finding::new(
                    FindingFamily::Crawl,
                    2,
                    Severity::Error,
                    &url,
                    format!("{} returned {}", page.url, page.status),
                    Locator::url(&page.url),
                    page.evidence.clone(),
                )
                .explained(
                    "A server error hides the URL from discovery.",
                    "Fix the origin response for this URL.",
                    "HTTP status is in the 2xx range.",
                ),
            );
        }
        if page.redirects.len() > 1 {
            findings.push(
                Finding::new(
                    FindingFamily::Crawl,
                    3,
                    Severity::Warn,
                    &url,
                    format!(
                        "{} has a redirect chain of {}",
                        page.url,
                        page.redirects.len()
                    ),
                    Locator::url(&page.url),
                    page.evidence.clone(),
                )
                .explained(
                    "Redirect chains waste crawl budget and dilute signals.",
                    "Point the first hop at the final URL.",
                    "A single hop, or none, reaches the canonical URL.",
                ),
            );
        }
    }
}
