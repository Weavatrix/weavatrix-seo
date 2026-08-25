//! JSON-LD block classification using blazingly-json.

use blazingly_json::Value;
use weavatrix_seo_model::JsonLd;

pub(super) fn parse_json_ld(raw: String) -> JsonLd {
    match blazingly_json::from_str::<Value>(&raw) {
        Ok(value) => {
            let mut types = Vec::new();
            let mut ids = Vec::new();
            let mut same_as = Vec::new();
            walk(&value, &mut types, &mut ids, &mut same_as);
            types.sort();
            types.dedup();
            ids.sort();
            ids.dedup();
            same_as.sort();
            same_as.dedup();
            JsonLd {
                types,
                valid_json: true,
                ids,
                same_as,
                raw,
            }
        }
        Err(_) => JsonLd {
            valid_json: false,
            raw,
            ..JsonLd::default()
        },
    }
}

fn walk(value: &Value, types: &mut Vec<String>, ids: &mut Vec<String>, same_as: &mut Vec<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, types, ids, same_as);
            }
        }
        Value::Object(object) => {
            let kinds = type_names(object.get("@type"));
            types.extend(kinds.iter().cloned());
            if kinds.iter().any(|kind| is_cite_type(kind)) {
                if let Some(Value::String(id)) = object.get("@id") {
                    ids.push(id.clone());
                }
                push_same_as(object.get("sameAs"), same_as);
            }
            for nested in object.values() {
                walk(nested, types, ids, same_as);
            }
        }
        _ => {}
    }
}

fn type_names(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) => vec![text.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(text) => Some(text.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn push_same_as(value: Option<&Value>, same_as: &mut Vec<String>) {
    match value {
        Some(Value::String(text)) => same_as.push(text.clone()),
        Some(Value::Array(items)) => {
            for item in items {
                if let Value::String(text) = item {
                    same_as.push(text.clone());
                }
            }
        }
        _ => {}
    }
}

fn is_cite_type(kind: &str) -> bool {
    let short = kind
        .rsplit('/')
        .next()
        .unwrap_or(kind)
        .rsplit(':')
        .next()
        .unwrap_or(kind);
    short.eq_ignore_ascii_case("organization") || short.eq_ignore_ascii_case("website")
}
