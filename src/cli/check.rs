//! Structure-first project validation used by the public `assura check` command.

use crate::cli::config::{ConfigDiscovery, ConfigError};
use crate::config::config::{Config, DirectoryNode, FileBundle, MarkdownBundle};
use crate::config::loader::ConfigLoader;
use crate::constraints::CaseConvention;
use glob::Pattern;
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

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
    WalkDir(#[from] walkdir::Error),
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

#[derive(Debug, Clone, Default)]
struct EffectiveRules {
    files: Option<FileBundle>,
    markdown: Option<MarkdownBundle>,
}

struct StructureChecker {
    project_root: PathBuf,
    config: Config,
    fail_fast: bool,
    configured_dirs: HashSet<PathBuf>,
    exclude_patterns: Vec<CompiledExclusion>,
    naming_regexes: HashMap<String, Regex>,
    rules_cache: HashMap<PathBuf, EffectiveRules>,
}

#[derive(Debug, Clone)]
struct CompiledExclusion {
    prefix: Option<String>,
    pattern: Option<Pattern>,
}

impl StructureChecker {
    fn new(project_root: PathBuf, config: Config, fail_fast: bool) -> Self {
        let mut configured_dirs = HashSet::new();
        let mut naming_regexes = HashMap::new();
        for (path, node) in &config.structure {
            let base = normalize_config_dir(path);
            collect_configured_dirs(base, node, &mut configured_dirs);
            collect_naming_regexes(node, &mut naming_regexes);
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
        for entry in WalkDir::new(&checked_path)
            .into_iter()
            .filter_entry(move |entry| {
                let rel = entry
                    .path()
                    .strip_prefix(&project_root)
                    .unwrap_or(entry.path());
                !is_excluded_rel_with(&exclude_patterns, rel)
            })
        {
            let entry = entry?;
            let path = entry.path();
            if path == checked_path && path.is_dir() {
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

    fn validate_directory(&mut self, path: &Path, report: &mut StructureCheckReport) {
        let rel = self.relative_path(path);
        if rel.as_os_str().is_empty() || self.configured_dirs.contains(&rel) {
            return;
        }

        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let rules = self.resolve_rules(parent_rel);
        let Some(files) = rules.files else {
            return;
        };

        let Some(naming) = files.naming.as_deref() else {
            return;
        };

        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !validate_name(name, naming, &self.naming_regexes) {
            self.push_violation(
                report,
                rel,
                "directory_naming",
                format!(
                    "Directory '{}' does not match naming convention '{}'",
                    name, naming
                ),
                severity_for_bundle(&files),
            );
        }
    }

    fn validate_file(&mut self, path: &Path, report: &mut StructureCheckReport) {
        let rel = self.relative_path(path);
        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let rules = self.resolve_rules(parent_rel);

        let needs_markdown =
            path.extension().and_then(|ext| ext.to_str()) == Some("md") && rules.markdown.is_some();
        let needs_file_content = rules.files.as_ref().is_some_and(|files| {
            files.max_lines.is_some()
                || (files.require_docs == Some(true)
                    && path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        });
        let content = if needs_file_content || needs_markdown {
            match fs::read_to_string(path) {
                Ok(content) => Some(content),
                Err(error) => {
                    let severity = rules
                        .files
                        .as_ref()
                        .map(severity_for_bundle)
                        .unwrap_or_else(|| "medium".to_string());
                    self.push_violation(
                        report,
                        rel.clone(),
                        "read_file",
                        format!("Could not read '{}': {}", display_rel(&rel), error),
                        severity,
                    );
                    None
                }
            }
        } else {
            None
        };

        if let Some(files) = rules.files {
            self.validate_file_bundle(path, &rel, &files, content.as_deref(), report);
        }

        if needs_markdown {
            if let (Some(markdown), Some(content)) = (rules.markdown, content.as_deref()) {
                self.validate_markdown(&rel, &markdown, content, report);
            }
        }
    }

    fn validate_file_bundle(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let allowed_by_name = files
            .allowed_names
            .as_ref()
            .map(|allowed| allowed.iter().any(|name| name == filename))
            .unwrap_or(false);

        if let Some(extensions) = &files.extensions {
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !extensions.iter().any(|allowed| allowed == ext) {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "extension",
                    format!("File '{}' has disallowed extension '{}'", filename, ext),
                    severity_for_bundle(files),
                );
            }
        }

        if !allowed_by_name {
            if let Some(naming) = &files.naming {
                let stem = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("");
                if !validate_file_stem(stem, naming, &self.naming_regexes) {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "file_naming",
                        format!(
                            "File '{}' does not match naming convention '{}'",
                            filename, naming
                        ),
                        severity_for_bundle(files),
                    );
                }
            }
        }

        if let Some(max_size) = &files.max_size {
            if let Some(max_bytes) = parse_size(max_size) {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() > max_bytes {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "max_size",
                            format!(
                                "File '{}' is {} bytes, exceeding limit {}",
                                display_rel(rel),
                                metadata.len(),
                                max_size
                            ),
                            severity_for_bundle(files),
                        );
                    }
                }
            }
        }

        if let (Some(max_lines), Some(content)) = (files.max_lines, content) {
            let line_count = content.lines().count();
            if line_count > max_lines {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "max_lines",
                    format!(
                        "File '{}' has {} lines, exceeding limit {}",
                        display_rel(rel),
                        line_count,
                        max_lines
                    ),
                    severity_for_bundle(files),
                );
            }
        }

        if files.require_docs == Some(true)
            && path.extension().and_then(|ext| ext.to_str()) == Some("rs")
        {
            if let Some(content) = content {
                if !content.contains("//!") && !content.contains("///") {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "require_docs",
                        format!("Rust file '{}' is missing rustdoc", display_rel(rel)),
                        severity_for_bundle(files),
                    );
                }
            }
        }
    }

    fn validate_markdown(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let frontmatter = parse_frontmatter(content);

        if markdown.require_frontmatter == Some(true) && frontmatter.is_none() {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "markdown_frontmatter",
                format!(
                    "Markdown file '{}' is missing YAML frontmatter",
                    display_rel(rel)
                ),
                "medium",
            );
        }

        if let Some(required_fields) = &markdown.required_fields {
            match frontmatter {
                Some(frontmatter) => match serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                    Ok(value) => {
                        for field in required_fields {
                            if value.get(field).is_none() {
                                self.push_violation(
                                    report,
                                    rel.to_path_buf(),
                                    "markdown_frontmatter_field",
                                    format!(
                                        "Markdown file '{}' is missing frontmatter field '{}'",
                                        display_rel(rel),
                                        field
                                    ),
                                    "medium",
                                );
                            }
                        }
                    }
                    Err(error) => {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "markdown_frontmatter_parse",
                            format!(
                                "Markdown file '{}' has invalid frontmatter: {}",
                                display_rel(rel),
                                error
                            ),
                            "medium",
                        );
                    }
                },
                None => {
                    for field in required_fields {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "markdown_frontmatter_field",
                            format!(
                                "Markdown file '{}' cannot satisfy required field '{}' without frontmatter",
                                display_rel(rel),
                                field
                            ),
                            "medium",
                        );
                    }
                }
            }
        }

        if let Some(max_depth) = markdown.max_heading_depth {
            for line in content.lines() {
                let depth = line.chars().take_while(|ch| *ch == '#').count();
                if depth > usize::from(max_depth) && line.chars().nth(depth) == Some(' ') {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "markdown_heading_depth",
                        format!(
                            "Markdown file '{}' has heading depth {}, exceeding limit {}",
                            display_rel(rel),
                            depth,
                            max_depth
                        ),
                        "medium",
                    );
                    break;
                }
            }
        }

        if let Some(required_sections) = &markdown.required_sections {
            let mut headings = HashSet::new();
            for line in content.lines() {
                if let Some(section) = line.strip_prefix("# ").or_else(|| line.strip_prefix("## "))
                {
                    headings.insert(section);
                }
            }

            for section in required_sections {
                if !headings.contains(section.as_str()) {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "markdown_required_section",
                        format!(
                            "Markdown file '{}' is missing required section '{}'",
                            display_rel(rel),
                            section
                        ),
                        "medium",
                    );
                }
            }
        }
    }

    fn resolve_rules(&mut self, dir_rel: &Path) -> EffectiveRules {
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
                markdown: merge_markdown_bundle(
                    inherited.markdown.as_ref(),
                    node.markdown.as_ref(),
                ),
            }
        } else {
            EffectiveRules {
                files: node.files.clone(),
                markdown: node.markdown.clone(),
            }
        };

        *result = effective.clone();

        if let Some(children) = &node.children {
            for (child_name, child) in children {
                let child_rel = join_config_child(node_rel, child_name);
                self.resolve_node(&child_rel, child, target_dir, &effective, result);
            }
        }
    }

    fn relative_path(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.project_root)
            .unwrap_or(path)
            .to_path_buf()
    }

    fn is_excluded_rel(&self, rel: &Path) -> bool {
        is_excluded_rel_with(&self.exclude_patterns, rel)
    }

    fn push_violation(
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

impl CompiledExclusion {
    fn new(pattern: &str) -> Self {
        Self {
            prefix: pattern.strip_suffix("/**").map(ToOwned::to_owned),
            pattern: Pattern::new(pattern).ok(),
        }
    }

    fn matches(&self, rel: &str) -> bool {
        if let Some(prefix) = &self.prefix {
            if rel == prefix || rel.starts_with(&format!("{}/", prefix)) {
                return true;
            }
        }

        self.pattern
            .as_ref()
            .map(|pattern| pattern.matches(rel))
            .unwrap_or(false)
    }
}

fn collect_configured_dirs(
    node_rel: PathBuf,
    node: &DirectoryNode,
    configured_dirs: &mut HashSet<PathBuf>,
) {
    configured_dirs.insert(node_rel.clone());

    if let Some(children) = &node.children {
        for (child_name, child) in children {
            let child_rel = join_config_child(&node_rel, child_name);
            collect_configured_dirs(child_rel, child, configured_dirs);
        }
    }
}

fn collect_naming_regexes(node: &DirectoryNode, regexes: &mut HashMap<String, Regex>) {
    if let Some(files) = &node.files {
        if let Some(naming) = &files.naming {
            collect_naming_regex(naming, regexes);
        }
    }

    if let Some(children) = &node.children {
        for child in children.values() {
            collect_naming_regexes(child, regexes);
        }
    }
}

fn collect_naming_regex(convention: &str, regexes: &mut HashMap<String, Regex>) {
    let Some(pattern) = convention.strip_prefix("regex:") else {
        return;
    };

    if regexes.contains_key(pattern) {
        return;
    }

    if let Ok(regex) = Regex::new(pattern) {
        regexes.insert(pattern.to_string(), regex);
    }
}

fn is_excluded_rel_with(patterns: &[CompiledExclusion], rel: &Path) -> bool {
    if rel.as_os_str().is_empty() {
        return false;
    }

    let rel = rel_to_string(rel);
    patterns.iter().any(|pattern| pattern.matches(&rel))
}

fn normalize_config_dir(path: &str) -> PathBuf {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "." {
        PathBuf::new()
    } else if let Some(stripped) = trimmed.strip_prefix("./") {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(trimmed)
    }
}

fn join_config_child(parent: &Path, child_name: &str) -> PathBuf {
    let child = normalize_config_dir(child_name);
    if parent.as_os_str().is_empty() {
        child
    } else {
        parent.join(child)
    }
}

fn dir_contains(node_rel: &Path, target_dir: &Path) -> bool {
    node_rel.as_os_str().is_empty() || target_dir == node_rel || target_dir.starts_with(node_rel)
}

fn merge_file_bundle(
    parent: Option<&FileBundle>,
    child: Option<&FileBundle>,
) -> Option<FileBundle> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(FileBundle {
            required: None,
            allowed_names: None,
            ..parent.clone()
        }),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => Some(FileBundle {
            naming: child.naming.clone().or_else(|| parent.naming.clone()),
            max_lines: child.max_lines.or(parent.max_lines),
            max_size: child.max_size.clone().or_else(|| parent.max_size.clone()),
            require_docs: child.require_docs.or(parent.require_docs),
            extensions: child
                .extensions
                .clone()
                .or_else(|| parent.extensions.clone()),
            severity: child.severity.clone().or_else(|| parent.severity.clone()),
            required: child.required.clone(),
            allowed_names: child.allowed_names.clone(),
        }),
    }
}

