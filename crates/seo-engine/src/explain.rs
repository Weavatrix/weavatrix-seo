//! Evidence chain: URL → route → producer span → revision.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{
    AuditReport, FactEdge, Finding, Inventory, Relation, SearchNodeKind, route_id, url_id,
};
use weavatrix_seo_nextjs::route_matches;

/// One hop in the Search Evidence Graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainHop {
    /// Node kind.
    pub kind: String,
    /// Node id.
    pub id: String,
    /// Human label.
    pub label: String,
    /// Incoming relation when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    /// Locator when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

/// Finding plus the source-aware chain competitors cannot produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    /// The finding.
    pub finding: Finding,
    /// Provenance hops.
    pub chain: Vec<ExplainHop>,
    /// Route families that render this URL.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub families: Vec<String>,
    /// Other measured URLs on the same families.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_urls: Vec<String>,
}

/// Explains a finding with its graph chain.
#[must_use]
pub fn explain_chain(report: &AuditReport, id: &str) -> Option<Explanation> {
    let finding = report.finding(id)?;
    let subject = finding.locator.subject_url();
    let (chain, families) = chain_for(&report.inventory, subject);
    let affected_urls = urls_on_families(&report.inventory, &families, subject);
    Some(Explanation {
        finding: finding.clone(),
        chain,
        families,
        affected_urls,
    })
}

fn chain_for(inventory: &Inventory, subject: &str) -> (Vec<ExplainHop>, Vec<String>) {
    let route_key = route_id(subject);
    if inventory
        .nodes
        .iter()
        .any(|node| node.kind == SearchNodeKind::RouteFamily && node.id == route_key)
    {
        return chain_from_route(inventory, subject);
    }
    if let Some(pattern) = family_for_source(inventory, subject) {
        return chain_from_route(inventory, &pattern);
    }
    let mut chain = Vec::new();
    let mut families = Vec::new();
    let url_key = url_id(subject);
    push_node(inventory, &mut chain, &url_key, None);
    for fact in facts_from(inventory, &url_key, Relation::RenderedBy) {
        push_node(inventory, &mut chain, &fact.target, Some(fact.relation));
        if let Some(label) = node_label(inventory, &fact.target) {
            families.push(label);
        }
        for producer in facts_from(inventory, &fact.target, Relation::MetadataFrom)
            .into_iter()
            .chain(facts_from(inventory, &fact.target, Relation::GeneratedBy))
        {
            push_node(
                inventory,
                &mut chain,
                &producer.target,
                Some(producer.relation),
            );
        }
    }
    push_domain(inventory, &mut chain, &url_key);
    for fact in facts_from(inventory, &url_key, Relation::ComparedAgainst)
        .into_iter()
        .chain(facts_from(inventory, &url_key, Relation::ChangedBy))
    {
        push_node(inventory, &mut chain, &fact.target, Some(fact.relation));
    }
    if chain.is_empty() {
        chain.push(ExplainHop {
            kind: "url".into(),
            id: url_key,
            label: subject.to_owned(),
            relation: None,
            locator: Some(subject.to_owned()),
        });
    }
    families.sort();
    families.dedup();
    (chain, families)
}

/// Walks the domain layer: what the URL claims, what that needs, and where it lives.
///
/// `URL ─CLAIMS→ Claim ─REQUIRES→ DataField ─DEFINED_AT→ span`, plus the policy
/// that governs the claim and the entities and market the page is about.
fn push_domain(inventory: &Inventory, chain: &mut Vec<ExplainHop>, url_key: &str) {
    for claim in facts_from(inventory, url_key, Relation::Claims) {
        push_node(inventory, chain, &claim.target, Some(claim.relation));
        for governs in facts_from(inventory, &claim.target, Relation::GovernedBy) {
            push_node(inventory, chain, &governs.target, Some(governs.relation));
        }
        for field in facts_from(inventory, &claim.target, Relation::Requires) {
            push_node(inventory, chain, &field.target, Some(field.relation));
            for definition in facts_from(inventory, &field.target, Relation::DefinedAt) {
                push_node(
                    inventory,
                    chain,
                    &definition.target,
                    Some(definition.relation),
                );
            }
        }
    }
    for about in facts_from(inventory, url_key, Relation::About) {
        push_node(inventory, chain, &about.target, Some(about.relation));
    }
}

