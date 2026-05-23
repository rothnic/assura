//! Optional stage timing wrapper for structure-first checks.

use super::{discover_project, CheckError, StructureCheckReport, StructureChecker};
use crate::config::loader::ConfigLoader;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

/// Stage timing details for one structure-first check run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StructureCheckTimings {
    /// Total elapsed time, including path preparation, config loading, and validation.
    pub total_ms: f64,
    /// Time spent resolving the checked path and discovering project/config paths.
    pub config_discovery_ms: f64,
    /// Time spent loading and parsing `.assura/config.yml`.
    pub config_load_ms: f64,
    /// Time spent compiling exclusions, globs, regexes, and configured directory metadata.
    pub checker_init_ms: f64,
    /// Time spent validating configured required files/directories before walking.
    pub configured_structure_ms: f64,
    /// Time spent walking the checked tree and applying file/directory validators.
    pub walk_and_validate_ms: f64,
    /// Time spent sorting the final violation list for deterministic output.
    pub report_sort_ms: f64,
}

/// Run structure-first validation and return stage timings for performance reporting.
pub fn run_structure_check_with_timings(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
) -> Result<(StructureCheckReport, StructureCheckTimings), CheckError> {
    let total_started = Instant::now();
    let mut timings = StructureCheckTimings::default();

    let discovery_started = Instant::now();
    let checked_path = match path {
        Some(path) => {
            if !path.exists() {
                return Err(CheckError::MissingPath(path));
            }
            path.canonicalize()?
        }
        None => std::env::current_dir()?,
    };
    let (project_root, config_path) = discover_project(&checked_path, config_path)?;
    timings.config_discovery_ms = discovery_started.elapsed().as_secs_f64() * 1000.0;

    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let config_load_started = Instant::now();
    let config = ConfigLoader::load(&config_path)?;
    timings.config_load_ms = config_load_started.elapsed().as_secs_f64() * 1000.0;

    let checker_init_started = Instant::now();
    let mut checker = StructureChecker::new(project_root.clone(), config, fail_fast);
    timings.checker_init_ms = checker_init_started.elapsed().as_secs_f64() * 1000.0;

    let report = checker.check(checked_path, config_path, &mut timings)?;
    timings.total_ms = total_started.elapsed().as_secs_f64() * 1000.0;
    Ok((report, timings))
}
