//! Inferred clusters, cannibalization, and internal-link recommendations.

use crate::embed::{self, MODEL};
use crate::profiles;
use weavatrix_graph::{AttributeValue, GraphBuilder, Node, NodeKind};
use weavatrix_semantic::{
    AnchorCandidate, AnchorConfig, AnchorMatcher, LinkConfig, SelectionMode, SemanticLinker,
    SemanticVector, SeoLinkPolicy,
};
use weavatrix_seo_architecture::Architecture;
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Indexability, Inventory,
    Locator, Opportunity, OpportunityAxes, Severity,
};

/// Semantic pass output. Evidence is always inferred.
pub struct SemanticPass {
    /// Cannibalization and near-duplicate findings.
    pub findings: Vec<Finding>,
    /// Link/topic opportunities.
    pub opportunities: Vec<Opportunity>,
}

/// Runs lexical embeddings through `weavatrix-semantic`.
#[must_use]
pub fn analyze(inventory: &Inventory, architecture: &Architecture) -> SemanticPass {
    let mut pass = SemanticPass {
        findings: Vec::new(),
        opportunities: Vec::new(),
    };
    let pages: Vec<_> = inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.indexability == Indexability::Indexable)
        .collect();
    if pages.len() < 2 {
        return pass;
    }
    let Ok(all_profiles) = profiles(inventory) else {
        return pass;
    };
    let mut vectors = Vec::new();
    let mut ids = Vec::new();
    for page in &pages {
        let Some(values) = embed::embed(&page.visible_text()) else {
            continue;
        };
        let id = format!("page:{}", page.url);
        if let Ok(vector) = SemanticVector::new(id.clone(), values) {
            ids.push(id);
            vectors.push(vector);
        }
    }
    if vectors.len() < 2 {
        return pass;
    }
    cannibalization(&pages, &vectors, &mut pass);
    let selected: Vec<_> = all_profiles
        .into_iter()
        .filter(|profile| ids.iter().any(|id| profile.node_id().as_str() == id))
        .collect();
    recommend(inventory, architecture, selected, &ids, &vectors, &mut pass);
    pass
}

fn inferred() -> Evidence {
    Evidence {
        kind: EvidenceKind::Inferred,
        source: EvidenceSource::Semantic,
        confidence: weavatrix_seo_model::Confidence::Medium,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    }
}

fn cannibalization(
    pages: &[&weavatrix_seo_model::ExtractedPage],
    vectors: &[SemanticVector],
    pass: &mut SemanticPass,
) {
    let by_id: std::collections::BTreeMap<_, _> = pages
        .iter()
        .map(|page| (format!("page:{}", page.url), *page))
        .collect();
    for (index, left) in vectors.iter().enumerate() {
        for right in vectors.iter().skip(index + 1) {
            let score = embed::cosine(left.values(), right.values());
            if score < 0.86 {
                continue;
            }
            let Some(a) = by_id.get(left.node_id().as_str()) else {
                continue;
            };
            let Some(b) = by_id.get(right.node_id().as_str()) else {
                continue;
            };
            if a.content_hash == b.content_hash {
                continue;
            }
            if !share_intent(a, b) {
                continue;
            }
            let subject = format!("{} {}", a.url, b.url);
            pass.findings.push(
                Finding::new(
                    FindingFamily::Cann,
                    1,
                    Severity::Warn,
                    &subject,
                    format!("{} and {} look like the same search intent", a.url, b.url),
                    Locator::url(&a.url),
                    inferred(),
                )
                .with_affected([b.url.to_string()])
                .explained(
                    "Two indexable URLs share heading intent and similar body; this is inferred, not a ranking proof.",
                    "Consolidate, differentiate unique facts, or canonicalise to one URL.",
                    "Each remaining URL has a distinct primary intent.",
                ),
            );
        }
    }
}

fn share_intent(
    left: &weavatrix_seo_model::ExtractedPage,
    right: &weavatrix_seo_model::ExtractedPage,
) -> bool {
    if different_service_same_city(left.url.path(), right.url.path()) {
        return false;
    }
    let lh = left
        .headings
        .iter()
        .find(|item| item.level == 1)
        .map(|item| item.text.as_str())
        .or(left.title.as_deref())
        .unwrap_or("");
    let rh = right
        .headings
        .iter()
        .find(|item| item.level == 1)
        .map(|item| item.text.as_str())
        .or(right.title.as_deref())
        .unwrap_or("");
    let lt = intent_tokens(lh);
    let rt = intent_tokens(rh);
    let overlap = lt.iter().filter(|token| rt.contains(token)).count();
    overlap >= 2
}

fn different_service_same_city(left: &str, right: &str) -> bool {
    let lp: Vec<&str> = left.split('/').filter(|part| !part.is_empty()).collect();
    let rp: Vec<&str> = right.split('/').filter(|part| !part.is_empty()).collect();
    if lp.len() < 3 || rp.len() != lp.len() {
        return false;
    }
    let last = lp.len() - 1;
    lp[last] == rp[last]
        && lp[last - 1] != rp[last - 1]
        && (lp[last].contains('-') || lp.contains(&"category"))
}

fn intent_tokens(heading: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "in", "the", "a", "an", "and", "or", "for", "of", "to", "on", "at", "wa", "il",
    ];
    heading
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .filter(|token| token.len() > 2 && !STOP.contains(&token.as_str()))
        .collect()
}

