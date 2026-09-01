//! `tsconfig.json` / `jsconfig.json` path aliases.

use std::fs;
use std::path::Path;

/// `(alias, target)` pairs such as `("@/*", "src/*")`.
#[must_use]
pub fn load(repo: &str) -> Vec<(String, String)> {
    for name in ["tsconfig.json", "jsconfig.json"] {
        let path = Path::new(repo).join(name);
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        let aliases = parse(&raw);
        if !aliases.is_empty() {
            return aliases;
        }
    }
    Vec::new()
}

fn parse(raw: &str) -> Vec<(String, String)> {
    let stripped = strip_comments(raw);
    let Some(start) = stripped.find("\"paths\"") else {
        return Vec::new();
    };
    let rest = &stripped[start..];
    let Some(brace) = rest.find('{') else {
        return Vec::new();
    };
    let body = &rest[brace + 1..];
    let end = body.find('}').unwrap_or(body.len());
    let mut aliases = Vec::new();
    let mut cursor = &body[..end];
    while let Some(key_start) = cursor.find('"') {
        cursor = &cursor[key_start + 1..];
        let Some(key_end) = cursor.find('"') else {
            break;
        };
        let key = cursor[..key_end].to_owned();
        cursor = &cursor[key_end + 1..];
        let Some(value_start) = cursor.find('"') else {
            break;
        };
        cursor = &cursor[value_start + 1..];
        let Some(value_end) = cursor.find('"') else {
            break;
        };
        let raw = cursor[..value_end].replace('\\', "/");
        cursor = &cursor[value_end + 1..];
        let value = raw.strip_prefix("./").unwrap_or(raw.as_str()).to_owned();
        aliases.push((key, value));
    }
    aliases
}

fn strip_comments(raw: &str) -> String {
    raw.lines()
        .map(|line| line.split("//").next().unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Resolves `specifier` against aliases, then `@/` / `~/`, then relatives.
#[must_use]
pub fn resolve(from_file: &str, specifier: &str, aliases: &[(String, String)]) -> String {
    let specifier = specifier.replace('\\', "/");
    for (alias, target) in aliases {
        if let Some(mapped) = apply_alias(alias, target, &specifier) {
            return mapped;
        }
    }
    crate::producers::join_relative(from_file, &specifier)
}

fn apply_alias(alias: &str, target: &str, specifier: &str) -> Option<String> {
    if let Some(prefix) = alias.strip_suffix("/*") {
        let rest = specifier.strip_prefix(prefix)?;
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        let target = target.strip_suffix("/*").unwrap_or(target);
        if rest.is_empty() {
            return Some(target.replace('\\', "/"));
        }
        return Some(format!("{target}/{rest}").replace('\\', "/"));
    }
    if specifier == alias {
        return Some(target.replace('\\', "/"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{parse, resolve};

    #[test]
    fn reads_star_aliases() {
        let aliases = parse(
            r#"{
              "compilerOptions": {
                "paths": {
                  "@/*": ["./src/*"],
                  "@lib/*": ["src/lib/*"]
                }
              }
            }"#,
        );
        assert!(
            aliases
                .iter()
                .any(|(key, value)| key == "@/*" && value == "src/*"),
            "{aliases:?}"
        );
        let path = resolve(
            "src/app/page.tsx",
            "@lib/citySeo",
            &[("@lib/*".into(), "src/lib/*".into())],
        );
        assert_eq!(path, "src/lib/citySeo");
    }
}
