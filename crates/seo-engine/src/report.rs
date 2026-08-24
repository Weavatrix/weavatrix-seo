//! Assemble findings and opportunities for one audit.

use crate::axes::axes;
use crate::plan_from;
use crate::request::AuditRequest;
use crate::source::{programmatic_findings, source_findings};
use weavatrix_seo_architecture::analyze as analyze_architecture;
use weavatrix_seo_claims::audit as integrity_audit;
use weavatrix_seo_competitor::compare_inventories;
use weavatrix_seo_content::exact_duplicates;
use weavatrix_seo_model::{AuditReport, Inventory};
use weavatrix_seo_observation::unmeasured as observations_unmeasured;
use weavatrix_seo_opportunity::opportunities;
use weavatrix_seo_programmatic::thin_city_variants;
use weavatrix_seo_quality::audit as quality_audit;
use weavatrix_seo_render::unmeasured as render_unmeasured;
use weavatrix_seo_rules::audit as rule_audit;
use weavatrix_seo_source::SourceSurface;

pub fn assemble(
    request: &AuditRequest,
    inventory: Inventory,
    surface: Option<&SourceSurface>,
    competitors: &[(String, Inventory)],
) -> AuditReport {
    let mut findings = rule_audit(&inventory);
    let (architecture, architecture_findings) = analyze_architecture(&inventory);
    findings.extend(architecture_findings);
    findings.extend(quality_audit(&inventory));
    findings.extend(exact_duplicates(&inventory));
    findings.extend(thin_city_variants(&inventory));
    findings.extend(integrity_audit(&inventory, request.repo.as_deref()));
    if let Some(surface) = &surface {
        findings.extend(source_findings(&inventory, surface));
        findings.extend(programmatic_findings(surface));
    }
    let mut items = opportunities(&inventory, &architecture);
    if request.mode == weavatrix_seo_model::AnalysisMode::Compare {
        items.extend(compare_inventories(&inventory, competitors));
    }
    let _ = render_unmeasured();
    let _ = observations_unmeasured();
    let _ = plan_from(&items);
    let axes = axes(&findings, surface.is_some(), !inventory.pages.is_empty());
    AuditReport {
        inventory,
        findings,
        axes,
        opportunities: items,
    }
}
