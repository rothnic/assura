//! Batch structure-check support for the lightweight check CLI.

use super::{
    discover_project, CheckError, StructureCheckReport, StructureCheckTimings, StructureChecker,
};
use crate::config::loader::ConfigLoader;
use std::path::PathBuf;

/// Run structure-first validation for multiple paths, reusing loaded config
/// and compiled checker state when paths belong to the same project.
pub fn run_structure_checks(
    paths: Vec<Option<PathBuf>>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
) -> Result<Vec<StructureCheckReport>, CheckError> {
    let mut prepared = prepare_paths(paths, config_path)?;
    let mut reports = Vec::with_capacity(prepared.len());

    while let Some((checked_path, project_root, discovered_config_path)) = prepared.pop() {
        let config = ConfigLoader::load(&discovered_config_path)?;
        let mut checker = StructureChecker::new(project_root.clone(), config, fail_fast);
        let mut timings = StructureCheckTimings::default();
        reports.push(checker.check(checked_path, discovered_config_path.clone(), &mut timings)?);

        let mut index = 0;
        while index < prepared.len() {
            if prepared[index].1 == project_root && prepared[index].2 == discovered_config_path {
                let (checked_path, _, _) = prepared.remove(index);
                let mut timings = StructureCheckTimings::default();
                reports.push(checker.check(
                    checked_path,
                    discovered_config_path.clone(),
                    &mut timings,
                )?);
            } else {
                index += 1;
            }
        }
    }

    reports.sort_by(|left, right| left.checked_path.cmp(&right.checked_path));
    Ok(reports)
}

fn prepare_paths(
    paths: Vec<Option<PathBuf>>,
    config_path: Option<PathBuf>,
) -> Result<Vec<(PathBuf, PathBuf, PathBuf)>, CheckError> {
    let mut prepared = Vec::with_capacity(paths.len());
    for path in paths {
        let requested_path = match path {
            Some(path) => path,
            None => std::env::current_dir()?,
        };

        if !requested_path.exists() {
            return Err(CheckError::MissingPath(requested_path));
        }

        let checked_path = requested_path.canonicalize()?;
        let (project_root, discovered_config_path) =
            discover_project(&checked_path, config_path.clone())?;
        if !checked_path.starts_with(&project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root,
            });
        }
        prepared.push((checked_path, project_root, discovered_config_path));
    }
    Ok(prepared)
}
