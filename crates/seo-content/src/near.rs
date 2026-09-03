//! Near-duplicate clustering via word shingles, `MinHash`, and LSH bands.

use crate::tokens::{shingles, tokens};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use weavatrix_seo_model::{
    ContentHash, ExtractedPage, Finding, FindingFamily, Indexability, Inventory, Locator,
    NearDuplicateGroup,
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
    let mut page_shingles: Vec<BTreeSet<String>> = Vec::new();
    for page in &pages {
        let toks = tokens(&page.visible_text());
        let shingle_list = shingles(&toks, 3);
        page_shingles.push(shingle_list.iter().cloned().collect());
        signatures.push((
            page.url.to_string(),
            page.content_hash,
            minhash(&shingle_list),
        ));
    }
    let mut lsh_buckets: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, (_, _, sig)) in signatures.iter().enumerate() {
        for key in band_keys(sig) {
            lsh_buckets.entry(key).or_default().push(index);
        }
    }
    let mut pairs: BTreeSet<(usize, usize)> = BTreeSet::new();
    for members in lsh_buckets.values() {
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
    let mut dsu = Dsu::new(signatures.len());
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
        dsu.union(left, right);
    }
    let mut clustered: Vec<NearDuplicateGroup> = Vec::new();
    let mut buckets: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for index in 0..signatures.len() {
        buckets.entry(dsu.find(index)).or_default().push(index);
    }
    for members in buckets.into_values() {
        if members.len() < 2 {
            continue;
        }
        let mut urls: Vec<String> = members
            .iter()
            .map(|index| signatures[*index].0.clone())
            .collect();
        urls.sort();
        urls.dedup();
        if urls.len() < 2 {
            continue;
        }
        let mut max_sim = 0_u16;
        for ((left, right), sim) in &similarity {
            if members.contains(left) && members.contains(right) {
                max_sim = max_sim.max(*sim);
            }
        }
        let witnesses = cluster_witnesses(&page_shingles, &members);
        clustered.push(NearDuplicateGroup {
            urls,
            similarity: max_sim,
            witness: witnesses.first().cloned(),
            witnesses,
        });
    }
    clustered.sort_by(|left, right| left.urls.cmp(&right.urls));
    let mut findings = Vec::new();
    for group in &clustered {
        findings.push(
            Finding::from_rule(
                FindingFamily::Dup,
                2,
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

struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(len: usize) -> Self {
        Self {
            parent: (0..len).collect(),
        }
    }

    fn find(&mut self, index: usize) -> usize {
        let mut root = index;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut cursor = index;
        while cursor != root {
            let next = self.parent[cursor];
            self.parent[cursor] = root;
            cursor = next;
        }
        root
    }

    fn union(&mut self, left: usize, right: usize) {
        let left = self.find(left);
        let right = self.find(right);
        if left != right {
            self.parent[right] = left;
        }
    }
}

fn cluster_witnesses(shingles: &[BTreeSet<String>], members: &[usize]) -> Vec<String> {
    let Some((first, rest)) = members.split_first() else {
        return Vec::new();
    };
    let mut shared = shingles.get(*first).cloned().unwrap_or_default();
    for index in rest {
        if let Some(other) = shingles.get(*index) {
            shared = shared.intersection(other).cloned().collect();
        }
    }
    let mut witnesses: Vec<String> = shared.into_iter().collect();
    witnesses.sort_by_key(|item| std::cmp::Reverse(item.len()));
    witnesses.truncate(3);
    witnesses
}

#[cfg(test)]
mod tests {
    use super::{estimated_jaccard, minhash};
    use crate::tokens::{shingles, tokens};

    #[test]
    fn dsu_merges_a_transitive_cluster() {
        let mut dsu = super::Dsu::new(3);
        dsu.union(0, 1);
        dsu.union(1, 2);
        assert_eq!(dsu.find(0), dsu.find(2));
    }

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
