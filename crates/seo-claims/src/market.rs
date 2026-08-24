//! Market packs and cross-market entity contamination.

use weavatrix_seo_model::{
    AbsoluteUrl, Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Inventory,
    Locator, Severity,
};

/// Declared or inferred public market of a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    /// Southwest Washington / US.
    UsWa,
    /// Israel.
    Israel,
    /// Not classified.
    Unknown,
}

#[derive(Clone, Copy)]
struct Entity {
    token: &'static str,
    label: &'static str,
    market: Market,
}

const ENTITIES: &[Entity] = &[
    Entity {
        token: "IEC",
        label: "Israel Electric Corporation",
        market: Market::Israel,
    },
    Entity {
        token: "Hevrat HaHashmal",
        label: "Hevrat HaHashmal",
        market: Market::Israel,
    },
    Entity {
        token: "Gush Dan",
        label: "Gush Dan",
        market: Market::Israel,
    },
    Entity {
        token: "Shabbat",
        label: "Shabbat",
        market: Market::Israel,
    },
    Entity {
        token: "Israel Electric",
        label: "Israel Electric",
        market: Market::Israel,
    },
    Entity {
        token: "חשמלאי",
        label: "Hebrew electrician title",
        market: Market::Israel,
    },
    Entity {
        token: "מוסמך",
        label: "Hebrew licensed-trade title",
        market: Market::Israel,
    },
    Entity {
        token: "Negev",
        label: "Negev",
        market: Market::Israel,
    },
    Entity {
        token: "Ministry of Energy",
        label: "Israeli energy regulator phrasing",
        market: Market::Israel,
    },
    Entity {
        token: "Washington State L&I",
        label: "Washington L&I",
        market: Market::UsWa,
    },
    Entity {
        token: "Clark County",
        label: "Clark County",
        market: Market::UsWa,
    },
];

/// Infers the page market from host, path, language, and body.
#[must_use]
pub fn infer_market(url: &AbsoluteUrl, html_lang: Option<&str>, text: &str) -> Market {
    let host = url.host();
    if host.ends_with(".co.il") || host.rsplit('.').next() == Some("il") {
        return Market::Israel;
    }
    if html_lang.is_some_and(|lang| lang.eq_ignore_ascii_case("he")) {
        return Market::Israel;
    }
    let path = url.path();
    if path.contains("-wa") || path.contains("washington") || host.rsplit('.').next() == Some("us")
    {
        return Market::UsWa;
    }
    let lower = text.to_ascii_lowercase();
    if lower.contains("southwest washington") || lower.contains("clark county") {
        return Market::UsWa;
    }
    if lower.contains("israel") {
        return Market::Israel;
    }
    Market::Unknown
}

/// Finds entities that belong to a different market than `owned`.
#[must_use]
pub fn foreign_entities(text: &str, owned: Market) -> Vec<&'static str> {
    if owned == Market::Unknown {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for entity in ENTITIES {
        if entity.market != owned && contains_token(text, entity.token) {
            hits.push(entity.label);
        }
    }
    if owned == Market::UsWa
        && contains_hebrew(text)
        && !hits.contains(&"Hebrew licensed-trade title")
    {
        hits.push("Hebrew script");
    }
    hits.sort_unstable();
    hits.dedup();
    hits
}

/// Page-level market findings.
#[must_use]
pub fn audit_pages(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    for page in &inventory.pages {
        if page.status >= 400 {
            continue;
        }
        let text = page_haystack(page);
        let market = infer_market(&page.url, page.html_lang.as_deref(), &text);
        let hits = foreign_entities(&text, market);
        if hits.is_empty() {
            continue;
        }
        let subject = format!("{}:{}", page.url, hits.join(","));
        findings.push(
            Finding::new(
                FindingFamily::Market,
                1,
                Severity::Error,
                &subject,
                format!(
                    "{} publishes {:?} market entities on a {:?} page",
                    page.url, hits, market
                ),
                Locator::dom(&page.url, "body"),
                Evidence {
                    kind: EvidenceKind::Deterministic,
                    source: EvidenceSource::Http,
                    confidence: weavatrix_seo_model::Confidence::High,
                    snapshot_id: None,
                    revision: None,
                    policy_version: None,
                },
            )
            .explained(
                "Public copy names a regulator, locale, or geography that does not belong to the page market.",
                "Split market packs so US pages cannot import Israeli entities, or stop rendering that module on this host.",
                "The live body and source module contain only entities of the page market.",
            ),
        );
    }
    findings
}

fn page_haystack(page: &weavatrix_seo_model::ExtractedPage) -> String {
    let mut out = String::new();
    if let Some(title) = &page.title {
        out.push_str(title);
        out.push(' ');
    }
    for heading in &page.headings {
        out.push_str(&heading.text);
        out.push(' ');
    }
    out.push_str(&page.text);
    out.push(' ');
    out.push_str(&page.payload);
    for block in &page.json_ld {
        out.push(' ');
        out.push_str(&block.raw);
    }
    out
}

fn contains_hebrew(text: &str) -> bool {
    text.chars()
        .any(|ch| ('\u{0590}'..='\u{05FF}').contains(&ch))
}

fn contains_token(hay: &str, needle: &str) -> bool {
    if !needle.is_ascii() {
        return hay.contains(needle);
    }
    let hay = hay.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    let mut start = 0;
    while let Some(index) = hay[start..].find(&needle) {
        let at = start + index;
        let before = hay[..at].chars().next_back();
        let after = hay[at + needle.len()..].chars().next();
        let edge_before = before.is_none_or(|ch| !ch.is_ascii_alphanumeric());
        let edge_after = after.is_none_or(|ch| !ch.is_ascii_alphanumeric());
        if edge_before && edge_after {
            return true;
        }
        start = at + 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{Market, foreign_entities, infer_market};
    use weavatrix_seo_model::AbsoluteUrl;

    #[test]
    fn flags_iec_on_washington_url() {
        let url = AbsoluteUrl::parse("https://kablay.us/category/electrician").unwrap();
        let text = "Need approval from the electric company (IEC)? Shabbat calls and Gush Dan.";
        let market = infer_market(&url, Some("en"), "Southwest Washington electrician");
        assert_eq!(market, Market::UsWa);
        let hits = foreign_entities(text, market);
        assert!(
            hits.contains(&"Israel Electric Corporation")
                && hits.contains(&"Gush Dan")
                && hits.contains(&"Shabbat")
        );
    }
}
