//! Evidence-directed extra crawl seeds. Additive to sitemap and internal links.

use weavatrix_seo_history::load as load_snapshot;
use weavatrix_seo_model::{AbsoluteUrl, DiscoverySource};
use weavatrix_seo_observation::{ObservationKind, load as load_gsc, load_any};

use crate::request::AuditRequest;

/// URLs from GSC, logs, citations, and previous snapshots that share `host`.
#[must_use]
pub fn directed_seeds(request: &AuditRequest, host: &str) -> Vec<(AbsoluteUrl, DiscoverySource)> {
    let mut out = Vec::new();
    if let Some(path) = request.observations.as_deref() {
        if let Ok(snapshot) = load_any(path) {
            for row in &snapshot.rows {
                let source = match row.kind {
                    ObservationKind::SearchPerformance => DiscoverySource::Gsc,
                    ObservationKind::BotCrawl => DiscoverySource::Log,
                    ObservationKind::AiCitation => DiscoverySource::AiCitation,
                    _ => continue,
                };
                push_host(&mut out, &row.url, host, source);
            }
        }
    } else if let Some(path) = request.gsc.as_deref()
        && let Ok(snapshot) = load_gsc(path)
    {
        for row in &snapshot.rows {
            push_host(&mut out, &row.url, host, DiscoverySource::Gsc);
        }
    }
    if let Some(dir) = request.history.as_deref() {
        push_history(&mut out, dir, host);
    }
    out
}

fn push_history(out: &mut Vec<(AbsoluteUrl, DiscoverySource)>, dir: &str, host: &str) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(path) = path.to_str() else {
            continue;
        };
        let Ok(snapshot) = load_snapshot(path) else {
            continue;
        };
        for page in snapshot.pages {
            push_host(out, &page.url, host, DiscoverySource::PreviousSnapshot);
        }
    }
}

fn push_host(
    out: &mut Vec<(AbsoluteUrl, DiscoverySource)>,
    url: &str,
    host: &str,
    source: DiscoverySource,
) {
    let Ok(parsed) = AbsoluteUrl::parse(url) else {
        return;
    };
    if parsed.host() != host {
        return;
    }
    if out.iter().any(|(existing, _)| existing == &parsed) {
        return;
    }
    out.push((parsed, source));
}

#[cfg(test)]
mod tests {
    use super::directed_seeds;
    use crate::request::AuditRequest;

    #[test]
    fn missing_imports_yield_no_seeds() {
        let request = AuditRequest::site("https://x.test/");
        assert!(directed_seeds(&request, "x.test").is_empty());
    }
}
