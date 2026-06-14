//! Source config freshness fingerprints for compiled artifacts.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct SourceConfigFingerprint {
    len: u64,
    modified_ns: u128,
    #[serde(default)]
    unix_dev: Option<u64>,
    #[serde(default)]
    unix_ino: Option<u64>,
    #[serde(default)]
    unix_ctime_ns: Option<i128>,
}

impl SourceConfigFingerprint {
    pub(super) fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        Ok(Self::from_metadata(&metadata))
    }

    pub(super) fn differs_from_path(&self, path: &Path) -> bool {
        self.has_strong_identity() && Self::from_path(path).is_ok_and(|current| &current != self)
    }

    fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        Self {
            len: metadata.len(),
            modified_ns: metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos())
                .unwrap_or(0),
            #[cfg(unix)]
            unix_dev: Some(unix_dev(metadata)),
            #[cfg(not(unix))]
            unix_dev: None,
            #[cfg(unix)]
            unix_ino: Some(unix_ino(metadata)),
            #[cfg(not(unix))]
            unix_ino: None,
            #[cfg(unix)]
            unix_ctime_ns: Some(unix_ctime_ns(metadata)),
            #[cfg(not(unix))]
            unix_ctime_ns: None,
        }
    }

    fn has_strong_identity(&self) -> bool {
        self.unix_dev.is_some() && self.unix_ino.is_some() && self.unix_ctime_ns.is_some()
    }
}

#[cfg(unix)]
fn unix_dev(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;

    metadata.dev()
}

#[cfg(unix)]
fn unix_ino(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;

    metadata.ino()
}

#[cfg(unix)]
fn unix_ctime_ns(metadata: &std::fs::Metadata) -> i128 {
    use std::os::unix::fs::MetadataExt;

    i128::from(metadata.ctime()) * 1_000_000_000 + i128::from(metadata.ctime_nsec())
}
