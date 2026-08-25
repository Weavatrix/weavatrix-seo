//! Assemble findings and opportunities for one audit.

use crate::axes::{Coverage, axes};
use crate::graph;
use crate::observe;
use crate::request::AuditRequest;
use crate::source::{programmatic_findings, source_findings};
use weavatrix_seo_architecture::{analyze as analyze_architecture, annotate_templates};
use weavatrix_seo_claims::audit as integrity_audit;
use weavatrix_seo_competitor::compare_inventories;
use weavatrix_seo_content::exact_duplicates;
use weavatrix_seo_model::{AuditReport, Inventory};
use weavatrix_seo_observation::{
    load as load_gsc, load_any, unmeasured as observations_unmeasured,
};
use weavatrix_seo_opportunity::{opportunities, rank};
use weavatrix_seo_programmatic::{SafetyVerdict, compile, thin_city_variants};
use weavatrix_seo_quality::audit as quality_audit;
use weavatrix_seo_render::{load as load_render, reconcile as reconcile_render};
use weavatrix_seo_rules::audit as rule_audit;
use weavatrix_seo_semantic::analyze as analyze_semantic;
use weavatrix_seo_source::SourceSurface;

pub fn assemble(
    request: &AuditRequest,
    mut inventory: Inventory,
    surface: Option<&SourceSurface>,
    competitors: &[(String, Inventory)],
) -> AuditReport {
    annotate_templates(&mut inventory);
    graph::bind(&mut inventory, surface);
    let mut findings = rule_audit(&inventory);
    let (architecture, architecture_findings) = analyze_architecture(&inventory);
    findings.extend(architecture_findings);
    findings.extend(quality_audit(&inventory));
    findings.extend(exact_duplicates(&inventory));
    findings.extend(thin_city_variants(&inventory));
    findings.extend(integrity_audit(&inventory, request.repo.as_deref()));
    let predicted = surface.map_or_else(
        || inventory.predicted_routes.clone(),
        weavatrix_seo_source::SourceSurface::patterns,
    );
    if let Some(surface) = &surface {
        findings.extend(source_findings(&inventory, surface));
        findings.extend(programmatic_findings(surface));
    }
    let matrices = compile(&inventory, &predicted);
    let semantic = analyze_semantic(&inventory, &architecture);
    findings.extend(semantic.findings);
    let mut items = opportunities(&inventory, &architecture);
    items.extend(semantic.opportunities);
    items.extend(matrix_opportunities(&matrices, inventory.policy.as_ref()));
    if request.mode == weavatrix_seo_model::AnalysisMode::Compare {
        items.extend(compare_inventories(&inventory, competitors));
    }
    let observations = request
        .observations
        .as_deref()
        .and_then(|path| load_any(path).ok())
        .or_else(|| request.gsc.as_deref().and_then(|path| load_gsc(path).ok()))
        .unwrap_or_else(observations_unmeasured);
    findings.extend(observe::decorate(&observations, &inventory, &mut items));
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
        },
    );
    AuditReport {
        inventory,
        findings,
        axes,
        opportunities: items,
    }
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
        items.push(weavatrix_seo_model::Opportunity::unmeasured_demand(
            kind,
            matrix.family.clone(),
            summary,
            "Programmatic compiler verdict from measured URLs and predicted families.",
            action,
        ));
    }
    items
}
