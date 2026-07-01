//! Serializable daemon health and changed-path response contracts.

use crate::intelligence::RepositoryReferenceEdge;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Daemon/session health state exposed to local clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonHealthState {
    /// The local project state is being prepared.
    Warming,
    /// Warm state is current enough to answer local requests.
    Running,
    /// Cached state is known stale and should not be trusted for success.
    Stale,
    /// The daemon can answer partially but has missed or degraded inputs.
    Degraded,
    /// No daemon/session state is available.
    Unavailable,
    /// The client and daemon/session contract versions are incompatible.
    Incompatible,
}

/// Runtime metadata paths for a project-local daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonRuntimePaths {
    /// Directory for daemon runtime metadata.
    pub status_dir: PathBuf,
    /// JSON status file path for lightweight clients.
    pub status_file: PathBuf,
    /// Log file path for runtime diagnostics.
    pub log_file: PathBuf,
}

impl DaemonRuntimePaths {
    pub(super) fn for_project(project_root: &Path) -> Self {
        let status_dir = project_root.join(".assura").join("daemon");
        Self {
            status_file: status_dir.join("status.json"),
            log_file: status_dir.join("daemon.log"),
            status_dir,
        }
    }
}

/// Health response shared by CLI, editor, hook, and agent clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonHealth {
    /// Machine-readable health state.
    pub state: DaemonHealthState,
    /// Human-readable reason for the current state.
    pub reason: String,
    /// Project root for this daemon/session state.
    pub project_root: PathBuf,
    /// Configuration file that controls this state.
    pub config_path: PathBuf,
    /// Monotonic in-process generation for rebuilt project state.
    pub generation: u64,
    /// Project-local runtime metadata paths.
    pub runtime_paths: DaemonRuntimePaths,
    /// One-shot command clients can run when daemon state is unavailable or
    /// stale.
    pub fallback_command: String,
}

impl DaemonHealth {
    /// Build a warming health response before a long refresh starts.
    pub fn warming(
        project_root: PathBuf,
        config_path: PathBuf,
        generation: u64,
        reason: impl Into<String>,
    ) -> Self {
        Self::with_state(
            DaemonHealthState::Warming,
            project_root,
            config_path,
            generation,
            reason,
        )
    }

    /// Build an unavailable health response for clients that cannot connect to
    /// a daemon/session.
    pub fn unavailable(
        project_root: PathBuf,
        config_path: PathBuf,
        reason: impl Into<String>,
    ) -> Self {
        Self::with_state(
            DaemonHealthState::Unavailable,
            project_root,
            config_path,
            0,
            reason,
        )
    }

    /// Build an incompatible health response for client/server contract drift.
    pub fn incompatible(
        project_root: PathBuf,
        config_path: PathBuf,
        reason: impl Into<String>,
    ) -> Self {
        Self::with_state(
            DaemonHealthState::Incompatible,
            project_root,
            config_path,
            0,
            reason,
        )
    }

    fn with_state(
        state: DaemonHealthState,
        project_root: PathBuf,
        config_path: PathBuf,
        generation: u64,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            state,
            reason: reason.into(),
            runtime_paths: DaemonRuntimePaths::for_project(&project_root),
            fallback_command: fallback_command(&project_root, Some(&config_path)),
            project_root,
            config_path,
            generation,
        }
    }
}

/// Bounds metadata for daemon reference responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonResponseBounds {
    /// Caller-requested maximum reference count.
    pub limit: usize,
    /// Number of references returned.
    pub returned: usize,
    /// Whether the result was truncated by `limit`.
    pub truncated: bool,
}

/// Repository-reference edge rendered for daemon clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonRepositoryReference {
    /// Stable edge ID.
    pub id: String,
    /// Repository-relative source path.
    pub source_path: PathBuf,
    /// One-based source line when known.
    pub source_line: Option<usize>,
    /// One-based source column when known.
    pub source_column: Option<usize>,
    /// Target resource ID when resolved.
    pub target_id: Option<String>,
    /// Repository-relative target path.
    pub target_path: PathBuf,
    /// Optional Markdown heading anchor without the leading `#`.
    pub target_anchor: Option<String>,
    /// Optional target line start.
    pub target_line_start: Option<usize>,
    /// Optional target line end.
    pub target_line_end: Option<usize>,
    /// Whether the target existed when the graph generation was built.
    pub target_exists: bool,
    /// Source reference kind.
    pub reference_kind: String,
    /// Rule ID associated with the reference.
    pub rule: String,
    /// Confidence level assigned by the scanner.
    pub confidence: String,
}

impl From<&RepositoryReferenceEdge> for DaemonRepositoryReference {
    fn from(edge: &RepositoryReferenceEdge) -> Self {
        Self {
            id: edge.id.to_string(),
            source_path: edge.source_path.clone(),
            source_line: edge.source_line,
            source_column: edge.source_column,
            target_id: edge.target_id.as_ref().map(ToString::to_string),
            target_path: edge.target_path.clone(),
            target_anchor: edge.target_anchor.clone(),
            target_line_start: edge.target_line_start,
            target_line_end: edge.target_line_end,
            target_exists: edge.target_exists,
            reference_kind: edge.reference_kind.clone(),
            rule: edge.rule.clone(),
            confidence: edge.confidence.clone(),
        }
    }
}

/// Changed-path reference response from daemon-ready state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonAffectedReferences {
    /// `source` for outbound references or `target` for inbound references.
    pub mode: &'static str,
    /// Repository-relative path requested by the caller.
    pub path: PathBuf,
    /// Current daemon/session health after freshness checks.
    pub health: DaemonHealth,
    /// Result bounds.
    pub bounds: DaemonResponseBounds,
    /// Affected repository-reference edges.
    pub references: Vec<DaemonRepositoryReference>,
}

/// Target-move reference response from daemon-ready state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonMovedTargetReferences {
    /// Previous repository-relative target path.
    pub previous_path: PathBuf,
    /// New repository-relative target path.
    pub new_path: PathBuf,
    /// Current daemon/session health after freshness checks.
    pub health: DaemonHealth,
    /// Result bounds.
    pub bounds: DaemonResponseBounds,
    /// Inbound references that pointed at `previous_path` in the warm graph.
    pub references: Vec<DaemonRepositoryReference>,
}

pub(super) fn response_bounds(total: usize, limit: usize) -> DaemonResponseBounds {
    DaemonResponseBounds {
        limit,
        returned: total.min(limit),
        truncated: total > limit,
    }
}

pub(super) fn fallback_command(project_root: &Path, config_path: Option<&Path>) -> String {
    let mut parts = vec!["assura".to_string()];
    if let Some(config_path) = config_path {
        parts.push("--config".to_string());
        parts.push(shell_quote_path(config_path));
    }
    parts.push("check".to_string());
    parts.push("--format".to_string());
    parts.push("json".to_string());
    parts.push(shell_quote_path(project_root));
    parts.join(" ")
}

fn shell_quote_path(path: &Path) -> String {
    let value = path.display().to_string();
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
    {
        value
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
