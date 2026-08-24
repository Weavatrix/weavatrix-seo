//! Repository scan for market packs and license facts.

use crate::license::false_facts;
use crate::market::{Market, foreign_entities};
use std::fs;
use weavatrix_scan::scan_repository;
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Locator, Severity,
};

/// Source contamination and license facts from a repository.
pub struct RepoSignals {
    /// `license_verified` appears in source.
    pub license_field: bool,
    /// A false literal was found.
    pub license_false: bool,
    /// Market findings already localized to files.
    pub findings: Vec<Finding>,
}

/// Walks the repository with weavatrix-scan and reads text files.
#[must_use]
pub fn scan(repo: &str) -> RepoSignals {
    let mut signals = RepoSignals {
        license_field: false,
        license_false: false,
        findings: Vec::new(),
    };
    let Ok(report) = scan_repository(repo) else {
        return signals;
    };
    for file in &report.files {
        let relative = file.relative.replace('\\', "/");
        if !is_source(&relative) || is_test(&relative) {
            continue;
        }
        let Ok(source) = fs::read_to_string(&file.absolute) else {
            continue;
        };
        if source.contains("license_verified") {
            signals.license_field = true;
        }
        if false_facts(&source) {
            signals.license_false = true;
        }
        let owned = if relative.contains("washington")
            || relative.contains("us-wa")
            || source.to_ascii_lowercase().contains("southwest washington")
        {
            Market::UsWa
        } else {
            continue;
        };
        let hits = foreign_entities(&source, owned);
        if hits.is_empty() {
            continue;
        }
        let subject = format!("{relative}:{}", hits.join(","));
        signals.findings.push(
            Finding::new(
                FindingFamily::Market,
                1,
                Severity::Error,
                &subject,
                format!("{relative} mixes {hits:?} into a Washington market pack"),
                Locator::Source {
                    path: relative,
                    start_line: None,
                },
                Evidence {
                    kind: EvidenceKind::Deterministic,
                    source: EvidenceSource::Repo,
                    confidence: weavatrix_seo_model::Confidence::Exact,
                    snapshot_id: None,
                    revision: None,
                    policy_version: None,
                },
            )
            .explained(
                "A US/Washington SEO module contains Israeli market entities.",
                "Split Israeli city/intent packs from the Washington renderer.",
                "The owning source file no longer names the foreign entities.",
            ),
        );
    }
    signals
}

fn is_test(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("/__tests__/") || lower.contains(".test.") || lower.contains(".spec.")
}

fn is_source(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.contains("/node_modules/") || lower.contains("/.git/") {
        return false;
    }
    matches!(
        std::path::Path::new(&lower)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(""),
        "ts" | "tsx" | "js" | "jsx" | "json" | "md"
    )
}
