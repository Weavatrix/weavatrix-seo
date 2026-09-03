//! Architecture compiler: opportunities become machine-checkable actions.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{AuditReport, Opportunity, OpportunityAxes};

/// Plan verb. Weavatrix SEO stays read-only; mutations belong elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlanKind {
    /// Create a missing URL or family.
    Create,
    /// Improve an existing URL.
    Improve,
    /// Merge overlapping URLs.
    Consolidate,
    /// Add an internal link.
    Link,
    /// Keep for users, drop from the index.
    Noindex,
    /// Remove a URL from the intended search surface.
    Delete,
}

impl std::fmt::Display for PlanKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Create => "CREATE",
            Self::Improve => "IMPROVE",
            Self::Consolidate => "CONSOLIDATE",
            Self::Link => "LINK",
            Self::Noindex => "NOINDEX",
            Self::Delete => "DELETE",
        })
    }
}

/// One construction action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanAction {
    /// Verb.
    pub kind: PlanKind,
    /// Subject URL or family.
    pub subject: String,
    /// Why this action exists.
    pub why: String,
    /// Finding fingerprints or opportunity ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    /// Other subjects that should happen first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Source implementation location when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<String>,
    /// Domain facts required before shipping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_facts: Vec<String>,
    /// Internal-link placements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_placements: Vec<String>,
    /// Schema types that must exist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_requirements: Vec<String>,
    /// Programmatic safety verdict when this is a matrix family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programmatic_verdict: Option<String>,
    /// Acceptance condition.
    pub acceptance: String,
    /// How to verify.
    pub verification: String,
    /// Priority axes copied from the opportunity.
    #[serde(default)]
    pub axes: OpportunityAxes,
}

/// One proposed source edit. Weavatrix SEO never applies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffTarget {
    /// Repository-relative path.
    pub path: String,
    /// Symbol when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Start line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    /// End line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// Plan verb.
    pub intent: String,
    /// URL or family this edit is for.
    pub subject: String,
    /// Facts that must be true after the edit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_facts: Vec<String>,
    /// Acceptance copied from the plan action.
    pub acceptance: String,
    /// Symbol extent hash when the producer recorded it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_hash: Option<String>,
    /// File content hash when the producer recorded it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

/// Read-only handoff toward Weavatrix Refactor.
///
/// SEO proves and plans. Refactor proposes a mutation after explicit approval.
/// Quality + SEO verify afterwards. This struct is the contract between them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefactorHandoff {
    /// Always `weavatrix-seo`.
    pub from: String,
    /// Always `weavatrix-refactor`.
    pub to: String,
    /// SEO does not write source.
    pub read_only: bool,
    /// Proposed source targets, spans included when producers recorded them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<HandoffTarget>,
    /// Worktree revision the plan was compiled against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_revision: Option<String>,
    /// Snapshot identity of the SEO run.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub snapshot_id: String,
}

/// One executable step in the plan DAG. Additive to [`PlanAction`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Stable step id `plan:{verb}:{subject-hash}:{stage}`.
    pub id: String,
    /// Stage verb: `ADD_FACT`, `CRAWL`, `OBSERVE_GSC`, …
    pub kind: String,
    /// URL or family this step is for.
    pub subject: String,
    /// What must already be true.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    /// What becomes true if the step succeeds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postconditions: Vec<String>,
    /// Finding fingerprints or opportunity ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

/// Directed relation between plan steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanEdge {
    /// Source step id.
    pub from: String,
    /// Target step id.
    pub to: String,
    /// `REQUIRES`, `BLOCKS`, `VERIFY_AFTER`, or `OPTIONAL`.
    pub relation: String,
}

/// Machine-checkable search architecture plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPlan {
    /// Ordered actions.
    pub actions: Vec<PlanAction>,
    /// Executable DAG nodes derived from actions. Empty when there is nothing to do.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<PlanStep>,
    /// DAG edges. `actions` stay the human-facing list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<PlanEdge>,
    /// Guarded mutation handoff. Empty when no source span is known.
    pub handoff: RefactorHandoff,
}

/// Compiles a plan from the current report. Does not draft copy or mutate source.
#[must_use]
pub fn plan_from(report: &AuditReport) -> SearchPlan {
    let actions: Vec<PlanAction> = report
        .opportunities
        .iter()
        .map(|item| from_opportunity(item, report))
        .collect();
    let (steps, edges) = dag(&actions);
    let handoff = handoff_from(&actions, report);
    SearchPlan {
        actions,
        steps,
        edges,
        handoff,
    }
}

