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
    Ok(path.to_string_lossy().into_owned())
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
        let _ = std::fs::remove_dir_all(dir);
    }
}
