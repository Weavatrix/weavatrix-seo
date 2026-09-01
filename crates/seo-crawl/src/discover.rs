//! robots.txt and sitemap discovery.

use crate::{Fetcher, Robots, parse_sitemap};
use std::collections::{BTreeSet, VecDeque};
use weavatrix_seo_model::AbsoluteUrl;

/// Fetches `/llms.txt` status. A 404 is measured absence, not a transport miss.
#[must_use]
pub fn fetch_llms_txt(fetcher: &Fetcher, seed: &AbsoluteUrl) -> Option<u16> {
    let Ok(url) = AbsoluteUrl::parse(&format!("{}/llms.txt", seed.origin())) else {
        return None;
    };
    fetcher.get(&url).ok().map(|response| response.status)
}

pub fn fetch_robots(fetcher: &Fetcher, seed: &AbsoluteUrl) -> Robots {
    let Ok(url) = AbsoluteUrl::parse(&format!("{}/robots.txt", seed.origin())) else {
        return Robots::default();
    };
    fetcher
        .get(&url)
        .ok()
        .filter(|response| response.status == 200)
        .map_or_else(Robots::default, |response| {
            Robots::parse(&response.body, "weavatrix-seo")
        })
}

pub fn fetch_sitemaps(fetcher: &Fetcher, seed: &AbsoluteUrl, robots: &Robots) -> Vec<AbsoluteUrl> {
    let mut declared = robots.sitemaps.clone();
    if declared.is_empty() {
        declared.push(format!("{}/sitemap.xml", seed.origin()));
        declared.push(format!("{}/sitemap.xml.gz", seed.origin()));
        declared.push(format!("{}/sitemap_index.xml", seed.origin()));
    }
    let mut queue = VecDeque::new();
    for item in declared {
        if let Ok(url) = AbsoluteUrl::parse(&item).or_else(|_| seed.join(&item)) {
            queue.push_back(url);
        }
    }
    let mut visited = BTreeSet::new();
    let mut locs = Vec::new();
    let mut documents = 0_usize;
    while let Some(url) = queue.pop_front() {
        if documents >= 64 || !visited.insert(url.clone()) {
            continue;
        }
        let Ok(response) = fetcher.get(&url) else {
            continue;
        };
        if response.status != 200 {
            continue;
        }
        documents += 1;
        let body = &response.body;
        let nested = parse_sitemap(body, seed);
        if is_sitemap_index(body) {
            queue.extend(nested);
            continue;
        }
        if is_urlset(body) || !nested.is_empty() {
            locs.extend(nested);
        }
    }
    locs.sort();
    locs.dedup();
    locs
}

fn is_sitemap_index(body: &str) -> bool {
    body.contains("<sitemapindex")
}

fn is_urlset(body: &str) -> bool {
    body.contains("<urlset")
}
