//! Bounded read-only query DSL over an audit report.
//!
//! Not SQL. Collections, AND-filters, and a hard limit keep this usable as an
//! MCP primitive without turning the engine into a database.

use serde::Serialize;
use std::collections::BTreeMap;
use weavatrix_seo_architecture::Architecture;
use weavatrix_seo_model::{AuditReport, Indexability, SearchIntelligence};

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;

/// Parsed query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// Collection name.
    pub collection: Collection,
    /// AND-ed filters.
    pub filters: Vec<Filter>,
    /// Selected fields. Empty means a default projection.
    pub fields: Vec<String>,
    /// Row cap.
    pub limit: usize,
}

/// Supported collections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    /// Measured URLs.
    Urls,
    /// Findings.
    Findings,
    /// Domain claims.
    Claims,
    /// Route families / matrices.
    RouteFamilies,
    /// Chunks.
    Chunks,
    /// Opportunities.
    Opportunities,
    /// Stored history runs.
    Runs,
}

/// One comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// Field name.
    pub field: String,
    /// Operator.
    pub op: Op,
    /// Literal value.
    pub value: String,
}

/// Comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `>`
    Gt,
    /// `<`
    Lt,
    /// `>=`
    Ge,
    /// `<=`
    Le,
    /// substring
    Contains,
}

/// Query result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueryResult {
    /// Collection.
    pub collection: String,
    /// Rows as string maps.
    pub rows: Vec<BTreeMap<String, String>>,
    /// Whether the limit truncated the result.
    pub truncated: bool,
    /// How these rows were established.
    pub evidence: String,
}

/// Parses a bounded DSL query.
///
/// # Errors
///
/// Unknown collections, operators, or malformed clauses are rejected.
pub fn parse(input: &str) -> Result<Query, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err("empty query".into());
    }
    let upper = raw.to_ascii_uppercase();
    if !upper.starts_with("FROM ") {
        return Err("query must start with FROM".into());
    }
    let rest = raw[5..].trim();
    let (collection_token, after) = split_keyword(rest, &["WHERE", "RETURN", "LIMIT"]);
    let collection = match collection_token.trim().to_ascii_lowercase().as_str() {
        "urls" => Collection::Urls,
        "findings" => Collection::Findings,
        "claims" => Collection::Claims,
        "route_families" | "families" => Collection::RouteFamilies,
        "chunks" => Collection::Chunks,
        "opportunities" => Collection::Opportunities,
        "runs" | "snapshots" => Collection::Runs,
        other => return Err(format!("unknown collection `{other}`")),
    };
    let mut cursor = after.trim();
    let mut filters = Vec::new();
    if let Some(stripped) = strip_prefix_ci(cursor, "WHERE ") {
        let (clause, rest) = split_keyword(stripped, &["RETURN", "LIMIT"]);
        filters = parse_filters(clause)?;
        cursor = rest;
    }
    let mut fields = Vec::new();
    if let Some(stripped) = strip_prefix_ci(cursor, "RETURN ") {
        let (clause, rest) = split_keyword(stripped, &["LIMIT"]);
        fields = clause
            .split(',')
            .map(|part| part.trim().to_ascii_lowercase())
            .filter(|part| !part.is_empty())
            .collect();
        cursor = rest;
    }
    let mut limit = DEFAULT_LIMIT;
    if let Some(stripped) = strip_prefix_ci(cursor, "LIMIT ") {
        limit = stripped
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("invalid LIMIT `{}`", stripped.trim()))?;
        limit = limit.clamp(1, MAX_LIMIT);
        cursor = "";
    }
    if !cursor.trim().is_empty() {
        return Err(format!("unexpected trailing input `{}`", cursor.trim()));
    }
    Ok(Query {
        collection,
        filters,
        fields,
        limit,
    })
}

fn parse_filters(clause: &str) -> Result<Vec<Filter>, String> {
    let mut filters = Vec::new();
    for part in split_and(clause) {
        let token = part.trim();
        if token.is_empty() {
            continue;
        }
        let (field, op, value) = parse_comparison(token)?;
        filters.push(Filter { field, op, value });
    }
    Ok(filters)
}

fn split_and(clause: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let chars: Vec<char> = clause.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '"' {
            in_string = !in_string;
            current.push('"');
            index += 1;
            continue;
        }
        if !in_string
            && chars[index].eq_ignore_ascii_case(&'a')
            && chars
                .get(index + 1)
                .is_some_and(|ch| ch.eq_ignore_ascii_case(&'n'))
            && chars
                .get(index + 2)
                .is_some_and(|ch| ch.eq_ignore_ascii_case(&'d'))
            && chars
                .get(index.wrapping_sub(1))
                .is_none_or(|ch| ch.is_whitespace())
            && chars.get(index + 3).is_none_or(|ch| ch.is_whitespace())
        {
            out.push(current.trim().to_owned());
            current.clear();
            index += 3;
            continue;
        }
        current.push(chars[index]);
        index += 1;
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_owned());
    }
    out
}

