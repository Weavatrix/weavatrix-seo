//! Chunking of page text for retrieval and citation suitability.

use crate::tokens::{is_fact_token, ratio, sentences, tokens};
use weavatrix_seo_model::{Chunk, ExtractedPage, Indexability, Inventory, chunk_id};

/// Splits indexable pages into heading-bounded chunks.
#[must_use]
pub fn chunks(inventory: &Inventory) -> Vec<Chunk> {
    let mut out = Vec::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && !page.visible_text().trim().is_empty()
    }) {
        out.extend(chunk_page(page));
    }
    out
}

fn chunk_page(page: &ExtractedPage) -> Vec<Chunk> {
    let url = page.url.to_string();
    if page.headings.len() >= 2 {
        let mut out = Vec::new();
        for (index, heading) in page.headings.iter().enumerate() {
            let next = page
                .headings
                .get(index + 1)
                .map_or("", |item| item.text.as_str());
            let body = slice_between(&page.text, &heading.text, next);
            out.push(build_chunk(&url, index, &heading.text, &body));
        }
        return out;
    }
    let sent = sentences(&page.visible_text());
    if sent.is_empty() {
        return vec![build_chunk(
            &url,
            0,
            page.title.as_deref().unwrap_or("page"),
            &page.text,
        )];
    }
    let mut out = Vec::new();
    for (index, group) in sent.chunks(3).enumerate() {
        let heading = group[0].clone();
        let body = group.join(" ");
        out.push(build_chunk(&url, index, &heading, &body));
    }
    out
}

fn slice_between(text: &str, start: &str, next: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let start_at = start
        .split_whitespace()
        .next()
        .and_then(|token| lower.find(&token.to_ascii_lowercase()))
        .unwrap_or(0);
    let end = if next.is_empty() {
        text.len()
    } else {
        next.split_whitespace()
            .next()
            .and_then(|token| lower[start_at..].find(&token.to_ascii_lowercase()))
            .map_or(text.len(), |rel| start_at + rel)
    };
    text.get(start_at..end.max(start_at))
        .unwrap_or(text)
        .trim()
        .to_owned()
}

fn build_chunk(url: &str, index: usize, heading: &str, text: &str) -> Chunk {
    let toks = tokens(text);
    let fact = toks.iter().filter(|token| is_fact_token(token)).count();
    let specific = toks
        .iter()
        .filter(|token| token.len() > 7 || token.chars().any(|ch| ch.is_ascii_digit()))
        .count();
    let cohesion = ratio(unique_ratio_proxy(&toks), 100);
    let self_contained = if heading.split_whitespace().count() >= 2 && toks.len() >= 8 {
        Some(80)
    } else if toks.len() >= 4 {
        Some(50)
    } else {
        Some(20)
    };
    let answer_density = if text.contains('?') || looks_like_answer(text) {
        Some(80)
    } else {
        ratio(fact, toks.len().max(1))
    };
    let specificity = ratio(specific, toks.len().max(1));
    let citation = match (self_contained, specificity, answer_density) {
        (Some(a), Some(b), Some(c)) => {
            Some(u16::try_from((u32::from(a) + u32::from(b) + u32::from(c)) / 3).unwrap_or(100))
        }
        _ => None,
    };
    Chunk {
        id: chunk_id(url, index),
        url: url.to_owned(),
        heading: heading.trim().to_owned(),
        text: text.trim().to_owned(),
        cohesion,
        self_contained,
        answer_density,
        specificity,
        citation_suitability: citation,
        witness: toks.iter().find(|token| token.len() > 5).cloned(),
    }
}

fn unique_ratio_proxy(toks: &[String]) -> usize {
    let mut set = std::collections::BTreeSet::new();
    for token in toks {
        set.insert(token);
    }
    if toks.is_empty() {
        0
    } else {
        (set.len() * 100) / toks.len()
    }
}

fn looks_like_answer(text: &str) -> bool {
    let hay = text.to_ascii_lowercase();
    hay.contains(" because ")
        || hay.contains(" costs ")
        || hay.contains(" takes ")
        || hay.contains(" located ")
        || hay.contains(" licensed ")
}

#[cfg(test)]
mod tests {
    use super::looks_like_answer;

    #[test]
    fn licensed_copy_looks_like_an_answer() {
        assert!(looks_like_answer(
            "The specialist is licensed in Washington."
        ));
        assert!(!looks_like_answer("Welcome."));
    }
}