fn from_opportunity(item: &Opportunity, report: &AuditReport) -> PlanAction {
    let kind = match item.kind.as_str() {
        "link_gap" | "link_rec" => PlanKind::Link,
        "cannibal" | "duplicate" => PlanKind::Consolidate,
        "create_family" => PlanKind::Create,
        "noindex" => PlanKind::Noindex,
        "delete" => PlanKind::Delete,
        _ => PlanKind::Improve,
    };
    let evidence: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| {
            finding.locator.subject_url() == item.subject
                || finding.affected_urls.iter().any(|url| url == &item.subject)
        })
        .map(|finding| finding.fingerprint.clone())
        .take(8)
        .collect();
    let source_location = source_for(item, report);
    let programmatic_verdict = item.programmatic_verdict.clone();
    let (required_facts, schema_requirements, link_placements) = extras(kind, item);
    PlanAction {
        kind,
        subject: item.subject.clone(),
        why: item.why.clone(),
        evidence,
        dependencies: dependencies(kind, item, source_location.as_deref()),
        source_location,
        required_facts,
        link_placements,
        schema_requirements,
        programmatic_verdict,
        acceptance: item.action.clone(),
        verification: match kind {
            PlanKind::Link => "The source HTML contains a crawlable link to the target.".into(),
            PlanKind::Consolidate => {
                "One indexable URL remains for this intent, or each URL has distinct facts.".into()
            }
            PlanKind::Create => "A live URL matches the predicted family.".into(),
            PlanKind::Noindex => "The URL returns noindex or is omitted from the sitemap.".into(),
            PlanKind::Delete => "The URL is gone from sitemap and internal links.".into(),
            PlanKind::Improve => "The listed acceptance condition is true on a new crawl.".into(),
        },
        axes: item.axes.clone(),
    }
}

fn dependencies(kind: PlanKind, item: &Opportunity, source_location: Option<&str>) -> Vec<String> {
    match kind {
        PlanKind::Create => {
            let mut deps = vec![
                "ADD first-party facts".into(),
                "BIND entity facts".into(),
                "ADD schema from measured facts".into(),
                "PASS programmatic distinctness".into(),
                "PASS claim integrity".into(),
            ];
            if let Some(source) = source_location {
                deps.insert(2, format!("producer:{source}"));
            }
            if let Some(verdict) = &item.programmatic_verdict {
                deps.push(format!("programmatic:{verdict}"));
            }
            deps
        }
        PlanKind::Link => vec!["target URL is crawlable from the seed".into()],
        PlanKind::Consolidate => vec![
            "PASS claim integrity".into(),
            "KEEP one indexable URL per intent".into(),
        ],
        PlanKind::Improve if item.kind == "content_gap" => {
            vec!["ADD one H1 that names the page purpose".into()]
        }
        PlanKind::Noindex => {
            vec!["KEEP the family out of the sitemap until unique value is proven".into()]
        }
        _ => Vec::new(),
    }
}

fn source_for(item: &Opportunity, report: &AuditReport) -> Option<String> {
    let from_fact = report.inventory.facts.iter().find_map(|fact| {
        if fact.source.contains(&item.subject) || fact.target.contains(&item.subject) {
            fact.locator.as_ref().map(|locator| match locator {
                weavatrix_seo_model::Locator::Source {
                    path,
                    start_line,
                    end_line,
                } => match (start_line, end_line) {
                    (Some(start), Some(end)) => format!("{path}:{start}-{end}"),
                    (Some(start), None) => format!("{path}:{start}"),
                    _ => path.clone(),
                },
                _ => locator.subject_url().to_owned(),
            })
        } else {
            None
        }
    });
    if from_fact.is_some() {
        return from_fact;
    }
    report.inventory.producers.iter().find_map(|producer| {
        let matches = producer.families.iter().any(|family| {
            item.subject == *family
                || item.subject.contains(family)
                || family.contains(&item.subject)
        });
        if !matches {
            return None;
        }
        Some(format_producer(producer))
    })
}

fn format_producer(producer: &weavatrix_seo_model::ProducerFact) -> String {
    match (producer.start_line, producer.end_line) {
        (Some(start), Some(end)) => {
            format!("{}#{}:{start}-{end}", producer.path, producer.name)
        }
        (Some(start), None) => format!("{}#{}:{start}", producer.path, producer.name),
        _ => producer.key(),
    }
}