fn parse_comparison(token: &str) -> Result<(String, Op, String), String> {
    let ops = [
        ("!=", Op::Ne),
        (">=", Op::Ge),
        ("<=", Op::Le),
        ("=", Op::Eq),
        (">", Op::Gt),
        ("<", Op::Lt),
    ];
    let lower = token.to_ascii_lowercase();
    if let Some(at) = lower.find(" contains ") {
        let field = token[..at].trim().to_ascii_lowercase();
        let value = unquote(token[at + " contains ".len()..].trim());
        return Ok((field, Op::Contains, value));
    }
    for (symbol, op) in ops {
        if let Some(at) = token.find(symbol) {
            let field = token[..at].trim().to_ascii_lowercase();
            let value = unquote(token[at + symbol.len()..].trim());
            if field.is_empty() {
                return Err(format!("missing field in `{token}`"));
            }
            return Ok((field, op, value));
        }
    }
    Err(format!("cannot parse filter `{token}`"))
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|item| item.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_owned()
}

fn strip_prefix_ci<'a>(input: &'a str, prefix: &str) -> Option<&'a str> {
    if input.len() < prefix.len() {
        return None;
    }
    if input[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&input[prefix.len()..])
    } else {
        None
    }
}

fn split_keyword<'a>(input: &'a str, keywords: &[&str]) -> (&'a str, &'a str) {
    let upper = input.to_ascii_uppercase();
    let mut best: Option<usize> = None;
    for keyword in keywords {
        if let Some(at) = find_word(&upper, keyword)
            && best.is_none_or(|current| at < current)
        {
            best = Some(at);
        }
    }
    match best {
        Some(at) => (&input[..at], input[at..].trim()),
        None => (input, ""),
    }
}

fn find_word(hay: &str, word: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(rel) = hay[start..].find(word) {
        let at = start + rel;
        let before = if at == 0 {
            true
        } else {
            hay.as_bytes()
                .get(at - 1)
                .is_some_and(u8::is_ascii_whitespace)
        };
        let after = hay
            .as_bytes()
            .get(at + word.len())
            .is_none_or(u8::is_ascii_whitespace);
        if before && after {
            return Some(at);
        }
        start = at + 1;
    }
    None
}

/// Evaluates a parsed query against a report and architecture.
#[must_use]
pub fn evaluate(query: &Query, report: &AuditReport, architecture: &Architecture) -> QueryResult {
    let intelligence = report.intelligence.as_ref();
    let rows = match query.collection {
        Collection::Urls => url_rows(report, architecture, intelligence),
        Collection::Findings => finding_rows(report),
        Collection::Claims => claim_rows(report),
        Collection::RouteFamilies => family_rows(intelligence),
        Collection::Chunks => chunk_rows(intelligence),
        Collection::Opportunities => opportunity_rows(report),
        Collection::Runs => run_rows(report),
    };
    finish(query, rows, "DETERMINISTIC")
}

fn finish(query: &Query, mut rows: Vec<BTreeMap<String, String>>, evidence: &str) -> QueryResult {
    rows.retain(|row| {
        query
            .filters
            .iter()
            .all(|filter| matches_filter(row, filter))
    });
    let truncated = rows.len() > query.limit;
    rows.truncate(query.limit);
    if !query.fields.is_empty() {
        for row in &mut rows {
            row.retain(|key, _| query.fields.iter().any(|field| field == key));
        }
    }
    QueryResult {
        collection: collection_name(query.collection).into(),
        rows,
        truncated,
        evidence: evidence.into(),
    }
}

fn collection_name(collection: Collection) -> &'static str {
    match collection {
        Collection::Urls => "urls",
        Collection::Findings => "findings",
        Collection::Claims => "claims",
        Collection::RouteFamilies => "route_families",
        Collection::Chunks => "chunks",
        Collection::Opportunities => "opportunities",
        Collection::Runs => "runs",
    }
}

/// Parses and evaluates in one step.
///
/// # Errors
///
/// Propagates parse errors.
pub fn run(
    input: &str,
    report: &AuditReport,
    architecture: &Architecture,
) -> Result<QueryResult, String> {
    let query = parse(input)?;
    Ok(evaluate(&query, report, architecture))
}

/// Evaluates a query against a report, rebuilding architecture from inventory.
///
/// # Errors
///
/// Propagates parse errors.
pub fn run_on_report(input: &str, report: &AuditReport) -> Result<QueryResult, String> {
    let (architecture, _) = weavatrix_seo_architecture::analyze(&report.inventory);
    run(input, report, &architecture)
}

