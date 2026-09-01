//! Per-page content profiles. Diagnostics, not authorship claims.

use crate::tokens::{FILLER_PHRASES, is_fact_token, is_function_word, ratio, sentences, tokens};
use weavatrix_seo_model::{
    ContentProfile, ExtractedPage, Indexability, Inventory, SignalLevel, SyntheticStyle,
};

/// Builds a content profile for every indexable HTML page with visible text.
#[must_use]
pub fn profiles(inventory: &Inventory) -> Vec<ContentProfile> {
    inventory
        .pages
        .iter()
        .filter(|page| {
            page.status == 200
                && page.indexability == Indexability::Indexable
                && !page.visible_text().trim().is_empty()
        })
        .map(profile_page)
        .collect()
}

fn profile_page(page: &ExtractedPage) -> ContentProfile {
    let text = page.visible_text();
    let toks = tokens(&text);
    let sent = sentences(&text);
    let mattr = mattr(&toks);
    let mtld = mtld(&toks);
    let term_entropy = entropy(&toks);
    let repetition = repeated_share(&toks);
    let entity_density = entity_share(&text, &toks);
    let numeric_density = numeric_share(&toks);
    let fact_density = fact_share(&toks);
    let function_word_ratio = function_share(&toks);
    let genericity = function_word_ratio;
    let specificity = specific_share(&toks);
    let filler_phrase_ratio = filler_share(&text);
    let (avg_sentence_length, long_sentence_share, sentence_variance) = sentence_stats(&sent);
    let sentence_redundancy = redundancy_level(repetition, &sent);
    let topic_cohesion = cohesion_level(term_entropy, repetition);
    let witness = toks.iter().find(|token| token.len() > 5).cloned();
    ContentProfile {
        url: page.url.to_string(),
        mattr,
        mtld,
        term_entropy,
        repetition,
        entity_density,
        numeric_density,
        fact_density,
        genericity,
        specificity,
        sentence_redundancy,
        topic_cohesion,
        function_word_ratio,
        filler_phrase_ratio,
        avg_sentence_length,
        long_sentence_share,
        synthetic: SyntheticStyle {
            semantic_redundancy: sentence_redundancy,
            sentence_variance,
            genericity: band(genericity),
            factual_specificity: band(specificity),
            template_reuse: SignalLevel::Unmeasured,
            authorship: "UNMEASURED".into(),
        },
        witness,
    }
}

fn mattr(toks: &[String]) -> Option<u16> {
    const WINDOW: usize = 50;
    if toks.len() < 20 {
        return ratio(unique_count(toks), toks.len());
    }
    if toks.len() < WINDOW {
        return ratio(unique_count(toks), toks.len());
    }
    let mut sum = 0_usize;
    let mut windows = 0_usize;
    for window in toks.windows(WINDOW) {
        sum += unique_count(window);
        windows += 1;
    }
    ratio(sum, windows.saturating_mul(WINDOW))
}

