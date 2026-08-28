//! Filesystem boundary for the agent surface.
//!
//! "No shell" is not "no filesystem capability". Every path a tool accepts —
//! repository, provider export, render snapshot, history directory, baseline,
//! diff sides — is resolved and checked against an allow-list before the engine
//! sees it. Resolution is done with [`std::fs::canonicalize`], so `..` segments
//! and symlinks cannot point outside a root.

use std::path::{Path, PathBuf};

/// Allowed filesystem roots for caller-supplied paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roots {
    allowed: Vec<PathBuf>,
}

impl Roots {
    /// Builds the boundary.
    ///
    /// An empty list falls back to the working directory, which is where a
    /// plugin launcher starts the server for the repository in scope.
    #[must_use]
    pub fn new(paths: &[String]) -> Self {
        let mut allowed: Vec<PathBuf> = paths
            .iter()
            .filter_map(|path| Path::new(path).canonicalize().ok())
            .collect();
        if allowed.is_empty()
            && let Ok(cwd) = std::env::current_dir()
            && let Ok(resolved) = cwd.canonicalize()
        {
            allowed.push(resolved);
        }
        Self { allowed }
    }

    /// Roots in effect, for diagnostics.
    #[must_use]
    pub fn allowed(&self) -> &[PathBuf] {
        &self.allowed
    }

    /// Resolves a caller path inside the boundary.
    ///
    /// # Errors
    ///
    /// Returns a message naming the input when the path escapes every root or
    /// cannot be resolved.
    pub fn resolve(&self, label: &str, path: &str) -> Result<String, String> {
        if self.allowed.is_empty() {
            return Err(format!(
                "{label} was refused: this server has no allowed filesystem root"
            ));
        }
        let resolved = canonical_target(path)
            .map_err(|error| format!("{label} `{path}` could not be resolved: {error}"))?;
        if self
            .allowed
            .iter()
            .any(|root| resolved.starts_with(root) || resolved == *root)
        {
            return Ok(resolved.to_string_lossy().into_owned());
        }
        Err(format!(
            "{label} `{path}` is outside the allowed roots ({}). Start the server with --allow-root to widen it.",
            self.allowed
                .iter()
                .map(|root| root.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    /// Resolves an optional path.
    ///
    /// # Errors
    ///
    /// Propagates [`Roots::resolve`].
    pub fn resolve_optional(
        &self,
        label: &str,
        path: Option<&String>,
    ) -> Result<Option<String>, String> {
        path.map(|value| self.resolve(label, value)).transpose()
    }
}

/// Canonical form of a target that may not exist yet.
///
/// A history directory is an output path, so the leaf can be missing. Every
/// existing ancestor is still resolved, which is what closes the symlink and
/// `..` escapes.
fn canonical_target(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    let absolute = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(raw)
    };
    canonical_existing(&absolute)
}

fn canonical_existing(path: &Path) -> Result<PathBuf, String> {
    if let Ok(resolved) = path.canonicalize() {
        return Ok(resolved);
    }
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Err("path does not exist and has no resolvable parent".to_owned());
    };
    Ok(canonical_existing(parent)?.join(name))
}

#[cfg(test)]
mod tests {
    use super::Roots;

    fn sandbox(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wvx-seo-roots-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("inside")).expect("sandbox");
        dir.canonicalize().expect("canonical sandbox")
    }

    #[test]
    fn a_path_inside_a_root_resolves() {
        let dir = sandbox("inside");
        let roots = Roots::new(&[dir.to_string_lossy().into_owned()]);
        let target = dir.join("inside");
        assert!(roots.resolve("repo", &target.to_string_lossy()).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_path_outside_every_root_is_refused() {
        let dir = sandbox("outside");
        let roots = Roots::new(&[dir.join("inside").to_string_lossy().into_owned()]);
        let error = roots
            .resolve("observations", &dir.to_string_lossy())
            .expect_err("must refuse");
        assert!(error.contains("outside the allowed roots"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_parent_escape_cannot_climb_out() {
        let dir = sandbox("escape");
        let roots = Roots::new(&[dir.join("inside").to_string_lossy().into_owned()]);
        let climb = dir.join("inside").join("..").join("elsewhere.json");
        let error = roots
            .resolve("gsc", &climb.to_string_lossy())
            .expect_err("must refuse");
        assert!(error.contains("outside the allowed roots"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_output_directory_need_not_exist_yet() {
        let dir = sandbox("output");
        let roots = Roots::new(&[dir.to_string_lossy().into_owned()]);
        let target = dir.join("history-not-created-yet");
        assert!(roots.resolve("history", &target.to_string_lossy()).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_absent_path_outside_a_root_is_still_refused() {
        let dir = sandbox("absent");
        let roots = Roots::new(&[dir.join("inside").to_string_lossy().into_owned()]);
        let target = dir.join("nope").join("still-nope.json");
        assert!(
            roots
                .resolve("baseline", &target.to_string_lossy())
                .is_err()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
