//! Candidate-page and chunk retrieval. Rust computes similarity; the LLM reasons.

use weavatrix_seo_model::{AuditReport, CandidatePage, Chunk, Indexability};
use weavatrix_seo_semantic::embed;

/// Retrieves candidate pages for a query using the first-party lexical model.
#[must_use]
pub fn retrieve(report: &AuditReport, query: &str, limit: usize) -> Vec<CandidatePage> {
    let Some(query_vec) = embed(query) else {
        return Vec::new();
    };
    let query_tokens: Vec<String> = query
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| part.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect();
    let mut scored = Vec::new();
    for page in report
        .inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.indexability == Indexability::Indexable)
    {
        let text = page.visible_text();
        let Some(page_vec) = embed(&text) else {
            continue;
        };
        let lexical = cosine_pct(&query_vec, &page_vec);
        let overlap = token_overlap(&query_tokens, &text);
        let language = page.html_lang.clone();
        let mut why = Vec::new();
        if lexical >= 40 {
            why.push(format!("lexical {lexical}"));
        }
        if overlap >= 1 {
            why.push(format!("{overlap} query tokens present"));
        }
        if why.is_empty() {
            continue;
        }
        scored.push(CandidatePage {
            url: page.url.to_string(),
            lexical,
            semantic: None,
            entities: overlapping_entities(&query_tokens, &text),
            language,
            impressions: report.opportunities.iter().find_map(|item| {
                (item.subject == page.url.to_string())
                    .then_some(item.axes.raw_impressions)
                    .flatten()
            }),
            why,
        });
    }
    scored.sort_by(|left, right| {
        right
            .lexical
            .cmp(&left.lexical)
            .then_with(|| right.entities.len().cmp(&left.entities.len()))
    });
    scored.truncate(limit.clamp(1, 50));
    scored
}

/// Pages most similar to `url` in the same report.
#[must_use]
pub fn similar(report: &AuditReport, url: &str, limit: usize) -> Vec<CandidatePage> {
    let needle = url.trim_end_matches('/');
    let Some(page) = report
        .inventory
        .pages
        .iter()
        .find(|page| page.url.to_string().trim_end_matches('/') == needle)
    else {
        return Vec::new();
    };
    retrieve(report, &page.visible_text(), limit)
        .into_iter()
        .filter(|item| item.url.trim_end_matches('/') != needle)
        .collect()
}

/// Chunks that best answer a query.
#[must_use]
pub fn chunks_for(report: &AuditReport, query: &str, limit: usize) -> Vec<Chunk> {
    let Some(intelligence) = &report.intelligence else {
        return Vec::new();
    };
    let Some(query_vec) = embed(query) else {
        return intelligence.chunks.iter().take(limit).cloned().collect();
    };
    let mut scored: Vec<(u16, Chunk)> = intelligence
        .chunks
        .iter()
        .filter_map(|chunk| {
            let vector = embed(&chunk.text)?;
            Some((cosine_pct(&query_vec, &vector), chunk.clone()))
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0));
    scored.truncate(limit.clamp(1, 50));
    scored
        .into_iter()
        .map(|(score, mut chunk)| {
            chunk.relevance = Some(score);
            chunk.retrieval_model = Some("wvx-seo-lexhash-v1".into());
            chunk.why = Some(format!("lexical {score}"));
            chunk
        })
        .collect()
}

fn cosine_pct(left: &[f32], right: &[f32]) -> u16 {
    let mut dot = 0.0_f64;
    let mut ln = 0.0_f64;
    let mut rn = 0.0_f64;
    for (a, b) in left.iter().zip(right) {
        dot += f64::from(*a) * f64::from(*b);
        ln += f64::from(*a) * f64::from(*a);
        rn += f64::from(*b) * f64::from(*b);
    }
    if ln == 0.0 || rn == 0.0 {
        return 0;
    }
    let cosine = (dot / (ln.sqrt() * rn.sqrt())).clamp(0.0, 1.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    ((cosine * 100.0).round() as u16)
}

fn token_overlap(query: &[String], text: &str) -> usize {
    let hay = text.to_ascii_lowercase();
    query
        .iter()
        .filter(|token| hay.contains(token.as_str()))
        .count()
}

fn overlapping_entities(query: &[String], text: &str) -> Vec<String> {
    let hay = text.to_ascii_lowercase();
    query
        .iter()
        .filter(|token| token.len() > 3 && hay.contains(token.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::cosine_pct;

    #[test]
    fn identical_vectors_are_100() {
        assert_eq!(cosine_pct(&[1.0, 0.0], &[1.0, 0.0]), 100);
        assert_eq!(cosine_pct(&[1.0, 0.0], &[0.0, 1.0]), 0);
    }
}
