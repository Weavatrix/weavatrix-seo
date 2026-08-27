//! Import cone: a helper's relative module is a producer of the same family.

use weavatrix_seo_source::{RouteFamily, SourceSymbol, unmeasured};

#[test]
fn hashes_relative_import_of_helper() {
    let dir = std::env::temp_dir().join(format!("wvx-seo-cone-{}", std::process::id()));
    let lib = dir.join("src").join("lib");
    std::fs::create_dir_all(&lib).expect("lib");
    std::fs::write(lib.join("cities.ts"), "export const CITY = \"A\";\n").expect("cities");
    std::fs::write(
        lib.join("citySeo.ts"),
        "import { CITY } from \"./cities\";\nexport function cityTitle() { return CITY; }\n",
    )
    .expect("helper");
    let mut surface = unmeasured(dir.to_str().unwrap());
    surface.families.push(RouteFamily {
        pattern: "/:locale/category/:city".into(),
        owner: None,
        has_metadata: false,
        has_static_params: false,
        page_symbol: None,
        metadata_symbol: None,
        static_params_symbol: None,
        json_ld_symbols: Vec::new(),
        helpers: vec![SourceSymbol {
            path: "src/lib/citySeo".into(),
            name: "cityTitle".into(),
            start_line: None,
            end_line: None,
        }],
        intercepting: None,
    });
    let facts = surface.producer_facts(dir.to_str().unwrap());
    assert!(
        facts.iter().any(|item| item.path.contains("cities")
            && item.families.iter().any(|family| family.contains(":city"))),
        "{facts:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
