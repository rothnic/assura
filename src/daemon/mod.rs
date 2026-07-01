//! Daemon-ready local project state contracts.
//!
//! This module does not start a background process. It provides the shared
//! warm-state contract that future daemon CLI, editor, hook, and agent
//! integrations can call without reimplementing project freshness or
//! repository-reference lookups.

use crate::cli::check::{CheckError, PreparedStructureCheck, StructureCheckReport};
use crate::cli::content_query::context::{ContentQueryError, QueryContext};
use crate::intelligence::{resource_id, RepositoryReferenceEdge};
use std::fs;
use std::path::{Path, PathBuf};

mod fingerprint;
mod types;

use fingerprint::ProjectFingerprint;
use types::{fallback_command, response_bounds};
pub(crate) use types::{serialize_optional_path, serialize_path};
pub use types::{
    DaemonAffectedReferences, DaemonHealth, DaemonHealthState, DaemonMovedTargetReferences,
    DaemonRepositoryReference, DaemonResponseBounds, DaemonRuntimePaths,
};

/// Errors returned by daemon-ready local state.
#[derive(Debug)]
pub enum DaemonCoreError {
    /// Structure-check preparation failed.
    Check(CheckError),
    /// Content/reference context loading failed.
    Content(String),
    /// Filesystem access failed.
    Io(std::io::Error),
    /// Cached state is stale and must be refreshed before success can be
    /// trusted.
    Stale(Box<DaemonHealth>),
}

impl std::fmt::Display for DaemonCoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Check(error) => write!(f, "{error}"),
            Self::Content(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::Stale(health) => write!(f, "daemon state is stale: {}", health.reason),
        }
    }
}

impl std::error::Error for DaemonCoreError {}

impl From<CheckError> for DaemonCoreError {
    fn from(error: CheckError) -> Self {
        Self::Check(error)
    }
}

impl From<ContentQueryError> for DaemonCoreError {
    fn from(error: ContentQueryError) -> Self {
        Self::Content(error.to_string())
    }
}

impl From<std::io::Error> for DaemonCoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Warm local project state for daemon/session integrations.
pub struct LocalDaemonCore {
    requested_path: PathBuf,
    requested_config: Option<PathBuf>,
    prepared: PreparedStructureCheck,
    context: QueryContext,
    fingerprint: ProjectFingerprint,
    config_hash: u64,
    generation: u64,
    state: DaemonHealthState,
    reason: String,
}

impl LocalDaemonCore {
    /// Load warm project state for a path and optional Assura config.
    pub fn load(path: PathBuf, config: Option<PathBuf>) -> Result<Self, DaemonCoreError> {
        let prepared =
            PreparedStructureCheck::load_for_path(Some(path.clone()), config.clone(), false)?;
        let context =
            QueryContext::load_for_path(path.clone(), config.clone(), false, false, true)?;
        let fingerprint = ProjectFingerprint::capture(&context.project_root)?;
        let config_hash = file_hash(&context.config_path)?;
        Ok(Self {
            requested_path: path,
            requested_config: config,
            prepared,
            context,
            fingerprint,
            config_hash,
            generation: 1,
            state: DaemonHealthState::Running,
            reason: "project state loaded".to_string(),
        })
    }

    /// Return current daemon/session health metadata.
    pub fn health(&self) -> DaemonHealth {
        DaemonHealth {
            state: self.state,
            reason: self.reason.clone(),
            project_root: self.context.project_root.clone(),
            config_path: self.context.config_path.clone(),
            generation: self.generation,
            runtime_paths: DaemonRuntimePaths::for_project(&self.context.project_root),
            fallback_command: self.fallback_command(),
        }
    }

    /// Return an observable warming response for clients before starting a
    /// long rebuild.
    pub fn warming_health(&self, reason: impl Into<String>) -> DaemonHealth {
        DaemonHealth {
            state: DaemonHealthState::Warming,
            reason: reason.into(),
            project_root: self.context.project_root.clone(),
            config_path: self.context.config_path.clone(),
            generation: self.generation,
            runtime_paths: DaemonRuntimePaths::for_project(&self.context.project_root),
            fallback_command: self.fallback_command(),
        }
    }

    /// Rebuild project state after a known stale or degraded condition.
    pub fn refresh(&mut self) -> Result<DaemonHealth, DaemonCoreError> {
        self.state = DaemonHealthState::Warming;
        self.reason = "rebuilding project state".to_string();
        self.prepared = PreparedStructureCheck::load_for_path(
            Some(self.requested_path.clone()),
            self.requested_config.clone(),
            false,
        )?;
        self.context = QueryContext::load_for_path(
            self.requested_path.clone(),
            self.requested_config.clone(),
            false,
            false,
            true,
        )?;
        self.fingerprint = ProjectFingerprint::capture(&self.context.project_root)?;
        self.config_hash = file_hash(&self.context.config_path)?;
        self.generation += 1;
        self.state = DaemonHealthState::Running;
        self.reason = "project state refreshed".to_string();
        Ok(self.health())
    }

    /// Mark watcher state degraded after a missed event or unreadable path.
    pub fn mark_degraded(&mut self, reason: impl Into<String>) -> DaemonHealth {
        self.state = DaemonHealthState::Degraded;
        self.reason = reason.into();
        self.health()
    }

