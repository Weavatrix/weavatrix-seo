//! Domain layer of the Search Evidence Graph.
//!
//! The detectors already establish these facts in order to raise findings.
//! Binding the same facts as nodes and edges is what turns an explanation from
//! "here is a finding" into the chain a fix can be planned against:
//!
//! ```text
//! URL ─CLAIMS→ Claim ─REQUIRES→ DataField ─DEFINED_AT→ source span
//!                └─GOVERNED_BY→ Policy
//! URL ─ABOUT→ Entity / Market
//! ```
//!
//! Nothing here re-detects anything. It reads the same pack rules and repo
//! signals the findings are built from, so the graph cannot disagree with them.

use crate::market::{contains_token, infer_market, page_haystack};
use crate::pack::{self, PolicyPack};
use crate::repo::RepoSignals;
use std::collections::BTreeSet;
use weavatrix_seo_model::{
    Confidence, Evidence, EvidenceKind, FactEdge, Inventory, Locator, Relation, SearchNode,
    SearchNodeKind, symbol_id, url_id,
};

/// Domain nodes and edges for one run.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DomainGraph {
    /// Claim, data-field, entity, market, and policy nodes.
    pub nodes: Vec<SearchNode>,
    /// Edges between them and the measured URLs.
    pub facts: Vec<FactEdge>,
}

/// Node id for a policy pack.
#[must_use]
pub fn policy_id(pack: &PolicyPack) -> String {
    format!("policy:{}", pack.id)
}

/// Node id for a jurisdiction.
#[must_use]
pub fn market_id(pack: &PolicyPack) -> String {
    format!("market:{}", pack.jurisdiction)
}

/// Node id for a public claim rule.
#[must_use]
pub fn claim_id(pack: &PolicyPack, rule: &str) -> String {
    format!("claim:{}#{rule}", pack.id)
}

/// Node id for a domain data field.
#[must_use]
pub fn field_id(pack: &PolicyPack, field: &str) -> String {
    format!("field:{}#{field}", pack.id)
}

/// Node id for a named entity.
#[must_use]
pub fn entity_id(pack: &PolicyPack, label: &str) -> String {
    format!("entity:{}#{label}", pack.id)
}

/// Builds the domain graph from measured pages and repository signals.
#[must_use]
pub fn domain_graph(inventory: &Inventory, signals: Option<&RepoSignals>) -> DomainGraph {
    let mut graph = DomainGraph::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for page in &inventory.pages {
        if page.status >= 400 {
            continue;
        }
        let hay = page_haystack(page);
        let market = infer_market(&page.url, page.html_lang.as_deref(), &hay);
        let Some(pack) = pack::for_market(market) else {
            continue;
        };
        let url = page.url.to_string();
        bind_market(&mut graph, &mut seen, &url, pack);
        bind_entities(&mut graph, &mut seen, &url, &hay);
        bind_claims(&mut graph, &mut seen, &url, &hay, pack, signals);
        bind_instances(&mut graph, &mut seen, &url, pack, signals);
    }
    graph
}

/// A jurisdiction inferred from host, path, language, and copy.
fn bind_market(graph: &mut DomainGraph, seen: &mut BTreeSet<String>, url: &str, pack: &PolicyPack) {
    let market = market_id(pack);
    push_node(
        graph,
        seen,
        SearchNode::new(SearchNodeKind::Market, market.clone(), pack.jurisdiction),
    );
    push_node(
        graph,
        seen,
        SearchNode::new(SearchNodeKind::Policy, policy_id(pack), pack.id),
    );
    graph.facts.push(FactEdge::new(
        url_id(url),
        SearchNodeKind::Url,
        market,
        SearchNodeKind::Market,
        Relation::About,
        inferred_http(),
    ));
}

/// Entities of any pack that this page actually names.
///
/// Not only the owning pack: a page that names a foreign entity is still about
/// it, and that is exactly what makes market contamination visible in the graph
/// rather than only in a finding.
fn bind_entities(graph: &mut DomainGraph, seen: &mut BTreeSet<String>, url: &str, hay: &str) {
    for pack in pack::all() {
        for entity in pack.entities {
            if !contains_token(hay, entity.token) {
                continue;
            }
            let id = entity_id(pack, entity.label);
            push_node(
                graph,
                seen,
                SearchNode::new(SearchNodeKind::Entity, id.clone(), entity.label),
            );
            graph.facts.push(FactEdge::new(
                url_id(url),
                SearchNodeKind::Url,
                id.clone(),
                SearchNodeKind::Entity,
                Relation::About,
                Evidence::http(),
            ));
            // An entity keeps its own jurisdiction even when a page of another
            // market names it. That edge is what makes contamination legible.
            graph.facts.push(FactEdge::new(
                id,
                SearchNodeKind::Entity,
                policy_id(pack),
                SearchNodeKind::Policy,
                Relation::GovernedBy,
                Evidence::policy(),
            ));
            push_node(
                graph,
                seen,
                SearchNode::new(SearchNodeKind::Policy, policy_id(pack), pack.id),
            );
        }
    }
}

