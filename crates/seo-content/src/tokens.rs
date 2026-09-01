//! Shared tokenisation for content intelligence.

pub fn tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| part.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let trimmed = current.trim();
            if trimmed.split_whitespace().count() >= 3 {
                out.push(trimmed.to_owned());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if trimmed.split_whitespace().count() >= 3 {
        out.push(trimmed.to_owned());
    }
    out
}

pub fn shingles(tokens: &[String], width: usize) -> Vec<String> {
    if tokens.len() < width {
        return tokens.to_vec();
    }
    tokens
        .windows(width)
        .map(|window| window.join(" "))
        .collect()
}

pub fn ratio(part: usize, whole: usize) -> Option<u16> {
    if whole == 0 {
        return None;
    }
    let scaled = part.saturating_mul(100) / whole;
    Some(u16::try_from(scaled.min(100)).unwrap_or(100))
}

pub const FUNCTION_WORDS: &[&str] = &[
    "the", "a", "an", "of", "to", "in", "for", "on", "with", "at", "by", "from", "as", "is", "are",
    "was", "were", "be", "been", "this", "that", "it", "and", "or", "but", "we", "you", "our",
    "your", "their",
];

pub const FILLER_PHRASES: &[&str] = &[
    "in order to",
    "it is important",
    "when it comes to",
    "in today's",
    "a wide range of",
    "comprehensive",
    "cutting-edge",
    "leverage",
    "best-in-class",
    "world-class",
    "at the end of the day",
];

pub fn is_function_word(token: &str) -> bool {
    FUNCTION_WORDS.contains(&token)
}

pub fn is_fact_token(token: &str) -> bool {
    token.chars().any(|ch| ch.is_ascii_digit())
        || matches!(
            token,
            "licensed"
                | "license"
                | "insured"
                | "certified"
                | "warranty"
                | "permit"
                | "bonded"
                | "price"
                | "same-day"
                | "haifa"
                | "vancouver"
        )
}

#[cfg(test)]
mod tests {
    use super::{ratio, tokens};

    #[test]
    fn splits_words_and_keeps_digits() {
        assert_eq!(
            tokens("Licensed electrician in Vancouver WA 98682"),
            ["licensed", "electrician", "in", "vancouver", "wa", "98682"]
        );
        assert_eq!(ratio(1, 4), Some(25));
        assert_eq!(ratio(0, 0), None);
    }
}
