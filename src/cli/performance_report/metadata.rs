//! Runtime metadata helpers for performance report envelopes.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SourceProvenance {
    pub(super) source_commit_sha: Option<String>,
    pub(super) source_branch: Option<String>,
    pub(super) source_patch_id: Option<String>,
    pub(super) source_worktree_dirty: Option<bool>,
}

pub(super) fn git_value<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

pub(super) fn utc_timestamp() -> String {
    match Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => format!("unix:{}", unix_seconds()),
    }
}

pub(super) fn source_provenance_from_env() -> SourceProvenance {
    SourceProvenance {
        source_commit_sha: env_value("ASSURA_SOURCE_COMMIT_SHA"),
        source_branch: env_value("ASSURA_SOURCE_BRANCH"),
        source_patch_id: env_value("ASSURA_SOURCE_PATCH_ID"),
        source_worktree_dirty: env_bool("ASSURA_SOURCE_WORKTREE_DIRTY"),
    }
}

pub(super) fn resolve_source_worktree_dirty(
    measured_worktree_dirty: bool,
    source_provenance: &SourceProvenance,
) -> bool {
    source_provenance
        .source_worktree_dirty
        .unwrap_or(measured_worktree_dirty)
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_bool(name: &str) -> Option<bool> {
    let value = env_value(name)?;
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::source_provenance_from_env;

    #[test]
    fn source_provenance_reads_env_values() {
        unsafe {
            std::env::set_var("ASSURA_SOURCE_COMMIT_SHA", "source-commit");
            std::env::set_var("ASSURA_SOURCE_BRANCH", "source-branch");
            std::env::set_var("ASSURA_SOURCE_PATCH_ID", "source-patch");
            std::env::set_var("ASSURA_SOURCE_WORKTREE_DIRTY", "true");
        }

        let provenance = source_provenance_from_env();

        assert_eq!(
            provenance.source_commit_sha.as_deref(),
            Some("source-commit")
        );
        assert_eq!(provenance.source_branch.as_deref(), Some("source-branch"));
        assert_eq!(provenance.source_patch_id.as_deref(), Some("source-patch"));
        assert_eq!(provenance.source_worktree_dirty, Some(true));

        unsafe {
            std::env::remove_var("ASSURA_SOURCE_COMMIT_SHA");
            std::env::remove_var("ASSURA_SOURCE_BRANCH");
            std::env::remove_var("ASSURA_SOURCE_PATCH_ID");
            std::env::remove_var("ASSURA_SOURCE_WORKTREE_DIRTY");
        }
    }
}