/// Public claims, the fields they require, and where those fields are defined.
fn bind_claims(
    graph: &mut DomainGraph,
    seen: &mut BTreeSet<String>,
    url: &str,
    hay: &str,
    pack: &PolicyPack,
    signals: Option<&RepoSignals>,
) {
    let compact = crate::license::compact(hay);
    for rule in pack.claims {
        if !rule
            .phrases
            .iter()
            .any(|phrase| compact.contains(&crate::license::compact(phrase)))
        {
            continue;
        }
        let claim = claim_id(pack, rule.id);
        let field = field_id(pack, rule.requires_fact);
        push_node(
            graph,
            seen,
            SearchNode::new(SearchNodeKind::Claim, claim.clone(), rule.id),
        );
        push_node(
            graph,
            seen,
            SearchNode::new(SearchNodeKind::DataField, field.clone(), rule.requires_fact),
        );
        graph.facts.push(FactEdge::new(
            url_id(url),
            SearchNodeKind::Url,
            claim.clone(),
            SearchNodeKind::Claim,
            Relation::Claims,
            Evidence::http(),
        ));
        graph.facts.push(FactEdge::new(
            claim.clone(),
            SearchNodeKind::Claim,
            field.clone(),
            SearchNodeKind::DataField,
            Relation::Requires,
            Evidence::policy(),
        ));
        graph.facts.push(FactEdge::new(
            claim,
            SearchNodeKind::Claim,
            policy_id(pack),
            SearchNodeKind::Policy,
            Relation::GovernedBy,
            Evidence::policy(),
        ));
        bind_definition(graph, seen, &field, rule.requires_fact, pack, signals);
    }
}

/// Where the analysed repository defines the required field, when it does.
fn bind_definition(
    graph: &mut DomainGraph,
    seen: &mut BTreeSet<String>,
    field: &str,
    name: &str,
    pack: &PolicyPack,
    signals: Option<&RepoSignals>,
) {
    let Some(signals) = signals else {
        return;
    };
    let Some((_, facts)) = signals.packs.iter().find(|(id, _)| *id == pack.id) else {
        return;
    };
    let Some((path, line)) = &facts.false_at else {
        return;
    };
    let symbol = symbol_id(path, name);
    push_node(
        graph,
        seen,
        SearchNode::new(SearchNodeKind::SourceSymbol, symbol.clone(), name)
            .at(Locator::source_span(path.clone(), *line, *line)),
    );
    graph.facts.push(
        FactEdge::new(
            field.to_owned(),
            SearchNodeKind::DataField,
            symbol,
            SearchNodeKind::SourceSymbol,
            Relation::DefinedAt,
            Evidence::repo(),
        )
        .at(Locator::source_span(path.clone(), *line, *line)),
    );
}

/// Entity-instance fields: `entity:specialist:123` and `field:specialist:123#license_verified`.
fn bind_instances(
    graph: &mut DomainGraph,
    seen: &mut BTreeSet<String>,
    url: &str,
    pack: &PolicyPack,
    signals: Option<&RepoSignals>,
) {
    let Some(signals) = signals else {
        return;
    };
    let Some((_, facts)) = signals.packs.iter().find(|(id, _)| *id == pack.id) else {
        return;
    };
    for instance in &facts.instances {
        if !url.contains(&instance.entity_id) {
            continue;
        }
        let entity = format!("entity:{}:instance:{}", pack.id, instance.entity_id);
        let field = format!(
            "field:{}:instance:{}#{}",
            pack.id, instance.entity_id, instance.field
        );
        push_node(
            graph,
            seen,
            SearchNode::new(
                SearchNodeKind::Entity,
                entity.clone(),
                instance.entity_id.clone(),
            ),
        );
        push_node(
            graph,
            seen,
            SearchNode::new(
                SearchNodeKind::DataField,
                field.clone(),
                format!("{}#{}", instance.entity_id, instance.field),
            ),
        );
        graph.facts.push(FactEdge::new(
            url_id(url),
            SearchNodeKind::Url,
            entity.clone(),
            SearchNodeKind::Entity,
            Relation::About,
            Evidence::repo(),
        ));
        let symbol = symbol_id(&instance.path, &instance.field);
        push_node(
            graph,
            seen,
            SearchNode::new(
                SearchNodeKind::SourceSymbol,
                symbol.clone(),
                instance.field.clone(),
            )
            .at(Locator::source_span(
                instance.path.clone(),
                instance.line,
                instance.line,
            )),
        );
        graph.facts.push(
            FactEdge::new(
                field,
                SearchNodeKind::DataField,
                symbol,
                SearchNodeKind::SourceSymbol,
                Relation::DefinedAt,
                Evidence::repo(),
            )
            .at(Locator::source_span(
                instance.path.clone(),
                instance.line,
                instance.line,
            )),
        );
    }
}

fn push_node(graph: &mut DomainGraph, seen: &mut BTreeSet<String>, node: SearchNode) {
    if seen.insert(node.id.clone()) {
        graph.nodes.push(node);
    }
}

/// Market classification is a heuristic over host, path, language, and copy.
fn inferred_http() -> Evidence {
    Evidence {
        kind: EvidenceKind::Inferred,
        confidence: Confidence::Medium,
        ..Evidence::http()
    }
}
