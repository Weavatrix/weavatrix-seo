//! Next.js App Router prediction.

use weavatrix_seo_nextjs::predict;

#[test]
fn predicts_locale_and_city_families() {
    let root = format!(
        "{}/tests/fixtures",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    let surface = predict(&root);
    let patterns = surface.patterns();
    assert!(
        patterns.iter().any(|item| item == "/:locale"),
        "{patterns:?}"
    );
    assert!(
        patterns
            .iter()
            .any(|item| item == "/:locale/category/:city"),
        "{patterns:?}"
    );
    let city = surface
        .families
        .iter()
        .find(|family| family.pattern == "/:locale/category/:city")
        .expect("city family");
    assert!(city.has_metadata);
    assert!(city.has_static_params);
    assert!(!surface.sitemaps.is_empty());
    assert_eq!(
        surface.evidence.kind,
        weavatrix_seo_model::EvidenceKind::Deterministic
    );
}
