//! Exact managed-artifact classification for Git hook pairs.

use super::{HookError, HookResult};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArtifactOwnership {
    Absent,
    Managed,
    Unmanaged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HookOwnership {
    pub(super) wrapper: ArtifactOwnership,
    pub(super) sidecar: ArtifactOwnership,
}

impl HookOwnership {
    pub(super) fn has_unmanaged(self) -> bool {
        self.wrapper == ArtifactOwnership::Unmanaged || self.sidecar == ArtifactOwnership::Unmanaged
    }

    pub(super) fn is_complete(self) -> bool {
        self.wrapper == ArtifactOwnership::Managed && self.sidecar == ArtifactOwnership::Managed
    }

    pub(super) fn has_managed_artifact(self) -> bool {
        self.wrapper == ArtifactOwnership::Managed || self.sidecar == ArtifactOwnership::Managed
    }

    pub(super) fn is_installed(self) -> bool {
        self.wrapper != ArtifactOwnership::Absent || self.sidecar != ArtifactOwnership::Absent
    }
}

pub(super) fn classify_artifact(
    path: &Path,
    expected_contents: &[String],
    parent_is_safe: bool,
) -> HookResult<ArtifactOwnership> {
    if !parent_is_safe {
        return Ok(ArtifactOwnership::Unmanaged);
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ArtifactOwnership::Absent)
        }
        Err(error) => return Err(error.into()),
    };
    if !metadata.file_type().is_file() {
        return Ok(ArtifactOwnership::Unmanaged);
    }
    let content = std::fs::read(path)?;
    if expected_contents
        .iter()
        .any(|expected| expected.as_bytes() == content)
    {
        Ok(ArtifactOwnership::Managed)
    } else {
        Ok(ArtifactOwnership::Unmanaged)
    }
}

pub(super) fn remove_file_if_still_managed(
    path: &Path,
    expected_contents: &[String],
) -> HookResult<()> {
    if classify_artifact(path, expected_contents, true)? != ArtifactOwnership::Managed {
        return Err(HookError::UnsafePath(path.to_path_buf()));
    }
    std::fs::remove_file(path)?;
    Ok(())
}

pub(super) fn plain_directory_or_absent(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_dir(),
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    }
}

pub(super) fn ensure_plain_directory(path: &Path) -> HookResult<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(()),
        Ok(_) => Err(HookError::UnsafePath(path.to_path_buf())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(path)?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) fn shell_single_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
