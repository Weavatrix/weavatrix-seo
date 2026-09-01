//! Near-duplicate clustering via word shingles, `MinHash`, and LSH bands.

use crate::tokens::{shingles, tokens};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use weavatrix_seo_model::{
    ContentHash, ExtractedPage, Finding, FindingFamily, Indexability, Inventory, Locator,
    NearDuplicateGroup, Severity,
};

const HASHES: usize = 64;
const BAND_ROWS: usize = 4;
const SIMILARITY_FLOOR: u16 = 70;

/// Clusters indexable pages that are similar but not byte-identical.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn near_duplicates(inventory: &Inventory) -> (Vec<NearDuplicateGroup>, Vec<Finding>) {
    let pages: Vec<&ExtractedPage> = inventory
        .pages
        .iter()
        .filter(|page| {
            page.status == 200
                && page.indexability == Indexability::Indexable
                && !page.visible_text().trim().is_empty()
        })
        .collect();
    let mut signatures = Vec::new();
    for page in &pages {
        let toks = tokens(&page.visible_text());
        let shingles = shingles(&toks, 3);
        signatures.push((page.url.to_string(), page.content_hash, minhash(&shingles)));
    }
    let mut buckets: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, (_, _, sig)) in signatures.iter().enumerate() {
        for key in band_keys(sig) {
            buckets.entry(key).or_default().push(index);
        }
    }
    let mut pairs: BTreeSet<(usize, usize)> = BTreeSet::new();
    for members in buckets.values() {
        if members.len() < 2 {
            continue;
        }
        let mut unique = members.clone();
        unique.sort_unstable();
        unique.dedup();
        for left in 0..unique.len() {
            for right in (left + 1)..unique.len() {
                pairs.insert((unique[left], unique[right]));
            }
        }
    }
    let mut groups: BTreeMap<u128, Vec<usize>> = BTreeMap::new();
    let mut similarity: BTreeMap<(usize, usize), u16> = BTreeMap::new();
    for (left, right) in pairs {
        if signatures[left].1 == signatures[right].1 {
            continue;
        }
        let sim = estimated_jaccard(&signatures[left].2, &signatures[right].2);
        if sim < SIMILARITY_FLOOR {
            continue;
        }
        similarity.insert((left, right), sim);
        let key = union_key(left, right);
        groups.entry(key).or_default().extend([left, right]);
    }
    let mut clustered: Vec<NearDuplicateGroup> = Vec::new();
    let mut seen: BTreeSet<usize> = BTreeSet::new();
    for members in groups.into_values() {
        let mut urls = Vec::new();
        let mut max_sim = 0_u16;
        for index in members {
            if seen.insert(index) {
                urls.push(signatures[index].0.clone());
            }
        }
        if urls.len() < 2 {
            continue;
        }
        urls.sort();
        urls.dedup();
        for ((left, right), sim) in &similarity {
            if urls.contains(&signatures[*left].0) && urls.contains(&signatures[*right].0) {
                max_sim = max_sim.max(*sim);
            }
        }
        clustered.push(NearDuplicateGroup {
            urls,
            similarity: max_sim,
            witness: None,
        });
    }
    clustered.sort_by(|left, right| left.urls.cmp(&right.urls));
    let mut findings = Vec::new();
    for group in &clustered {
        findings.push(
            Finding::new(
                FindingFamily::Dup,
                2,
                Severity::Info,
                &group.urls.join(" "),
                format!(
                    "{} URLs are near-duplicates (~{}% MinHash overlap)",
                    group.urls.len(),
                    group.similarity
                ),
                Locator::Url(group.urls[0].clone()),
                weavatrix_seo_model::Evidence::http(),
            )
            .with_affected(group.urls.clone())
            .explained(
                "Near-duplicate bodies split attention even when they are not byte-identical.",
                "Differentiate unique facts or canonicalize the cluster.",
                "Each remaining indexable URL has distinct remaining content after template tokens.",
            ),
        );
    }
    (clustered, findings)
}

fn minhash(shingles: &[String]) -> [u64; HASHES] {
    let mut mins = [u64::MAX; HASHES];
    if shingles.is_empty() {
        return mins;
    }
    for shingle in shingles {
        let base = ContentHash::of_str(shingle).hex();
        for (index, slot) in mins.iter_mut().enumerate() {
            let mixed = ContentHash::of_str(&format!("{index}:{base}")).hex();
            let value = u64::from_str_radix(&mixed[..16], 16).unwrap_or(u64::MAX);
            if value < *slot {
                *slot = value;
            }
        }
    }
    mins
}

fn band_keys(sig: &[u64; HASHES]) -> Vec<String> {
    sig.chunks(BAND_ROWS)
        .enumerate()
        .map(|(band, rows)| {
            let mut key = format!("{band}:");
            for value in rows {
                let _ = write!(key, "{value:x}-");
            }
            key
        })
        .collect()
}

fn estimated_jaccard(left: &[u64; HASHES], right: &[u64; HASHES]) -> u16 {
    let matched = left
        .iter()
        .zip(right.iter())
        .filter(|(a, b)| a == b)
        .count();
    u16::try_from((matched * 100) / HASHES).unwrap_or(100)
}

fn union_key(left: usize, right: usize) -> u128 {
    let (a, b) = if left < right {
        (left, right)
    } else {
        (right, left)
    };
    (u128::from(u64::try_from(a).unwrap_or(0)) << 64) | u128::from(u64::try_from(b).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{estimated_jaccard, minhash};
    use crate::tokens::{shingles, tokens};

    #[test]
    fn similar_copy_has_higher_minhash_overlap_than_unrelated() {
        let left = minhash(&shingles(
            &tokens("Licensed electrician in Vancouver WA serving Clark County"),
            3,
        ));
        let right = minhash(&shingles(
            &tokens("Licensed electrician in Camas WA serving Clark County"),
            3,
        ));
        let other = minhash(&shingles(
            &tokens("Tomato soup recipes and garden soil compost"),
            3,
        ));
        assert!(estimated_jaccard(&left, &right) > estimated_jaccard(&left, &other));
    }
}
