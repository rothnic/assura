//! Context loading and freshness state shared by the JSON-line sessions.

use super::context::{ContentQueryError, QueryContext};
use crate::stable_hash::stable_hash;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub(super) struct SessionContext {
    path: PathBuf,
    config: Option<PathBuf>,
    context: QueryContext,
    fingerprint: ProjectFingerprint,
    first_response: bool,
}

impl SessionContext {
    pub(super) fn load(path: PathBuf, config: Option<PathBuf>) -> Result<Self, ContentQueryError> {
        let context = QueryContext::load_for_path(path.clone(), config.clone(), false, true, true)?;
        let fingerprint = ProjectFingerprint::capture(&context.project_root)?;
        Ok(Self {
            path,
            config,
            context,
            fingerprint,
            first_response: true,
        })
    }

    pub(super) fn context(&self) -> &QueryContext {
        &self.context
    }

    pub(super) fn ensure_fresh(&mut self) -> Result<SessionReloadOutput, ContentQueryError> {
        let latest = ProjectFingerprint::capture(&self.context.project_root)?;
        let first = std::mem::take(&mut self.first_response);
        if latest == self.fingerprint {
            return Ok(if first {
                SessionReloadOutput::initial(&self.context)
            } else {
                SessionReloadOutput::reused(&self.context)
            });
        }

        let context =
            QueryContext::load_for_path(self.path.clone(), self.config.clone(), false, true, true)?;
        let fingerprint = ProjectFingerprint::capture(&context.project_root)?;
        self.context = context;
        self.fingerprint = fingerprint;
        Ok(SessionReloadOutput::reloaded(&self.context))
    }
}

#[derive(Debug, Serialize)]
pub(super) struct SessionReloadOutput {
    pub(super) state: &'static str,
    pub(super) reason: &'static str,
    pub(super) project_root: Option<PathBuf>,
    pub(super) config_path: Option<PathBuf>,
}

impl SessionReloadOutput {
    fn initial(context: &QueryContext) -> Self {
        Self::from_context("initial_load", "session context loaded", context)
    }

    fn reused(context: &QueryContext) -> Self {
        Self::from_context("reused", "project fingerprint unchanged", context)
    }

    fn reloaded(context: &QueryContext) -> Self {
        Self::from_context(
            "reloaded",
            "project fingerprint changed; context rebuilt",
            context,
        )
    }

    pub(super) fn failed() -> Self {
        Self {
            state: "reload_failed",
            reason: "project fingerprint changed but context could not be rebuilt",
            project_root: None,
            config_path: None,
        }
    }

    pub(super) fn not_checked() -> Self {
        Self {
            state: "not_checked",
            reason: "request did not parse",
            project_root: None,
            config_path: None,
        }
    }

    fn from_context(state: &'static str, reason: &'static str, context: &QueryContext) -> Self {
        Self {
            state,
            reason,
            project_root: Some(context.project_root.clone()),
            config_path: Some(context.config_path.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectFingerprint {
    entries: BTreeMap<PathBuf, FileFingerprint>,
}

impl ProjectFingerprint {
    fn capture(root: &Path) -> Result<Self, ContentQueryError> {
        let mut entries = BTreeMap::new();
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !is_ignored_fingerprint_entry(entry))
        {
            let entry = entry.map_err(|error| {
                ContentQueryError::runtime(format!("failed to scan project fingerprint: {error}"))
            })?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(|error| {
                ContentQueryError::runtime(format!(
                    "failed to read metadata for {}: {error}",
                    path.display()
                ))
            })?;
            let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
            let fingerprint = FileFingerprint::from_path(path, &metadata)?;
            entries.insert(relative, fingerprint);
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileFingerprint {
    content_hash: Option<u64>,
    len: u64,
    is_dir: bool,
}

impl FileFingerprint {
    fn from_path(path: &Path, metadata: &std::fs::Metadata) -> Result<Self, ContentQueryError> {
        let content_hash = if metadata.is_file() {
            let content = fs::read(path).map_err(|error| {
                ContentQueryError::runtime(format!(
                    "failed to read fingerprint content for {}: {error}",
                    path.display()
                ))
            })?;
            Some(stable_hash(&content))
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

fn is_ignored_fingerprint_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        ".git" | "target" | "node_modules" | "dist" | ".astro" | ".next"
    )
}
