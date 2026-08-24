//! CLI usage and version.

use weavatrix_seo_cli::{run, usage};

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

#[test]
fn help_and_version() {
    let help = run(&argv(&["--help"]));
    assert_eq!(help.code, 0);
    assert!(help.stdout.contains("weavatrix-seo audit"));
    let version = run(&argv(&["--version"]));
    assert_eq!(version.code, 0);
    assert!(version.stdout.contains("weavatrix-seo"));
}

#[test]
fn missing_site_is_usage() {
    let output = run(&argv(&["audit"]));
    assert_eq!(output.code, 2);
    assert!(output.stderr.contains("--site") || output.stderr.contains("Usage"));
    let _ = usage();
}
