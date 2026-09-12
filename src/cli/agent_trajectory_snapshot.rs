//! Atomic Git-local storage for one bounded trajectory snapshot per worktree.

use super::{git::GitInput, TrajectorySnapshot};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_CACHE_BYTES: usize = 1024 * 1024;

pub(super) struct CacheKey {
    path: PathBuf,
    key: String,
}

pub(super) struct CacheRead {
    pub(super) snapshot: Option<TrajectorySnapshot>,
    pub(super) source: &'static str,
}

impl CacheKey {
    pub(super) fn new(input: &GitInput, classification_version: &str) -> Self {
        let key = digest(&format!(
            "{classification_version}|{}|{}|{}|{}|{}|{}",
            input.repo_root.display(),
            input.common_dir.display(),
            input.git_dir.display(),
            input.requested_integration_ref,
            input.window.kind(),
            input.window.value()
        ));
        let path = input
            .common_dir
            .join("assura")
            .join("trajectory")
            .join(format!("{key}.json"));
        Self { path, key }
    }
}

pub(super) fn read(key: &CacheKey) -> CacheRead {
    let bytes = match read_bounded(&key.path) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            return CacheRead {
                snapshot: None,
                source: "oversized",
            }
        }
        Err(_) => {
            return CacheRead {
                snapshot: None,
                source: "miss",
            }
        }
    };
    let value = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(value) => value,
        Err(_) => {
            return CacheRead {
                snapshot: None,
                source: "corrupt",
            }
        }
    };
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(super::SNAPSHOT_SCHEMA) {
        return CacheRead {
            snapshot: None,
            source: "schema_mismatch",
        };
    }
    match serde_json::from_value::<TrajectorySnapshot>(value) {
        Ok(snapshot) => CacheRead {
            snapshot: Some(snapshot),
            source: "cache",
        },
        Err(_) => CacheRead {
            snapshot: None,
            source: "corrupt",
        },
    }
}

fn read_bounded(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take((MAX_CACHE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok((bytes.len() <= MAX_CACHE_BYTES).then_some(bytes))
}

pub(super) fn write(key: &CacheKey, snapshot: &TrajectorySnapshot) -> Result<(), String> {
    let directory = key
        .path
        .parent()
        .ok_or_else(|| "trajectory cache has no parent directory".to_string())?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let temporary = directory.join(format!(".{}.{}.tmp", key.key, std::process::id()));
    let bytes = serde_json::to_vec(snapshot).map_err(|error| error.to_string())?;
    let result = (|| {
        fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &key.path).map_err(|error| error.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
