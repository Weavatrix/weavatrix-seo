//! Repository scan for policy packs and license facts.

use crate::license::fact_is_false;
use crate::market::foreign_entities;
use crate::pack::{self, Market};
use std::fs;
use weavatrix_scan::scan_repository;
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Locator, Severity,
};

/// One entity-bound field assignment from source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactInstance {
    /// Stable entity id, for example `specialist:123`.
    pub entity_id: String,
    /// Field name.
    pub field: String,
    /// Repository-relative path.
    pub path: String,
    /// Line of the assignment.
    pub line: Option<u32>,
    /// True when the assignment is a false/empty literal.
    pub is_false: bool,
}

/// Per-pack source facts.
#[derive(Debug, Clone, Default)]
pub struct PackFacts {
    /// Fact field appears.
    pub license_field: bool,
    /// A false literal was found in this pack.
    pub license_false: bool,
    /// File that set the false fact, with optional line.
    pub false_at: Option<(String, Option<u32>)>,
    /// Entity-instance field assignments.
    pub instances: Vec<FactInstance>,
}

/// Source contamination and pack facts from a repository.
pub struct RepoSignals {
    /// Pack id → facts.
    pub packs: Vec<(&'static str, PackFacts)>,
    /// Extra packs loaded from `.weavatrix/seo.pack.yaml`.
    pub extra_packs: Vec<crate::decl::OwnedPack>,
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
        extra_packs: crate::decl::load(repo),
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
        if relative.rsplit('.').next() == Some("md") {
            continue;
        }
        if pack::file_belongs(&pack::US_WA, &relative, &source) {
            record_foreign(
                &mut signals,
                &relative,
                &source,
                Market::UsWa,
                pack::US_WA.id,
            );
        }
        if pack::file_belongs(&pack::ISRAEL, &relative, &source) {
            record_foreign(
                &mut signals,
                &relative,
                &source,
                Market::Israel,
                pack::ISRAEL.id,
            );
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
                    !pack::all().iter().any(|other| {
                        other.id != pack.id && pack::file_belongs(other, relative, source)
                    })
                })
                .map(|pack| pack.id),
        );
    }
    for id in owners {
        let Some((_, facts)) = signals.packs.iter_mut().find(|(pack, _)| *pack == id) else {
            continue;
        };
        let Some(pack) = pack::all().iter().find(|item| item.id == id) else {
            continue;
        };
        for rule in pack.facts {
            if source.contains(rule.field) {
                facts.license_field = true;
            }
            if fact_is_false(source, rule.field) {
                facts.license_false = true;
                if facts.false_at.is_none() {
                    let line = source
                        .lines()
                        .position(|row| fact_is_false(row, rule.field))
                        .map(|index| u32::try_from(index + 1).unwrap_or(0));
                    facts.false_at = Some((relative.to_owned(), line));
                    if let Some(entity) =
                        nearby_entity_id(source, usize::try_from(line.unwrap_or(1)).unwrap_or(1))
                    {
                        facts.instances.push(FactInstance {
                            entity_id: entity,
                            field: rule.field.to_owned(),
                            path: relative.to_owned(),
                            line,
                            is_false: true,
                        });
                    }
                }
            }
        }
    }
}

fn record_foreign(
    signals: &mut RepoSignals,
    relative: &str,
    source: &str,
    owned: Market,
    pack_id: &str,
) {
    let hits = foreign_entities(source, owned);
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
            format!("{relative} mixes {hits:?} into pack {pack_id}"),
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
            "A market pack module contains entities that belong to another jurisdiction.",
            "Split city/intent packs so each renderer owns one market.",
            "The owning source file no longer names the foreign entities.",
        ),
    );
}

fn nearby_entity_id(source: &str, line: usize) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let index = line.saturating_sub(1).min(lines.len().saturating_sub(1));
    let start = index.saturating_sub(16);
    let end = (index + 16).min(lines.len());
    for row in &lines[start..end] {
        if let Some(id) = capture_id(row) {
            return Some(id);
        }
    }
    None
}

fn capture_id(row: &str) -> Option<String> {
    let lower = row.to_ascii_lowercase();
    for key in ["specialist_id", "specialistid", "entity_id", "entityid"] {
        if let Some(at) = lower.find(key) {
            return first_token(&row[at + key.len()..]);
        }
    }
    let trimmed = row.trim_start();
    if trimmed.starts_with("id:") || trimmed.starts_with("id :") || trimmed.starts_with("\"id\"") {
        return first_token(trimmed.split(':').nth(1).unwrap_or(""));
    }
    None
}

fn first_token(rest: &str) -> Option<String> {
    let token = rest
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .find(|part| !part.is_empty() && *part != "false" && *part != "true")?;
    if token.chars().all(|ch| ch.is_ascii_digit()) || token.len() >= 3 {
        Some(token.to_owned())
    } else {
        None
    }
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

#[cfg(test)]
mod instance_tests {
    use super::{capture_id, nearby_entity_id};

    #[test]
    fn captures_a_specialist_id_near_a_false_fact() {
        let source =
            "export const specialist = {\n  specialistId: 42,\n  license_verified: false,\n};\n";
        assert_eq!(capture_id("  specialistId: 42,").as_deref(), Some("42"));
        assert_eq!(nearby_entity_id(source, 3).as_deref(), Some("42"));
    }
}
