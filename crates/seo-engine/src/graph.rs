//! Bind live URLs to route families, producers, schema, and revision.

use weavatrix_seo_model::{
    Evidence, FactEdge, Inventory, Relation, SearchNode, SearchNodeKind, route_id, symbol_id,
    url_id,
};
use weavatrix_seo_nextjs::route_matches;
use weavatrix_seo_source::SourceSurface;

/// Extends the inventory with heterogeneous Search Evidence Graph facts.
pub fn bind(inventory: &mut Inventory, surface: Option<&SourceSurface>) {
    let evidence = inventory.pages.first().map_or_else(Evidence::http, |page| {
        page.evidence.clone()
    });
    if let Some(revision) = &inventory.repo_revision {
        let id = format!("revision:{revision}");
        inventory.nodes.push(
            SearchNode::new(SearchNodeKind::Revision, id.clone(), revision.clone())
                .at(weavatrix_seo_model::Locator::source_span(".git", None, None)),
        );
        if let Some(site) = &inventory.site {
            inventory.facts.push(FactEdge::new(
                url_id(site),
                SearchNodeKind::Url,
                id,
                SearchNodeKind::Revision,
                Relation::ChangedBy,
                evidence.clone(),
            ));
        }
    }
    for page in &inventory.pages {
        inventory.nodes.push(SearchNode::new(
            SearchNodeKind::Url,
            url_id(&page.url.to_string()),
            page.url.to_string(),
        ));
        for block in &page.json_ld {
            for type_name in &block.types {
                let schema_id = format!("schema:{}#{type_name}", page.url);
                inventory.nodes.push(SearchNode::new(
                    SearchNodeKind::SchemaObject,
                    schema_id.clone(),
                    type_name.clone(),
                ));
                inventory.facts.push(FactEdge::new(
                    url_id(&page.url.to_string()),
                    SearchNodeKind::Url,
                    schema_id,
                    SearchNodeKind::SchemaObject,
                    Relation::Declares,
                    evidence.clone(),
                ));
            }
        }
    }
    let Some(surface) = surface else {
        return;
    };
    for family in &surface.families {
        let route = route_id(&family.pattern);
        inventory.nodes.push(
            SearchNode::new(SearchNodeKind::RouteFamily, route.clone(), family.pattern.clone())
                .at(weavatrix_seo_model::Locator::source_span(
                    family.owner.clone().unwrap_or_default(),
                    None,
                    None,
                )),
        );
        bind_symbol(inventory, &route, family.page_symbol.as_ref(), Relation::GeneratedBy, &evidence);
        bind_symbol(
            inventory,
            &route,
            family.metadata_symbol.as_ref(),
            Relation::MetadataFrom,
            &evidence,
        );
        for helper in &family.helpers {
            bind_symbol(inventory, &route, Some(helper), Relation::GeneratedBy, &evidence);
        }
        for page in &inventory.pages {
            if route_matches(&family.pattern, page.url.path()) {
                inventory.facts.push(FactEdge::new(
                    url_id(&page.url.to_string()),
                    SearchNodeKind::Url,
                    route.clone(),
                    SearchNodeKind::RouteFamily,
                    Relation::RenderedBy,
                    evidence.clone(),
                ));
            }
        }
    }
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
        SearchNode::new(SearchNodeKind::SourceSymbol, id.clone(), symbol.name.clone()).at(symbol.locator()),
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
