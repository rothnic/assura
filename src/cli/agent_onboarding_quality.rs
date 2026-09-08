//! Native quality-tool discovery for generated onboarding policy.

use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub(super) struct QualityAdvice {
    pub(super) tool: &'static str,
    pub(super) status: &'static str,
}

pub(super) fn python_quality_advice(project_root: &Path) -> Vec<QualityAdvice> {
    let Ok(contents) = fs::read_to_string(project_root.join("pyproject.toml")) else {
        return Vec::new();
    };
    let Ok(pyproject) = contents.parse::<toml::Value>() else {
        return Vec::new();
    };
    let pytest_configured = pyproject
        .get("tool")
        .and_then(|tool| tool.get("pytest"))
        .and_then(|pytest| pytest.get("ini_options"))
        .is_some_and(toml::Value::is_table);
    if !pytest_configured {
        return Vec::new();
    }
    vec![QualityAdvice {
        tool: "pytest",
        status: if executable_on_path("pytest") {
            "available"
        } else {
            "unavailable"
        },
    }]
}

fn executable_on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|directory| directory.join(name).is_file())
}

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
