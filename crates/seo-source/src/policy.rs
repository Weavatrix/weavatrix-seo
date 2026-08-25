//! Load `.weavatrix/seo.json` or a tiny YAML subset.

use std::path::Path;
use weavatrix_seo_model::SearchPolicy;

/// Reads the repository search-policy contract when present.
#[must_use]
pub fn load(repo: &str) -> Option<SearchPolicy> {
    let root = Path::new(repo).join(".weavatrix");
    let json = root.join("seo.json");
    if json.is_file() {
        return std::fs::read_to_string(&json)
            .ok()
            .and_then(|raw| blazingly_json::from_str(&raw).ok());
    }
    let yaml = root.join("seo.yaml");
    if yaml.is_file() {
        return std::fs::read_to_string(&yaml)
            .ok()
            .map(|raw| from_yaml(&raw));
    }
    None
}

/// Whether a family belongs on the intended indexable surface.
#[must_use]
pub fn allows_family(policy: Option<&SearchPolicy>, pattern: &str) -> bool {
    match policy {
        Some(policy) => policy.allows_family(pattern) && !super::is_private_pattern(pattern),
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
    use super::from_yaml;

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
