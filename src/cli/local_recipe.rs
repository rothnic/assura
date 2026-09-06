//! Explicit, local Assura YAML recipe loading and overlay helpers.

use super::init_support::StarterInitError;
use sha2::{Digest, Sha256};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

/// A non-destructive local recipe merge result.
pub struct RecipeMergeOutcome {
    /// Conflicting leaf values retained from the existing project policy.
    pub conflicts: Vec<RecipeConflict>,
}

/// One preserved collision between existing and explicit local policy.
pub struct RecipeConflict {
    path: String,
    existing: serde_yaml::Value,
    incoming: serde_yaml::Value,
}

impl RecipeMergeOutcome {
    /// Render preserved conflicts with their path and both competing values.
    pub fn render_conflicts(&self) -> String {
        let conflicts = self
            .conflicts
            .iter()
            .map(|conflict| {
                format!(
                    "{} (existing: {}, incoming: {})",
                    conflict.path,
                    render_value(&conflict.existing),
                    render_value(&conflict.incoming)
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("local recipe conflicts; no project policy was written: {conflicts}")
    }
}

/// Merge one explicit local recipe into project-owned policy after validation.
pub fn merge_recipe_file(
    config_path: &Path,
    recipe_file: &Path,
) -> Result<RecipeMergeOutcome, StarterInitError> {
    let existing_source = std::fs::read_to_string(config_path).map_err(|error| {
        StarterInitError::Runtime(format!("failed to read {}: {error}", config_path.display()))
    })?;
    let existing: serde_yaml::Value = serde_yaml::from_str(&existing_source).map_err(|error| {
        StarterInitError::Configuration(format!(
            "{} is not valid YAML: {error}",
            config_path.display()
        ))
    })?;
    let recipe_source = std::fs::read_to_string(recipe_file).map_err(|error| {
        StarterInitError::Runtime(format!(
            "failed to read local recipe {}: {error}",
            recipe_file.display()
        ))
    })?;
    let recipe: serde_yaml::Value = serde_yaml::from_str(&recipe_source).map_err(|error| {
        StarterInitError::Configuration(format!(
            "local recipe {} is not valid YAML: {error}",
            recipe_file.display()
        ))
    })?;
    let mut prospective = existing.clone();
    let mut conflicts = Vec::new();
    merge_preserving_existing(&mut prospective, recipe, "", &mut conflicts);
    if !conflicts.is_empty() {
        return Ok(RecipeMergeOutcome { conflicts });
    }
    let rendered = serde_yaml::to_string(&prospective).map_err(|error| {
        StarterInitError::Runtime(format!("failed to render merged recipe: {error}"))
    })?;
    crate::config::config::ConfigLoader::parse_validated(&rendered).map_err(|error| {
        StarterInitError::Configuration(format!("merged local recipe is invalid: {error}"))
    })?;
    if rendered != existing_source {
        write_config_atomically(config_path, &rendered)?;
    }
    Ok(RecipeMergeOutcome { conflicts })
}

/// Replace a validated project config with an atomic same-directory rename.
pub fn write_config_atomically(config_path: &Path, contents: &str) -> Result<(), StarterInitError> {
    let parent = config_path.parent().ok_or_else(|| {
        StarterInitError::Runtime(format!("{} has no parent directory", config_path.display()))
    })?;
    let name = config_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.yml");
    for attempt in 0..128 {
        let temporary = parent.join(format!(
            ".{name}.{}.{}.assura-tmp",
            std::process::id(),
            attempt
        ));
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(StarterInitError::Runtime(format!(
                    "failed to create {}: {error}",
                    temporary.display()
                )));
            }
        };
        if let Err(error) = file.write_all(contents.as_bytes()) {
            let _ = std::fs::remove_file(&temporary);
            return Err(StarterInitError::Runtime(format!(
                "failed to write {}: {error}",
                temporary.display()
            )));
        }
        drop(file);
        return std::fs::rename(&temporary, config_path).map_err(|error| {
            let _ = std::fs::remove_file(&temporary);
            StarterInitError::Runtime(format!(
                "failed to replace {}: {error}",
                config_path.display()
            ))
        });
    }
    Err(StarterInitError::Runtime(format!(
        "failed to allocate a temporary config beside {}",
        config_path.display()
    )))
}

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

fn merge_preserving_existing(
    destination: &mut serde_yaml::Value,
    source: serde_yaml::Value,
    path: &str,
    conflicts: &mut Vec<RecipeConflict>,
) {
    match (destination, source) {
        (serde_yaml::Value::Mapping(destination), serde_yaml::Value::Mapping(source)) => {
            for (key, value) in source {
                let segment = key.as_str().unwrap_or("<key>");
                let child_path = if path.is_empty() {
                    segment.to_string()
                } else {
                    format!("{path}.{segment}")
                };
                if let Some(existing) = destination.get_mut(&key) {
                    merge_preserving_existing(existing, value, &child_path, conflicts);
                } else {
                    destination.insert(key, value);
                }
            }
        }
        (serde_yaml::Value::Sequence(destination), serde_yaml::Value::Sequence(source))
            if path == "exclude" =>
        {
            for value in source {
                if !destination.contains(&value) {
                    destination.push(value);
                }
            }
        }
        (destination, source) if *destination == source => {}
        (destination, source) => conflicts.push(RecipeConflict {
            path: path.to_string(),
            existing: destination.clone(),
            incoming: source,
        }),
    }
}

fn render_value(value: &serde_yaml::Value) -> String {
    serde_yaml::to_string(value)
        .unwrap_or_else(|_| "<unrenderable>".to_string())
        .trim()
        .to_string()
}
