//! Path and glob helpers for agent guidance contract checks.

use crate::cli::check::CheckError;
use glob::Pattern;
use std::path::{Component, Path, PathBuf};

pub(super) struct CompiledPattern {
    raw: String,
    pattern: Pattern,
}

pub(super) fn compile_agent_guidance_patterns(
    patterns: &[String],
) -> Result<Vec<CompiledPattern>, CheckError> {
    patterns
        .iter()
        .map(|raw| {
            Pattern::new(raw)
                .map(|pattern| CompiledPattern {
                    raw: raw.clone(),
                    pattern,
                })
                .map_err(|error| {
                    CheckError::Config(crate::cli::config::ConfigError::Invalid(format!(
                        "agent guidance pattern `{raw}` is invalid: {error}"
                    )))
                })
        })
        .collect()
}

pub(super) fn pattern_matches_any(patterns: &[CompiledPattern], rel: &Path) -> bool {
    patterns.iter().any(|compiled| {
        compiled.pattern.matches_path(rel) && pattern_depth_allows(&compiled.raw, rel)
    })
}

fn pattern_depth_allows(pattern: &str, path: &Path) -> bool {
    pattern.contains("**")
        || pattern.split('/').filter(|part| !part.is_empty()).count() == path.components().count()
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
