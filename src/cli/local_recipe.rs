//! Explicit, local Assura YAML recipe loading and overlay helpers.

use super::init_support::StarterInitError;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Overlay each explicit local recipe in argument order onto a YAML document.
pub fn apply_recipe_file(
    destination: &mut serde_yaml::Value,
    recipe_file: Option<&PathBuf>,
) -> Result<(), StarterInitError> {
    let Some(recipe_file) = recipe_file else {
        return Ok(());
    };
    let source = std::fs::read_to_string(recipe_file).map_err(|error| {
        StarterInitError::Runtime(format!(
            "failed to read local recipe {}: {error}",
            recipe_file.display()
        ))
    })?;
    let recipe = serde_yaml::from_str(&source).map_err(|error| {
        StarterInitError::Configuration(format!(
            "local recipe {} is not valid YAML: {error}",
            recipe_file.display()
        ))
    })?;
    overlay_value(destination, recipe);
    Ok(())
}

/// Write the provenance for one explicit local recipe outside policy semantics.
pub fn write_profile_selection(
    project_root: &Path,
    recipe_file: &Path,
) -> Result<PathBuf, StarterInitError> {
    let source = std::fs::read(recipe_file).map_err(|error| {
        StarterInitError::Runtime(format!(
            "failed to read local recipe {}: {error}",
            recipe_file.display()
        ))
    })?;
    let profile = serde_json::json!({
        "schema": "assura.profile-selection.v1",
        "source": recipe_file.display().to_string(),
        "source_hash": format!("{:x}", Sha256::digest(source)),
    });
    let directory = project_root.join(".assura/onboarding");
    std::fs::create_dir_all(&directory).map_err(|error| {
        StarterInitError::Runtime(format!("failed to create {}: {error}", directory.display()))
    })?;
    let path = directory.join("profile-selection.json");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&profile).expect("profile selection is serializable"),
    )
    .map_err(|error| {
        StarterInitError::Runtime(format!("failed to write {}: {error}", path.display()))
    })?;
    Ok(path)
}

fn overlay_value(destination: &mut serde_yaml::Value, source: serde_yaml::Value) {
    match (destination, source) {
        (serde_yaml::Value::Mapping(destination), serde_yaml::Value::Mapping(source)) => {
            for (key, value) in source {
                if let Some(existing) = destination.get_mut(&key) {
                    overlay_value(existing, value);
                } else {
                    destination.insert(key, value);
                }
            }
        }
        (destination, source) => *destination = source,
    }
}