fn recommend(
    inventory: &Inventory,
    architecture: &Architecture,
    profiles: Vec<weavatrix_semantic::SeoPage>,
    ids: &[String],
    vectors: &[SemanticVector],
    pass: &mut SemanticPass,
) {
    let profiles = annotate_profiles(profiles, inventory, architecture);
    let Ok(policy) = SeoLinkPolicy::new(profiles) else {
        return;
    };
    let Ok(kind) = NodeKind::custom("page") else {
        return;
    };
    let mut builder = GraphBuilder::new();
    for id in ids {
        let Ok(node) = Node::new(id.clone(), id.clone(), kind.clone()) else {
            continue;
        };
        let _ = builder.add_node(node);
    }
    let Ok(graph) = builder.build() else {
        return;
    };
    let Ok(linker) = SemanticLinker::new(
        LinkConfig::new(MODEL, 0.12, 3).with_selection(SelectionMode::Directed),
    ) else {
        return;
    };
    let Ok(report) = linker.link_with_policy(&graph, vectors, &policy) else {
        return;
    };
    let candidates = heading_candidates(inventory);
    let placements = AnchorMatcher::new(AnchorConfig::new(MODEL, 0.05, 2))
        .ok()
        .and_then(|matcher| matcher.match_links(&report, vectors, &candidates).ok());
    for edge in report.edges() {
        emit_link(edge, placements.as_ref(), pass);
    }
}

fn annotate_profiles(
    mut profiles: Vec<weavatrix_semantic::SeoPage>,
    inventory: &Inventory,
    architecture: &Architecture,
) -> Vec<weavatrix_semantic::SeoPage> {
    let authority: std::collections::BTreeMap<_, _> = architecture
        .pages
        .iter()
        .map(|page| (page.url.to_string(), page.authority))
        .collect();
    let existing: std::collections::BTreeSet<_> = inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == weavatrix_seo_model::Relation::LinksTo)
        .map(|edge| (edge.source.to_string(), edge.target.to_string()))
        .collect();
    for profile in &mut profiles {
        let url = profile.node_id().as_str().trim_start_matches("page:");
        if authority.get(url).is_some_and(|score| *score > 0.15) || url.ends_with('/') {
            *profile = profile.clone().with_cornerstone(true);
        }
        for (source, target) in &existing {
            if profile.node_id().as_str() == format!("page:{source}")
                && let Ok(node) = weavatrix_graph::NodeId::new(format!("page:{target}"))
            {
                *profile = profile.clone().with_existing_target(node);
            }
        }
    }
    profiles
}

fn emit_link(
    edge: &weavatrix_graph::Edge,
    placements: Option<&weavatrix_semantic::AnchorMatchReport>,
    pass: &mut SemanticPass,
) {
    let source = edge.source.as_str().trim_start_matches("page:");
    let target = edge.target.as_str().trim_start_matches("page:");
    let sim = match edge.attributes.get("similarity") {
        Some(AttributeValue::Float(value)) => format!("{:.2}", value.get()),
        _ => "inferred".into(),
    };
    let orphan = edge
        .attributes
        .get("target_orphan")
        .is_some_and(|value| matches!(value, AttributeValue::Bool(true)));
    let placement = placements.and_then(|report| {
        report.links().iter().find(|link| {
            link.source().as_str() == edge.source.as_str()
                && link.target().as_str() == edge.target.as_str()
        })
    });
    let anchor = placement
        .and_then(|link| link.suggestions().first())
        .map_or_else(
            || "contextual mention".into(),
            |item| item.anchor_text().to_owned(),
        );
    pass.findings.push(
        Finding::new(
            FindingFamily::Link,
            4,
            if orphan { Severity::Warn } else { Severity::Info },
            &format!("{source}->{target}"),
            format!("{source} is missing a topical internal link to {target} (cosine {sim})"),
            Locator::Url(source.into()),
            inferred(),
        )
        .explained(
            "weavatrix-semantic selected this directed link under SEO policy. Similarity is inferred, not a ranking factor.",
            format!("Add a crawlable HTML link with anchor `{anchor}` toward {target}."),
            "The source template links to the target with a contextual anchor.",
        ),
    );
    pass.opportunities.push(
        Opportunity::unmeasured_demand(
            "link_rec",
            target,
            format!("Internal link from {source} to {target}"),
            "Topical similarity and graph policy say this target is eligible.",
            format!("Place `{anchor}` on {source} pointing at {target}."),
        )
        .with_axes(OpportunityAxes {
            graph_leverage: Some(if orphan { 80 } else { 40 }),
            topical_fit: Some(70),
            confidence: Some(50),
            implementation_cost: Some(20),
            risk: Some(15),
            ..OpportunityAxes::default()
        }),
    );
}

fn heading_candidates(inventory: &Inventory) -> Vec<AnchorCandidate> {
    let mut out = Vec::new();
    for page in &inventory.pages {
        let heading = page
            .headings
            .iter()
            .find(|item| item.level == 1)
            .map(|item| item.text.as_str())
            .or(page.title.as_deref())
            .unwrap_or("this page");
        let Some(values) = embed::embed(heading) else {
            continue;
        };
        let Ok(node) = weavatrix_graph::NodeId::new(format!("page:{}", page.url)) else {
            continue;
        };
        if let Ok(candidate) = AnchorCandidate::new(node, "h1", heading, heading, values) {
            out.push(candidate);
        }
    }
    out
}
