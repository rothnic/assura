//! Path and glob helpers for agent guidance contract checks.

use crate::cli::check::patterns::DepthAwareGlob;
use crate::cli::check::CheckError;
use std::path::{Component, Path, PathBuf};

pub(super) fn compile_agent_guidance_patterns(
    patterns: &[String],
) -> Result<Vec<DepthAwareGlob>, CheckError> {
    patterns
        .iter()
        .map(|raw| {
            DepthAwareGlob::new(raw).map_err(|error| {
                CheckError::Config(crate::cli::config::ConfigError::Invalid(format!(
                    "agent guidance pattern `{raw}` is invalid: {error}"
                )))
            })
        })
        .collect()
}

pub(super) fn safe_agent_guidance_path(path: &str) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || rel.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(format!(
                "agent guidance path `{path}` must be a safe project-relative path"
            )),
        ));
    }
    Ok(rel)
}

pub(super) fn checked_path_relevant_to(checked_path: &Path, target_abs: &Path) -> bool {
    checked_path == target_abs || checked_path.is_dir() && target_abs.starts_with(checked_path)
}

pub(super) fn checked_path_relevant_to_any(
    checked_path: &Path,
    project_root: &Path,
    targets_rel: &[PathBuf],
) -> bool {
    targets_rel.iter().any(|target| {
        let target_abs = project_root.join(target);
        checked_path == target_abs || checked_path.is_dir() && target_abs.starts_with(checked_path)
    })
}
