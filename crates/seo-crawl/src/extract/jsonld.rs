//! JSON-LD block classification using blazingly-json.
//!
//! Nodes keep their own identity. The flat `types`, `ids`, and `same_as` fields
//! are derived from them so the block cannot disagree with itself.

use blazingly_json::Value;
use weavatrix_seo_model::{JsonLd, JsonLdNode};

pub(super) fn parse_json_ld(raw: String) -> JsonLd {
    match blazingly_json::from_str::<Value>(&raw) {
        Ok(value) => {
            let mut nodes = Vec::new();
            walk(&value, &mut nodes);
            let mut types: Vec<String> = nodes.iter().flat_map(|node| node.types.clone()).collect();
            let mut ids: Vec<String> = nodes
                .iter()
                .filter(|node| node.types.iter().any(|kind| is_cite_type(kind)))
                .filter_map(|node| node.id.clone())
                .collect();
            let mut same_as: Vec<String> = nodes
                .iter()
                .filter(|node| node.types.iter().any(|kind| is_cite_type(kind)))
                .flat_map(|node| node.same_as.clone())
                .collect();
            types.sort();
            types.dedup();
            ids.sort();
            ids.dedup();
            same_as.sort();
            same_as.dedup();
            JsonLd {
                nodes,
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

fn walk(value: &Value, nodes: &mut Vec<JsonLdNode>) {
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, nodes);
            }
        }
        Value::Object(object) => {
            let types = type_names(object.get("@type"));
            if !types.is_empty() {
                nodes.push(JsonLdNode {
                    id: match object.get("@id") {
                        Some(Value::String(id)) => Some(id.clone()),
                        _ => None,
                    },
                    types,
                    same_as: same_as_values(object.get("sameAs")),
                });
            }
            for nested in object.values() {
                walk(nested, nodes);
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

fn same_as_values(value: Option<&Value>) -> Vec<String> {
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
