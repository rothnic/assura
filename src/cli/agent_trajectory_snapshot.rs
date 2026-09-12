//! Atomic Git-local storage for one bounded trajectory snapshot per worktree.

use super::{git::GitInput, TrajectorySnapshot};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_CACHE_BYTES: usize = 1024 * 1024;
const MAX_CACHE_FILES: usize = 128;

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

pub(super) fn read_latest_for_worktree(common_dir: &Path, worktree_root: &Path) -> CacheRead {
    let directory = common_dir.join("assura").join("trajectory");
    let mut newest = None;
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => {
            return CacheRead {
                snapshot: None,
                source: "miss",
            }
        }
    };
    for entry in entries.take(MAX_CACHE_FILES).flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Some(bytes) = read_bounded(&path).ok().flatten() else {
            continue;
        };
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        if value.get("schema").and_then(serde_json::Value::as_str) != Some(super::SNAPSHOT_SCHEMA) {
            continue;
        }
        let Ok(snapshot) = serde_json::from_value::<TrajectorySnapshot>(value) else {
            continue;
        };
        if snapshot.worktree.root != worktree_root.to_string_lossy() {
            continue;
        }
        if newest
            .as_ref()
            .is_none_or(|current: &TrajectorySnapshot| current.captured_at < snapshot.captured_at)
        {
            newest = Some(snapshot);
        }
    }
    CacheRead {
        snapshot: newest,
        source: "cache_scan",
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
        crate::cli::replace_file(&temporary, &key.path).map_err(|error| error.to_string())
    })();
    if result.is_ok() {
        prune_cache(directory);
    }
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn prune_cache(directory: &Path) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut snapshots = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return None;
            }
            entry
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(|modified| (modified, path))
        })
        .collect::<Vec<_>>();
    let remove_count = snapshots.len().saturating_sub(MAX_CACHE_FILES);
    if remove_count == 0 {
        return;
    }
    snapshots.sort_by_key(|(modified, _)| *modified);
    for (_, path) in snapshots.into_iter().take(remove_count) {
        let _ = fs::remove_file(path);
    }
}

fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cache_retention_is_bounded() {
        let directory = tempdir().expect("cache directory");
        for index in 0..=MAX_CACHE_FILES {
            fs::write(directory.path().join(format!("{index}.json")), b"{}").expect("cache file");
        }

        prune_cache(directory.path());

        let count = fs::read_dir(directory.path())
            .expect("cache entries")
            .flatten()
            .filter(|entry| {
                entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            })
            .count();
        assert_eq!(count, MAX_CACHE_FILES);
    }
}