fn handoff_from(actions: &[PlanAction], report: &AuditReport) -> RefactorHandoff {
    let mut targets = Vec::new();
    for action in actions {
        let Some(producer) = report.inventory.producers.iter().find(|item| {
            action.source_location.as_deref().is_some_and(|location| {
                location.contains(&item.path) && location.contains(&item.name)
            }) || item
                .families
                .iter()
                .any(|family| action.subject == *family || family.contains(&action.subject))
        }) else {
            continue;
        };
        if producer.path.starts_with('@') || producer.name == "import" {
            continue;
        }
        targets.push(HandoffTarget {
            path: producer.path.clone(),
            symbol: Some(producer.name.clone()),
            start_line: producer.start_line,
            end_line: producer.end_line,
            intent: action.kind.to_string(),
            subject: action.subject.clone(),
            required_facts: action.required_facts.clone(),
            acceptance: action.acceptance.clone(),
            symbol_hash: producer
                .symbol_hash
                .map(weavatrix_seo_model::ContentHash::hex),
            content_hash: Some(producer.content_hash.hex()),
        });
    }
    targets.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.subject.cmp(&right.subject))
    });
    targets.dedup_by(|left, right| left.path == right.path && left.subject == right.subject);
    RefactorHandoff {
        from: "weavatrix-seo".into(),
        to: "weavatrix-refactor".into(),
        read_only: true,
        targets,
        repo_revision: report.inventory.repo_revision.clone(),
        snapshot_id: report.inventory.snapshot_id.clone(),
    }
}

/// Reasons Refactor should refuse this handoff against the current worktree.
///
/// Empty means the source identity still matches. A later revision or symbol
/// hash change is stale; SEO does not apply the edit.
#[must_use]
pub fn stale_targets(
    handoff: &RefactorHandoff,
    repo_revision: Option<&str>,
    producers: &[weavatrix_seo_model::ProducerFact],
) -> Vec<String> {
    let mut reasons = Vec::new();
    if let (Some(planned), Some(now)) = (handoff.repo_revision.as_deref(), repo_revision)
        && planned != now
    {
        reasons.push(format!(
            "source changed since SEO analysis ({planned} → {now})"
        ));
    }
    for target in &handoff.targets {
        let Some(producer) = producers.iter().find(|item| {
            item.path == target.path && target.symbol.as_deref() == Some(item.name.as_str())
        }) else {
            reasons.push(format!(
                "{}#{} is gone from the current worktree",
                target.path,
                target.symbol.as_deref().unwrap_or("?")
            ));
            continue;
        };
        if let (Some(planned), Some(now)) = (
            target.symbol_hash.as_deref(),
            producer
                .symbol_hash
                .map(weavatrix_seo_model::ContentHash::hex),
        ) && planned != now
        {
            reasons.push(format!(
                "{}#{} symbol hash changed since SEO analysis",
                target.path, producer.name
            ));
        }
    }
    reasons
}

fn extras(kind: PlanKind, item: &Opportunity) -> (Vec<String>, Vec<String>, Vec<String>) {
    match kind {
        PlanKind::Link => (Vec::new(), Vec::new(), vec![item.action.clone()]),
        PlanKind::Improve if item.kind == "content_gap" => (Vec::new(), Vec::new(), Vec::new()),
        PlanKind::Create => (
            vec!["unique local facts per combination".into()],
            vec!["Service".into()],
            Vec::new(),
        ),
        _ => (Vec::new(), Vec::new(), Vec::new()),
    }
}

