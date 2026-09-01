//! Content-intelligence throughput on a synthetic inventory.

use std::hint::black_box;
use std::time::Instant;
use weavatrix_seo_content::audit;
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Heading, Indexability,
    Inventory, MediaKind,
};

fn main() {
    let inventory = synthetic(48);
    let started = Instant::now();
    let pass = audit(&inventory);
    println!(
        "weavatrix-seo content pages={} profiles={} chunks={} families={} near={} findings={} in {:?}",
        inventory.pages.len(),
        pass.profiles.len(),
        pass.chunks.len(),
        pass.families.len(),
        pass.near_duplicates.len(),
        pass.findings.len(),
        started.elapsed()
    );
    black_box(pass);
}

fn synthetic(count: usize) -> Inventory {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = (0..count)
        .map(|index| {
            let path = if index % 6 == 0 {
                format!("/category/electrician/city-{index}")
            } else {
                format!("/p{index}")
            };
            let fact = if index % 3 == 0 {
                format!("permit {index} licensed electrician Clark County")
            } else {
                "same-day service filler copy used across the family".into()
            };
            let h1 = format!("Electrician {index}");
            let url = AbsoluteUrl::parse(&format!("https://bench.test{path}")).expect("url");
            ExtractedPage {
                url: url.clone(),
                requested: url,
                status: 200,
                redirects: Vec::new(),
                content_type: Some("text/html".into()),
                media: MediaKind::Html,
                canonical: None,
                robots: Vec::new(),
                title: Some(h1.clone()),
                description: None,
                html_lang: Some("en".into()),
                alternates: Vec::new(),
                headings: vec![Heading {
                    level: 1,
                    text: h1.clone(),
                }],
                links: Vec::new(),
                link_refs: Vec::new(),
                images: Vec::new(),
                json_ld: Vec::new(),
                text: fact,
                heading_text: h1.clone(),
                main_text: String::new(),
                payload: String::new(),
                arbitrary_script: String::new(),
                og_title: None,
                og_description: None,
                og_image: None,
                headers: Vec::new(),
                csp_meta: None,
                body_bytes: 64,
                fetch_ms: 1,
                has_main: true,
                unlabeled_controls: 0,
                content_hash: ContentHash::of_str(&h1),
                indexability: Indexability::Indexable,
                in_sitemap: true,
                linked_from_page: true,
                evidence: Evidence::http(),
            }
            .finalize()
        })
        .collect();
    inventory
}