fn merge_markdown_bundle(
    parent: Option<&MarkdownBundle>,
    child: Option<&MarkdownBundle>,
) -> Option<MarkdownBundle> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(parent.clone()),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => Some(MarkdownBundle {
            require_frontmatter: child.require_frontmatter.or(parent.require_frontmatter),
            required_fields: child
                .required_fields
                .clone()
                .or_else(|| parent.required_fields.clone()),
            max_heading_depth: child.max_heading_depth.or(parent.max_heading_depth),
            check_links: child.check_links.or(parent.check_links),
            required_sections: child
                .required_sections
                .clone()
                .or_else(|| parent.required_sections.clone()),
        }),
    }
}

fn validate_name(name: &str, convention: &str, regexes: &HashMap<String, Regex>) -> bool {
    if let Some(pattern) = convention.strip_prefix("regex:") {
        return regexes
            .get(pattern)
            .map(|regex| regex.is_match(name))
            .unwrap_or(false);
    }

    match convention_to_case(convention) {
        Some(case) => case.validate(name),
        None => false,
    }
}

fn validate_file_stem(stem: &str, convention: &str, regexes: &HashMap<String, Regex>) -> bool {
    validate_name(stem, convention, regexes)
        || stem
            .split_once('.')
            .map(|(base, _)| validate_name(base, convention, regexes))
            .unwrap_or(false)
}

