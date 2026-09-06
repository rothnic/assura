//! Project fingerprinting for daemon freshness checks.

use crate::stable_hash::stable_hash;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectFingerprint {
    entries: BTreeMap<PathBuf, ProjectFingerprintEntry>,
}

impl ProjectFingerprint {
    pub(super) fn capture(root: &Path) -> Result<Self, std::io::Error> {
        let mut entries = BTreeMap::new();
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !ignored_fingerprint_entry(entry))
        {
            let entry = entry.map_err(std::io::Error::other)?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(std::io::Error::other)?;
            let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
            entries.insert(
                relative,
                ProjectFingerprintEntry::from_path(path, &metadata)?,
            );
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectFingerprintEntry {
    content_hash: Option<u64>,
    len: u64,
    is_dir: bool,
}

impl ProjectFingerprintEntry {
    fn from_path(path: &Path, metadata: &std::fs::Metadata) -> Result<Self, std::io::Error> {
        let content_hash = if metadata.is_file() {
            Some(stable_hash(&fs::read(path)?))
        } else {
            None
        };
        Ok(Self {
            content_hash,
            len: metadata.len(),
            is_dir: metadata.is_dir(),
        })
    }
}

fn ignored_fingerprint_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        ".git" | "target" | "node_modules" | "dist" | ".astro" | ".next"
    ) || is_daemon_runtime_path(entry.path())
}

fn is_daemon_runtime_path(path: &Path) -> bool {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();
    components
        .windows(2)
        .any(|window| window[0] == ".assura" && window[1] == "daemon")
}
