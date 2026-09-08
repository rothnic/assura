//! Narrow, byte-preserving pre-commit configuration integration.

use std::io::Write;
use std::path::Path;
use std::process::Command;

const PRE_COMMIT_VERSION: &str = "pre-commit 4.6.2";
const PRE_COMMIT_BLOCK_START: &str = "# >>> Assura pre-commit integration >>>";
const PRE_COMMIT_BLOCK_END: &str = "# <<< Assura pre-commit integration <<<";
const PRE_COMMIT_BLOCK: &str = "# >>> Assura pre-commit integration >>>\n- repo: local\n  hooks:\n  - id: assura-pre-push\n    name: Assura pre-push validation\n    entry: assura hooks run pre-push\n    language: system\n    stages: [pre-push]\n    always_run: true\n    pass_filenames: false\n# <<< Assura pre-commit integration <<<\n";

/// Append Assura's exact pre-push hook to a safely owned pre-commit config.
pub fn append_pre_push(project_root: &Path) -> Result<(), String> {
    let path = project_root.join(".pre-commit-config.yaml");
    let existing = match std::fs::read_to_string(&path) {
        Ok(existing) => existing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    require_pinned_pre_commit()?;
    require_assura_on_path()?;
    if let Some(prefix) = existing.strip_suffix(PRE_COMMIT_BLOCK) {
        if is_terminal_root_repos_sequence(prefix) && !has_assura_pre_push_id(prefix)? {
            return Ok(());
        }
        return Err("unsupported or already-owned configuration shape".to_string());
    }
    if !is_terminal_root_repos_sequence(&existing)
        || existing.matches(PRE_COMMIT_BLOCK_START).count() != 0
        || existing.matches(PRE_COMMIT_BLOCK_END).count() != 0
        || has_assura_pre_push_id(&existing)?
    {
        return Err("unsupported or already-owned configuration shape".to_string());
    }
    let mut candidate = existing.clone();
    if !candidate.ends_with('\n') {
        candidate.push('\n');
    }
    candidate.push_str(PRE_COMMIT_BLOCK);
    serde_yaml::from_str::<serde_yaml::Value>(&candidate).map_err(|error| error.to_string())?;
    validate_candidate(project_root, &candidate)?;
    super::local_recipe::write_config_atomically(&path, &candidate)
        .map_err(|error| format!("atomic config replacement failed: {error:?}"))
}

fn require_pinned_pre_commit() -> Result<(), String> {
    let version = Command::new("pre-commit")
        .arg("--version")
        .output()
        .map_err(|error| format!("pre-commit unavailable: {error}"))?;
    if version.status.success()
        && String::from_utf8_lossy(&version.stdout).trim() == PRE_COMMIT_VERSION
    {
        Ok(())
    } else {
        Err(format!("{PRE_COMMIT_VERSION} is required"))
    }
}

fn require_assura_on_path() -> Result<(), String> {
    let assura = Command::new("assura")
        .arg("--help")
        .output()
        .map_err(|error| format!("assura unavailable on PATH: {error}"))?;
    if assura.status.success() {
        Ok(())
    } else {
        Err("assura must resolve on PATH".to_string())
    }
}

fn is_terminal_root_repos_sequence(source: &str) -> bool {
    let mut saw_repos = false;
    let mut saw_item = false;
    for line in source.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !saw_repos {
            if line == "repos:" {
                saw_repos = true;
                continue;
            }
            return false;
        }
        if line.starts_with('-') {
            saw_item = true;
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        return false;
    }
    saw_repos && saw_item
}

fn has_assura_pre_push_id(source: &str) -> Result<bool, String> {
    let value =
        serde_yaml::from_str::<serde_yaml::Value>(source).map_err(|error| error.to_string())?;
    Ok(value_contains_assura_pre_push_id(&value))
}

fn value_contains_assura_pre_push_id(value: &serde_yaml::Value) -> bool {
    match value {
        serde_yaml::Value::Mapping(mapping) => mapping.iter().any(|(key, value)| {
            (key.as_str() == Some("id") && value.as_str() == Some("assura-pre-push"))
                || value_contains_assura_pre_push_id(value)
        }),
        serde_yaml::Value::Sequence(sequence) => {
            sequence.iter().any(value_contains_assura_pre_push_id)
        }
        _ => false,
    }
}

fn validate_candidate(project_root: &Path, candidate: &str) -> Result<(), String> {
    let temporary = project_root.join(format!(".assura-pre-commit-{}.yaml", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("create candidate config: {error}"))?;
    file.write_all(candidate.as_bytes())
        .map_err(|error| format!("write candidate config: {error}"))?;
    drop(file);

    let output = Command::new("pre-commit")
        .args(["validate-config", temporary.to_string_lossy().as_ref()])
        .current_dir(project_root)
        .output()
        .map_err(|error| format!("validate candidate config: {error}"));
    let _ = std::fs::remove_file(&temporary);
    let output = output?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "pre-commit rejected candidate: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
