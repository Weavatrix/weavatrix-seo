//! Builds `weavatrix-semantic` page profiles. Does not run cosine similarity.

#![forbid(unsafe_code)]

use weavatrix_graph::NodeId;
use weavatrix_semantic::{SemanticError, SeoPage};
use weavatrix_seo_model::{Indexability, Inventory};

/// Converts crawled pages into SEO link-policy profiles.
///
/// # Errors
///
/// Returns a semantic error when a profile cannot be constructed.
pub fn profiles(inventory: &Inventory) -> Result<Vec<SeoPage>, SemanticError> {
    let mut pages = Vec::new();
    for page in &inventory.pages {
        let node = NodeId::new(format!("page:{}", page.url))?;
        let canonical = page
            .canonical
            .clone()
            .unwrap_or_else(|| page.url.to_string());
        let mut profile = SeoPage::new(node, page.url.host(), canonical)?;
        if let Some(lang) = &page.html_lang {
            profile = profile.with_language(lang)?;
        }
        let eligible = page.indexability == Indexability::Indexable && page.status == 200;
        profile = profile
            .with_source_eligible(eligible)
            .with_target_eligible(eligible)
            .with_orphan(!page.linked_from_page && page.url.path() != "/");
        pages.push(profile);
    }
    Ok(pages)
}
