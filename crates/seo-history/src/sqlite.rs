//! Local `SQLite` index of history snapshots. JSON snapshots stay the archive.

use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use weavatrix_seo_model::{
    AuditReport, FindingFamily, Indexability, Inventory, ProducerFact, Relation, SearchNodeKind,
    Severity,
};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS runs (
  snapshot_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL DEFAULT '',
  site TEXT,
  repo_revision TEXT,
  mode TEXT NOT NULL,
  recorded_at INTEGER NOT NULL,
  measured_urls INTEGER NOT NULL,
  findings INTEGER NOT NULL,
  errors INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS pages (
  snapshot_id TEXT NOT NULL,
  url TEXT NOT NULL,
  status INTEGER NOT NULL,
  indexable TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  title TEXT,
  family TEXT,
  producer_key TEXT,
  producer_hash TEXT,
  inbound INTEGER,
  gsc_clicks INTEGER,
  gsc_impressions INTEGER,
  citations INTEGER,
  PRIMARY KEY (snapshot_id, url)
);
CREATE TABLE IF NOT EXISTS findings (
  snapshot_id TEXT NOT NULL,
  fingerprint TEXT NOT NULL,
  code TEXT NOT NULL,
  severity TEXT NOT NULL,
  url TEXT NOT NULL,
  summary TEXT NOT NULL,
  PRIMARY KEY (snapshot_id, fingerprint)
);
CREATE TABLE IF NOT EXISTS families (
  snapshot_id TEXT NOT NULL,
  family TEXT NOT NULL,
  measured_urls INTEGER NOT NULL,
  verdict TEXT,
  gsc_clicks INTEGER,
  gsc_impressions INTEGER,
  error_findings INTEGER,
  unique_fact_ratio INTEGER,
  PRIMARY KEY (snapshot_id, family)
);
CREATE TABLE IF NOT EXISTS chunks (
  snapshot_id TEXT NOT NULL,
  id TEXT NOT NULL,
  url TEXT NOT NULL,
  heading TEXT NOT NULL,
  citation_suitability INTEGER,
  PRIMARY KEY (snapshot_id, id)
);
CREATE TABLE IF NOT EXISTS claims (
  snapshot_id TEXT NOT NULL,
  id TEXT NOT NULL,
  claim TEXT NOT NULL,
  support_state TEXT NOT NULL,
  citations INTEGER,
  PRIMARY KEY (snapshot_id, id)
);
CREATE TABLE IF NOT EXISTS opportunities (
  snapshot_id TEXT NOT NULL,
  id TEXT NOT NULL,
  kind TEXT NOT NULL,
  subject TEXT NOT NULL,
  summary TEXT NOT NULL,
  PRIMARY KEY (snapshot_id, id)
);
";

const DAY: i64 = 86_400;
const WINDOW_28D: i64 = 28 * DAY;
const WINDOW_SLACK: i64 = 10 * DAY;

/// Writes one report into `{dir}/weavatrix-seo.sqlite`.
///
/// # Errors
///
/// Returns IO or `SQLite` errors.
pub fn ingest(dir: &str, report: &AuditReport) -> Result<(), String> {
    ingest_at(dir, report, unix_now())
}

/// Ingest with an explicit unix timestamp (tests and rebuilds).
///
/// # Errors
///
/// Returns IO or `SQLite` errors.
pub fn ingest_at(dir: &str, report: &AuditReport, recorded_at: i64) -> Result<(), String> {
    let mut conn = open(dir)?;
    ingest_report(&mut conn, report, recorded_at)
}

/// Bounded maps for the latest snapshot, with deltas against earlier runs.
///
/// # Errors
///
/// Returns IO or `SQLite` errors, or when the directory has no runs.
pub fn query_maps(dir: &str, collection: &str) -> Result<Vec<BTreeMap<String, String>>, String> {
    let conn = open(dir)?;
    let latest = latest_run(&conn)?
        .ok_or_else(|| "history has no runs; pass --history DIR on an audit first".to_owned())?;
    match collection {
        "runs" | "snapshots" => runs_maps(&conn),
        "urls" => url_maps(&conn, &latest),
        "findings" => finding_maps(&conn, &latest.id),
        "route_families" | "families" => family_maps(&conn, &latest),
        "chunks" => chunk_maps(&conn, &latest),
        "claims" => claim_maps(&conn, &latest.id),
        "opportunities" => opportunity_maps(&conn, &latest.id),
        other => Err(format!("unknown collection `{other}`")),
    }
}

fn open(dir: &str) -> Result<Connection, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = Path::new(dir).join("weavatrix-seo.sqlite");
    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|error| error.to_string())?;
    conn.execute_batch(SCHEMA)
        .map_err(|error| error.to_string())?;
    Ok(conn)
}

