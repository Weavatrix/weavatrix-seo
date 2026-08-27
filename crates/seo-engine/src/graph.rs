//! Bind live URLs to route families, producers, schema, and revision.

use weavatrix_seo_model::{
    Confidence, Evidence, EvidenceKind, FactEdge, Inventory, Locator, Relation, SearchNode,
    SearchNodeKind, route_id, symbol_id, url_id,
};
use weavatrix_seo_nextjs::route_matches;
use weavatrix_seo_render::RenderSnapshot;
use weavatrix_seo_source::SourceSurface;

/// Extends the inventory with heterogeneous Search Evidence Graph facts.
pub fn bind(inventory: &mut Inventory, surface: Option<&SourceSurface>) {
    let http = inventory
        .pages
        .first()
        .map_or_else(Evidence::http, |page| page.evidence.clone());
    let repo = repo_evidence(inventory);
    bind_revision(inventory, &repo);
    bind_pages(inventory, &http);
    if let Some(surface) = surface {
        bind_surface(inventory, surface, &repo);
    }
    stamp_facts(inventory);
}

/// Provenance for facts a repository parser established.
fn repo_evidence(inventory: &Inventory) -> Evidence {
    let mut evidence = Evidence::repo();
    if let Some(revision) = &inventory.repo_revision {
        evidence.revision = Some(revision.clone());
    }
    evidence
}

/// Provenance for a fact that only exists because two layers were compared.
fn cross_layer_evidence(inventory: &Inventory) -> Evidence {
    let mut evidence = repo_evidence(inventory);
    evidence.kind = EvidenceKind::Inferred;
    evidence.confidence = Confidence::High;
    evidence
}

fn bind_revision(inventory: &mut Inventory, evidence: &Evidence) {
    let Some(revision) = &inventory.repo_revision else {
        return;
    };
    let id = format!("revision:{revision}");
    inventory.nodes.push(
        SearchNode::new(SearchNodeKind::Revision, id.clone(), revision.clone())
            .at(Locator::source_span(".git", None, None)),
    );
    if let Some(site) = &inventory.site {
        // The crawl measured production and the worktree sits at this revision.
        // Nothing here proves production was built from it, so the relation is
        // a comparison, not causation.
        inventory.facts.push(FactEdge::new(
            url_id(site),
            SearchNodeKind::Url,
            id,
            SearchNodeKind::Revision,
            Relation::ComparedAgainst,
            evidence.clone(),
        ));
    }
}

fn bind_pages(inventory: &mut Inventory, evidence: &Evidence) {
    let mut schema: Vec<(String, String, String)> = Vec::new();
    for page in &inventory.pages {
        let url = page.url.to_string();
        inventory.nodes.push(SearchNode::new(
            SearchNodeKind::Url,
            url_id(&url),
            url.clone(),
        ));
        for block in &page.json_ld {
            if block.ids.is_empty() {
                for type_name in &block.types {
                    schema.push((
                        url.clone(),
                        format!("schema:{url}#{type_name}"),
                        type_name.clone(),
                    ));
                }
            } else {
                for id in &block.ids {
                    let label = block.types.first().cloned().unwrap_or_else(|| id.clone());
                    schema.push((url.clone(), id.clone(), label));
                }
            }
        }
    }
    for (url, schema_id, label) in schema {
        bind_schema(inventory, &url, &schema_id, &label, evidence);
    }
}

