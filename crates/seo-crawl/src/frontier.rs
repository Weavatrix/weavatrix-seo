//! Link-first crawl frontier. Sitemap URLs stay cold until the hot queue drains.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use weavatrix_seo_model::AbsoluteUrl;

/// Three-lane URL schedule.
#[derive(Debug, Default)]
pub struct Frontier {
    urgent: VecDeque<(AbsoluteUrl, u32)>,
    linked: VecDeque<(AbsoluteUrl, u32)>,
    sitemap: VecDeque<(AbsoluteUrl, u32)>,
    scheduled: BTreeSet<AbsoluteUrl>,
    sampled_cities: BTreeMap<String, u8>,
}

impl Frontier {
    /// Enqueues the seed as urgent.
    pub fn seed(&mut self, url: AbsoluteUrl) {
        self.push_urgent(url, 0);
    }

    /// Enqueues a sitemap loc without blocking link discovery.
    /// City variants are sampled even when they are sitemap-only.
    pub fn push_sitemap(&mut self, url: AbsoluteUrl) {
        if !self.scheduled.insert(url.clone()) {
            return;
        }
        if self.sample_city(&url) {
            self.urgent.push_back((url, 0));
            return;
        }
        self.sitemap.push_back((url, 0));
    }

    /// Marks a URL as seen without enqueueing (redirect hops, already fetched).
    pub fn remember(&mut self, url: AbsoluteUrl) {
        self.scheduled.insert(url);
    }

    /// Enqueues a hyperlink. Promotes a sitemap-only URL into the hot lane.
    /// First city URL per family is urgent so uniqueness can be measured.
    pub fn push_link(&mut self, url: AbsoluteUrl, depth: u32) {
        let lane = self.lane(&url);
        if self.scheduled.contains(&url) {
            if let Some(position) = self.sitemap.iter().position(|(item, _)| item == &url) {
                let _ = self.sitemap.remove(position);
                self.push_lane(lane, url, depth);
            }
            return;
        }
        self.scheduled.insert(url.clone());
        self.push_lane(lane, url, depth);
    }

    fn lane(&mut self, url: &AbsoluteUrl) -> Lane {
        if is_landing(url) {
            return Lane::Urgent;
        }
        if self.sample_city(url) {
            return Lane::Urgent;
        }
        Lane::Linked
    }

    fn sample_city(&mut self, url: &AbsoluteUrl) -> bool {
        let Some(family) = city_family(url) else {
            return false;
        };
        let taken = self.sampled_cities.entry(family).or_insert(0);
        if *taken < 2 {
            *taken += 1;
            true
        } else {
            false
        }
    }

    fn push_lane(&mut self, lane: Lane, url: AbsoluteUrl, depth: u32) {
        match lane {
            Lane::Urgent => self.urgent.push_back((url, depth)),
            Lane::Linked => self.linked.push_back((url, depth)),
        }
    }

    /// Pops up to `count` URLs from a single lane so landings stay ahead of sitemaps.
    pub fn pop_batch(&mut self, count: usize) -> Vec<(AbsoluteUrl, u32)> {
        if count == 0 {
            return Vec::new();
        }
        let lane = if !self.urgent.is_empty() {
            &mut self.urgent
        } else if !self.linked.is_empty() {
            &mut self.linked
        } else {
            &mut self.sitemap
        };
        let take = count.min(lane.len());
        lane.drain(..take).collect()
    }

    fn push_urgent(&mut self, url: AbsoluteUrl, depth: u32) {
        if !self.scheduled.insert(url.clone()) {
            return;
        }
        self.urgent.push_back((url, depth));
    }
}

#[derive(Clone, Copy)]
enum Lane {
    Urgent,
    Linked,
}

fn city_family(url: &AbsoluteUrl) -> Option<String> {
    let parts: Vec<&str> = url
        .path()
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let rest = if parts
        .first()
        .is_some_and(|part| matches!(*part, "en" | "ru" | "he" | "es" | "fr" | "de"))
    {
        parts.get(1..)?
    } else {
        &parts
    };
    match rest {
        ["category" | "services", slug, city] if city.contains('-') => {
            Some(format!("category/{slug}"))
        }
        _ => None,
    }
}

