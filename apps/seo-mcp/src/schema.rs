//! Strict JSON schemas for the eight MCP tools.

use mcport::json;

pub fn site() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "description": "site, repo, hybrid, or compare." },
            "site": { "type": "string", "description": "Absolute http(s) URL." },
            "repo": { "type": "string", "description": "Repository path." },
            "competitor": { "type": "string", "description": "Public competitor origin." },
            "competitors": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Public competitor origins."
            },
            "max_pages": { "type": "integer", "minimum": 1, "description": "Crawl page cap." },
            "scope": { "type": "string", "description": "Optional URL glob." },
            "render": { "type": "string", "description": "WVQ render snapshot JSON path." }
        },
        "additionalProperties": false
    })
}

pub fn explain() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Finding fingerprint or code." },
            "site": { "type": "string", "description": "Site used to rebuild the audit." },
            "max_pages": { "type": "integer", "minimum": 1 }
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

pub fn diff() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "repo": { "type": "string" },
            "base": { "type": "string", "description": "Base snapshot or audit JSON path." },
            "head": { "type": "string", "description": "Head snapshot or audit JSON path." }
        },
        "additionalProperties": false
    })
}
