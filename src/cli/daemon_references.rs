//! Reference selector parsing for daemon CLI queries.

use crate::daemon::{DaemonAffectedReferences, DaemonMovedTargetReferences};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(super) enum DaemonReferenceRequest {
    Source(PathBuf),
    Target(PathBuf),
    MovedTarget {
        previous_path: PathBuf,
        new_path: PathBuf,
    },
}

pub(super) fn reference_request(
    source: Option<PathBuf>,
    target: Option<PathBuf>,
    moved_target: Option<PathBuf>,
    new_target: Option<PathBuf>,
) -> Result<DaemonReferenceRequest, String> {
    let selected = source.is_some() as u8 + target.is_some() as u8 + moved_target.is_some() as u8;
    if selected != 1 {
        return Err(
            "daemon references requires exactly one of --source, --target, or --moved-target"
                .to_string(),
        );
    }
    if let Some(path) = source {
        return Ok(DaemonReferenceRequest::Source(path));
    }
    if let Some(path) = target {
        return Ok(DaemonReferenceRequest::Target(path));
    }
    let previous_path = moved_target.expect("moved_target selected");
    let Some(new_path) = new_target else {
        return Err("--moved-target requires --new-target".to_string());
    };
    Ok(DaemonReferenceRequest::MovedTarget {
        previous_path,
        new_path,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct DaemonReferenceIpcRequest {
    pub(super) mode: String,
    pub(super) path: String,
    pub(super) new_path: Option<String>,
    pub(super) limit: usize,
}

impl DaemonReferenceIpcRequest {
    pub(super) fn from_cli(request: &DaemonReferenceRequest, limit: usize) -> Self {
        match request {
            DaemonReferenceRequest::Source(path) => Self::new("source", path, None, limit),
            DaemonReferenceRequest::Target(path) => Self::new("target", path, None, limit),
            DaemonReferenceRequest::MovedTarget {
                previous_path,
                new_path,
            } => Self::new("moved-target", previous_path, Some(new_path), limit),
        }
    }

    fn new(mode: &str, path: &Path, new_path: Option<&Path>, limit: usize) -> Self {
        Self {
            mode: mode.to_string(),
            path: path.to_string_lossy().to_string(),
            new_path: new_path.map(|path| path.to_string_lossy().to_string()),
            limit,
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DaemonReferenceIpcOutput {
    schema: &'static str,
    protocol_version: &'static str,
    #[serde(flatten)]
    response: DaemonAffectedReferences,
}

impl DaemonReferenceIpcOutput {
    pub(super) fn affected(
        response: DaemonAffectedReferences,
        protocol_version: &'static str,
    ) -> Self {
        Self {
            schema: "assura.daemon.references.v1",
            protocol_version,
            response,
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DaemonMovedReferenceIpcOutput {
    schema: &'static str,
    protocol_version: &'static str,
    #[serde(flatten)]
    response: DaemonMovedTargetReferences,
}

impl DaemonMovedReferenceIpcOutput {
    pub(super) fn new(
        response: DaemonMovedTargetReferences,
        protocol_version: &'static str,
    ) -> Self {
        Self {
            schema: "assura.daemon.references.v1",
            protocol_version,
            response,
        }
    }
}
