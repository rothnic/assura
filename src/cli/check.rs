//! Structure-first project validation used by the public `assura check` command.

mod artifact_check;
#[cfg(feature = "yaml-config")]
mod batch;
#[cfg(all(feature = "yaml-config", feature = "json-output"))]
mod cache;
mod case;
mod command_surface_docs;
mod compiled_artifact;
#[cfg(test)]
mod compiled_artifact_tests;
mod compiled_config;
mod compiled_fingerprint;
mod compiled_plan_artifact;
mod configured_structure;
mod custom_constraints;
mod direct_contents;
#[cfg(all(feature = "yaml-config", feature = "json-output"))]
pub mod fast_cli;
mod ls_fast;
mod ls_fast_counts;
mod ls_fast_naming;
#[cfg(feature = "full-cli")]
mod ls_fast_parallel;
mod ls_fast_plan;
#[cfg(test)]
mod ls_fast_plan_tests;
mod ls_fast_target;
#[cfg(feature = "yaml-config")]
mod markdown;
mod patterns;
#[cfg(feature = "yaml-config")]
mod prepared;
#[cfg(feature = "yaml-config")]
mod profiling;
mod report;
mod rule_plan;
mod rules;
mod scope_patterns;
mod traversal;
mod validators;

#[cfg(feature = "yaml-config")]
use crate::cli::config::ConfigDiscovery;
use crate::config::config::Config;
#[cfg(feature = "yaml-config")]
use crate::config::loader::ConfigLoader;
pub use artifact_check::{
    run_structure_check_with_artifact, run_structure_check_with_fast_artifact,
    run_structure_check_with_prechecked_fast_artifact,
};
#[cfg(feature = "yaml-config")]
pub use batch::run_structure_checks;
#[cfg(all(feature = "yaml-config", feature = "json-output"))]
pub use cache::run_structure_check_cached;
pub use compiled_artifact::CompiledStructureConfigArtifact;
use compiled_config::CompiledStructureConfig;
use glob::Pattern;
use ls_fast_plan::FastScope;
#[cfg(feature = "yaml-config")]
pub use prepared::PreparedStructureCheck;
#[cfg(feature = "yaml-config")]
pub use profiling::{run_structure_check_with_timings, StructureCheckTimings};
pub use report::{CheckError, StructureCheckReport, StructureViolation};
#[cfg(not(feature = "yaml-config"))]
#[derive(Debug, Default)]
pub struct StructureCheckTimings {
    /// Time spent finding the configuration file.
    pub config_discovery_ms: f64,
    /// Time spent loading and parsing configuration.
    pub config_load_ms: f64,
    /// Time spent constructing the checker.
    pub checker_init_ms: f64,
    /// Time spent validating configured required paths.
    pub configured_structure_ms: f64,
    /// Time spent walking and validating files/directories.
    pub walk_and_validate_ms: f64,
    /// Time spent sorting report violations.
    pub report_sort_ms: f64,
}
use regex_lite::Regex;
use rule_plan::{rules_for_dir, RuleScope};
use rules::{is_excluded_rel_with, normalize_config_dir, CompiledExclusion, EffectiveRules};
use scope_patterns::{path_has_scope_magic, path_matches_scope_pattern};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// How a non-root checked path should be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckTargetMode {
    /// Validate directories recursively. This is Assura's native changed-path
    /// behavior for real-time feedback.
    Recursive,
    /// Validate only the explicitly provided target, matching LS-Lint's path
    /// argument behavior for compatibility checks.
    LsLint,
}

/// Run structure-first validation for a path.
#[cfg(feature = "yaml-config")]
pub fn run_structure_check(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
) -> Result<StructureCheckReport, CheckError> {
    run_structure_check_with_target_mode(path, config_path, fail_fast, CheckTargetMode::Recursive)
}

/// Run structure-first validation for a path with explicit target semantics.
#[cfg(feature = "yaml-config")]
pub fn run_structure_check_with_target_mode(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
    target_mode: CheckTargetMode,
) -> Result<StructureCheckReport, CheckError> {
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
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let config = ConfigLoader::load(&config_path)?;
    let compiled = if target_mode == CheckTargetMode::LsLint {
        CompiledStructureConfig::new(config, fail_fast)
    } else {
        CompiledStructureConfig::new_for_check(config, fail_fast)
    };
    let mut checker = StructureChecker::from_compiled_owned(project_root, compiled, fail_fast);
    let mut timings = StructureCheckTimings::default();
    checker.check(checked_path, config_path, target_mode, &mut timings)
}

