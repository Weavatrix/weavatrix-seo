//! Assemble findings and opportunities for one audit.

use crate::axes::{Coverage, axes};
use crate::graph;
use crate::observe;
use crate::request::AuditRequest;
use crate::source::{programmatic_findings, source_findings};
use weavatrix_seo_architecture::{analyze as analyze_architecture, annotate_templates};
use weavatrix_seo_claims::audit_with_graph as integrity_audit;
use weavatrix_seo_competitor::compare_inventories;
use weavatrix_seo_content::audit as content_audit;
use weavatrix_seo_model::{
    AuditReport, Evidence, EvidenceSemantics, FamilyMatrix, Finding, FindingFamily, Inventory,
    Locator, SearchIntelligence, Severity,
};
use weavatrix_seo_observation::load_state;
use weavatrix_seo_opportunity::{opportunities, rank};
use weavatrix_seo_programmatic::{SafetyVerdict, compile, enrich, thin_city_variants};
use weavatrix_seo_quality::audit as quality_audit;
use weavatrix_seo_render::{load as load_render, reconcile as reconcile_render};
use weavatrix_seo_rules::audit as rule_audit;
use weavatrix_seo_semantic::analyze as analyze_semantic;
use weavatrix_seo_source::SourceSurface;

#[allow(clippy::too_many_lines)]
pub fn assemble(
    request: &AuditRequest,
    mut inventory: Inventory,
    surface: Option<&SourceSurface>,
    competitors: &[(String, Inventory)],
) -> AuditReport {
    annotate_templates(&mut inventory);
    graph::bind(&mut inventory, surface);
    let mut findings = rule_audit(&inventory);
    if let Some(error) = inventory.policy_error.clone() {
        findings.push(policy_contract_finding(&inventory, &error));
    }
    let (architecture, architecture_findings) = analyze_architecture(&inventory);
    findings.extend(architecture_findings);
    findings.extend(quality_audit(&inventory));
    let mut content = content_audit(&inventory);
    findings.extend(content.findings.clone());
    findings.extend(thin_city_variants(&inventory));
    let (integrity_findings, domain) = integrity_audit(&inventory, request.repo.as_deref());
    findings.extend(integrity_findings);
    graph::bind_domain(&mut inventory, domain);
    let predicted = surface.map_or_else(
        || inventory.predicted_routes.clone(),
        weavatrix_seo_source::SourceSurface::patterns,
    );
    if let Some(surface) = &surface {
        findings.extend(source_findings(&inventory, surface));
        findings.extend(programmatic_findings(surface));
    }
    let matrices = enrich(compile(&inventory, &predicted), &content.families);
    let semantic = analyze_semantic(&inventory, &architecture);
    findings.extend(semantic.findings);
    let mut items = opportunities(&inventory, &architecture);
    items.extend(semantic.opportunities);
    items.extend(matrix_opportunities(&matrices, inventory.policy.as_ref()));
    if request.mode == weavatrix_seo_model::AnalysisMode::Compare {
        items.extend(compare_inventories(&inventory, competitors));
    }
    let observations = if request.observations.is_some() {
        load_state(request.observations.as_deref(), "GSC")
    } else {
        load_state(request.gsc.as_deref(), "GSC")
    };
    findings.extend(observe::decorate(
        &observations,
        &inventory,
        &architecture,
        &content.chunks,
        &mut content.families,
        &mut items,
    ));
    for family in &mut content.families {
        let errors = findings
            .iter()
            .filter(|finding| finding.severity == Severity::Error)
            .filter(|finding| finding.locator.subject_url().contains(&family.family))
            .count();
        if errors > 0 {
            family.error_findings = Some(u32::try_from(errors).unwrap_or(u32::MAX));
        }
    }
    let outcomes = weavatrix_seo_observation::outcome_metrics(&observations);
    let url_metrics = weavatrix_seo_observation::url_metrics(&observations);
    let (mut ai_funnels, funnel_findings) =
        weavatrix_seo_observation::analyze_funnel(&observations);
    findings.extend(funnel_findings);
    for funnel in &mut ai_funnels {
        if let Some(producer) = inventory.producers.iter().find(|item| {
            item.families
                .iter()
                .any(|family| funnel.url.contains(family))
        }) {
            funnel.producer = Some(producer.key());
            funnel.family = producer.families.first().cloned();
        }
    }
    let items = rank(items);
    let render = request
        .render
        .as_deref()
        .and_then(|path| load_render(path).ok());
    let has_render = render
        .as_ref()
        .is_some_and(weavatrix_seo_render::RenderSnapshot::connected);
    if let Some(snapshot) = &render {
        let (_report, render_findings) = reconcile_render(&inventory, snapshot);
        findings.extend(render_findings);
        graph::bind_render(&mut inventory, snapshot);
    }
    let axes = axes(
        &findings,
        Coverage {
            source: surface.is_some(),
            http: !inventory.pages.is_empty(),
            obs: observations.connected,
            render: has_render,
            ai_citations: observations.has(weavatrix_seo_observation::ObservationKind::AiCitation),
        },
    );
    graph::bind_chunks(&mut inventory, &content.chunks);
    inventory.stamp_findings(&mut findings);
    let mut semantics = EvidenceSemantics::current();
    let extra = request
        .repo
        .as_deref()
        .map(weavatrix_seo_claims::extra_pack_digest)
        .unwrap_or_default();
    semantics.policy_pack_digest = weavatrix_seo_model::ContentHash::of_str(&format!(
        "{}\n{extra}",
        weavatrix_seo_claims::pack_digest()
    ))
    .hex();
    inventory.semantics = Some(semantics.clone());
    let intelligence = SearchIntelligence {
        semantics,
        profiles: content.profiles,
        families: content.families,
        matrices: matrices
            .iter()
            .map(|matrix| FamilyMatrix {
                family: matrix.family.clone(),
                measured_urls: matrix.measured_urls,
                verdict: matrix.verdict.label().to_owned(),
                dimensions: matrix.dimensions.clone(),
                estimated_cardinality: matrix.estimated_cardinality,
                fact_coverage: matrix.fact_coverage,
                unique_fact_ratio: matrix.unique_fact_ratio,
                template_boilerplate_ratio: matrix.template_boilerplate_ratio,
                semantic_distinctness: matrix.semantic_distinctness,
                unmet_requirements: matrix.unmet_requirements.clone(),
                requirements: matrix.requirements.clone(),
            })
            .collect(),
        chunks: content.chunks,
        intents: content.intents,
        outcomes,
        near_duplicates: content.near_duplicates,
        url_metrics,
        ai_funnels,
        prompts: observations.prompts.clone(),
    };
    AuditReport {
        inventory,
        findings,
        axes,
        opportunities: items,
        intelligence: Some(intelligence),
    }
}

