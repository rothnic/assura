//! Git-owned hook-directory resolution for the hook lifecycle.
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{HookError, HookResult};

/// Resolves the effective Git hook directory for a project.
///
/// Git owns `core.hooksPath` precedence and relative-path semantics, so use its
/// path resolver whenever the project is a real Git repository. The metadata
/// fallback keeps the lifecycle fixtures and partial repositories supported.
pub(super) fn resolve_git_hooks_dir(project_root: &Path) -> HookResult<PathBuf> {
    if let Some(hooks_dir) = git_effective_hooks_dir(project_root) {
        return Ok(hooks_dir);
    }

    let git_path = project_root.join(".git");
    if git_path.is_dir() {
        return Ok(git_path.join("hooks"));
    }
    if git_path.is_file() {
        let git_dir = resolve_gitdir_file(&git_path)?;
        return Ok(resolve_common_git_dir(&git_dir).join("hooks"));
    }
    Err(HookError::GitNotFound)
}

fn git_effective_hooks_dir(project_root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--path-format=absolute", "--git-path", "hooks"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    git_path_from_output(&output.stdout)
}

fn git_path_from_output(output: &[u8]) -> Option<PathBuf> {
    let output = output.strip_suffix(b"\n")?;

    #[cfg(unix)]
    {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        Some(PathBuf::from(OsString::from_vec(output.to_vec())))
    }

    #[cfg(not(unix))]
    {
        Some(PathBuf::from(std::str::from_utf8(output).ok()?))
    }
}

fn resolve_gitdir_file(git_path: &Path) -> HookResult<PathBuf> {
    let content = std::fs::read_to_string(git_path)?;
    let Some(first_line) = content.lines().next() else {
        return Err(HookError::GitNotFound);
    };
    let Some(raw_git_dir) = first_line.trim().strip_prefix("gitdir:") else {
        return Err(HookError::GitNotFound);
    };
    let git_dir = PathBuf::from(raw_git_dir.trim());
    if git_dir.is_absolute() {
        Ok(git_dir)
    } else {
        Ok(git_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(git_dir))
    }
}

fn resolve_common_git_dir(git_dir: &Path) -> PathBuf {
    let common_dir_file = git_dir.join("commondir");
    let Ok(content) = std::fs::read_to_string(&common_dir_file) else {
        return git_dir.to_path_buf();
    };
    let Some(first_line) = content.lines().next() else {
        return git_dir.to_path_buf();
    };
    let common_dir = PathBuf::from(first_line.trim());
    let resolved = if common_dir.is_absolute() {
        common_dir
    } else {
        git_dir.join(common_dir)
    };
    std::fs::canonicalize(&resolved).unwrap_or(resolved)
}
