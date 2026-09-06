//! Explicit, local Assura YAML recipe loading and overlay helpers.

use super::init_support::StarterInitError;
use std::path::PathBuf;

/// Overlay each explicit local recipe in argument order onto a YAML document.
pub fn apply_recipe_files(
    destination: &mut serde_yaml::Value,
    recipe_files: &[PathBuf],
) -> Result<(), StarterInitError> {
    for recipe_file in recipe_files {
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
    }
    Ok(())
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
