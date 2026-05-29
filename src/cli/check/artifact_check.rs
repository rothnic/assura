//! Structure-check execution for compiled config artifacts.

use super::{
    CheckError, CompiledStructureConfig, CompiledStructureConfigArtifact, StructureCheckReport,
    StructureCheckTimings, StructureChecker,
};
use crate::cli::config::ConfigError;
use std::path::PathBuf;

/// Run one structure check against a precompiled config artifact.
pub fn run_structure_check_with_artifact(
    project_root: PathBuf,
    config_path: PathBuf,
    checked_path: PathBuf,
    artifact: CompiledStructureConfigArtifact,
    fail_fast: bool,
) -> Result<StructureCheckReport, CheckError> {
    if !checked_path.exists() {
        return Err(CheckError::MissingPath(checked_path));
    }

    let checked_path = checked_path.canonicalize()?;
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let compiled = match artifact.into_fast_compiled_config(fail_fast) {
        Ok(compiled) => compiled,
        Err(artifact) => (*artifact).into_compiled_config(fail_fast)?,
    };
    run_with_compiled_config(project_root, config_path, checked_path, compiled, fail_fast)
}

/// Run one LS-Lint-compatible structure check from a fast-only artifact.
///
/// This path intentionally never falls back to the full structure checker. It
/// exists for compiled LS-Lint-compatible artifacts where YAML/config loading
/// already happened in the compiler process and the runtime binary should not
/// link the YAML-dependent fallback validator graph.
pub fn run_structure_check_with_fast_artifact(
    project_root: PathBuf,
    config_path: PathBuf,
    checked_path: PathBuf,
    artifact: CompiledStructureConfigArtifact,
) -> Result<StructureCheckReport, CheckError> {
    if !checked_path.exists() {
        return Err(CheckError::MissingPath(checked_path));
    }

    let checked_path = checked_path.canonicalize()?;
    run_structure_check_with_prechecked_fast_artifact(
        project_root,
        config_path,
        checked_path,
        artifact,
    )
}

/// Run one LS-Lint-compatible structure check from a canonicalized path.
///
/// Callers must pass a checked path that already exists and has already been
/// resolved relative to the project root. This lets latency-sensitive compiled
/// CLI callers avoid duplicate existence and canonicalization work after they
/// have already resolved the path for project discovery.
pub fn run_structure_check_with_prechecked_fast_artifact(
    project_root: PathBuf,
    config_path: PathBuf,
    checked_path: PathBuf,
    artifact: CompiledStructureConfigArtifact,
) -> Result<StructureCheckReport, CheckError> {
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let compiled = artifact
        .into_fast_compiled_config(false)
        .map_err(|_| ConfigError::Invalid("compiled config is not fast-path compatible".into()))?;
    let mut checker = StructureChecker::from_compiled_owned(project_root.clone(), compiled, false);
    let mut report = StructureCheckReport {
        success: true,
        project_root,
        config_path,
        checked_path: checked_path.clone(),
        files_checked: 0,
        dirs_checked: 0,
        violations: Vec::new(),
    };
    let mut timings = StructureCheckTimings::default();
    if !checker.try_check_lslint_fast(&checked_path, &mut report, &mut timings)? {
        return Err(CheckError::Config(ConfigError::Invalid(
            "compiled config is not fast-path compatible".into(),
        )));
    }

    report.success = report.violations.is_empty();
    Ok(report)
}

fn run_with_compiled_config(
    project_root: PathBuf,
    config_path: PathBuf,
    checked_path: PathBuf,
    compiled: CompiledStructureConfig,
    fail_fast: bool,
) -> Result<StructureCheckReport, CheckError> {
    let mut checker = StructureChecker::from_compiled_owned(project_root, compiled, fail_fast);
    let mut timings = StructureCheckTimings::default();
    checker.check(
        checked_path,
        config_path,
        super::CheckTargetMode::Recursive,
        &mut timings,
    )
}
