//! Prepared structure-check plans for hot validation sessions.

use super::rules::is_excluded_rel_with;
use super::{
    compiled_fingerprint::SourceConfigFingerprint, discover_project, CheckError, CheckTargetMode,
    CompiledStructureConfig, StructureCheckReport, StructureCheckTimings, StructureChecker,
};
use crate::config::config::{Config, DirectoryNode, FileBundle};
use crate::config::loader::ConfigLoader;
use crate::stable_hash::stable_hash;
use std::fs;
use std::path::{Path, PathBuf};

/// A parsed, validated, and compiled structure-check configuration.
///
/// This is intended for long-lived editor or daemon sessions where the
/// configuration usually stays unchanged while checked files change. It keeps
/// rule planning separate from per-run traversal and reloads only when the
/// configuration bytes change.
pub struct PreparedStructureCheck {
    project_root: PathBuf,
    config_path: PathBuf,
    config_hash: u64,
    config_fingerprint: Option<SourceConfigFingerprint>,
    incremental_path_safe: bool,
    fail_fast: bool,
    check_compiled: CompiledStructureConfig,
    compiled: CompiledStructureConfig,
}

impl PreparedStructureCheck {
    /// Load and compile the configuration discovered for a checked path.
    pub fn load_for_path(
        path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        fail_fast: bool,
    ) -> Result<Self, CheckError> {
        let requested_path = match path {
            Some(path) => path,
            None => std::env::current_dir()?,
        };

        if !requested_path.exists() {
            return Err(CheckError::MissingPath(requested_path));
        }

        let checked_path = requested_path.canonicalize()?;
        let (project_root, discovered_config_path) = discover_project(&checked_path, config_path)?;
        if !checked_path.starts_with(&project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root,
            });
        }

        Self::load_project(project_root, discovered_config_path, fail_fast)
    }

    /// Validate one path using the prepared configuration.
    pub fn check_path(&self, path: PathBuf) -> Result<StructureCheckReport, CheckError> {
        if !path.exists() {
            return Err(CheckError::MissingPath(path));
        }

        let checked_path = path.canonicalize()?;
        if !checked_path.starts_with(&self.project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root: self.project_root.clone(),
            });
        }

        let mut checker = StructureChecker::from_compiled(
            self.project_root.clone(),
            &self.check_compiled,
            self.fail_fast,
        );
        let mut timings = StructureCheckTimings::default();
        checker.check(
            checked_path,
            self.config_path.clone(),
            CheckTargetMode::Recursive,
            &mut timings,
        )
    }

    /// Validate one changed path and its direct aggregate scopes without
    /// traversing the whole project.
    ///
    /// This is intended for editor and daemon integrations that already keep a
    /// project-level result warm and need a low-latency answer for the file or
    /// directory currently being edited. It does not prove whole-project
    /// success; callers that need a complete report should use `check_path`.
    pub fn check_changed_path(&self, path: PathBuf) -> Result<StructureCheckReport, CheckError> {
        if !self.incremental_path_safe {
            return self.check_path(self.project_root.clone());
        }

        let checked_path = self.resolve_changed_path(path)?;
        if !checked_path.starts_with(&self.project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root: self.project_root.clone(),
            });
        }

        let mut report = StructureCheckReport {
            success: true,
            project_root: self.project_root.clone(),
            config_path: self.config_path.clone(),
            checked_path: checked_path.clone(),
            files_checked: 0,
            dirs_checked: 0,
            violations: Vec::new(),
        };

        let mut checker = StructureChecker::from_compiled(
            self.project_root.clone(),
            &self.compiled,
            self.fail_fast,
        );
        checker.validate_configured_structure(&mut report);
        checker.validate_one_changed_path(&checked_path, &mut report);
        report.sort_violations_staged();
        report.refresh_success();
        Ok(report)
    }

    fn resolve_changed_path(&self, path: PathBuf) -> Result<PathBuf, CheckError> {
        if path.exists() {
            return Ok(path.canonicalize()?);
        }

        let path = if path.is_absolute() {
            path
        } else {
            self.project_root.join(path)
        };
        let mut cursor = path.as_path();
        let mut missing_components = Vec::new();
        while !cursor.exists() {
            let Some(name) = cursor.file_name() else {
                return Ok(path);
            };
            missing_components.push(name.to_os_string());
            let Some(parent) = cursor.parent() else {
                return Ok(path);
            };
            cursor = parent;
        }

        let mut resolved = cursor.canonicalize()?;
        for component in missing_components.iter().rev() {
            resolved.push(component);
        }
        Ok(resolved)
    }

    /// Return the project root this prepared checker was built for.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Return the configuration path this prepared checker watches.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Return whether one changed path can prove the configured policy locally.
    ///
    /// Repository-wide extensions and content relationships require a full
    /// validation pass because their result can depend on paths other than the
    /// file that triggered the check.
    pub fn supports_incremental_path_checks(&self) -> bool {
        self.incremental_path_safe
    }

    /// Return whether the configuration bytes differ from the prepared plan.
    pub fn config_content_changed(&self) -> Result<bool, CheckError> {
        let content = fs::read(&self.config_path).map_err(CheckError::Io)?;
        Ok(stable_hash(&content) != self.config_hash)
    }

    /// Return whether a project path is excluded by the prepared configuration.
    pub fn is_excluded_path(&self, path: &Path) -> bool {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.project_root.join(path)
        };
        absolute
            .strip_prefix(&self.project_root)
            .is_ok_and(|relative| is_excluded_rel_with(&self.compiled.exclude_patterns, relative))
    }

    /// Build a prepared checker from an already parsed config.
    pub fn from_config(
        project_root: PathBuf,
        config_path: PathBuf,
        config: Config,
        fail_fast: bool,
    ) -> Self {
        let incremental_path_safe = config_supports_incremental_path_checks(&config);
        let check_compiled = CompiledStructureConfig::new_for_check(config.clone(), fail_fast);
        let compiled = CompiledStructureConfig::new(config, fail_fast);
        Self {
            project_root,
            config_path,
            config_hash: 0,
            config_fingerprint: None,
            incremental_path_safe,
            fail_fast,
            check_compiled,
            compiled,
        }
    }

    /// Reload the compiled plan only when the configuration content changed.
    pub fn reload_if_config_changed(&mut self) -> Result<bool, CheckError> {
        let content = fs::read_to_string(&self.config_path).map_err(CheckError::Io)?;
        let config_hash = stable_hash(content.as_bytes());
        if config_hash == self.config_hash {
            self.config_fingerprint = SourceConfigFingerprint::from_path(&self.config_path).ok();
            return Ok(false);
        }

        let config = ConfigLoader::parse_validated(&content)?;
        self.incremental_path_safe = config_supports_incremental_path_checks(&config);
        self.check_compiled =
            CompiledStructureConfig::new_for_check(config.clone(), self.fail_fast);
        self.compiled = CompiledStructureConfig::new(config, self.fail_fast);
        self.config_hash = config_hash;
        self.config_fingerprint = SourceConfigFingerprint::from_path(&self.config_path).ok();
        Ok(true)
    }

    fn load_project(
        project_root: PathBuf,
        config_path: PathBuf,
        fail_fast: bool,
    ) -> Result<Self, CheckError> {
        let content = fs::read_to_string(&config_path).map_err(CheckError::Io)?;
        let config_hash = stable_hash(content.as_bytes());
        let config = ConfigLoader::parse_validated(&content)?;
        let incremental_path_safe = config_supports_incremental_path_checks(&config);
        let check_compiled = CompiledStructureConfig::new_for_check(config.clone(), fail_fast);
        let compiled = CompiledStructureConfig::new(config, fail_fast);
        let config_fingerprint = SourceConfigFingerprint::from_path(&config_path).ok();

        Ok(Self {
            project_root,
            config_path,
            config_hash,
            config_fingerprint,
            incremental_path_safe,
            fail_fast,
            check_compiled,
            compiled,
        })
    }
}

