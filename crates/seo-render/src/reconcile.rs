//! HTTP response versus rendered DOM. Missing render stays unmeasured.

use crate::{RenderMode, RenderReport, RenderSnapshot, RenderedPage};
use weavatrix_seo_model::{
    AbsoluteUrl, Evidence, ExtractedPage, Finding, FindingFamily, Indexability, Inventory,
    LayerState, Locator, Severity,
};

/// Reconciles crawled HTTP pages with imported render observations.
#[must_use]
pub fn reconcile(inventory: &Inventory, snapshot: &RenderSnapshot) -> (RenderReport, Vec<Finding>) {
    if !snapshot.connected() {
        return (super::unmeasured(), Vec::new());
    }
    let mut evidence = snapshot.evidence();
    if !inventory.snapshot_id.is_empty() {
        evidence.snapshot_id = Some(inventory.snapshot_id.clone());
    }
    let mut states = Vec::new();
    let mut findings = Vec::new();
    for rendered in &snapshot.pages {
        match find_http(inventory, &rendered.url) {
            None => {
                states.push((rendered.url.clone(), LayerState::RenderOnly));
                findings.push(
                    Finding::new(
                        FindingFamily::Render,
                        7,
                        Severity::Info,
                        &rendered.url,
                        format!(
                            "{} was rendered but not present in this crawl",
                            rendered.url
                        ),
                        Locator::Url(rendered.url.clone()),
                        evidence.clone(),
                    )
                    .explained(
                        "WVQ measured a URL this HTTP snapshot did not crawl.",
                        "Raise the crawl budget or add the URL to the seed.",
                        "The URL is in the inventory or is intentionally out of scope.",
                    ),
                );
            }
            Some(http) => {
                let contradicted = emit_mismatches(http, rendered, &evidence, &mut findings);
                states.push((
                    rendered.url.clone(),
                    if contradicted {
                        LayerState::Contradicted
                    } else {
                        LayerState::Expected
                    },
                ));
            }
        }
    }
    (
        RenderReport {
            mode: RenderMode::Requested,
            states,
            evidence,
        },
        findings,
    )
}

fn emit_mismatches(
    http: &ExtractedPage,
    rendered: &RenderedPage,
    evidence: &Evidence,
    findings: &mut Vec<Finding>,
) -> bool {
    if http.indexability != Indexability::Indexable && http.status != 200 {
        return false;
    }
    let mut hit = false;
    hit |= mismatch(
        findings,
        evidence,
        &rendered.url,
        3,
        "title",
        http.title.as_deref(),
        rendered.title.as_deref(),
    );
    let http_canon = http.canonical.as_deref().or(Some(http.url.path()));
    hit |= !canonicals_match(http_canon, rendered.canonical.as_deref())
        && mismatch(
            findings,
            evidence,
            &rendered.url,
            4,
            "canonical",
            http_canon,
            rendered.canonical.as_deref(),
        );
    let http_h1 = http
        .headings
        .iter()
        .find(|heading| heading.level == 1)
        .map(|heading| heading.text.as_str());
    hit |= mismatch(
        findings,
        evidence,
        &rendered.url,
        5,
        "h1",
        http_h1,
        rendered.h1.as_deref(),
    );
    if !rendered.json_ld_types.is_empty() {
        let http_types: Vec<_> = http
            .json_ld
            .iter()
            .flat_map(|block| block.types.iter().cloned())
            .collect();
        for kind in &rendered.json_ld_types {
            if !http_types.iter().any(|item| item == kind) {
                hit = true;
                findings.push(
                    Finding::new(
                        FindingFamily::Render,
                        6,
                        Severity::Warn,
                        &format!("{}:{kind}", rendered.url),
                        format!(
                            "{} renders JSON-LD `{kind}` that the HTTP response did not declare",
                            rendered.url
                        ),
                        Locator::dom(&http.url, "script[type='application/ld+json']"),
                        evidence.clone(),
                    )
                    .explained(
                        "Hydration or a client script injected schema the raw response lacks.",
                        "Emit the type from the server template or stop injecting it after render.",
                        "HTTP and rendered JSON-LD types match.",
                    ),
                );
            }
        }
    }
    hit
}

