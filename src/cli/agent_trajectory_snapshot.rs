//! Atomic Git-local storage for one bounded trajectory snapshot per worktree.

use super::{git::GitInput, TrajectorySnapshot};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_CACHE_BYTES: usize = 1024 * 1024;
const MAX_CACHE_FILES: usize = 128;
const MAX_POINTER_BYTES: usize = 4096;
const CACHE_LOCK_SECONDS: i64 = 30;
const POINTER_SCHEMA: &str = "assura.agent-trajectory-pointer.v1";

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
    let pointer = latest_pointer_path(&directory, worktree_root);
    let Some(bytes) = read_bounded_with_limit(&pointer, MAX_POINTER_BYTES)
        .ok()
        .flatten()
    else {
        return CacheRead {
            snapshot: None,
            source: "miss",
        };
    };
    let Ok(pointer) = serde_json::from_slice::<LatestPointer>(&bytes) else {
        return CacheRead {
            snapshot: None,
            source: "pointer_corrupt",
        };
    };
    if pointer.schema != POINTER_SCHEMA
        || pointer.worktree_root != worktree_root.to_string_lossy()
        || !valid_cache_key(&pointer.key)
    {
        return CacheRead {
            snapshot: None,
            source: "pointer_invalid",
        };
    }
    let key = CacheKey {
        path: directory.join(format!("{}.json", pointer.key)),
        key: pointer.key,
    };
    let result = read(&key);
    if result.snapshot.is_some() {
        CacheRead {
            snapshot: result.snapshot,
            source: "cache_pointer",
        }
    } else {
        result
    }
}

fn read_bounded(path: &Path) -> Result<Option<Vec<u8>>, String> {
    read_bounded_with_limit(path, MAX_CACHE_BYTES)
}

fn read_bounded_with_limit(path: &Path, limit: usize) -> Result<Option<Vec<u8>>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64)
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
    let Some(_lease) = CacheLease::acquire(&key.path) else {
        return Err("trajectory cache write in flight".to_string());
    };
    if let Some(current) = read(key).snapshot.filter(|current| {
        current.generation == snapshot.generation && current.captured_at >= snapshot.captured_at
    }) {
        return update_latest_pointer(directory, &current, &key.key);
    }
    let temporary = directory.join(format!(".{}.{}.tmp", key.key, std::process::id()));
    let bytes = serde_json::to_vec(snapshot).map_err(|error| error.to_string())?;
    let result = (|| {
        fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
        crate::cli::replace_file(&temporary, &key.path).map_err(|error| error.to_string())
    })();
    if result.is_ok() {
        prune_cache(directory);
        update_latest_pointer(directory, snapshot, &key.key)?;
    }
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(super) fn acquire_refresh(key: &CacheKey) -> Option<CacheLease> {
    CacheLease::acquire_path(key.path.with_extension("refresh.lock"))
}

#[derive(Debug, Deserialize, Serialize)]
struct LatestPointer {
    schema: String,
    worktree_root: String,
    captured_at: i64,
    key: String,
}

pub(super) struct CacheLease {
    path: PathBuf,
    token: String,
}

impl CacheLease {
    fn acquire(cache: &Path) -> Option<Self> {
        Self::acquire_path(cache.with_extension("lock"))
    }

    fn acquire_path(path: PathBuf) -> Option<Self> {
        fs::create_dir_all(path.parent()?).ok()?;
        if let Ok(metadata) = fs::metadata(&path) {
            let stale = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|value| {
                    crate::cli::agent_nudge_delivery::now_millis()
                        .saturating_sub(value.as_secs() as i64)
                        > CACHE_LOCK_SECONDS
                })
                .unwrap_or(false);
            if stale {
                let _ = fs::remove_file(&path);
            }
        }
        let token = format!(
            "{}:{}",
            std::process::id(),
            crate::cli::agent_nudge_delivery::now_millis()
        );
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()?;
        if fs::write(&path, &token).is_err() {
            let _ = fs::remove_file(&path);
            return None;
        }
        Some(Self { path, token })
    }
}

impl Drop for CacheLease {
    fn drop(&mut self) {
        if fs::read_to_string(&self.path).ok().as_deref() == Some(self.token.as_str()) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn latest_pointer_path(directory: &Path, worktree_root: &Path) -> PathBuf {
    directory.join(format!(
        ".latest-{}.pointer",
        digest(&worktree_root.to_string_lossy())
    ))
}

fn update_latest_pointer(
    directory: &Path,
    snapshot: &TrajectorySnapshot,
    key: &str,
) -> Result<(), String> {
    let path = latest_pointer_path(directory, Path::new(&snapshot.worktree.root));
    let Some(_lease) = CacheLease::acquire(&path) else {
        return Ok(());
    };
    if read_bounded_with_limit(&path, MAX_POINTER_BYTES)
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<LatestPointer>(&bytes).ok())
        .is_some_and(|current| current.captured_at > snapshot.captured_at)
    {
        return Ok(());
    }
    let pointer = LatestPointer {
        schema: POINTER_SCHEMA.to_string(),
        worktree_root: snapshot.worktree.root.clone(),
        captured_at: snapshot.captured_at,
        key: key.to_string(),
    };
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(
        &temporary,
        serde_json::to_vec(&pointer).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    crate::cli::replace_file(&temporary, &path).map_err(|error| error.to_string())
}

fn valid_cache_key(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
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