/// Evaluates a bounded DSL query against the `SQLite` history in `dir`.
///
/// JSON snapshots stay on disk; this reads `{dir}/weavatrix-seo.sqlite`.
///
/// # Errors
///
/// Propagates parse errors and `SQLite` errors.
pub fn run_on_history(input: &str, dir: &str) -> Result<QueryResult, String> {
    let query = parse(input)?;
    let rows = weavatrix_seo_history::query_maps(dir, collection_name(query.collection))?;
    Ok(finish(&query, rows, "DETERMINISTIC"))
}

fn matches_filter(row: &BTreeMap<String, String>, filter: &Filter) -> bool {
    let Some(actual) = row.get(&filter.field) else {
        return false;
    };
    match filter.op {
        Op::Eq => actual.eq_ignore_ascii_case(&filter.value),
        Op::Ne => !actual.eq_ignore_ascii_case(&filter.value),
        Op::Contains => actual
            .to_ascii_lowercase()
            .contains(&filter.value.to_ascii_lowercase()),
        Op::Gt | Op::Lt | Op::Ge | Op::Le => {
            let Ok(left) = actual.parse::<f64>() else {
                return false;
            };
            let Ok(right) = filter.value.parse::<f64>() else {
                return false;
            };
            match filter.op {
                Op::Gt => left > right,
                Op::Lt => left < right,
                Op::Ge => left >= right,
                Op::Le => left <= right,
                _ => false,
            }
        }
    }
}

fn url_rows(
    report: &AuditReport,
    architecture: &Architecture,
    intelligence: Option<&SearchIntelligence>,
) -> Vec<BTreeMap<String, String>> {
    let inbound: BTreeMap<String, usize> = architecture
        .pages
        .iter()
        .map(|page| (page.url.to_string(), page.inbound))
        .collect();
    let authority: BTreeMap<String, f64> = architecture
        .pages
        .iter()
        .map(|page| (page.url.to_string(), page.authority))
        .collect();
    report
        .inventory
        .pages
        .iter()
        .map(|page| {
            let url = page.url.to_string();
            let mut row = BTreeMap::new();
            row.insert("url".into(), url.clone());
            row.insert("status".into(), page.status.to_string());
            row.insert(
                "indexable".into(),
                (page.indexability == Indexability::Indexable).to_string(),
            );
            row.insert(
                "inbound_links".into(),
                inbound.get(&url).copied().unwrap_or(0).to_string(),
            );
            if let Some(score) = authority.get(&url) {
                row.insert("authority".into(), format!("{score:.6}"));
            }
            row.insert("in_sitemap".into(), page.in_sitemap.to_string());
            if let Some(title) = &page.title {
                row.insert("title".into(), title.clone());
            }
            if let Some(intel) = intelligence {
                if let Some(profile) = intel.profiles.iter().find(|item| item.url == url) {
                    if let Some(value) = profile.fact_density {
                        row.insert("fact_density".into(), value.to_string());
                    }
                    if let Some(value) = profile.genericity {
                        row.insert("genericity".into(), value.to_string());
                    }
                }
                if let Some(family) = intel
                    .families
                    .iter()
                    .find(|item| url.contains(&item.family))
                {
                    row.insert("route_family".into(), family.family.clone());
                    if let Some(value) = family.template_shared_ratio {
                        row.insert("boilerplate_ratio".into(), value.to_string());
                    }
                }
                if let Some(metric) = intel.url_metrics.iter().find(|item| item.url == url) {
                    if let Some(value) = metric.gsc_clicks {
                        row.insert("gsc_clicks".into(), value.to_string());
                    }
                    if let Some(value) = metric.gsc_impressions {
                        row.insert("gsc_impressions".into(), value.to_string());
                    }
                    if let Some(value) = metric.citations {
                        row.insert("citations".into(), value.to_string());
                    }
                }
            }
            row
        })
        .collect()
}

fn run_rows(report: &AuditReport) -> Vec<BTreeMap<String, String>> {
    let mut row = BTreeMap::new();
    row.insert("snapshot_id".into(), report.inventory.snapshot_id.clone());
    if let Some(site) = &report.inventory.site {
        row.insert("site".into(), site.clone());
    }
    row.insert(
        "mode".into(),
        format!("{:?}", report.inventory.mode).to_ascii_lowercase(),
    );
    row.insert(
        "measured_urls".into(),
        report.inventory.pages.len().to_string(),
    );
    row.insert("findings".into(), report.findings.len().to_string());
    vec![row]
}

