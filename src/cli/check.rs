//! Structure-first project validation used by the public `assura check` command.

mod direct_contents;
mod markdown;
mod patterns;
mod rules;
mod validators;

use crate::cli::config::{ConfigDiscovery, ConfigError};
use crate::config::config::{Config, DirectoryNode};
use crate::config::loader::ConfigLoader;
use glob::Pattern;
use regex::Regex;
use rules::{
    collect_configured_dirs, collect_naming_regexes, dir_contains, display_rel,
    is_excluded_rel_with, join_config_child, merge_directory_bundle, merge_file_bundle,
    merge_markdown_bundle, normalize_config_dir, severity_for_bundle, severity_for_node,
    strip_direct_content_policy, CompiledExclusion, EffectiveRules,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Result of running a structure-first check.
#[derive(Debug, Clone, Serialize)]
pub struct StructureCheckReport {
    /// Whether the checked path passed all configured validations.
    pub success: bool,
    /// Project root used to resolve relative config paths.
    pub project_root: PathBuf,
    /// Configuration file used for validation.
    pub config_path: PathBuf,
    /// Path that was checked.
    pub checked_path: PathBuf,
    /// Number of files checked.
    pub files_checked: usize,
    /// Number of directories checked.
    pub dirs_checked: usize,
    /// Validation violations.
    pub violations: Vec<StructureViolation>,
}

impl StructureCheckReport {
    /// Number of validation violations.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// A single structure validation violation.
#[derive(Debug, Clone, Serialize)]
pub struct StructureViolation {
    /// Path associated with the violation.
    pub path: PathBuf,
    /// Rule that produced the violation.
    pub rule: String,
    /// Human-readable violation message.
    pub message: String,
    /// Violation severity.
    pub severity: String,
}

impl StructureViolation {
    fn new(
        path: PathBuf,
        rule: impl Into<String>,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) -> Self {
        Self {
            path,
            rule: rule.into(),
            message: message.into(),
            severity: severity.into(),
        }
    }
}

/// Errors produced while preparing or running a structure check.
#[derive(Debug, Error)]
pub enum CheckError {
    /// The target path does not exist.
    #[error("checked path does not exist: {0:?}")]
    MissingPath(PathBuf),
    /// No Assura configuration was found.
    #[error("no .assura/config.yml found for {0:?}")]
    NoConfig(PathBuf),
    /// The configured project root could not be determined.
    #[error("could not determine project root for config {0:?}")]
    InvalidConfigLocation(PathBuf),
    /// The checked path is outside the discovered project root.
    #[error("checked path {checked_path:?} is outside project root {project_root:?}")]
    OutsideProject {
        /// Path requested by the user.
        checked_path: PathBuf,
        /// Discovered project root.
        project_root: PathBuf,
    },
    /// Filesystem I/O failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Directory walking failed.
    #[error(transparent)]
    WalkDir(#[from] jwalk::Error),
    /// Configuration loading failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
}

/// Run structure-first validation for a path.
pub fn run_structure_check(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
) -> Result<StructureCheckReport, CheckError> {
    let requested_path = match path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };

    if !requested_path.exists() {
        return Err(CheckError::MissingPath(requested_path));
    }

    let checked_path = requested_path.canonicalize()?;
    let (project_root, config_path) = discover_project(&checked_path, config_path)?;

    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let config = ConfigLoader::load(&config_path)?;
    let mut checker = StructureChecker::new(project_root.clone(), config, fail_fast);
    checker.check(checked_path, config_path)
}

fn discover_project(
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

struct StructureChecker {
    pub(super) project_root: PathBuf,
    pub(super) config: Config,
    pub(super) fail_fast: bool,
    pub(super) configured_dirs: HashSet<PathBuf>,
    pub(super) exclude_patterns: Vec<CompiledExclusion>,
    pub(super) naming_regexes: HashMap<String, Regex>,
    pub(super) glob_patterns: HashMap<String, Pattern>,
    pub(super) rules_cache: HashMap<PathBuf, EffectiveRules>,
}

impl StructureChecker {
    fn new(project_root: PathBuf, config: Config, fail_fast: bool) -> Self {
        let mut configured_dirs = HashSet::new();
        let mut naming_regexes = HashMap::new();
        let mut glob_patterns = HashMap::new();
        for (path, node) in &config.structure {
            let base = normalize_config_dir(path);
            collect_configured_dirs(base, node, &mut configured_dirs);
            collect_naming_regexes(node, &mut naming_regexes);
            patterns::collect_glob_patterns(node, &mut glob_patterns);
        }

        let exclude_patterns = config
            .exclude
            .iter()
            .map(|pattern| CompiledExclusion::new(pattern))
            .collect();

        Self {
            project_root,
            config,
            fail_fast,
            configured_dirs,
            exclude_patterns,
            naming_regexes,
            glob_patterns,
            rules_cache: HashMap::new(),
        }
    }

    fn check(
        &mut self,
        checked_path: PathBuf,
        config_path: PathBuf,
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

        self.validate_configured_structure(&mut report);

        if self.fail_fast && !report.violations.is_empty() {
            report.success = false;
            return Ok(report);
        }

        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let mut walker = jwalk::WalkDir::new(&checked_path)
            .skip_hidden(false)
            .parallelism(jwalk::Parallelism::Serial);
        if self.fail_fast {
            walker = walker.sort(true);
        }

        for entry in walker.process_read_dir(move |_depth, _path, _state, children| {
            children.retain_mut(|entry| {
                let Ok(entry) = entry else {
                    return true;
                };
                let path = entry.path();
                let rel = path.strip_prefix(&project_root).unwrap_or(&path);
                if is_excluded_rel_with(&exclude_patterns, rel) {
                    entry.read_children_path = None;
                    return false;
                }
                true
            });
        }) {
            let entry = entry?;
            let path = entry.path();
            if path == checked_path && path.is_dir() {
                self.validate_directory_contents(&path, &mut report);
                continue;
            }

            if path.is_dir() {
                report.dirs_checked += 1;
                self.validate_directory(&path, &mut report);
            } else if path.is_file() {
                report.files_checked += 1;
                self.validate_file(&path, &mut report);
            }

            if self.fail_fast && !report.violations.is_empty() {
                break;
            }
        }

        report
            .violations
            .sort_by(|left, right| left.path.cmp(&right.path).then(left.rule.cmp(&right.rule)));
        report.success = report.violations.is_empty();
        Ok(report)
    }

    fn validate_configured_structure(&self, report: &mut StructureCheckReport) {
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

        let mut result = EffectiveRules::default();
        for (path, node) in &self.config.structure {
            let base = normalize_config_dir(path);
            self.resolve_node(
                &base,
                node,
                dir_rel,
                &EffectiveRules::default(),
                &mut result,
            );
        }
        self.rules_cache
            .insert(dir_rel.to_path_buf(), result.clone());
        result
    }

    fn resolve_node(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        target_dir: &Path,
        inherited: &EffectiveRules,
        result: &mut EffectiveRules,
    ) {
        if !dir_contains(node_rel, target_dir) {
            return;
        }

        let effective = if node.inherit {
            EffectiveRules {
                files: merge_file_bundle(inherited.files.as_ref(), node.files.as_ref()),
                directories: merge_directory_bundle(
                    inherited.directories.as_ref(),
                    node.directories.as_ref(),
                ),
                markdown: merge_markdown_bundle(
                    inherited.markdown.as_ref(),
                    node.markdown.as_ref(),
                ),
            }
        } else {
            EffectiveRules {
                files: node.files.clone().map(std::sync::Arc::new),
                directories: node.directories.clone().map(std::sync::Arc::new),
                markdown: node.markdown.clone().map(std::sync::Arc::new),
            }
        };

        *result = if target_dir == node_rel {
            effective.clone()
        } else {
            strip_direct_content_policy(effective.clone())
        };

        if let Some(children) = &node.children {
            for (child_name, child) in children {
                let child_rel = join_config_child(node_rel, child_name);
                self.resolve_node(&child_rel, child, target_dir, &effective, result);
            }
        }
    }

    pub(super) fn relative_path(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.project_root)
            .unwrap_or(path)
            .to_path_buf()
    }

    fn is_excluded_rel(&self, rel: &Path) -> bool {
        is_excluded_rel_with(&self.exclude_patterns, rel)
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
