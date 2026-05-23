//! Structure-first project validation used by the public `assura check` command.

mod artifact_check;
#[cfg(feature = "yaml-config")]
mod batch;
#[cfg(all(feature = "yaml-config", feature = "json-output"))]
mod cache;
mod case;
mod compiled_artifact;
#[cfg(test)]
mod compiled_artifact_tests;
mod compiled_config;
mod compiled_fingerprint;
mod compiled_plan_artifact;
mod direct_contents;
#[cfg(all(feature = "yaml-config", feature = "json-output"))]
pub mod fast_cli;
mod ls_fast;
mod ls_fast_counts;
mod ls_fast_naming;
mod ls_fast_plan;
#[cfg(test)]
mod ls_fast_plan_tests;
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
mod traversal;
mod validators;

#[cfg(feature = "yaml-config")]
use crate::cli::config::ConfigDiscovery;
use crate::config::config::{Config, DirectoryNode};
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
use rules::{
    display_rel, is_excluded_rel_with, join_config_child, normalize_config_dir,
    severity_for_bundle, severity_for_node, CompiledExclusion, EffectiveRules,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Run structure-first validation for a path.
#[cfg(feature = "yaml-config")]
pub fn run_structure_check(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
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
    let mut checker = StructureChecker::new(project_root, config, fail_fast);
    let mut timings = StructureCheckTimings::default();
    checker.check(checked_path, config_path, &mut timings)
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
    checker.check(checked_path, config_path, &mut timings)
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
                ConfigDiscovery::find_project_root(checked_path)
                    .ok_or_else(|| CheckError::NoConfig(checked_path.to_path_buf()))?
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
    required_dirs: Vec<PathBuf>,
    exclude_patterns: Vec<CompiledExclusion>,
    naming_regexes: HashMap<String, Regex>,
    glob_patterns: HashMap<String, Pattern>,
    rule_scopes: Vec<RuleScope>,
    lslint_fast_scopes: Option<Vec<FastScope>>,
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
            required_dirs: compiled.required_dirs.clone(),
            exclude_patterns: compiled.exclude_patterns.clone(),
            naming_regexes: compiled.naming_regexes.clone(),
            glob_patterns: compiled.glob_patterns.clone(),
            rule_scopes: compiled.rule_scopes.clone(),
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
        Self {
            project_root,
            config: compiled.config,
            fail_fast,
            configured_dirs: compiled.configured_dirs,
            required_dirs: compiled.required_dirs,
            exclude_patterns: compiled.exclude_patterns,
            naming_regexes: compiled.naming_regexes,
            glob_patterns: compiled.glob_patterns,
            rule_scopes: compiled.rule_scopes,
            lslint_fast_scopes: compiled.lslint_fast_scopes,
            has_direct_count_constraints: compiled.has_direct_count_constraints,
            rules_cache: HashMap::new(),
        }
    }

    pub(in crate::cli::check) fn check(
        &mut self,
        checked_path: PathBuf,
        config_path: PathBuf,
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

        let sort_started = Instant::now();
        report
            .violations
            .sort_by(|left, right| left.path.cmp(&right.path).then(left.rule.cmp(&right.rule)));
        timings.report_sort_ms = sort_started.elapsed().as_secs_f64() * 1000.0;
        report.success = report.violations.is_empty();
        Ok(report)
    }

    pub(in crate::cli::check) fn validate_configured_structure(
        &self,
        report: &mut StructureCheckReport,
    ) {
        for (path, node) in &self.config.structure {
            let base = normalize_config_dir(path);
            self.validate_node_requirements(&base, node, report);
        }
    }

    fn validate_node_requirements(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        if self.is_excluded_rel(node_rel) {
            return;
        }

        let node_abs = self.project_root.join(node_rel);
        if !node_abs.is_dir() {
            if !node.required {
                return;
            }
            self.push_violation(
                report,
                node_rel.to_path_buf(),
                "required_directory",
                format!(
                    "Configured directory '{}' is missing",
                    display_rel(node_rel)
                ),
                severity_for_node(node),
            );
            return;
        }

        if let Some(files) = &node.files {
            if let Some(required) = &files.required {
                for file in required {
                    let file_rel = node_rel.join(file);
                    if !self.project_root.join(&file_rel).is_file() {
                        self.push_violation(
                            report,
                            file_rel.clone(),
                            "required_file",
                            format!("Required file '{}' is missing", display_rel(&file_rel)),
                            severity_for_bundle(files),
                        );
                    }
                }
            }
        }

        if let Some(directories) = &node.directories {
            if let Some(required) = &directories.required {
                for directory in required {
                    let dir_rel = node_rel.join(directory);
                    if !self.project_root.join(&dir_rel).is_dir() {
                        self.push_violation(
                            report,
                            dir_rel.clone(),
                            "required_directory",
                            format!("Required directory '{}' is missing", display_rel(&dir_rel)),
                            directories
                                .severity
                                .clone()
                                .unwrap_or_else(|| "medium".to_string()),
                        );
                    }
                }
            }
        }

        if let Some(exists) = &node.exists {
            if let Some(files) = &exists.files {
                for file in files {
                    let file_rel = node_rel.join(file);
                    if !self.project_root.join(&file_rel).is_file() {
                        self.push_violation(
                            report,
                            file_rel.clone(),
                            "required_file",
                            format!("Required file '{}' is missing", display_rel(&file_rel)),
                            severity_for_node(node),
                        );
                    }
                }
            }

            if let Some(directories) = &exists.directories {
                for directory in directories {
                    let dir_rel = node_rel.join(directory);
                    if !self.project_root.join(&dir_rel).is_dir() {
                        self.push_violation(
                            report,
                            dir_rel.clone(),
                            "required_directory",
                            format!("Required directory '{}' is missing", display_rel(&dir_rel)),
                            severity_for_node(node),
                        );
                    }
                }
            }
        }

        if let Some(children) = &node.children {
            for (child_name, child) in children {
                let child_rel = join_config_child(node_rel, child_name);
                self.validate_node_requirements(&child_rel, child, report);
            }
        }
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
        self.configured_dirs
            .binary_search_by(|configured| configured.as_path().cmp(rel))
            .is_ok()
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
