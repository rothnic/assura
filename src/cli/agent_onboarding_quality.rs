//! Native quality detection and policy rendering for generated onboarding.

use super::agent_onboarding::DetectedSection;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub(super) struct QualityAdvice {
    pub(super) tool: &'static str,
    pub(super) status: &'static str,
}

pub(super) fn python_quality_advice(project_root: &Path) -> Vec<QualityAdvice> {
    configured_python_tools(project_root)
        .into_iter()
        .map(|tool| QualityAdvice {
            tool,
            status: if executable_on_path(tool) {
                "available"
            } else {
                "unavailable"
            },
        })
        .collect()
}

/// Gather setup advice for all configured native-quality runtimes.
pub(super) fn onboarding_quality_advice(project_root: &Path) -> Vec<QualityAdvice> {
    let mut advice = python_quality_advice(project_root);
    advice.extend(bun_quality_advice(
        project_root,
        project_root.join("package.json").is_file(),
    ));
    advice
}

/// Return configured Python quality tools that can actually run locally.
pub(super) fn available_python_quality_tools(project_root: &Path) -> Vec<&'static str> {
    configured_python_tools(project_root)
        .into_iter()
        .filter(|tool| executable_on_path(tool))
        .collect()
}

/// Report an explicit Bun project whose runtime is not locally executable.
pub(super) fn bun_quality_advice(
    project_root: &Path,
    has_package_json: bool,
) -> Vec<QualityAdvice> {
    if !explicit_bun_project(project_root, has_package_json) {
        return Vec::new();
    }
    vec![QualityAdvice {
        tool: "bun",
        status: if bun_available(project_root, has_package_json) {
            "available"
        } else {
            "unavailable"
        },
    }]
}

/// Whether an explicit Bun project has a runnable local Bun executable.
pub(super) fn bun_available(project_root: &Path, has_package_json: bool) -> bool {
    explicit_bun_project(project_root, has_package_json) && executable_on_path("bun")
}

/// Render the generated quality policy for the detected local project.
pub(super) fn quality_config(detected: &DetectedSection) -> String {
    let mut config = String::from(QUALITY_POLICY_SCOPES);
    if detected.project_type == "rust" {
        config.push_str(rust_quality_scope());
        return config;
    }
    if !detected.python_quality_tools.is_empty() {
        let frequent = if detected.python_quality_tools.contains(&"ruff") {
            "      frequent:\n        - \"ruff check .\"\n"
        } else {
            ""
        };
        let pre_push = if detected.python_quality_tools.contains(&"pytest") {
            "      pre_push:\n        - \"pytest\"\n"
        } else {
            ""
        };
        let pr = if detected.python_quality_tools.contains(&"mypy") {
            "      pr:\n        - \"mypy .\"\n"
        } else {
            ""
        };
        config.push_str(&format!(
            r#"    python:
      paths:
        - "src/**"
        - "tests/**"
        - "pyproject.toml"
      always:
        - "assura check"
{frequent}{pre_push}{pr}"#
        ));
        return config;
    }
    if !detected.bun_available {
        return config;
    }
    let frequent = if detected.bun_scripts.iter().any(|script| script == "lint") {
        "      frequent:\n        - \"bun run lint\"\n"
    } else {
        ""
    };
    let pre_push = if detected.bun_scripts.iter().any(|script| script == "test") {
        "      pre_push:\n        - \"bun run test\"\n"
    } else {
        "      pre_push:\n        - \"bun test\"\n"
    };
    config.push_str(&format!(
        "    bun:\n      paths:\n        - \"src/**\"\n        - \"tests/**\"\n        - \"package.json\"\n        - \"bun.lock\"\n      always:\n        - \"assura check\"\n{frequent}{pre_push}"
    ));
    config
}

const QUALITY_POLICY_SCOPES: &str = r#"quality:
  scopes:
    policy:
      paths:
        - "**"
      always:
        - "assura check"
    project-config:
      paths:
        - ".assura/**"
        - "AGENTS.md"
        - ".agents/**"
        - ".github/workflows/**"
        - ".gitlab-ci.yml"
        - ".circleci/**"
      always:
        - "assura check"
"#;

fn rust_quality_scope() -> &'static str {
    r#"    rust:
      paths:
        - "src/**"
        - "tests/**"
        - "examples/**"
        - "benches/**"
        - "crates/**"
        - "Cargo.toml"
        - "**/Cargo.toml"
        - "Cargo.lock"
        - "**/Cargo.lock"
        - "build.rs"
        - "**/build.rs"
        - "rust-toolchain"
        - "rust-toolchain.toml"
        - "**/rust-toolchain"
        - "**/rust-toolchain.toml"
        - ".cargo/**"
        - "**/.cargo/**"
        - "rustfmt.toml"
        - ".rustfmt.toml"
        - "clippy.toml"
        - ".clippy.toml"
        - "**/rustfmt.toml"
        - "**/.rustfmt.toml"
        - "**/clippy.toml"
        - "**/.clippy.toml"
        - ".assura/**"
        - "AGENTS.md"
        - ".agents/**"
        - ".github/workflows/**"
        - ".gitlab-ci.yml"
        - ".circleci/**"
      always:
        - "assura check"
      frequent:
        - "cargo fmt --all -- --check"
      pre_push:
        - "cargo test --locked"
      pr:
        - "cargo clippy --all-targets -- -D warnings"
"#
}

fn configured_python_tools(project_root: &Path) -> Vec<&'static str> {
    let Ok(contents) = fs::read_to_string(project_root.join("pyproject.toml")) else {
        return Vec::new();
    };
    let Ok(pyproject) = contents.parse::<toml::Value>() else {
        return Vec::new();
    };
    let tool = pyproject.get("tool");
    [
        (
            tool.and_then(|tool| tool.get("pytest"))
                .and_then(|pytest| pytest.get("ini_options"))
                .is_some_and(toml::Value::is_table),
            "pytest",
        ),
        (
            tool.and_then(|tool| tool.get("ruff"))
                .is_some_and(toml::Value::is_table),
            "ruff",
        ),
        (
            tool.and_then(|tool| tool.get("mypy"))
                .is_some_and(toml::Value::is_table),
            "mypy",
        ),
    ]
    .into_iter()
    .filter_map(|(configured, tool)| configured.then_some(tool))
    .collect()
}

#[cfg(unix)]
fn executable_on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|directory| executable_file(&directory.join(name)))
}

#[cfg(unix)]
fn executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn executable_on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let extensions = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    std::env::split_paths(&path).any(|directory| {
        extensions
            .iter()
            .any(|extension| directory.join(format!("{name}{extension}")).is_file())
    })
}

#[cfg(not(any(unix, windows)))]
fn executable_on_path(_name: &str) -> bool {
    false
}

/// Return declared quality scripts only for an explicit Bun project.
pub(super) fn declared_bun_quality_scripts(
    project_root: &Path,
    has_package_json: bool,
) -> Vec<String> {
    if !bun_available(project_root, has_package_json) {
        return Vec::new();
    }
    let Ok(contents) = fs::read_to_string(project_root.join("package.json")) else {
        return Vec::new();
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return Vec::new();
    };
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

fn explicit_bun_project(project_root: &Path, has_package_json: bool) -> bool {
    if !has_package_json {
        return false;
    }
    let Ok(contents) = fs::read_to_string(project_root.join("package.json")) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&contents)
        .ok()
        .and_then(|manifest| {
            manifest
                .get("packageManager")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|manager| manager.starts_with("bun@"))
}
