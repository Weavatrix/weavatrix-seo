//! Meta and link extraction.

use super::ExtractedPageDraft;
use super::tag::{Tag, attr};
use weavatrix_seo_model::Alternate;

pub fn apply_meta(draft: &mut ExtractedPageDraft, tag: &Tag) {
    let content = attr(tag, "content");
    if let Some(equiv) = attr(tag, "http-equiv")
        && equiv.eq_ignore_ascii_case("content-security-policy")
    {
        draft.csp_meta.clone_from(&content);
    }
    if let Some(name) = attr(tag, "name") {
        match name.to_ascii_lowercase().as_str() {
            "description" => draft.description.clone_from(&content),
            "robots" => {
                if let Some(content) = content.clone() {
                    draft.robots.push(content);
                }
            }
            _ => {}
        }
    }
    if let Some(property) = attr(tag, "property").or_else(|| attr(tag, "name")) {
        match property.to_ascii_lowercase().as_str() {
            "og:title" => draft.og_title = content,
            "og:description" => draft.og_description = content,
            "og:image" => draft.og_image = content,
            _ => {}
        }
    }
}

pub fn apply_link(draft: &mut ExtractedPageDraft, tag: &Tag) {
    let rel = attr(tag, "rel").map(|value| value.to_ascii_lowercase());
    let href = attr(tag, "href");
    match (rel.as_deref(), href) {
        (Some("canonical"), Some(href)) => draft.canonical = Some(href),
        (Some("alternate"), Some(href)) => {
            if let Some(hreflang) = attr(tag, "hreflang") {
                draft.alternates.push(Alternate { hreflang, href });
            }
        }
        _ => {}
    }
}
