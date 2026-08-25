//! First-party lexical hash embeddings. Not a ranking model.

use weavatrix_seo_model::ContentHash;

/// Model id stored on semantic edges.
pub const MODEL: &str = "wvx-seo-lexhash-v1";

/// Embedding width.
pub const DIM: usize = 64;

/// Hashed bag-of-tokens vector. Empty text yields `None`.
#[must_use]
pub fn embed(text: &str) -> Option<Vec<f32>> {
    let mut values = vec![0.0_f32; DIM];
    let mut saw = false;
    for token in tokens(text) {
        saw = true;
        let hex = ContentHash::of_str(&token).hex();
        let index = usize::from_str_radix(&hex[..4], 16).unwrap_or(0) % DIM;
        let sign = if hex.as_bytes()[4] & 1 == 0 {
            1.0
        } else {
            -1.0
        };
        values[index] += sign;
    }
    saw.then_some(values)
}

pub(crate) fn cosine(left: &[f32], right: &[f32]) -> f64 {
    let mut dot = 0.0;
    let mut ln = 0.0;
    let mut rn = 0.0;
    for (a, b) in left.iter().zip(right) {
        dot += f64::from(*a) * f64::from(*b);
        ln += f64::from(*a) * f64::from(*a);
        rn += f64::from(*b) * f64::from(*b);
    }
    if ln == 0.0 || rn == 0.0 {
        0.0
    } else {
        dot / (ln.sqrt() * rn.sqrt())
    }
}

fn tokens(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| part.len() >= 2)
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
mod tests {
    use super::embed;

    #[test]
    fn similar_copy_is_closer_than_unrelated() {
        let left = embed("Electrician in Vancouver WA licensed contractor").unwrap();
        let right = embed("Electrician in Camas WA licensed contractor").unwrap();
        let other = embed("Tomato soup recipes and garden soil").unwrap();
        assert!(super::cosine(&left, &right) > super::cosine(&left, &other));
    }
}