#[allow(clippy::too_many_lines)]
fn ingest_report(
    conn: &mut Connection,
    report: &AuditReport,
    recorded_at: i64,
) -> Result<(), String> {
    let snapshot_id = if report.inventory.snapshot_id.is_empty() {
        "snapshot".to_owned()
    } else {
        report.inventory.snapshot_id.clone()
    };
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    clear_snapshot(&tx, &snapshot_id)?;
    let errors = report
        .findings
        .iter()
        .filter(|item| item.severity == Severity::Error)
        .count();
    tx.execute(
        "INSERT INTO runs(
            snapshot_id, run_id, site, repo_revision, mode, recorded_at,
            measured_urls, findings, errors
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            snapshot_id,
            report.inventory.run_id,
            report.inventory.site,
            report.inventory.repo_revision,
            format!("{:?}", report.inventory.mode).to_ascii_lowercase(),
            recorded_at,
            i64::try_from(report.inventory.pages.len()).unwrap_or(i64::MAX),
            i64::try_from(report.findings.len()).unwrap_or(i64::MAX),
            i64::try_from(errors).unwrap_or(i64::MAX),
        ],
    )
    .map_err(|error| error.to_string())?;
    let inbound = inbound_counts(&report.inventory);
    let metrics: BTreeMap<&str, &weavatrix_seo_model::UrlMetric> = report
        .intelligence
        .as_ref()
        .map(|intel| {
            intel
                .url_metrics
                .iter()
                .map(|item| (item.url.as_str(), item))
                .collect()
        })
        .unwrap_or_default();
    let families = report
        .intelligence
        .as_ref()
        .map_or(&[][..], |intel| intel.families.as_slice());
    for page in &report.inventory.pages {
        let url = page.url.to_string();
        let family = family_for(&url, families, &report.inventory.producers);
        let (producer_key, producer_hash) = producer_for(&url, &report.inventory.producers);
        let metric = metrics.get(url.as_str());
        tx.execute(
            "INSERT INTO pages(
                snapshot_id, url, status, indexable, content_hash, title, family,
                producer_key, producer_hash, inbound, gsc_clicks, gsc_impressions, citations
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                snapshot_id,
                url,
                i64::from(page.status),
                indexable_label(page.indexability),
                page.content_hash.hex(),
                page.title,
                family,
                producer_key,
                producer_hash,
                i64::try_from(inbound.get(&url).copied().unwrap_or(0)).unwrap_or(i64::MAX),
                metric.and_then(|item| item.gsc_clicks).map(i64::from),
                metric.and_then(|item| item.gsc_impressions).map(i64::from),
                metric.and_then(|item| item.citations).map(i64::from),
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    for finding in &report.findings {
        tx.execute(
            "INSERT OR REPLACE INTO findings(
                snapshot_id, fingerprint, code, severity, url, summary
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                snapshot_id,
                finding.fingerprint,
                finding.code,
                format!("{:?}", finding.severity).to_ascii_lowercase(),
                finding.locator.subject_url(),
                finding.summary,
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    if let Some(intel) = &report.intelligence {
        for family in &intel.families {
            let verdict = intel
                .matrices
                .iter()
                .find(|matrix| matrix.family == family.family)
                .map(|matrix| matrix.verdict.as_str());
            tx.execute(
                "INSERT INTO families(
                    snapshot_id, family, measured_urls, verdict, gsc_clicks,
                    gsc_impressions, error_findings, unique_fact_ratio
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    snapshot_id,
                    family.family,
                    i64::try_from(family.measured_urls).unwrap_or(i64::MAX),
                    verdict,
                    family.gsc_clicks.map(i64::from),
                    family.gsc_impressions.map(i64::from),
                    family.error_findings.map(i64::from),
                    family.unique_fact_ratio.map(i64::from),
                ],
            )
            .map_err(|error| error.to_string())?;
        }
        for chunk in &intel.chunks {
            tx.execute(
                "INSERT INTO chunks(
                    snapshot_id, id, url, heading, citation_suitability
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    snapshot_id,
                    chunk.id,
                    chunk.url,
                    chunk.heading,
                    chunk.citation_suitability.map(i64::from),
                ],
            )
            .map_err(|error| error.to_string())?;
        }
    }
    let claim_hits = claim_findings(report);
    for node in report
        .inventory
        .nodes
        .iter()
        .filter(|node| node.kind == SearchNodeKind::Claim)
    {
        let support = if claim_hits
            .iter()
            .any(|hit| hit.contains(&node.id) || hit.contains(&node.label) || node.id.contains(hit))
        {
            "unsupported"
        } else {
            "unmeasured"
        };
        tx.execute(
            "INSERT INTO claims(snapshot_id, id, claim, support_state, citations)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![snapshot_id, node.id, node.label, support],
        )
        .map_err(|error| error.to_string())?;
    }
    for item in &report.opportunities {
        tx.execute(
            "INSERT OR REPLACE INTO opportunities(
                snapshot_id, id, kind, subject, summary
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![snapshot_id, item.id, item.kind, item.subject, item.summary,],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(())
}

fn clear_snapshot(tx: &rusqlite::Transaction<'_>, snapshot_id: &str) -> Result<(), String> {
    for table in [
        "pages",
        "findings",
        "families",
        "chunks",
        "claims",
        "opportunities",
        "runs",
    ] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE snapshot_id = ?1"),
            params![snapshot_id],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

struct Run {
    id: String,
    recorded_at: i64,
    repo_revision: Option<String>,
}

fn latest_run(conn: &Connection) -> Result<Option<Run>, String> {
    conn.query_row(
        "SELECT snapshot_id, recorded_at, repo_revision FROM runs
         ORDER BY recorded_at DESC, snapshot_id DESC LIMIT 1",
        [],
        |row| {
            Ok(Run {
                id: row.get(0)?,
                recorded_at: row.get(1)?,
                repo_revision: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn previous_run(conn: &Connection, latest: &Run) -> Result<Option<Run>, String> {
    conn.query_row(
        "SELECT snapshot_id, recorded_at, repo_revision FROM runs
         WHERE recorded_at < ?1 OR (recorded_at = ?1 AND snapshot_id < ?2)
         ORDER BY recorded_at DESC, snapshot_id DESC LIMIT 1",
        params![latest.recorded_at, latest.id],
        |row| {
            Ok(Run {
                id: row.get(0)?,
                recorded_at: row.get(1)?,
                repo_revision: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn window_run(conn: &Connection, latest: &Run) -> Result<Option<Run>, String> {
    let target = latest.recorded_at.saturating_sub(WINDOW_28D);
    let lo = target.saturating_sub(WINDOW_SLACK);
    let hi = target.saturating_add(WINDOW_SLACK);
    conn.query_row(
        "SELECT snapshot_id, recorded_at, repo_revision FROM runs
         WHERE snapshot_id != ?1 AND recorded_at BETWEEN ?2 AND ?3
         ORDER BY ABS(recorded_at - ?4) ASC, snapshot_id DESC LIMIT 1",
        params![latest.id, lo, hi, target],
        |row| {
            Ok(Run {
                id: row.get(0)?,
                recorded_at: row.get(1)?,
                repo_revision: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn url_maps(conn: &Connection, latest: &Run) -> Result<Vec<BTreeMap<String, String>>, String> {
    let previous = previous_run(conn, latest)?;
    let window = window_run(conn, latest)?;
    let prev_pages = previous
        .as_ref()
        .map(|run| load_pages(conn, &run.id))
        .transpose()?
        .unwrap_or_default();
    let window_pages = window
        .as_ref()
        .map(|run| load_pages(conn, &run.id))
        .transpose()?
        .unwrap_or_default();
    let revision_changed = previous
        .as_ref()
        .is_some_and(|run| run.repo_revision != latest.repo_revision);
    let mut rows = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT url, status, indexable, inbound, family, producer_hash,
                    gsc_clicks, gsc_impressions, citations, title
             FROM pages WHERE snapshot_id = ?1 ORDER BY url",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![latest.id], |row| {
            Ok(PageRec {
                url: row.get(0)?,
                status: row.get(1)?,
                indexable: row.get(2)?,
                inbound: row.get(3)?,
                family: row.get(4)?,
                producer_hash: row.get(5)?,
                gsc_clicks: row.get(6)?,
                gsc_impressions: row.get(7)?,
                citations: row.get(8)?,
                title: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;
    for rec in iter {
        let rec = rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("url".into(), rec.url.clone());
        row.insert("status".into(), rec.status.to_string());
        row.insert("indexable".into(), rec.indexable);
        if let Some(inbound) = rec.inbound {
            row.insert("inbound_links".into(), inbound.to_string());
        }
        if let Some(family) = rec.family {
            row.insert("route_family".into(), family);
        }
        if let Some(title) = rec.title {
            row.insert("title".into(), title);
        }
        insert_opt(&mut row, "gsc_clicks", rec.gsc_clicks);
        insert_opt(&mut row, "gsc_impressions", rec.gsc_impressions);
        insert_opt(&mut row, "citations", rec.citations);
        if let Some(prev) = prev_pages.get(&rec.url) {
            row.insert(
                "producer_changed".into(),
                (rec.producer_hash != prev.producer_hash).to_string(),
            );
            if let (Some(now), Some(before)) = (rec.gsc_clicks, prev.gsc_clicks)
                && let Some(delta) = pct_delta(now, before)
            {
                row.insert("clicks_delta".into(), delta.to_string());
            }
        }
        if let Some(before) = window_pages.get(&rec.url)
            && let (Some(now), Some(prev_clicks)) = (rec.gsc_clicks, before.gsc_clicks)
            && let Some(delta) = pct_delta(now, prev_clicks)
        {
            row.insert("clicks_delta_28d".into(), delta.to_string());
        }
        if previous.is_some() {
            row.insert(
                "source_revision_changed".into(),
                revision_changed.to_string(),
            );
        }
        rows.push(row);
    }
    Ok(rows)
}

struct PageRec {
    url: String,
    status: i64,
    indexable: String,
    inbound: Option<i64>,
    family: Option<String>,
    producer_hash: Option<String>,
    gsc_clicks: Option<i64>,
    gsc_impressions: Option<i64>,
    citations: Option<i64>,
    title: Option<String>,
}

fn load_pages(conn: &Connection, snapshot_id: &str) -> Result<BTreeMap<String, PageRec>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT url, status, indexable, inbound, family, producer_hash,
                    gsc_clicks, gsc_impressions, citations, title
             FROM pages WHERE snapshot_id = ?1",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![snapshot_id], |row| {
            Ok(PageRec {
                url: row.get(0)?,
                status: row.get(1)?,
                indexable: row.get(2)?,
                inbound: row.get(3)?,
                family: row.get(4)?,
                producer_hash: row.get(5)?,
                gsc_clicks: row.get(6)?,
                gsc_impressions: row.get(7)?,
                citations: row.get(8)?,
                title: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;
    let mut out = BTreeMap::new();
    for rec in iter {
        let rec = rec.map_err(|error| error.to_string())?;
        out.insert(rec.url.clone(), rec);
    }
    Ok(out)
}

fn finding_maps(
    conn: &Connection,
    snapshot_id: &str,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT fingerprint, code, severity, url, summary
             FROM findings WHERE snapshot_id = ?1 ORDER BY code, url",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![snapshot_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for rec in iter {
        let (fingerprint, code, severity, url, summary) = rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("fingerprint".into(), fingerprint);
        row.insert("code".into(), code);
        row.insert("severity".into(), severity);
        row.insert("url".into(), url);
        row.insert("summary".into(), summary);
        rows.push(row);
    }
    Ok(rows)
}

fn family_maps(conn: &Connection, latest: &Run) -> Result<Vec<BTreeMap<String, String>>, String> {
    let previous = previous_run(conn, latest)?;
    let prev: BTreeMap<String, i64> = if let Some(run) = &previous {
        let mut stmt = conn
            .prepare("SELECT family, error_findings FROM families WHERE snapshot_id = ?1")
            .map_err(|error| error.to_string())?;
        let iter = stmt
            .query_map(params![run.id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
            })
            .map_err(|error| error.to_string())?;
        let mut map = BTreeMap::new();
        for rec in iter {
            let (family, errors) = rec.map_err(|error| error.to_string())?;
            map.insert(family, errors.unwrap_or(0));
        }
        map
    } else {
        BTreeMap::new()
    };
    let mut stmt = conn
        .prepare(
            "SELECT family, measured_urls, verdict, gsc_clicks, gsc_impressions,
                    error_findings, unique_fact_ratio
             FROM families WHERE snapshot_id = ?1 ORDER BY family",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![latest.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for rec in iter {
        let (family, measured, verdict, clicks, impressions, errors, unique) =
            rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("family".into(), family.clone());
        row.insert("measured_urls".into(), measured.to_string());
        if let Some(verdict) = verdict {
            row.insert("verdict".into(), verdict);
        }
        insert_opt(&mut row, "gsc_clicks", clicks);
        insert_opt(&mut row, "gsc_impressions", impressions);
        insert_opt(&mut row, "error_findings", errors);
        insert_opt(&mut row, "unique_fact_ratio", unique);
        if let Some(before) = prev.get(&family) {
            let now = errors.unwrap_or(0);
            row.insert("errors_delta".into(), (now - *before).to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

fn chunk_maps(conn: &Connection, latest: &Run) -> Result<Vec<BTreeMap<String, String>>, String> {
    let previous = previous_run(conn, latest)?;
    let revision_changed = previous
        .as_ref()
        .is_some_and(|run| run.repo_revision != latest.repo_revision);
    let citations = load_page_citations(conn, &latest.id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, url, heading, citation_suitability
             FROM chunks WHERE snapshot_id = ?1 ORDER BY url, id",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![latest.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for rec in iter {
        let (id, url, heading, suitability) = rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("id".into(), id);
        row.insert("url".into(), url.clone());
        row.insert("heading".into(), heading);
        insert_opt(&mut row, "citation_suitability", suitability);
        if let Some(hits) = citations.get(&url) {
            row.insert("citation_hits".into(), hits.to_string());
        }
        if previous.is_some() {
            row.insert(
                "source_revision_changed".into(),
                revision_changed.to_string(),
            );
        }
        rows.push(row);
    }
    Ok(rows)
}

fn claim_maps(
    conn: &Connection,
    snapshot_id: &str,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, claim, support_state, citations
             FROM claims WHERE snapshot_id = ?1 ORDER BY id",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![snapshot_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for rec in iter {
        let (id, claim, support, citations) = rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("id".into(), id);
        row.insert("claim".into(), claim);
        row.insert("support_state".into(), support);
        insert_opt(&mut row, "citations", citations);
        rows.push(row);
    }
    Ok(rows)
}

fn opportunity_maps(
    conn: &Connection,
    snapshot_id: &str,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, subject, summary
             FROM opportunities WHERE snapshot_id = ?1 ORDER BY id",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![snapshot_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for rec in iter {
        let (id, kind, subject, summary) = rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("id".into(), id);
        row.insert("kind".into(), kind);
        row.insert("subject".into(), subject);
        row.insert("summary".into(), summary);
        rows.push(row);
    }
    Ok(rows)
}

fn runs_maps(conn: &Connection) -> Result<Vec<BTreeMap<String, String>>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT snapshot_id, site, mode, recorded_at, measured_urls, findings, errors
             FROM runs ORDER BY recorded_at DESC, snapshot_id DESC",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for rec in iter {
        let (id, site, mode, recorded_at, urls, findings, errors) =
            rec.map_err(|error| error.to_string())?;
        let mut row = BTreeMap::new();
        row.insert("snapshot_id".into(), id);
        if let Some(site) = site {
            row.insert("site".into(), site);
        }
        row.insert("mode".into(), mode);
        row.insert("recorded_at".into(), recorded_at.to_string());
        row.insert("measured_urls".into(), urls.to_string());
        row.insert("findings".into(), findings.to_string());
        row.insert("errors".into(), errors.to_string());
        rows.push(row);
    }
    Ok(rows)
}

fn load_page_citations(
    conn: &Connection,
    snapshot_id: &str,
) -> Result<BTreeMap<String, i64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT url, citations FROM pages WHERE snapshot_id = ?1 AND citations IS NOT NULL",
        )
        .map_err(|error| error.to_string())?;
    let iter = stmt
        .query_map(params![snapshot_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|error| error.to_string())?;
    let mut out = BTreeMap::new();
    for rec in iter {
        let (url, citations) = rec.map_err(|error| error.to_string())?;
        out.insert(url, citations);
    }
    Ok(out)
}

fn inbound_counts(inventory: &Inventory) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        *counts.entry(edge.target.to_string()).or_insert(0) += 1;
    }
    counts
}

fn family_for(
    url: &str,
    families: &[weavatrix_seo_model::FamilyContent],
    producers: &[ProducerFact],
) -> Option<String> {
    families
        .iter()
        .find(|family| url.contains(&family.family))
        .map(|family| family.family.clone())
        .or_else(|| {
            producers
                .iter()
                .find(|producer| producer.families.iter().any(|family| url.contains(family)))
                .and_then(|producer| producer.families.first().cloned())
        })
}

fn producer_for(url: &str, producers: &[ProducerFact]) -> (Option<String>, Option<String>) {
    producers
        .iter()
        .find(|producer| producer.families.iter().any(|family| url.contains(family)))
        .map_or((None, None), |producer| {
            let hash = producer.symbol_hash.unwrap_or(producer.content_hash);
            (Some(producer.key()), Some(hash.hex()))
        })
}

fn claim_findings(report: &AuditReport) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| finding.family == FindingFamily::Claim)
        .flat_map(|finding| {
            [
                finding.locator.subject_url().to_owned(),
                finding.summary.clone(),
            ]
        })
        .collect()
}

fn indexable_label(indexability: Indexability) -> String {
    match indexability {
        Indexability::Indexable => "true".into(),
        _ => "false".into(),
    }
}

fn pct_delta(current: i64, previous: i64) -> Option<i64> {
    if previous == 0 {
        return None;
    }
    Some((current - previous) * 100 / previous)
}

fn insert_opt(row: &mut BTreeMap<String, String>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        row.insert(key.to_owned(), value.to_string());
    }
}

fn unix_now() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs()),
    )
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{ingest_at, query_maps};
    use weavatrix_seo_model::{
        AbsoluteUrl, AnalysisMode, AuditReport, Chunk, ContentHash, Evidence, ExtractedPage,
        FamilyContent, Heading, Indexability, Inventory, MediaKind, ProducerFact,
        SearchIntelligence, UrlMetric,
    };

    fn url(path: &str) -> AbsoluteUrl {
        AbsoluteUrl::parse(&format!("https://x.test{path}")).expect("url")
    }

    fn page(path: &str) -> ExtractedPage {
        let parsed = url(path);
        ExtractedPage {
            url: parsed.clone(),
            requested: parsed,
            status: 200,
            redirects: Vec::new(),
            content_type: Some("text/html".into()),
            media: MediaKind::Html,
            canonical: None,
            robots: Vec::new(),
            title: Some(path.into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: vec![Heading {
                level: 1,
                text: path.into(),
            }],
            links: Vec::new(),
            link_refs: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: path.into(),
            heading_text: path.into(),
            main_text: String::new(),
            payload: String::new(),
            arbitrary_script: String::new(),
            og_title: None,
            og_description: None,
            og_image: None,
            headers: Vec::new(),
            csp_meta: None,
            body_bytes: 1,
            fetch_ms: 1,
            has_main: true,
            unlabeled_controls: 0,
            content_hash: ContentHash::of_str(path),
            indexability: Indexability::Indexable,
            in_sitemap: true,
            linked_from_page: true,
            evidence: Evidence::http(),
        }
        .finalize()
    }

    fn report(
        snapshot: &str,
        clicks: u32,
        errors: u32,
        producer: &str,
        revision: Option<&str>,
    ) -> AuditReport {
        let page_url = "https://x.test/category/electrician/haifa";
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.snapshot_id = snapshot.into();
        inventory.site = Some("https://x.test/".into());
        inventory.repo_revision = revision.map(str::to_owned);
        inventory.pages = vec![page("/category/electrician/haifa")];
        inventory.producers = vec![ProducerFact {
            path: "src/page.tsx".into(),
            name: "Page".into(),
            content_hash: ContentHash::of_str(producer),
            families: vec!["category/electrician".into()],
            symbol_hash: Some(ContentHash::of_str(producer)),
            start_line: None,
            end_line: None,
        }];
        let intel = SearchIntelligence {
            url_metrics: vec![UrlMetric {
                url: page_url.into(),
                gsc_clicks: Some(clicks),
                gsc_impressions: Some(800),
                citations: Some(3),
            }],
            families: vec![FamilyContent {
                family: "category/electrician".into(),
                measured_urls: 1,
                template_shared_ratio: None,
                parameter_substitution_ratio: None,
                unique_fact_ratio: Some(20),
                unique_semantic_ratio: None,
                local_fact_coverage: None,
                schema_fact_coverage: None,
                primary_producer: None,
                gsc_clicks: Some(clicks),
                gsc_impressions: Some(800),
                error_findings: Some(errors),
            }],
            chunks: vec![Chunk {
                id: "chunk:haifa#0".into(),
                url: page_url.into(),
                heading: "Electrician".into(),
                text: "Licensed.".into(),
                cohesion: None,
                self_contained: None,
                answer_density: None,
                specificity: None,
                citation_suitability: Some(70),
                witness: None,
                relevance: None,
                retrieval_model: None,
                why: None,
            }],
            ..SearchIntelligence::default()
        };
        AuditReport {
            inventory,
            findings: Vec::new(),
            axes: Vec::new(),
            opportunities: Vec::new(),
            intelligence: Some(intel),
        }
    }

    fn temp_dir() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let path =
            std::env::temp_dir().join(format!("wvx-seo-sqlite-{}-{nanos}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        path
    }

    #[test]
    fn historical_query_exposes_deltas_and_producer_change() {
        let dir = temp_dir();
        let dir_s = dir.to_string_lossy().into_owned();
        let t0 = 1_700_000_000;
        ingest_at(&dir_s, &report("snap-a", 40, 1, "old-producer", None), t0).expect("ingest a");
        ingest_at(
            &dir_s,
            &report("snap-b", 10, 3, "new-producer", Some("abc")),
            t0 + 28 * 86_400,
        )
        .expect("ingest b");
        let urls = query_maps(&dir_s, "urls").expect("urls");
        assert_eq!(urls.len(), 1);
        assert_eq!(
            urls[0].get("clicks_delta_28d").map(String::as_str),
            Some("-75")
        );
        assert_eq!(
            urls[0].get("producer_changed").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            urls[0].get("source_revision_changed").map(String::as_str),
            Some("true")
        );
        let families = query_maps(&dir_s, "route_families").expect("families");
        assert_eq!(
            families[0].get("errors_delta").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            families[0].get("gsc_clicks").map(String::as_str),
            Some("10")
        );
        let chunks = query_maps(&dir_s, "chunks").expect("chunks");
        assert_eq!(
            chunks[0].get("citation_hits").map(String::as_str),
            Some("3")
        );
        let runs = query_maps(&dir_s, "runs").expect("runs");
        assert_eq!(runs.len(), 2);
        let _ = std::fs::remove_dir_all(dir);
    }
}
