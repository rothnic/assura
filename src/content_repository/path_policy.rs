//! Project-relative path policy for content model artifacts.

use super::model::ContentFinding;
use std::path::{Component, Path, PathBuf};

pub(super) fn normalize_rel_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => normalized.push(".."),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
        }
    }
    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

pub(super) fn project_relative_model_artifact_path(
    value: &str,
    field: &str,
    path_escape_code: &'static str,
    findings: &mut Vec<ContentFinding>,
) -> Option<PathBuf> {
    let rel_path = project_relative_path(value, field, path_escape_code, findings)?;
    if rel_path.starts_with(Path::new(".assura"))
        && !rel_path.starts_with(Path::new(".assura/models"))
    {
        findings.push(ContentFinding::new(
            "content_model_artifact_outside_models_dir",
            Some(rel_path.clone()),
            format!(
                "{field} uses '{}'. Model artifacts inside .assura must live under .assura/models/**; move the artifact there or choose a path outside .assura/.",
                rel_path.display()
            ),
        ));
        None
    } else {
        Some(rel_path)
    }
}

fn project_relative_path(
    value: &str,
    field: &str,
    code: &'static str,
    findings: &mut Vec<ContentFinding>,
) -> Option<PathBuf> {
    let path = Path::new(value);
    let invalid = value.trim().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)));
    if invalid {
        findings.push(ContentFinding::new(
            code,
            None,
            format!("{field} must be a non-empty project-relative path"),
        ));
        None
    } else {
        Some(normalize_rel_path(path.to_path_buf()))
    }
}
