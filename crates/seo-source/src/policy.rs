//! Load `.weavatrix/seo.json` or a tiny YAML subset.

use std::path::Path;
use weavatrix_seo_model::SearchPolicy;

/// Outcome of reading the repository search contract.
///
/// A present-but-unreadable file is not the same as no file. Collapsing the two
/// makes a typo in a versioned contract invisible.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicyLoad {
    /// Parsed contract when the file was present and readable.
    pub policy: Option<SearchPolicy>,
    /// Why a present contract could not be used.
    pub error: Option<String>,
}

impl PolicyLoad {
    fn absent() -> Self {
        Self::default()
    }

    fn parsed(policy: SearchPolicy) -> Self {
        Self {
            policy: Some(policy),
            error: None,
        }
    }

    fn failed(path: &Path, reason: impl std::fmt::Display) -> Self {
        Self {
            policy: None,
            error: Some(format!("{} could not be read: {reason}", path.display())),
        }
    }
}

/// Reads the repository search-policy contract when present.
#[must_use]
pub fn load(repo: &str) -> PolicyLoad {
    let root = Path::new(repo).join(".weavatrix");
    let json = root.join("seo.json");
    if json.is_file() {
        return match std::fs::read_to_string(&json) {
            Err(error) => PolicyLoad::failed(&json, error),
            Ok(raw) => match blazingly_json::from_str(weavatrix_seo_model::strip_bom(&raw)) {
                Ok(policy) => PolicyLoad::parsed(policy),
                Err(error) => PolicyLoad::failed(&json, error),
            },
        };
    }
    let yaml = root.join("seo.yaml");
    if yaml.is_file() {
        return match std::fs::read_to_string(&yaml) {
            Ok(raw) => PolicyLoad::parsed(from_yaml(&raw)),
            Err(error) => PolicyLoad::failed(&yaml, error),
        };
    }
    PolicyLoad::absent()
}

/// Whether a family belongs on the intended indexable surface.
///
/// An explicit contract wins. The private-pattern heuristic exists only to guess
/// when the project declared nothing; a project that says `/profile/:username`
/// is a public landing family must not be overruled by a built-in guess.
#[must_use]
pub fn allows_family(policy: Option<&SearchPolicy>, pattern: &str) -> bool {
    match policy {
        Some(policy) => policy.allows_family(pattern),
        None => !super::is_private_pattern(pattern),
    }
}

fn from_yaml(raw: &str) -> SearchPolicy {
    let mut policy = SearchPolicy {
        schema: "weavatrix-seo-policy/v1".into(),
        ..SearchPolicy::default()
    };
    let mut section = "";
    let mut list: Option<&str> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            match list {
                Some("include") => policy.indexability.include.push(item.trim().to_owned()),
                Some("exclude") => policy.indexability.exclude.push(item.trim().to_owned()),
                _ => {}
            }
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        match key {
            "schema" if !value.is_empty() => value.clone_into(&mut policy.schema),
            "indexability" => {
                section = "indexability";
                list = None;
            }
            "international" => {
                section = "international";
                list = None;
            }
            "include" if section == "indexability" => list = Some("include"),
            "exclude" if section == "indexability" => list = Some("exclude"),
            "x_default" if section == "international" && !value.is_empty() => {
                policy.international.x_default = Some(value.to_owned());
            }
            _ => {}
        }
    }
    policy
}

#[cfg(test)]
mod tests {
    use super::{allows_family, from_yaml, load};

    #[test]
    fn explicit_contract_beats_the_private_heuristic() {
        let policy = from_yaml("indexability:\n  include:\n    - /profile/**\n");
        assert!(
            crate::is_private_pattern("/profile/:username"),
            "the heuristic must still guess this is private when nothing is declared"
        );
        assert!(!allows_family(None, "/profile/:username"));
        assert!(
            allows_family(Some(&policy), "/profile/:username"),
            "a declared public landing family is not overruled by a built-in guess"
        );
    }

    #[test]
    fn malformed_contract_is_not_an_absent_contract() {
        let dir = std::env::temp_dir().join(format!(
            "wvx-seo-policy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".weavatrix")).expect("contract dir");
        std::fs::write(dir.join(".weavatrix").join("seo.json"), "{ oops").expect("write");
        let loaded = load(&dir.to_string_lossy());
        assert!(loaded.policy.is_none());
        assert!(
            loaded.error.is_some(),
            "a typo must not read as no contract"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn absent_contract_reports_no_error() {
        let dir = std::env::temp_dir().join(format!(
            "wvx-seo-nopolicy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        let loaded = load(&dir.to_string_lossy());
        assert!(loaded.policy.is_none());
        assert!(loaded.error.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn yaml_subset_reads_include_exclude() {
        let policy = from_yaml(
            "indexability:\n  include:\n    - /:locale/category/**\n  exclude:\n    - /:locale/auth/**\ninternational:\n  x_default: en\n",
        );
        assert!(policy.allows_family("/:locale/category/:city"));
        assert!(!policy.allows_family("/:locale/auth/verify"));
        assert_eq!(policy.international.x_default.as_deref(), Some("en"));
    }
}
