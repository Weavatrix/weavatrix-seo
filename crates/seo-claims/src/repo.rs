//! Repository scan for policy packs and license facts.

use crate::license::fact_is_false;
use crate::market::foreign_entities;
use crate::pack::{self, Market};
use std::fs;
use weavatrix_scan::scan_repository;
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Locator, Severity,
};

/// Per-pack source facts.
#[derive(Debug, Clone, Default)]
pub struct PackFacts {
    /// Fact field appears.
    pub license_field: bool,
    /// A false literal was found in this pack.
    pub license_false: bool,
    /// File that set the false fact, with optional line.
    pub false_at: Option<(String, Option<u32>)>,
}

/// Source contamination and pack facts from a repository.
pub struct RepoSignals {
    /// Pack id → facts.
    pub packs: Vec<(&'static str, PackFacts)>,
    /// Market findings already localized to files.
    pub findings: Vec<Finding>,
}

impl RepoSignals {
    /// False-fact locations for packs that actually assigned false.
    #[must_use]
    pub fn pack_false(&self) -> Vec<(&'static str, String, Option<u32>)> {
        self.packs
            .iter()
            .filter_map(|(id, facts)| {
                let (path, line) = facts.false_at.clone()?;
                Some((*id, path, line))
            })
            .collect()
    }
}

/// Walks the repository with weavatrix-scan and reads text files.
#[must_use]
pub fn scan(repo: &str) -> RepoSignals {
    let mut signals = RepoSignals {
        packs: pack::all()
            .iter()
            .map(|pack| (pack.id, PackFacts::default()))
            .collect(),
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
        record_facts(&mut signals, &relative, &source);
        if pack::file_belongs(&pack::US_WA, &relative, &source) {
            record_foreign(&mut signals, &relative, &source);
        }
    }
    signals
}

fn record_facts(signals: &mut RepoSignals, relative: &str, source: &str) {
    let mut owners: Vec<&str> = pack::all()
        .iter()
        .filter(|pack| pack::file_belongs(pack, relative, source))
        .map(|pack| pack.id)
        .collect();
    if owners.is_empty() {
        owners.extend(
            pack::all()
                .iter()
                .filter(|pack| !pack.facts.is_empty() && source.contains(pack.facts[0].field))
                .filter(|pack| {
                    !pack::all()
                        .iter()
                        .any(|other| other.id != pack.id && pack::file_belongs(other, relative, source))
                })
                .map(|pack| pack.id),
        );
    }
    for id in owners {
        let Some((_, facts)) = signals.packs.iter_mut().find(|(pack, _)| *pack == id) else {
            continue;
        };
        if source.contains("license_verified") {
            facts.license_field = true;
        }
        if fact_is_false(source, "license_verified") {
            facts.license_false = true;
            if facts.false_at.is_none() {
                let line = source
                    .lines()
                    .position(|row| fact_is_false(row, "license_verified"))
                    .map(|index| u32::try_from(index + 1).unwrap_or(0));
                facts.false_at = Some((relative.to_owned(), line));
            }
        }
    }
}

fn record_foreign(signals: &mut RepoSignals, relative: &str, source: &str) {
    let hits = foreign_entities(source, Market::UsWa);
    if hits.is_empty() {
        return;
    }
    let subject = format!("{relative}:{}", hits.join(","));
    signals.findings.push(
        Finding::new(
            FindingFamily::Market,
            1,
            Severity::Error,
            &subject,
            format!("{relative} mixes {hits:?} into pack {}", pack::US_WA.id),
            Locator::source_span(relative, None, None),
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
            "A US/Washington SEO module contains entities from another market pack.",
            "Split foreign city/intent packs from the Washington renderer.",
            "The owning source file no longer names the foreign entities.",
        ),
    );
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
