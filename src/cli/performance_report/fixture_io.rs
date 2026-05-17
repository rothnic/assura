//! Fixture config and file writing helpers.

use crate::config::ls_compat::convert_ls_lint_to_config;
use std::fs;
use std::path::Path;

pub(super) fn write_lslint_compatible_configs(
    root: &Path,
    ls_lint_config: &str,
) -> Result<(), String> {
    let config = convert_ls_lint_to_config(ls_lint_config)
        .map_err(|error| format!("convert LS-Lint config: {error}"))?;
    let assura_config =
        serde_yaml::to_string(&config).map_err(|error| format!("serialize config: {error}"))?;
    write_configs(root, &assura_config, ls_lint_config)
}

pub(super) fn write_configs(
    root: &Path,
    assura_config: &str,
    ls_lint_config: &str,
) -> Result<(), String> {
    let assura_dir = root.join(".assura");
    fs::create_dir_all(&assura_dir)
        .map_err(|error| format!("create {}: {error}", assura_dir.display()))?;
    fs::write(assura_dir.join("config.yml"), assura_config)
        .map_err(|error| format!("write Assura config: {error}"))?;
    fs::write(root.join(".ls-lint.yml"), ls_lint_config)
        .map_err(|error| format!("write LS-Lint config: {error}"))?;
    Ok(())
}

pub(super) fn write_file(path: impl AsRef<Path>, content: &str) -> Result<(), String> {
    let path = path.as_ref();
    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))
}