    /// Validate one changed path against prepared structure rules.
    pub fn check_changed_path(
        &mut self,
        path: PathBuf,
    ) -> Result<StructureCheckReport, DaemonCoreError> {
        self.ensure_fresh()?;
        Ok(self.prepared.check_changed_path(path)?)
    }

    /// Return outbound repository references from a changed source path.
    pub fn changed_source_references(
        &mut self,
        path: PathBuf,
        limit: usize,
    ) -> Result<DaemonAffectedReferences, DaemonCoreError> {
        self.ensure_fresh()?;
        let rel_path = self.normalize_project_path(path);
        let all_references = self
            .context
            .store
            .repository_references_from_path(&rel_path);
        Ok(self.reference_response("source", rel_path, all_references, limit))
    }

    /// Return inbound repository references to a changed target path.
    pub fn changed_target_references(
        &mut self,
        path: PathBuf,
        limit: usize,
    ) -> Result<DaemonAffectedReferences, DaemonCoreError> {
        self.ensure_config_fresh()?;
        self.mark_degraded_if_project_changed()?;
        let rel_path = self.normalize_project_path(path);
        let target_id = resource_id(&rel_path);
        let all_references = self.context.store.repository_references_to(&target_id);
        Ok(self.reference_response("target", rel_path, all_references, limit))
    }

    /// Return inbound references for a target move using the previous path as
    /// the lookup key and the new path as caller context.
    pub fn moved_target_references(
        &mut self,
        previous_path: PathBuf,
        new_path: PathBuf,
        limit: usize,
    ) -> Result<DaemonMovedTargetReferences, DaemonCoreError> {
        self.ensure_config_fresh()?;
        self.mark_degraded_if_project_changed()?;
        let previous_path = self.normalize_project_path(previous_path);
        let new_path = self.normalize_project_path(new_path);
        let target_id = resource_id(&previous_path);
        let all_references = self.context.store.repository_references_to(&target_id);
        let bounds = response_bounds(all_references.len(), limit);
        let references = all_references
            .into_iter()
            .take(limit)
            .map(DaemonRepositoryReference::from)
            .collect();
        Ok(DaemonMovedTargetReferences {
            previous_path,
            new_path,
            health: self.health(),
            bounds,
            references,
        })
    }

    fn ensure_fresh(&mut self) -> Result<(), DaemonCoreError> {
        self.ensure_config_fresh()?;

        let latest = ProjectFingerprint::capture(&self.context.project_root)?;
        if latest == self.fingerprint {
            self.state = DaemonHealthState::Running;
            self.reason = "project state is current".to_string();
            return Ok(());
        }

        self.refresh()?;
        Ok(())
    }

    fn ensure_config_fresh(&mut self) -> Result<(), DaemonCoreError> {
        match file_hash(&self.context.config_path) {
            Ok(hash) if hash == self.config_hash => Ok(()),
            Ok(_) => self.stale_config(
                "configuration changed; refresh required before daemon results are trusted",
            ),
            Err(error) => self.stale_config(format!(
                "configuration unavailable or unreadable: {error}; refresh required before daemon results are trusted"
            )),
        }
    }

    fn stale_config(&mut self, reason: impl Into<String>) -> Result<(), DaemonCoreError> {
        self.state = DaemonHealthState::Stale;
        self.reason = reason.into();
        Err(DaemonCoreError::Stale(Box::new(self.health())))
    }

    fn mark_degraded_if_project_changed(&mut self) -> Result<(), DaemonCoreError> {
        let latest = ProjectFingerprint::capture(&self.context.project_root)?;
        if latest == self.fingerprint {
            self.state = DaemonHealthState::Running;
            self.reason = "project state is current".to_string();
        } else {
            self.state = DaemonHealthState::Degraded;
            self.reason =
                "project changed; target feedback uses prior warm reference graph".to_string();
        }
        Ok(())
    }

    fn normalize_project_path(&self, path: PathBuf) -> PathBuf {
        if path.is_absolute() {
            path.strip_prefix(&self.context.project_root)
                .map(Path::to_path_buf)
                .unwrap_or(path)
        } else {
            path
        }
    }

    fn reference_response(
        &self,
        mode: &'static str,
        path: PathBuf,
        all_references: Vec<&RepositoryReferenceEdge>,
        limit: usize,
    ) -> DaemonAffectedReferences {
        let total = all_references.len();
        let references = all_references
            .into_iter()
            .take(limit)
            .map(DaemonRepositoryReference::from)
            .collect();
        DaemonAffectedReferences {
            mode,
            path,
            health: self.health(),
            bounds: response_bounds(total, limit),
            references,
        }
    }

    fn fallback_command(&self) -> String {
        fallback_command(
            &self.context.project_root,
            self.requested_config
                .as_ref()
                .map(|_| self.context.config_path.as_path()),
        )
    }
}

fn file_hash(path: &Path) -> Result<u64, std::io::Error> {
    fs::read(path).map(|content| crate::stable_hash::stable_hash(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_states_are_serialized_for_clients() {
        let states = [
            DaemonHealthState::Running,
            DaemonHealthState::Warming,
            DaemonHealthState::Stale,
            DaemonHealthState::Degraded,
            DaemonHealthState::Unavailable,
            DaemonHealthState::Incompatible,
        ];
        let rendered = serde_json::to_value(states).unwrap();
        assert_eq!(
            rendered,
            serde_json::json!([
                "running",
                "warming",
                "stale",
                "degraded",
                "unavailable",
                "incompatible"
            ])
        );
    }
}