/// Run one structure check against an already parsed configuration.
///
/// This powers one-shot compiled-config CLI execution where configuration
/// discovery and YAML parsing already happened outside the measured check
/// process.
pub fn run_structure_check_with_config(
    project_root: PathBuf,
    config_path: PathBuf,
    checked_path: PathBuf,
    config: Config,
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

    let mut checker = StructureChecker::new(project_root, config, fail_fast);
    let mut timings = StructureCheckTimings::default();
    checker.check(
        checked_path,
        config_path,
        CheckTargetMode::Recursive,
        &mut timings,
    )
}

#[cfg(feature = "yaml-config")]
pub(super) fn discover_project(
    checked_path: &Path,
    config_path: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf), CheckError> {
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

        return Ok((project_root.canonicalize()?, config_path));
    }

    if checked_path.is_dir() {
        let direct_config_path = checked_path.join(".assura/config.yml");
        if direct_config_path.exists() {
            return Ok((checked_path.to_path_buf(), direct_config_path));
        }
    }

    let config_path = ConfigDiscovery::find_config_path(checked_path)
        .ok_or_else(|| CheckError::NoConfig(checked_path.to_path_buf()))?;

    if !config_path.exists() {
        return Err(CheckError::NoConfig(checked_path.to_path_buf()));
    }

    let project_root = config_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| CheckError::InvalidConfigLocation(config_path.clone()))?
        .canonicalize()?;

    Ok((project_root, config_path.canonicalize()?))
}

pub(in crate::cli::check) struct StructureChecker {
    project_root: PathBuf,
    config: Config,
    fail_fast: bool,
    configured_dirs: Vec<PathBuf>,
    exclude_patterns: Vec<CompiledExclusion>,
    naming_regexes: HashMap<String, Regex>,
    glob_patterns: HashMap<String, Pattern>,
    rule_scopes: Vec<RuleScope>,
    lslint_fast_scopes: Option<Vec<FastScope>>,
    lslint_fast_scope_index: Option<HashMap<PathBuf, usize>>,
    has_direct_count_constraints: bool,
    rules_cache: HashMap<PathBuf, EffectiveRules>,
}

impl StructureChecker {
    pub(in crate::cli::check) fn new(
        project_root: PathBuf,
        config: Config,
        fail_fast: bool,
    ) -> Self {
        let compiled = CompiledStructureConfig::new_for_check(config, fail_fast);
        Self::from_compiled_owned(project_root, compiled, fail_fast)
    }

    #[cfg(feature = "yaml-config")]
    pub(in crate::cli::check) fn from_compiled(
        project_root: PathBuf,
        compiled: &CompiledStructureConfig,
        fail_fast: bool,
    ) -> Self {
        Self {
            project_root,
            config: compiled.config.clone(),
            fail_fast,
            configured_dirs: compiled.configured_dirs.clone(),
            exclude_patterns: compiled.exclude_patterns.clone(),
            naming_regexes: compiled.naming_regexes.clone(),
            glob_patterns: compiled.glob_patterns.clone(),
            rule_scopes: compiled.rule_scopes.clone(),
            lslint_fast_scope_index: index_lslint_fast_scopes(
                compiled.lslint_fast_scopes.as_deref(),
            ),
            lslint_fast_scopes: compiled.lslint_fast_scopes.clone(),
            has_direct_count_constraints: compiled.has_direct_count_constraints,
            rules_cache: HashMap::new(),
        }
    }

    pub(super) fn from_compiled_owned(
        project_root: PathBuf,
        compiled: CompiledStructureConfig,
        fail_fast: bool,
    ) -> Self {
        let lslint_fast_scope_index =
            index_lslint_fast_scopes(compiled.lslint_fast_scopes.as_deref());
        Self {
            project_root,
            config: compiled.config,
            fail_fast,
            configured_dirs: compiled.configured_dirs,
            exclude_patterns: compiled.exclude_patterns,
            naming_regexes: compiled.naming_regexes,
            glob_patterns: compiled.glob_patterns,
            rule_scopes: compiled.rule_scopes,
            lslint_fast_scope_index,
            lslint_fast_scopes: compiled.lslint_fast_scopes,
            has_direct_count_constraints: compiled.has_direct_count_constraints,
            rules_cache: HashMap::new(),
        }
    }