fn mtld(toks: &[String]) -> Option<u16> {
    if toks.len() < 20 {
        return None;
    }
    let mut factors = 0_usize;
    let mut seen = std::collections::BTreeSet::new();
    let mut run = 0_usize;
    for token in toks {
        run += 1;
        seen.insert(token);
        if run >= 10 {
            let ttr = (seen.len() * 100) / run;
            if ttr < 72 {
                factors += 1;
                seen.clear();
                run = 0;
            }
        }
    }
    if factors == 0 {
        return Some(100);
    }
    ratio(toks.len(), factors.saturating_mul(10))
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn entropy(toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    for token in toks {
        *counts.entry(token.as_str()).or_default() += 1;
    }
    let len = toks.len() as f64;
    let mut h = 0.0;
    for count in counts.values() {
        let p = f64::from(u32::try_from(*count).unwrap_or(0)) / len;
        if p > 0.0 {
            h -= p * p.log2();
        }
    }
    let max = (counts.len() as f64).log2();
    if max == 0.0 {
        return Some(0);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(((h / max) * 100.0).round().clamp(0.0, 100.0) as u16)
}

fn repeated_share(toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    let unique = unique_count(toks);
    ratio(toks.len().saturating_sub(unique), toks.len())
}

fn unique_count(toks: &[String]) -> usize {
    let mut set = std::collections::BTreeSet::new();
    for token in toks {
        set.insert(token);
    }
    set.len()
}

fn entity_share(text: &str, toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    let titled = text
        .split_whitespace()
        .filter(|word| {
            word.chars()
                .next()
                .is_some_and(|ch| ch.is_uppercase() && ch.is_alphabetic())
        })
        .count();
    ratio(titled, toks.len())
}

fn numeric_share(toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    ratio(
        toks.iter()
            .filter(|token| token.chars().any(|ch| ch.is_ascii_digit()))
            .count(),
        toks.len(),
    )
}

fn fact_share(toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    ratio(
        toks.iter().filter(|token| is_fact_token(token)).count(),
        toks.len(),
    )
}

fn function_share(toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    ratio(
        toks.iter().filter(|token| is_function_word(token)).count(),
        toks.len(),
    )
}

fn specific_share(toks: &[String]) -> Option<u16> {
    if toks.is_empty() {
        return None;
    }
    ratio(
        toks.iter()
            .filter(|token| token.len() > 8 || token.chars().any(|ch| ch.is_ascii_digit()))
            .count(),
        toks.len(),
    )
}

fn filler_share(text: &str) -> Option<u16> {
    let hay = text.to_ascii_lowercase();
    if hay.is_empty() {
        return None;
    }
    let hits = FILLER_PHRASES
        .iter()
        .filter(|phrase| hay.contains(*phrase))
        .count();
    ratio(hits, FILLER_PHRASES.len())
}

fn sentence_stats(sent: &[String]) -> (Option<u16>, Option<u16>, SignalLevel) {
    if sent.is_empty() {
        return (None, None, SignalLevel::Unmeasured);
    }
    let lengths: Vec<usize> = sent
        .iter()
        .map(|sentence| sentence.split_whitespace().count())
        .collect();
    let sum: usize = lengths.iter().sum();
    let avg =
        Some(u16::try_from((sum / lengths.len()).min(usize::from(u16::MAX))).unwrap_or(u16::MAX));
    let long = lengths.iter().filter(|len| **len > 30).count();
    let long_share = ratio(long, lengths.len());
    let variance = length_variance(&lengths);
    (avg, long_share, variance)
}

#[allow(clippy::cast_precision_loss)]
fn length_variance(lengths: &[usize]) -> SignalLevel {
    if lengths.len() < 2 {
        return SignalLevel::Unmeasured;
    }
    let mean = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
    let var = lengths
        .iter()
        .map(|len| {
            let d = *len as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / lengths.len() as f64;
    if var < 8.0 {
        SignalLevel::Low
    } else if var < 40.0 {
        SignalLevel::Medium
    } else {
        SignalLevel::High
    }
}

fn redundancy_level(repetition: Option<u16>, sent: &[String]) -> SignalLevel {
    if sent.len() < 2 && repetition.is_none() {
        return SignalLevel::Unmeasured;
    }
    match repetition.unwrap_or(0) {
        0..=20 => SignalLevel::Low,
        21..=45 => SignalLevel::Medium,
        _ => SignalLevel::High,
    }
}

fn cohesion_level(entropy: Option<u16>, repetition: Option<u16>) -> SignalLevel {
    match (entropy, repetition) {
        (None, None) => SignalLevel::Unmeasured,
        (Some(entropy), _) if entropy >= 70 => SignalLevel::High,
        (Some(entropy), Some(rep)) if entropy >= 45 && rep <= 40 => SignalLevel::Medium,
        _ => SignalLevel::Low,
    }
}

fn band(value: Option<u16>) -> SignalLevel {
    match value {
        None => SignalLevel::Unmeasured,
        Some(0..=33) => SignalLevel::Low,
        Some(34..=66) => SignalLevel::Medium,
        Some(_) => SignalLevel::High,
    }
}

#[cfg(test)]
mod tests {
    use super::mattr;
    use crate::tokens::tokens;

    #[test]
    fn diverse_copy_has_higher_mattr_than_repeats() {
        let diverse =
            tokens("Licensed electrician serving Vancouver Camas Ridgefield and Battle Ground");
        let repeats = tokens("Great great great great great great great great great great");
        assert!(mattr(&diverse).unwrap() > mattr(&repeats).unwrap());
    }
}
