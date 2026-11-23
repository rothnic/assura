//! Prepared structure-check plans for hot validation sessions.

use super::rules::is_excluded_rel_with;
use super::{
    project::discover_project_with_origin, CheckError, CheckTargetMode, CompiledStructureConfig,
    StructureCheckReport, StructureCheckTimings, StructureChecker,
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
/// rule planning separate from per-run traversal and reloads when its
/// configuration selection or bytes change.
pub struct PreparedStructureCheck {
    project_root: PathBuf,
    config_path: PathBuf,
    config_hash: u64,
    discovery_anchor: Option<PathBuf>,
    git_source: Option<(String, PathBuf)>,
    incremental_path_safe: bool,
    fail_fast: bool,
    check_compiled: CompiledStructureConfig,
    compiled: CompiledStructureConfig,
    config: Config,
}

/// Results for one bounded changed-path validation batch.
///
/// A repository-wide policy is represented by one full-project report and an
/// explicit fallback count instead of repeating the same expensive report for
/// every changed path.
#[cfg(feature = "full-cli")]
#[derive(Debug)]
pub(crate) struct PreparedChangedPathsReport {
    pub(crate) reports: Vec<StructureCheckReport>,
    pub(crate) requested_paths: usize,
    pub(crate) full_project_fallbacks: usize,
    pub(crate) coverage: &'static str,
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
        let discovery_anchor = config_path.is_none().then(|| checked_path.clone());
        let (project_root, config_path, origin) =
            discover_project_with_origin(&checked_path, config_path)?;
        if !checked_path.starts_with(&project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root,
            });
        }

        let git_source = git_source(&config_path, origin)?;
        let content = fs::read_to_string(&config_path)?;
        let config = ConfigLoader::parse_validated(&content)?;
        let mut prepared = Self::from_config(project_root, config_path, config, fail_fast);
        prepared.config_hash = stable_hash(content.as_bytes());
        prepared.discovery_anchor = discovery_anchor;
        prepared.git_source = git_source;
        Ok(prepared)
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

        let mut checker = StructureChecker::from_compiled(
            self.project_root.clone(),
            &self.compiled,
            self.fail_fast,
        );
        self.check_changed_path_with_checker(path, &mut checker)
    }

    /// Validate a bounded batch of changed paths while reusing one checker.
    ///
    /// Cross-path policies intentionally produce one authoritative
    /// full-project report for the batch. The returned coverage and fallback
    /// count make that reduction observable to callers.
    #[cfg(feature = "full-cli")]
    pub(crate) fn check_changed_paths(
        &self,
        paths: Vec<PathBuf>,
    ) -> Result<PreparedChangedPathsReport, CheckError> {
        let requested_paths = paths.len();
        if requested_paths == 0 {
            return Ok(PreparedChangedPathsReport {
                reports: Vec::new(),
                requested_paths,
                full_project_fallbacks: 0,
                coverage: "none",
            });
        }

        if !self.incremental_path_safe {
            return Ok(PreparedChangedPathsReport {
                reports: vec![self.check_path(self.project_root.clone())?],
                requested_paths,
                full_project_fallbacks: 1,
                coverage: "full_project",
            });
        }

        let mut checker = StructureChecker::from_compiled(
            self.project_root.clone(),
            &self.compiled,
            self.fail_fast,
        );
        let mut reports = Vec::with_capacity(requested_paths);
        for path in paths {
            reports.push(self.check_changed_path_with_checker(path, &mut checker)?);
        }
        Ok(PreparedChangedPathsReport {
            reports,
            requested_paths,
            full_project_fallbacks: 0,
            coverage: "changed_paths",
        })
    }

    fn check_changed_path_with_checker(
        &self,
        path: PathBuf,
        checker: &mut StructureChecker,
    ) -> Result<StructureCheckReport, CheckError> {
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

    #[cfg(feature = "full-cli")]
    pub(crate) fn uses_git_config(&self) -> bool {
        self.git_source.is_some()
    }

    /// Return the parsed project configuration for lightweight consumers.
    pub(crate) fn config(&self) -> &Config {
        &self.config
    }

    /// Return whether one changed path can prove the configured policy locally.
    ///
    /// Repository-wide extensions and content relationships require a full
    /// validation pass because their result can depend on paths other than the
    /// file that triggered the check.
    pub fn supports_incremental_path_checks(&self) -> bool {
        self.incremental_path_safe
    }

    /// Return whether the selected configuration or its bytes differ from the prepared plan.
    pub fn config_content_changed(&self) -> Result<bool, CheckError> {
        self.config_update().map(|update| update.is_some())
    }

    // Watch event normalization checks only the known file. Selection runs once per batch.
    #[cfg(feature = "full-cli")]
    pub(crate) fn watched_config_content_changed(&self) -> Result<bool, CheckError> {
        let content = fs::read(&self.config_path)?;
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
        let compiled = CompiledStructureConfig::new(config.clone(), fail_fast);
        Self {
            project_root,
            config_path,
            config_hash: 0,
            discovery_anchor: None,
            git_source: None,
            incremental_path_safe,
            fail_fast,
            check_compiled,
            compiled,
            config,
        }
    }

    /// Rediscover implicit configuration and reload the compiled plan if its source or bytes changed.
    pub fn reload_if_config_changed(&mut self) -> Result<bool, CheckError> {
        let Some(update) = self.config_update()? else {
            return Ok(false);
        };
        let config = ConfigLoader::parse_validated(&config_text(update.content)?)?;
        let mut next = Self::from_config(
            update.project_root,
            update.config_path,
            config,
            self.fail_fast,
        );
        next.config_hash = update.config_hash;
        next.discovery_anchor = self.discovery_anchor.take();
        next.git_source = update.git_source;
        *self = next;
        Ok(true)
    }

    fn config_update(&self) -> Result<Option<ConfigUpdate>, CheckError> {
        let selection = self
            .discovery_anchor
            .as_deref()
            .map(|anchor| {
                let (root, config, origin) = discover_project_with_origin(anchor, None)?;
                if !anchor.starts_with(&root) {
                    return Err(CheckError::OutsideProject {
                        checked_path: anchor.to_path_buf(),
                        project_root: root,
                    });
                }
                Ok((root, config, origin))
            })
            .transpose()?;
        let config_path = selection
            .as_ref()
            .map_or(self.config_path.as_path(), |(_, path, _)| path.as_path());
        let content = fs::read(config_path)?;
        let config_hash = stable_hash(&content);
        let (project_root, config_path, git_source) = match selection {
            Some((root, path, origin)) => {
                let git_source = git_source(&path, origin)?;
                (root, path, git_source)
            }
            None if config_hash == self.config_hash => return Ok(None),
            None => (self.project_root.clone(), self.config_path.clone(), None),
        };
        let changed = project_root != self.project_root
            || config_path != self.config_path
            || git_source != self.git_source
            || config_hash != self.config_hash;
        Ok(changed.then_some(ConfigUpdate {
            project_root,
            config_path,
            git_source,
            content,
            config_hash,
        }))
    }
}

struct ConfigUpdate {
    project_root: PathBuf,
    config_path: PathBuf,
    git_source: Option<(String, PathBuf)>,
    content: Vec<u8>,
    config_hash: u64,
}

fn git_source(
    path: &Path,
    origin: Option<String>,
) -> Result<Option<(String, PathBuf)>, CheckError> {
    origin
        .map(|origin| Ok((origin, path.canonicalize()?)))
        .transpose()
}

fn config_text(bytes: Vec<u8>) -> Result<String, CheckError> {
    String::from_utf8(bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error).into())
}

fn config_supports_incremental_path_checks(config: &Config) -> bool {
    config.extensions.as_ref().is_none_or(|extensions| {
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
            .is_none_or(file_bundle_supports_incremental_path_checks)
        && node.children.as_ref().is_none_or(|children| {
            children
                .values()
                .all(directory_node_supports_incremental_path_checks)
        })
}

#[cfg(test)]
#[path = "prepared_tests.rs"]
mod tests;
