//! Project root and configuration discovery for working-tree checks.

use super::CheckError;
use crate::cli::config::{ConfigDiscovery, ConfigError};
use crate::policy::source_selection::{select_worktree_project, SourceOrigin};
use std::path::{Path, PathBuf};

pub(super) fn discover_project(
    checked_path: &Path,
    config_path: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf), CheckError> {
    discover_project_with_origin(checked_path, config_path).map(|(root, config, _)| (root, config))
}

pub(super) fn discover_project_with_origin(
    checked_path: &Path,
    config_path: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf, Option<String>), CheckError> {
    if let Some(config_path) = config_path {
        let config_path = config_path.canonicalize()?;
        let assura_dir = config_path
            .parent()
            .ok_or_else(|| CheckError::InvalidConfigLocation(config_path.clone()))?;
        let project_root =
            if assura_dir.file_name().and_then(|name| name.to_str()) == Some(".assura") {
                assura_dir
                    .parent()
                    .ok_or_else(|| CheckError::InvalidConfigLocation(config_path.clone()))?
                    .to_path_buf()
            } else {
                checked_path
                    .is_file()
                    .then(|| {
                        checked_path.parent().map(|parent| {
                            if parent.as_os_str().is_empty() {
                                PathBuf::from(".")
                            } else {
                                parent.to_path_buf()
                            }
                        })
                    })
                    .flatten()
                    .unwrap_or_else(|| checked_path.to_path_buf())
            };

        return Ok((project_root.canonicalize()?, config_path, None));
    }

    let start = if checked_path.is_dir() {
        checked_path
    } else {
        checked_path.parent().unwrap_or(checked_path)
    };
    if ConfigDiscovery::find_git_dir(checked_path).is_some()
        || std::env::var_os("GIT_DIR").is_some()
    {
        if let Some((root, source)) = select_worktree_project(start)
            .map_err(|error| CheckError::Config(ConfigError::Invalid(error.to_string())))?
        {
            let origin = match source.origin {
                SourceOrigin::Git(origin) => origin,
                #[cfg(feature = "full-cli")]
                _ => unreachable!("Git project selection always records its origin"),
            };
            return Ok((root, source.path, Some(origin)));
        }
    }

    if checked_path.is_dir() {
        let direct_config_path = checked_path.join(".assura/config.yml");
        if direct_config_path.exists() {
            return Ok((checked_path.to_path_buf(), direct_config_path, None));
        }
    }

    let config_path = ConfigDiscovery::find_config_path(checked_path)
        .ok_or_else(|| CheckError::no_config(checked_path))?;

    if !config_path.exists() {
        return Err(CheckError::no_config(checked_path));
    }

    let project_root = config_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| CheckError::InvalidConfigLocation(config_path.clone()))?
        .canonicalize()?;

    Ok((project_root, config_path.canonicalize()?, None))
}
