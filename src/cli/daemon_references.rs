//! Reference selector parsing for daemon CLI queries.

use std::path::PathBuf;

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