fn dag(actions: &[PlanAction]) -> (Vec<PlanStep>, Vec<PlanEdge>) {
    let mut steps = Vec::new();
    let mut edges = Vec::new();
    let mut last_by_subject: Vec<(String, PlanKind, String)> = Vec::new();
    for action in actions {
        let prefix = format!(
            "plan:{}:{}",
            action.kind,
            weavatrix_seo_model::ContentHash::of_str(&action.subject).short()
        );
        let stages = stages(action.kind);
        let mut previous: Option<String> = None;
        for (index, stage) in stages.iter().enumerate() {
            let id = format!("{prefix}:{stage}");
            let verify = *stage == "CRAWL" || *stage == "OBSERVE_GSC" || *stage == "DIFF";
            steps.push(PlanStep {
                id: id.clone(),
                kind: (*stage).to_owned(),
                subject: action.subject.clone(),
                preconditions: action.dependencies.clone(),
                postconditions: vec![action.acceptance.clone()],
                evidence: action.evidence.clone(),
            });
            if let Some(from) = &previous {
                edges.push(PlanEdge {
                    from: from.clone(),
                    to: id.clone(),
                    relation: if verify {
                        "VERIFY_AFTER".into()
                    } else {
                        "REQUIRES".into()
                    },
                });
            }
            if index + 1 == stages.len() {
                last_by_subject.push((action.subject.clone(), action.kind, id.clone()));
            }
            previous = Some(id);
        }
    }
    for action in actions.iter().filter(|item| item.kind == PlanKind::Link) {
        let prefix = format!(
            "plan:{}:{}",
            action.kind,
            weavatrix_seo_model::ContentHash::of_str(&action.subject).short()
        );
        let link_start = format!("{prefix}:ADD_INTERNAL_LINKS");
        if let Some((_, _, create_end)) = last_by_subject
            .iter()
            .find(|(subject, kind, _)| *kind == PlanKind::Create && subject == &action.subject)
        {
            edges.push(PlanEdge {
                from: create_end.clone(),
                to: link_start,
                relation: "REQUIRES".into(),
            });
        }
    }
    (steps, edges)
}

fn stages(kind: PlanKind) -> &'static [&'static str] {
    match kind {
        PlanKind::Create => &[
            "ADD_FACT",
            "BIND_ENTITY",
            "ADD_SCHEMA",
            "CRAWL",
            "OBSERVE_GSC",
        ],
        PlanKind::Improve => &["UPDATE_CONTENT", "CRAWL", "DIFF", "OBSERVE_GSC"],
        PlanKind::Link => &["ADD_INTERNAL_LINKS", "CRAWL"],
        PlanKind::Consolidate => &["PICK_CANONICAL", "CRAWL", "DIFF"],
        PlanKind::Noindex | PlanKind::Delete => &["SET_NOINDEX", "CRAWL"],
    }
}

#[cfg(test)]
mod tests {
    use super::{PlanKind, dag};
    use crate::plan::PlanAction;
    use weavatrix_seo_model::OpportunityAxes;

    fn action(kind: PlanKind, subject: &str) -> PlanAction {
        PlanAction {
            kind,
            subject: subject.into(),
            why: "test".into(),
            evidence: Vec::new(),
            dependencies: Vec::new(),
            source_location: None,
            required_facts: Vec::new(),
            link_placements: Vec::new(),
            schema_requirements: Vec::new(),
            programmatic_verdict: None,
            acceptance: "done".into(),
            verification: "later".into(),
            axes: OpportunityAxes::default(),
        }
    }

    #[test]
    fn create_pipeline_requires_facts_before_schema() {
        let (steps, edges) = dag(&[action(PlanKind::Create, "category/electrician")]);
        assert!(steps.iter().any(|step| step.kind == "ADD_FACT"));
        assert!(edges.iter().any(|edge| {
            edge.relation == "REQUIRES"
                && edge.from.contains("ADD_FACT")
                && edge.to.contains("BIND_ENTITY")
        }));
        assert!(
            edges
                .iter()
                .any(|edge| edge.relation == "VERIFY_AFTER" && edge.to.contains("CRAWL"))
        );
    }

    #[test]
    fn stale_guard_fires_when_revision_moves() {
        let handoff = super::RefactorHandoff {
            from: "weavatrix-seo".into(),
            to: "weavatrix-refactor".into(),
            read_only: true,
            targets: Vec::new(),
            repo_revision: Some("aaa".into()),
            snapshot_id: "snap".into(),
        };
        let reasons = super::stale_targets(&handoff, Some("bbb"), &[]);
        assert!(reasons.iter().any(|item| item.contains("source changed")));
        assert!(super::stale_targets(&handoff, Some("aaa"), &[]).is_empty());
    }

    #[test]
    fn link_requires_create_on_the_same_subject() {
        let (_steps, edges) = dag(&[
            action(PlanKind::Create, "https://x.test/a"),
            action(PlanKind::Link, "https://x.test/a"),
        ]);
        assert!(
            edges.iter().any(|edge| {
                edge.relation == "REQUIRES" && edge.to.contains("ADD_INTERNAL_LINKS")
            })
        );
    }
}