fn is_landing(url: &AbsoluteUrl) -> bool {
    let parts: Vec<&str> = url
        .path()
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        return true;
    }
    let rest = if matches!(parts[0], "en" | "ru" | "he" | "es" | "fr" | "de") {
        &parts[1..]
    } else {
        &parts[..]
    };
    matches!(
        rest,
        [] | ["category"
            | "categories"
            | "services"
            | "specialists"
            | "about"
            | "blog"
            | "how-it-works"
            | "reviews"]
            | ["category" | "services", _]
    )
}

#[cfg(test)]
mod tests {
    use super::Frontier;
    use weavatrix_seo_model::AbsoluteUrl;

    #[test]
    fn linked_category_beats_sitemap_flood() {
        let mut frontier = Frontier::default();
        let home = AbsoluteUrl::parse("https://x.test/").unwrap();
        let electrician = AbsoluteUrl::parse("https://x.test/category/electrician").unwrap();
        frontier.seed(home);
        for index in 0..40 {
            frontier.push_sitemap(
                AbsoluteUrl::parse(&format!("https://x.test/blog/post-{index}")).unwrap(),
            );
        }
        frontier.push_sitemap(electrician.clone());
        frontier.push_link(electrician.clone(), 1);
        let first = frontier.pop_batch(1);
        assert_eq!(first[0].0.path(), "/");
        let second = frontier.pop_batch(1);
        assert_eq!(second[0].0, electrician);
    }

    #[test]
    fn batch_stays_in_one_lane() {
        let mut frontier = Frontier::default();
        frontier.seed(AbsoluteUrl::parse("https://x.test/").unwrap());
        for index in 0..8 {
            frontier
                .push_sitemap(AbsoluteUrl::parse(&format!("https://x.test/blog/{index}")).unwrap());
        }
        let first = frontier.pop_batch(5);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].0.path(), "/");
        let rest = frontier.pop_batch(5);
        assert_eq!(rest.len(), 5);
        assert!(rest.iter().all(|(url, _)| url.path().starts_with("/blog/")));
    }

    #[test]
    fn sitemap_only_cities_are_sampled() {
        let mut frontier = Frontier::default();
        frontier.seed(AbsoluteUrl::parse("https://x.test/").unwrap());
        let first = AbsoluteUrl::parse("https://x.test/category/electrician/vancouver-wa").unwrap();
        let second = AbsoluteUrl::parse("https://x.test/category/electrician/camas-wa").unwrap();
        let third =
            AbsoluteUrl::parse("https://x.test/category/electrician/ridgefield-wa").unwrap();
        frontier.push_sitemap(first.clone());
        frontier.push_sitemap(second.clone());
        frontier.push_sitemap(third.clone());
        let _home = frontier.pop_batch(1);
        let sampled = frontier.pop_batch(2);
        assert_eq!(sampled.len(), 2);
        assert!(sampled.iter().any(|(url, _)| *url == first));
        assert!(sampled.iter().any(|(url, _)| *url == second));
        let rest = frontier.pop_batch(4);
        assert!(rest.iter().any(|(url, _)| *url == third));
    }

    #[test]
    fn first_city_variant_is_urgent() {
        let mut frontier = Frontier::default();
        frontier.seed(AbsoluteUrl::parse("https://x.test/").unwrap());
        let city = AbsoluteUrl::parse("https://x.test/category/electrician/vancouver-wa").unwrap();
        let other = AbsoluteUrl::parse("https://x.test/category/electrician/camas-wa").unwrap();
        frontier.push_link(city.clone(), 1);
        frontier.push_link(other.clone(), 1);
        let _ = frontier.pop_batch(1);
        let batch = frontier.pop_batch(2);
        assert_eq!(batch[0].0, city);
        assert_eq!(batch[1].0, other);
    }
}