fn mismatch(
    findings: &mut Vec<Finding>,
    evidence: &Evidence,
    url: &str,
    number: u16,
    field: &str,
    http: Option<&str>,
    rendered: Option<&str>,
) -> bool {
    let Some(rendered) = rendered.map(norm) else {
        return false;
    };
    let http = http.map(norm).unwrap_or_default();
    if http == rendered {
        return false;
    }
    findings.push(
        Finding::new(
            FindingFamily::Render,
            number,
            Severity::Warn,
            &format!("{url}:{field}"),
            format!("{url} HTTP {field} differs from the rendered DOM"),
            Locator::Url(url.into()),
            evidence.clone(),
        )
        .explained(
            "The raw HTTP search surface does not match what WVQ measured after render.",
            format!("Make the server {field} match the hydrated document, or stop changing it client-side."),
            format!("HTTP and rendered {field} are identical."),
        ),
    );
    true
}

fn norm(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn canonicals_match(http: Option<&str>, rendered: Option<&str>) -> bool {
    let Some(rendered) = rendered.map(norm).filter(|item| !item.is_empty()) else {
        return true;
    };
    let Some(http) = http.map(norm).filter(|item| !item.is_empty()) else {
        return false;
    };
    if http == rendered {
        return true;
    }
    path_of(&http) == path_of(&rendered)
}

fn path_of(value: &str) -> String {
    AbsoluteUrl::parse(value).map_or_else(|_| value.to_owned(), |url| url.path().to_owned())
}

fn find_http<'a>(inventory: &'a Inventory, url: &str) -> Option<&'a ExtractedPage> {
    if let Ok(parsed) = AbsoluteUrl::parse(url)
        && let Some(page) = inventory.page(&parsed)
    {
        return Some(page);
    }
    let path = AbsoluteUrl::parse(url)
        .ok()
        .map_or_else(|| url.to_owned(), |item| item.path().to_owned());
    inventory
        .pages
        .iter()
        .find(|page| page.url.path() == path || page.url.to_string() == url)
}

#[cfg(test)]
mod tests {
    use super::reconcile;
    use crate::{RenderSnapshot, RenderedPage};
    use weavatrix_seo_model::{
        AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
        MediaKind,
    };

    #[test]
    fn title_drift_is_render_warn() {
        let url = AbsoluteUrl::parse("https://x.test/").unwrap();
        let page = ExtractedPage {
            url: url.clone(),
            requested: url,
            status: 200,
            redirects: Vec::new(),
            content_type: Some("text/html".into()),
            media: MediaKind::Html,
            canonical: Some("/".into()),
            robots: Vec::new(),
            title: Some("HTTP Home".into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: vec![weavatrix_seo_model::Heading {
                level: 1,
                text: "Home".into(),
            }],
            links: Vec::new(),
            link_refs: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: "Home".into(),
            heading_text: "Home".into(),
            main_text: String::new(),
            payload: String::new(),
            arbitrary_script: String::new(),
            og_title: None,
            og_description: None,
            og_image: None,
            headers: Vec::new(),
            body_bytes: 4,
            fetch_ms: 1,
            has_main: false,
            unlabeled_controls: 0,
            content_hash: ContentHash::of_str("Home"),
            indexability: Indexability::Indexable,
            in_sitemap: true,
            linked_from_page: true,
            evidence: Evidence::http(),
        }
        .finalize();
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.pages = vec![page];
        let snapshot = RenderSnapshot {
            schema: "weavatrix-seo-render/v1".into(),
            source: "wvq".into(),
            pages: vec![RenderedPage {
                url: "https://x.test/".into(),
                title: Some("Rendered Home".into()),
                canonical: Some("/".into()),
                h1: Some("Home".into()),
                description: None,
                json_ld_types: Vec::new(),
                html_lang: Some("en".into()),
            }],
        };
        let (_, findings) = reconcile(&inventory, &snapshot);
        assert!(
            findings
                .iter()
                .any(|item| item.code == "WVX-SEO-RENDER-003"),
            "{findings:?}"
        );
    }
}
