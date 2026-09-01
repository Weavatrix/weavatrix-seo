//! Directory persistence for compact snapshots.

use crate::StoredSnapshot;
use std::fs;
use std::path::Path;
use weavatrix_seo_model::AuditReport;

/// Writes `{dir}/{snapshot_id}.json`. Creates `dir` when missing.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn save(dir: &str, report: &AuditReport) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let snapshot = StoredSnapshot::from_report(report);
    let name = if snapshot.snapshot_id.is_empty() {
        "snapshot".into()
    } else {
        snapshot.snapshot_id.clone()
    };
    let path = Path::new(dir).join(format!("{name}.json"));
    let body = blazingly_json::to_string(&snapshot).map_err(|error| error.to_string())?;
    fs::write(&path, body).map_err(|error| error.to_string())?;
    append_index(dir, report, &snapshot);
    Ok(path.to_string_lossy().into_owned())
}

/// One line in `{dir}/index.jsonl`. Cheap history for `seo_query`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryIndexRow {
    /// Snapshot id.
    pub snapshot_id: String,
    /// Site seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Mode.
    pub mode: weavatrix_seo_model::AnalysisMode,
    /// Measured URL count.
    pub measured_urls: usize,
    /// Finding count.
    pub findings: usize,
    /// Error findings.
    pub errors: usize,
}

fn append_index(dir: &str, report: &AuditReport, snapshot: &StoredSnapshot) {
    let row = HistoryIndexRow {
        snapshot_id: snapshot.snapshot_id.clone(),
        site: snapshot.site.clone(),
        mode: snapshot.mode,
        measured_urls: snapshot.pages.len(),
        findings: snapshot.findings.len(),
        errors: report
            .findings
            .iter()
            .filter(|item| item.severity == weavatrix_seo_model::Severity::Error)
            .count(),
    };
    let Ok(line) = blazingly_json::to_string(&row) else {
        return;
    };
    let path = Path::new(dir).join("index.jsonl");
    let mut body = fs::read_to_string(&path).unwrap_or_default();
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(&line);
    body.push('\n');
    let _ = fs::write(path, body);
}

/// Reads `{dir}/index.jsonl` when present.
///
/// # Errors
///
/// Returns IO errors. Malformed lines are skipped.
pub fn load_index(dir: &str) -> Result<Vec<HistoryIndexRow>, String> {
    let path = Path::new(dir).join("index.jsonl");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(row) = blazingly_json::from_str::<HistoryIndexRow>(line) {
            rows.push(row);
        }
    }
    Ok(rows)
}

/// Loads a compact snapshot or a full audit JSON.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load(path: &str) -> Result<StoredSnapshot, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let raw = weavatrix_seo_model::strip_bom(&raw);
    if let Ok(snapshot) = blazingly_json::from_str::<StoredSnapshot>(raw)
        && snapshot.schema.starts_with("weavatrix-seo-snapshot")
    {
        return Ok(snapshot);
    }
    let report: AuditReport = blazingly_json::from_str(raw).map_err(|error| error.to_string())?;
    Ok(StoredSnapshot::from_report(&report))
}

#[cfg(test)]
mod tests {
    use super::load;
    use crate::StoredSnapshot;
    use weavatrix_seo_model::{AnalysisMode, AuditReport, Inventory};

    #[test]
    fn roundtrip_blank_report() {
        let report = AuditReport {
            inventory: Inventory::blank(AnalysisMode::Site),
            findings: Vec::new(),
            axes: Vec::new(),
            opportunities: Vec::new(),
            intelligence: None,
        };
        let dir = std::env::temp_dir().join(format!(
            "wvx-hist-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let path = super::save(dir.to_string_lossy().as_ref(), &report).expect("save");
        let loaded = load(&path).expect("load");
        assert_eq!(loaded.schema, StoredSnapshot::from_report(&report).schema);
        let index = super::load_index(dir.to_string_lossy().as_ref()).expect("index");
        assert_eq!(index.len(), 1);
        assert_eq!(index[0].snapshot_id, report.inventory.snapshot_id);
        let _ = std::fs::remove_dir_all(dir);
    }
}