fn bind_surface(inventory: &mut Inventory, surface: &SourceSurface, evidence: &Evidence) {
    for family in &surface.families {
        let route = route_id(&family.pattern);
        inventory.nodes.push(
            SearchNode::new(
                SearchNodeKind::RouteFamily,
                route.clone(),
                family.pattern.clone(),
            )
            .at(Locator::source_span(
                family.owner.clone().unwrap_or_default(),
                None,
                None,
            )),
        );
        bind_symbol(
            inventory,
            &route,
            family.page_symbol.as_ref(),
            Relation::GeneratedBy,
            evidence,
        );
        bind_symbol(
            inventory,
            &route,
            family.metadata_symbol.as_ref(),
            Relation::MetadataFrom,
            evidence,
        );
        for helper in &family.helpers {
            bind_symbol(
                inventory,
                &route,
                Some(helper),
                Relation::GeneratedBy,
                evidence,
            );
        }
        let matched = cross_layer_evidence(inventory);
        for page in &inventory.pages {
            if route_matches(&family.pattern, page.url.path()) {
                // A measured URL matching a predicted pattern is an inference
                // across two layers, not a deterministic repository fact.
                inventory.facts.push(FactEdge::new(
                    url_id(&page.url.to_string()),
                    SearchNodeKind::Url,
                    route.clone(),
                    SearchNodeKind::RouteFamily,
                    Relation::RenderedBy,
                    matched.clone(),
                ));
            }
        }
    }
}

fn stamp_facts(inventory: &mut Inventory) {
    let snapshot = inventory.snapshot_id.clone();
    let policy = inventory.policy_version.clone();
    for fact in &mut inventory.facts {
        if fact.evidence.snapshot_id.is_none() {
            fact.evidence.snapshot_id = Some(snapshot.clone());
        }
        if fact.evidence.policy_version.is_none() {
            fact.evidence.policy_version = Some(policy.clone());
        }
    }
}

/// Attaches imported render observations as `OBSERVED_AS` facts.
pub fn bind_render(inventory: &mut Inventory, snapshot: &RenderSnapshot) {
    if !snapshot.connected() {
        return;
    }
    let mut evidence = snapshot.evidence();
    if !inventory.snapshot_id.is_empty() {
        evidence.snapshot_id = Some(inventory.snapshot_id.clone());
    }
    if evidence.policy_version.is_none() && !inventory.policy_version.is_empty() {
        evidence.policy_version = Some(inventory.policy_version.clone());
    }
    for page in &snapshot.pages {
        let id = format!("render:{}", page.url);
        inventory.nodes.push(
            SearchNode::new(
                SearchNodeKind::SearchObservation,
                id.clone(),
                page.url.clone(),
            )
            .at(Locator::Url(page.url.clone())),
        );
        inventory.facts.push(FactEdge::new(
            url_id(&page.url),
            SearchNodeKind::Url,
            id,
            SearchNodeKind::SearchObservation,
            Relation::ObservedAs,
            evidence.clone(),
        ));
    }
}

fn bind_symbol(
    inventory: &mut Inventory,
    route: &str,
    symbol: Option<&weavatrix_seo_source::SourceSymbol>,
    relation: Relation,
    evidence: &Evidence,
) {
    let Some(symbol) = symbol else {
        return;
    };
    let id = symbol_id(&symbol.path, &symbol.name);
    inventory.nodes.push(
        SearchNode::new(
            SearchNodeKind::SourceSymbol,
            id.clone(),
            symbol.name.clone(),
        )
        .at(symbol.locator()),
    );
    inventory.facts.push(
        FactEdge::new(
            route.to_owned(),
            SearchNodeKind::RouteFamily,
            id,
            SearchNodeKind::SourceSymbol,
            relation,
            evidence.clone(),
        )
        .at(symbol.locator()),
    );
}

fn bind_schema(
    inventory: &mut Inventory,
    page_url: &str,
    schema_id: &str,
    label: &str,
    evidence: &Evidence,
) {
    inventory.nodes.push(SearchNode::new(
        SearchNodeKind::SchemaObject,
        schema_id,
        label,
    ));
    inventory.facts.push(FactEdge::new(
        url_id(page_url),
        SearchNodeKind::Url,
        schema_id.to_owned(),
        SearchNodeKind::SchemaObject,
        Relation::Declares,
        evidence.clone(),
    ));
}
