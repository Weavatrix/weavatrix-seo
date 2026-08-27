//! Strict JSON schemas for the MCP tools.

use mcport::json;

/// Crawl-backed tools. Every evidence import the CLI accepts is here too.
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
            "workers": { "type": "integer", "minimum": 1, "description": "Parallel fetches." },
            "render": { "type": "string", "description": "WVQ render snapshot JSON path." },
            "gsc": { "type": "string", "description": "Search Console export JSON path." },
            "observations": {
                "type": "string",
                "description": "GSC, Bing, or bot-log JSON path. Takes precedence over gsc."
            },
            "history": {
                "type": "string",
                "description": "Directory for a compact snapshot. Enables later seo_diff."
            }
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
            "repo": { "type": "string", "description": "Repository path for source chain." },
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

/// Evidence gate. Mirrors CLI `--ci` / `--baseline`.
pub fn gate() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "description": "site, repo, or hybrid." },
            "site": { "type": "string", "description": "Absolute http(s) URL." },
            "repo": { "type": "string", "description": "Repository path." },
            "max_pages": { "type": "integer", "minimum": 1 },
            "workers": { "type": "integer", "minimum": 1 },
            "render": { "type": "string", "description": "WVQ render snapshot JSON path." },
            "gsc": { "type": "string", "description": "Search Console export JSON path." },
            "observations": { "type": "string", "description": "Provider JSON path." },
            "baseline": {
                "type": "string",
                "description": "Previous audit JSON or compact baseline. Omit to gate on errors only."
            }
        },
        "additionalProperties": false
    })
}

/// Imported provider evidence.
pub fn observations() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "observations": {
                "type": "string",
                "description": "GSC, Bing, or bot-log JSON path."
            },
            "gsc": { "type": "string", "description": "Search Console export JSON path." },
            "provider": { "type": "string", "description": "Filter rows by provider name." },
            "limit": { "type": "integer", "minimum": 1, "description": "Returned rows. Default 200." }
        },
        "additionalProperties": false
    })
}
