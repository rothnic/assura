//! Native quality-tool discovery for generated onboarding policy.

use std::fs;
use std::path::Path;

/// Return declared quality scripts only for an explicit Bun project.
pub(super) fn declared_bun_quality_scripts(
    project_root: &Path,
    has_package_json: bool,
) -> Vec<String> {
    if !has_package_json {
        return Vec::new();
    }
    let Ok(contents) = fs::read_to_string(project_root.join("package.json")) else {
        return Vec::new();
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return Vec::new();
    };
    let is_bun = manifest
        .get("packageManager")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|manager| manager.starts_with("bun@"));
    if !is_bun {
        return Vec::new();
    }
    ["lint", "test"]
        .into_iter()
        .filter(|name| {
            manifest["scripts"]
                .get(*name)
                .and_then(serde_json::Value::as_str)
                .is_some()
        })
        .map(str::to_string)
        .collect()
}
