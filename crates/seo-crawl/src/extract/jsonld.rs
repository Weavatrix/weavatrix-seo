//! JSON-LD block classification using blazingly-json.

use blazingly_json::Value;
use weavatrix_seo_model::JsonLd;

pub(super) fn parse_json_ld(raw: String) -> JsonLd {
    match blazingly_json::from_str::<Value>(&raw) {
        Ok(value) => JsonLd {
            types: collect_types(&value),
            valid_json: true,
            raw,
        },
        Err(_) => JsonLd {
            types: Vec::new(),
            valid_json: false,
            raw,
        },
    }
}

fn collect_types(value: &Value) -> Vec<String> {
    let mut types = Vec::new();
    push_types(value, &mut types);
    types.sort();
    types.dedup();
    types
}

fn push_types(value: &Value, types: &mut Vec<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                push_types(item, types);
            }
        }
        Value::Object(object) => {
            if let Some(kind) = object.get("@type") {
                match kind {
                    Value::String(text) => types.push(text.clone()),
                    Value::Array(items) => {
                        for item in items {
                            if let Value::String(text) = item {
                                types.push(text.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            if let Some(graph) = object.get("@graph") {
                push_types(graph, types);
            }
        }
        _ => {}
    }
}