fn convention_to_case(convention: &str) -> Option<CaseConvention> {
    match convention {
        "snake_case" => Some(CaseConvention::SnakeCase),
        "camelCase" => Some(CaseConvention::CamelCase),
        "PascalCase" => Some(CaseConvention::PascalCase),
        "kebab-case" => Some(CaseConvention::KebabCase),
        "SCREAMING_SNAKE_CASE" => Some(CaseConvention::ScreamingSnakeCase),
        "dot.case" => Some(CaseConvention::DotCase),
        "flatcase" => Some(CaseConvention::FlatCase),
        "FLATCASE" => Some(CaseConvention::ScreamingFlatCase),
        "COBOL-CASE" => Some(CaseConvention::CobolCase),
        "Train-Case" => Some(CaseConvention::TrainCase),
        "lowercase" => Some(CaseConvention::LowerCase),
        "UPPERCASE" => Some(CaseConvention::UpperCase),
        _ => None,
    }
}

fn parse_size(size: &str) -> Option<u64> {
    let size = size.trim();
    let split = size
        .find(|ch: char| !ch.is_ascii_digit() && !ch.is_ascii_whitespace())
        .unwrap_or(size.len());
    let amount: u64 = size[..split].trim().parse().ok()?;
    let unit = size[split..].trim().to_ascii_uppercase();

    match unit.as_str() {
        "B" => Some(amount),
        "KB" => Some(amount * 1024),
        "MB" => Some(amount * 1024 * 1024),
        "GB" => Some(amount * 1024 * 1024 * 1024),
        "TB" => Some(amount * 1024 * 1024 * 1024 * 1024),
        _ => None,
    }
}

fn parse_frontmatter(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }

    let start = 4;
    let end = content[start..].find("\n---")?;
    Some(&content[start..start + end])
}

fn rel_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn display_rel(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        rel_to_string(path)
    }
}

fn severity_for_node(node: &DirectoryNode) -> String {
    node.files
        .as_ref()
        .and_then(|files| files.severity.clone())
        .unwrap_or_else(|| "medium".to_string())
}

fn severity_for_bundle(files: &FileBundle) -> String {
    files
        .severity
        .clone()
        .unwrap_or_else(|| "medium".to_string())
}