fn family_for_source(inventory: &Inventory, path: &str) -> Option<String> {
    if let Some(family) = inventory.nodes.iter().find_map(|node| {
        if node.kind != SearchNodeKind::RouteFamily {
            return None;
        }
        let locator = node.locator.as_ref()?.subject_url();
        (locator == path).then(|| node.label.clone())
    }) {
        return Some(family);
    }
    inventory.producers.iter().find_map(|producer| {
        (producer.path == path || path.contains(&producer.path))
            .then(|| producer.families.first().cloned())
            .flatten()
    })
}

fn chain_from_route(inventory: &Inventory, pattern: &str) -> (Vec<ExplainHop>, Vec<String>) {
    let mut chain = Vec::new();
    let route = route_id(pattern);
    push_node(inventory, &mut chain, &route, None);
    for producer in facts_from(inventory, &route, Relation::MetadataFrom)
        .into_iter()
        .chain(facts_from(inventory, &route, Relation::GeneratedBy))
    {
        push_node(
            inventory,
            &mut chain,
            &producer.target,
            Some(producer.relation),
        );
    }
    (chain, vec![pattern.to_owned()])
}

fn facts_from<'a>(inventory: &'a Inventory, source: &str, relation: Relation) -> Vec<&'a FactEdge> {
    inventory
        .facts
        .iter()
        .filter(|fact| fact.source == source && fact.relation == relation)
        .collect()
}

fn push_node(
    inventory: &Inventory,
    chain: &mut Vec<ExplainHop>,
    id: &str,
    relation: Option<Relation>,
) {
    if chain.iter().any(|hop| hop.id == id) {
        return;
    }
    let node = inventory.nodes.iter().find(|item| item.id == id);
    chain.push(ExplainHop {
        kind: node.map_or("node", |item| kind_name(item.kind)).into(),
        id: id.to_owned(),
        label: node.map_or_else(|| id.to_owned(), |item| item.label.clone()),
        relation: relation.map(relation_name).map(str::to_owned),
        locator: node
            .and_then(|item| item.locator.as_ref())
            .map(|locator| locator.subject_url().to_owned()),
    });
}

fn kind_name(kind: SearchNodeKind) -> &'static str {
    match kind {
        SearchNodeKind::Url => "url",
        SearchNodeKind::RouteFamily => "route",
        SearchNodeKind::SourceSymbol => "symbol",
        SearchNodeKind::Revision => "revision",
        SearchNodeKind::SchemaObject => "schema",
        SearchNodeKind::SearchObservation => "observation",
        SearchNodeKind::Claim => "claim",
        SearchNodeKind::DataField => "field",
        SearchNodeKind::Entity => "entity",
        SearchNodeKind::Market => "market",
        SearchNodeKind::Policy => "policy",
        SearchNodeKind::LegalRequirement => "requirement",
        SearchNodeKind::Topic => "topic",
    }
}

fn relation_name(relation: Relation) -> &'static str {
    match relation {
        Relation::RenderedBy => "RENDERED_BY",
        Relation::GeneratedBy => "GENERATED_BY",
        Relation::MetadataFrom => "METADATA_FROM",
        Relation::ChangedBy => "CHANGED_BY",
        Relation::ComparedAgainst => "COMPARED_AGAINST",
        Relation::ObservedAs => "OBSERVED_AS",
        Relation::Claims => "CLAIMS",
        Relation::Requires => "REQUIRES",
        Relation::DefinedAt => "DEFINED_AT",
        Relation::GovernedBy => "GOVERNED_BY",
        Relation::About => "ABOUT",
        Relation::Declares => "DECLARES",
        _ => "RELATED",
    }
}

fn node_label(inventory: &Inventory, id: &str) -> Option<String> {
    inventory
        .nodes
        .iter()
        .find(|item| item.id == id && item.kind == SearchNodeKind::RouteFamily)
        .map(|item| item.label.clone())
}

fn urls_on_families(inventory: &Inventory, families: &[String], subject: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for page in &inventory.pages {
        let path = page.url.path();
        if families.iter().any(|pattern| route_matches(pattern, path))
            && page.url.to_string() != subject
        {
            urls.push(page.url.to_string());
        }
    }
    urls
}
