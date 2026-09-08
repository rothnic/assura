//! Status model for managed Git hook pairs.

use super::HookType;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct HookStatus {
    pub hook_type: HookType,
    pub is_installed: bool,
    pub is_managed: bool,
    /// Whether both managed artifacts match the current safe generator.
    pub is_current: bool,
    pub git_runnable: bool,
    pub assura_runnable: bool,
    pub git_path: PathBuf,
    pub assura_path: PathBuf,
}

impl HookStatus {
    pub fn is_ready(&self) -> bool {
        self.is_installed
            && self.is_managed
            && self.is_current
            && self.git_runnable
            && self.assura_runnable
    }

    pub fn display(&self) -> String {
        let status = if self.is_ready() {
            "✓ installed (managed by assura, runnable)"
        } else if self.is_installed {
            if self.is_managed && !self.is_current {
                "⚠ installed (managed by assura, upgrade required)"
            } else if self.is_managed {
                "⚠ installed (managed by assura, not runnable)"
            } else {
                "⚠ installed (not managed by assura)"
            }
        } else {
            "✗ not installed"
        };

        format!("{:<20} {}", self.hook_type.as_str(), status)
    }
}

#[cfg(unix)]
pub(super) fn is_runnable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.symlink_metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
pub(super) fn is_runnable(path: &Path) -> bool {
    path.is_file()
}