/// A present-but-unreadable search contract leaves the indexable surface undefined.
fn policy_contract_finding(inventory: &Inventory, error: &str) -> Finding {
    let subject = inventory
        .repo
        .clone()
        .unwrap_or_else(|| ".weavatrix".to_owned());
    Finding::new(
        FindingFamily::Idx,
        1,
        Severity::Error,
        &subject,
        format!("the repository search contract could not be read: {error}"),
        Locator::source_span(".weavatrix", None, None),
        Evidence::repo(),
    )
    .explained(
        "A malformed contract falls back to built-in private-path guesses, so a typo is indistinguishable from having no contract at all.",
        "Fix the contract file, or remove it to accept the default heuristic on purpose.",
        "The contract parses and its include/exclude globs decide which families may be indexable.",
    )
}

fn matrix_opportunities(
    matrices: &[weavatrix_seo_programmatic::PageMatrix],
    policy: Option<&weavatrix_seo_model::SearchPolicy>,
) -> Vec<weavatrix_seo_model::Opportunity> {
    let mut items = Vec::new();
    for matrix in matrices {
        if !weavatrix_seo_source::allows_family(policy, &matrix.family) {
            continue;
        }
        let (kind, summary, action) = match matrix.verdict {
            SafetyVerdict::Consolidate => (
                "cannibal",
                format!("{} variants should be consolidated", matrix.family),
                "Merge thin combinations or add unique facts per URL.",
            ),
            SafetyVerdict::Unmeasured if matrix.family.contains(':') => (
                "create_family",
                format!("{} is predicted but unmeasured", matrix.family),
                "Generate a representative URL only after unique facts exist.",
            ),
            SafetyVerdict::SafeIfRequirementsMet => (
                "create_family",
                format!("{} needs unique facts before expansion", matrix.family),
                "Add unique local facts, then generate the rest of the matrix.",
            ),
            SafetyVerdict::NoindexByDefault => (
                "noindex",
                format!("{} should stay out of the index by default", matrix.family),
                "Keep the family noindexed until unique value is proven.",
            ),
            _ => continue,
        };
        items.push(
            weavatrix_seo_model::Opportunity::unmeasured_demand(
                kind,
                matrix.family.clone(),
                summary,
                "Programmatic compiler verdict from measured URLs and predicted families.",
                action,
            )
            .with_programmatic_verdict(matrix.verdict.label()),
        );
    }
    items
}
