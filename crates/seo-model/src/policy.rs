//! Optional repository search-policy contract.

use serde::{Deserialize, Serialize};

/// Indexability include/exclude globs for route families.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IndexabilityPolicy {
    /// Families that may be indexable / CREATE.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    /// Families that must stay out of the intended search surface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// International contract.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InternationalPolicy {
    /// Required `x-default` hreflang when locale twins exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_default: Option<String>,
}

/// `.weavatrix/seo.json` (or yaml) contract. Absence stays unmeasured.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SearchPolicy {
    /// Artifact schema.
    #[serde(default)]
    pub schema: String,
    /// Indexability rules.
    #[serde(default)]
    pub indexability: IndexabilityPolicy,
    /// Hreflang contract.
    #[serde(default)]
    pub international: InternationalPolicy,
}

impl SearchPolicy {
    /// True when `pattern` is in the intended indexable surface.
    #[must_use]
    pub fn allows_family(&self, pattern: &str) -> bool {
        if self
            .indexability
            .exclude
            .iter()
            .any(|glob| glob_match(glob, pattern))
        {
            return false;
        }
        if self.indexability.include.is_empty() {
            return true;
        }
        self.indexability
            .include
            .iter()
            .any(|glob| glob_match(glob, pattern))
    }
}

/// Glob over route families. `**` is any tail; `*` and `:name` are one segment.
/// `/:locale` is optional, matching default-locale URLs without the prefix.
#[must_use]
pub fn glob_match(glob: &str, pattern: &str) -> bool {
    if match_glob(glob, pattern) {
        return true;
    }
    if let Some(rest) = glob.strip_prefix("/:locale") {
        let rest = if rest.is_empty() { "/" } else { rest };
        return match_glob(rest, pattern);
    }
    false
}

fn match_glob(glob: &str, pattern: &str) -> bool {
    let glob: Vec<&str> = glob.split('/').filter(|part| !part.is_empty()).collect();
    let path: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    match_parts(&glob, &path)
}

fn match_parts(glob: &[&str], path: &[&str]) -> bool {
    let mut gi = 0;
    let mut pi = 0;
    while gi < glob.len() {
        if glob[gi] == "**" {
            if gi + 1 == glob.len() {
                return true;
            }
            gi += 1;
            while pi <= path.len() {
                if match_parts(&glob[gi..], &path[pi..]) {
                    return true;
                }
                pi += 1;
            }
            return false;
        }
        if pi >= path.len() {
            return false;
        }
        if glob[gi] != "*" && !glob[gi].starts_with(':') && glob[gi] != path[pi] {
            return false;
        }
        gi += 1;
        pi += 1;
    }
    pi == path.len()
}

#[cfg(test)]
mod tests {
    use super::{SearchPolicy, glob_match};

    #[test]
    fn glob_double_star_covers_tail() {
        assert!(glob_match("/:locale/auth/**", "/:locale/auth/verify"));
        assert!(glob_match(
            "/:locale/category/**",
            "/:locale/category/:city"
        ));
        assert!(!glob_match("/:locale/auth/**", "/:locale/about"));
    }

    #[test]
    fn include_and_exclude() {
        let policy = SearchPolicy {
            indexability: super::IndexabilityPolicy {
                include: vec!["/:locale/category/**".into()],
                exclude: vec!["/:locale/admin/**".into()],
            },
            ..SearchPolicy::default()
        };
        assert!(policy.allows_family("/:locale/category/:city"));
        assert!(!policy.allows_family("/:locale/about"));
        assert!(!policy.allows_family("/:locale/admin/dashboard"));
        assert!(policy.allows_family("/category/:city"));
        assert!(glob_match(
            "/:locale/category/:slug/:city",
            "/category/cleaning/camas-wa"
        ));
        assert!(glob_match(
            "/:locale/category/:slug/:city",
            "/en/category/cleaning/camas-wa"
        ));
        assert!(!glob_match("/:locale/category/:slug/:city", "/blog/post"));
    }
}
