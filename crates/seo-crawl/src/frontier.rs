//! Link-first crawl frontier. Sitemap URLs stay cold until the hot queue drains.

use std::collections::{BTreeSet, VecDeque};
use weavatrix_seo_model::AbsoluteUrl;

/// Three-lane URL schedule.
#[derive(Debug, Default)]
pub struct Frontier {
    urgent: VecDeque<(AbsoluteUrl, u32)>,
    linked: VecDeque<(AbsoluteUrl, u32)>,
    sitemap: VecDeque<(AbsoluteUrl, u32)>,
    scheduled: BTreeSet<AbsoluteUrl>,
}

impl Frontier {
    /// Enqueues the seed as urgent.
    pub fn seed(&mut self, url: AbsoluteUrl) {
        self.push_urgent(url, 0);
    }

    /// Enqueues a sitemap loc without blocking link discovery.
    pub fn push_sitemap(&mut self, url: AbsoluteUrl) {
        if !self.scheduled.insert(url.clone()) {
            return;
        }
        self.sitemap.push_back((url, 0));
    }

    /// Enqueues a hyperlink. Promotes a sitemap-only URL into the hot lane.
    pub fn push_link(&mut self, url: AbsoluteUrl, depth: u32) {
        if self.scheduled.contains(&url) {
            if let Some(position) = self.sitemap.iter().position(|(item, _)| item == &url) {
                let _ = self.sitemap.remove(position);
                if is_landing(&url) {
                    self.urgent.push_back((url, depth));
                } else {
                    self.linked.push_back((url, depth));
                }
            }
            return;
        }
        self.scheduled.insert(url.clone());
        if is_landing(&url) {
            self.urgent.push_back((url, depth));
        } else {
            self.linked.push_back((url, depth));
        }
    }

    /// Pops urgent, then linked, then sitemap.
    pub fn pop(&mut self) -> Option<(AbsoluteUrl, u32)> {
        self.urgent
            .pop_front()
            .or_else(|| self.linked.pop_front())
            .or_else(|| self.sitemap.pop_front())
    }

    fn push_urgent(&mut self, url: AbsoluteUrl, depth: u32) {
        if !self.scheduled.insert(url.clone()) {
            return;
        }
        self.urgent.push_back((url, depth));
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
        let _ = frontier.pop();
        let second = frontier.pop().unwrap().0;
        assert_eq!(second, electrician);
    }
}