fn config_supports_incremental_path_checks(config: &Config) -> bool {
    config.extensions.as_ref().map_or(true, |extensions| {
        extensions.custom_constraints.is_empty()
            && extensions.release_contracts.is_empty()
            && extensions.support_matrices.is_empty()
            && extensions.manifest_semantics.is_empty()
            && extensions.test_relationships.is_empty()
            && extensions.module_topologies.is_empty()
            && extensions.docs_lifecycles.is_empty()
            && extensions.repository_references.is_empty()
            && extensions.agent_guidance.is_empty()
            && extensions.requirements_traceability.is_empty()
            && extensions.computed_checks.is_empty()
            && extensions.relationships.is_empty()
    }) && config.models.is_none()
        && config.collections.is_empty()
        && config.relations.is_empty()
        && config.code_symbols.is_empty()
        && config
            .patterns
            .values()
            .all(file_bundle_supports_incremental_path_checks)
        && config
            .structure
            .values()
            .all(directory_node_supports_incremental_path_checks)
}

fn file_bundle_supports_incremental_path_checks(files: &FileBundle) -> bool {
    files.markdown_patterns.is_none() && files.require_docs != Some(true)
}

fn directory_node_supports_incremental_path_checks(node: &DirectoryNode) -> bool {
    node.markdown.is_none()
        && node
            .files
            .as_ref()
            .map_or(true, file_bundle_supports_incremental_path_checks)
        && node.children.as_ref().map_or(true, |children| {
            children
                .values()
                .all(directory_node_supports_incremental_path_checks)
        })
}

#[cfg(test)]
#[path = "prepared_tests.rs"]
mod tests;