fn finding_rows(report: &AuditReport) -> Vec<BTreeMap<String, String>> {
    report
        .findings
        .iter()
        .map(|finding| {
            let mut row = BTreeMap::new();
            row.insert("code".into(), finding.code.clone());
            row.insert("fingerprint".into(), finding.fingerprint.clone());
            row.insert(
                "severity".into(),
                format!("{:?}", finding.severity).to_ascii_lowercase(),
            );
            row.insert("summary".into(), finding.summary.clone());
            row.insert("url".into(), finding.locator.subject_url().to_owned());
            row.insert("authority".into(), format!("{:?}", finding.authority));
            row
        })
        .collect()
}

fn claim_rows(report: &AuditReport) -> Vec<BTreeMap<String, String>> {
    report
        .inventory
        .nodes
        .iter()
        .filter(|node| node.kind == weavatrix_seo_model::SearchNodeKind::Claim)
        .map(|node| {
            let mut row = BTreeMap::new();
            row.insert("claim".into(), node.label.clone());
            row.insert("id".into(), node.id.clone());
            row.insert("support_state".into(), "unmeasured".into());
            row
        })
        .collect()
}

fn family_rows(intelligence: Option<&SearchIntelligence>) -> Vec<BTreeMap<String, String>> {
    let Some(intelligence) = intelligence else {
        return Vec::new();
    };
    intelligence
        .matrices
        .iter()
        .map(|matrix| {
            let mut row = BTreeMap::new();
            row.insert("family".into(), matrix.family.clone());
            row.insert("measured_urls".into(), matrix.measured_urls.to_string());
            row.insert("verdict".into(), matrix.verdict.clone());
            if let Some(value) = matrix.template_boilerplate_ratio {
                row.insert("boilerplate_ratio".into(), value.to_string());
            }
            if let Some(value) = matrix.unique_fact_ratio {
                row.insert("unique_fact_ratio".into(), value.to_string());
            }
            if let Some(family) = intelligence
                .families
                .iter()
                .find(|item| item.family == matrix.family)
            {
                if let Some(value) = family.gsc_clicks {
                    row.insert("gsc_clicks".into(), value.to_string());
                }
                if let Some(value) = family.gsc_impressions {
                    row.insert("gsc_impressions".into(), value.to_string());
                }
                if let Some(value) = family.error_findings {
                    row.insert("error_findings".into(), value.to_string());
                }
            }
            row
        })
        .collect()
}

fn chunk_rows(intelligence: Option<&SearchIntelligence>) -> Vec<BTreeMap<String, String>> {
    let Some(intelligence) = intelligence else {
        return Vec::new();
    };
    intelligence
        .chunks
        .iter()
        .map(|chunk| {
            let mut row = BTreeMap::new();
            row.insert("id".into(), chunk.id.clone());
            row.insert("url".into(), chunk.url.clone());
            row.insert("heading".into(), chunk.heading.clone());
            if let Some(value) = chunk.citation_suitability {
                row.insert("citation_suitability".into(), value.to_string());
            }
            if let Some(value) = chunk.relevance {
                row.insert("relevance".into(), value.to_string());
            }
            row
        })
        .collect()
}

fn opportunity_rows(report: &AuditReport) -> Vec<BTreeMap<String, String>> {
    report
        .opportunities
        .iter()
        .map(|item| {
            let mut row = BTreeMap::new();
            row.insert("id".into(), item.id.clone());
            row.insert("kind".into(), item.kind.clone());
            row.insert("subject".into(), item.subject.clone());
            row.insert("summary".into(), item.summary.clone());
            if let Some(demand) = item.axes.demand {
                row.insert("demand".into(), demand.to_string());
            }
            if let Some(value) = item.axes.expected_ctr {
                row.insert("expected_ctr".into(), value.to_string());
            }
            if let Some(value) = item.axes.recoverable_clicks {
                row.insert("recoverable_clicks".into(), value.to_string());
            }
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Collection, Op, parse};

    #[test]
    fn parses_a_url_orphan_query() {
        let query = parse(
            "FROM urls WHERE inbound_links = 0 AND indexable = true RETURN url, inbound_links LIMIT 20",
        )
        .expect("parse");
        assert_eq!(query.collection, Collection::Urls);
        assert_eq!(query.limit, 20);
        assert_eq!(query.filters.len(), 2);
        assert_eq!(query.filters[0].op, Op::Eq);
        assert_eq!(query.fields, ["url", "inbound_links"]);
    }

    #[test]
    fn rejects_unknown_collection() {
        assert!(parse("FROM bananas RETURN url").is_err());
    }

    #[test]
    fn parses_historical_runs() {
        let query = parse("FROM runs WHERE measured_urls > 0 LIMIT 5").expect("parse");
        assert_eq!(query.collection, Collection::Runs);
    }
}