    pub(in crate::cli::check) fn check(
        &mut self,
        checked_path: PathBuf,
        config_path: PathBuf,
        target_mode: CheckTargetMode,
        timings: &mut StructureCheckTimings,
    ) -> Result<StructureCheckReport, CheckError> {
        let mut report = StructureCheckReport {
            success: true,
            project_root: self.project_root.clone(),
            config_path,
            checked_path: checked_path.clone(),
            files_checked: 0,
            dirs_checked: 0,
            violations: Vec::new(),
        };

        if target_mode == CheckTargetMode::LsLint && checked_path != self.project_root {
            let walk_started = Instant::now();
            if let Some(scopes) = self.lslint_fast_scopes.as_deref() {
                if self.try_check_lslint_explicit_target(&checked_path, &mut report, scopes)? {
                    timings.walk_and_validate_ms = walk_started.elapsed().as_secs_f64() * 1000.0;
                    let sort_started = Instant::now();
                    report.violations.sort_by(|left, right| {
                        left.path.cmp(&right.path).then(left.rule.cmp(&right.rule))
                    });
                    timings.report_sort_ms = sort_started.elapsed().as_secs_f64() * 1000.0;
                    report.success = report.violations.is_empty();
                    return Ok(report);
                }
            }
            let has_direct_count_constraints = self.has_direct_count_constraints;
            self.has_direct_count_constraints = false;
            self.validate_one_existing_path(&checked_path, &mut report);
            self.has_direct_count_constraints = has_direct_count_constraints;
            timings.walk_and_validate_ms = walk_started.elapsed().as_secs_f64() * 1000.0;
            let sort_started = Instant::now();
            report
                .violations
                .sort_by(|left, right| left.path.cmp(&right.path).then(left.rule.cmp(&right.rule)));
            timings.report_sort_ms = sort_started.elapsed().as_secs_f64() * 1000.0;
            report.success = report.violations.is_empty();
            return Ok(report);
        }

        if self.try_check_lslint_fast(&checked_path, &mut report, timings)? {
            report.success = report.violations.is_empty();
            return Ok(report);
        }

        let configured_started = Instant::now();
        self.validate_configured_structure(&mut report);
        timings.configured_structure_ms = configured_started.elapsed().as_secs_f64() * 1000.0;

        if self.fail_fast && !report.violations.is_empty() {
            report.success = false;
            return Ok(report);
        }

        let walk_started = Instant::now();
        self.walk_and_validate(&checked_path, &mut report)?;
        timings.walk_and_validate_ms = walk_started.elapsed().as_secs_f64() * 1000.0;

        if !self.fail_fast || report.violations.is_empty() {
            self.validate_custom_constraints(&checked_path, &mut report)?;
        }

        let sort_started = Instant::now();
        report
            .violations
            .sort_by(|left, right| left.path.cmp(&right.path).then(left.rule.cmp(&right.rule)));
        timings.report_sort_ms = sort_started.elapsed().as_secs_f64() * 1000.0;
        report.success = report.violations.is_empty();
        Ok(report)
    }

    pub(super) fn resolve_rules(&mut self, dir_rel: &Path) -> EffectiveRules {
        if let Some(cached) = self.rules_cache.get(dir_rel) {
            return cached.clone();
        }

        let result = rules_for_dir(dir_rel, &self.rule_scopes);
        self.rules_cache
            .insert(dir_rel.to_path_buf(), result.clone());
        result
    }

    pub(super) fn relative_path(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.project_root)
            .unwrap_or(path)
            .to_path_buf()
    }

    fn is_excluded_rel(&self, rel: &Path) -> bool {
        is_excluded_rel_with(&self.exclude_patterns, rel)
    }

    pub(super) fn is_configured_dir(&self, rel: &Path) -> bool {
        if self
            .configured_dirs
            .binary_search_by(|configured| configured.as_path().cmp(rel))
            .is_ok()
        {
            return true;
        }

        self.configured_dirs
            .iter()
            .filter(|configured| path_has_scope_magic(configured))
            .any(|configured| path_matches_scope_pattern(configured, rel))
    }

    pub(super) fn push_violation(
        &self,
        report: &mut StructureCheckReport,
        path: PathBuf,
        rule: impl Into<String>,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) {
        if self.is_excluded_rel(&path) {
            return;
        }

        report
            .violations
            .push(StructureViolation::new(path, rule, message, severity));
    }
}

fn index_lslint_fast_scopes(scopes: Option<&[FastScope]>) -> Option<HashMap<PathBuf, usize>> {
    let scopes = scopes?;
    if scopes.iter().any(FastScope::has_scope_magic) {
        return None;
    }

    let mut index = HashMap::with_capacity(scopes.len());
    for (scope_index, scope) in scopes.iter().enumerate() {
        index.insert(scope.parts().0.to_path_buf(), scope_index);
    }
    Some(index)
}
