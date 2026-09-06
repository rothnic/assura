//! Deterministic specialization evidence for agent onboarding.

use super::agent_onboarding::DetectedSection;
use super::agent_onboarding_report::CheckItem;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Write the evidence record describing the selected onboarding specialization.
pub(super) fn write_specialization_profile(
    project_root: &Path,
    detected: &DetectedSection,
    recipe_file: Option<&Path>,
    verified: &[CheckItem],
) -> Result<(), String> {
    let (profile, source, source_path, stack) = match recipe_file {
        Some(recipe_file) => (
            "local-policy",
            recipe_file.display().to_string(),
            recipe_file.to_path_buf(),
            detected.project_type,
        ),
        None => match detected.project_type {
            "rust" => (
                "rust-library",
                "Cargo.toml".to_string(),
                project_root.join("Cargo.toml"),
                "rust",
            ),
            "node" => (
                "typescript-bun-utility",
                "package.json".to_string(),
                project_root.join("package.json"),
                "node",
            ),
            "python" => (
                "python-pytest",
                "pyproject.toml".to_string(),
                project_root.join("pyproject.toml"),
                "python",
            ),
            _ => (
                "repository-default",
                "repository inspection".to_string(),
                project_root.join(".assura/config.yml"),
                detected.project_type,
            ),
        },
    };
    let source_hash = fs::read(&source_path)
        .map(|contents| format!("{:x}", Sha256::digest(contents)))
        .unwrap_or_else(|_| format!("{:x}", Sha256::digest(source.as_bytes())));
    let config_status = verified
        .iter()
        .find(|item| item.name == "structure_config")
        .map(|item| item.status)
        .unwrap_or("fail");
    let profile = serde_json::json!({
        "schema": "assura.profile-selection.v1",
        "profile": profile,
        "source": source,
        "source_hash": source_hash,
        "decisions": [{"key": "stack", "value": stack, "evidence": source}],
        "conflicts": [],
        "verification": {"config": config_status},
    });
    let directory = project_root.join(".assura/onboarding");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    fs::write(
        directory.join("profile-selection.json"),
        serde_json::to_vec_pretty(&profile).expect("specialization profile is serializable"),
    )
    .map_err(|error| error.to_string())
}
