//! How a URL entered the crawl frontier.

use serde::{Deserialize, Serialize};

/// Provenance of a discovered URL. First-party link and sitemap stay; extra
/// seeds from observations sit beside them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    /// Caller-supplied seed.
    Explicit,
    /// Google Search Console or search-performance import.
    Gsc,
    /// Server or CDN bot log.
    Log,
    /// Generative-search citation.
    AiCitation,
    /// URL present in a previous snapshot.
    PreviousSnapshot,
    /// Internal hyperlink.
    InternalLink,
    /// Sitemap loc.
    Sitemap,
}

impl DiscoverySource {
    /// Stronger provenance wins when the same URL is found twice.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Explicit => 6,
            Self::Gsc => 5,
            Self::Log => 4,
            Self::AiCitation => 3,
            Self::PreviousSnapshot => 2,
            Self::InternalLink => 1,
            Self::Sitemap => 0,
        }
    }

    /// Keep the stronger of two sources.
    #[must_use]
    pub fn stronger(self, other: Self) -> Self {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiscoverySource;

    #[test]
    fn gsc_outranks_a_sitemap_loc() {
        assert_eq!(
            DiscoverySource::Sitemap.stronger(DiscoverySource::Gsc),
            DiscoverySource::Gsc
        );
    }
}
