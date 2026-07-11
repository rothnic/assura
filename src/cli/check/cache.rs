//! Opt-in hot result cache for repeated LS-Lint-compatible checks.

use super::{
    compiled_fingerprint::SourceConfigFingerprint, discover_project, rules::is_excluded_rel_with,
    rules::CompiledExclusion, CheckError, StructureCheckReport, StructureCheckTimings,
    StructureChecker,
};
use crate::config::loader::ConfigLoader;
use crate::stable_hash::{stable_hash, StableHasher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "cache/snapshot.rs"]
mod snapshot;
#[path = "cache/status.rs"]
mod status;
use snapshot::{
    collect_directory_snapshot, collect_file_snapshot, DirectoryFingerprint, FileFingerprint,
};
use status::{cache_root_for_entry, ensure_cache_root, set_private_permissions};
pub use status::{
    clean_check_cache, default_check_cache_dir, inspect_check_cache, CheckCacheStatus,
};

const CACHE_SCHEMA_VERSION: u32 = 5;
const CACHE_ROOT_SCHEMA: &str = "assura.check-cache-root.v1";
const CACHE_ROOT_MARKER: &str = ".assura-cache-root.json";

#[derive(Debug, Deserialize, Serialize)]
struct CachedCheckReport {
    schema_version: u32,
    assura_version: String,
    config_hash: u64,
    #[serde(default)]
    config_fingerprint: Option<SourceConfigFingerprint>,
    project_root: PathBuf,
    config_path: PathBuf,
    checked_path: PathBuf,
    exclude_patterns: Vec<String>,
    #[serde(default)]
    dir_snapshot: Vec<DirectoryFingerprint>,
    #[serde(default)]
    file_snapshot: Option<FileFingerprint>,
    report: StructureCheckReport,
}

/// Run structure validation with an opt-in hot cache for unchanged naming-only trees.
pub fn run_structure_check_cached(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_fast: bool,
    cache_dir: PathBuf,
) -> Result<StructureCheckReport, CheckError> {
    let requested_path = match path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };

    if !requested_path.exists() {
        return Err(CheckError::MissingPath(requested_path));
    }

    let checked_path = requested_path.canonicalize()?;
    let (project_root, discovered_config_path) = discover_project(&checked_path, config_path)?;
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let cache_path = worktree_cache_file_path(
        &cache_dir,
        &project_root,
        &discovered_config_path,
        &checked_path,
    );

    let cached = (!fail_fast).then(|| read_cache(&cache_path)).flatten();
    if !fail_fast {
        if let Some(report) = fresh_cached_report(
            cached.as_ref(),
            None,
            &project_root,
            &discovered_config_path,
            &checked_path,
        ) {
            return Ok(report);
        }
    }

    let config_content = fs::read_to_string(&discovered_config_path).map_err(CheckError::Io)?;
    let config_hash = stable_hash(config_content.as_bytes());
    if !fail_fast {
        if let Some(report) = fresh_cached_report(
            cached.as_ref(),
            Some(config_hash),
            &project_root,
            &discovered_config_path,
            &checked_path,
        ) {
            return Ok(report);
        }
    }
    let shared_path = (!fail_fast)
        .then(|| shared_cache_file_path(&cache_dir, &project_root, &checked_path, config_hash))
        .flatten();
    if let Some(path) = shared_path.as_ref() {
        if let Some(cached) = read_cache(path) {
            if let Some(report) = fresh_shared_report(
                &cached,
                config_hash,
                &project_root,
                &discovered_config_path,
                &checked_path,
            ) {
                if shared_cache_file_path(&cache_dir, &project_root, &checked_path, config_hash)
                    .as_ref()
                    == Some(path)
                {
                    return Ok(report);
                }
            }
        }
    }

    let config = ConfigLoader::parse_validated(&config_content)?;
    let exclude_patterns = config.exclude.clone();
    let mut checker = StructureChecker::new(project_root.clone(), config, fail_fast);
    let before_dir_snapshot = collect_directory_snapshot(
        checked_path.as_path(),
        checked_path.as_path(),
        &checker.exclude_patterns,
    )?;
    let before_file_snapshot = collect_file_snapshot(checked_path.as_path())?;
    let mut timings = StructureCheckTimings::default();
    let report = checker.check(
        checked_path.clone(),
        discovered_config_path.clone(),
        super::CheckTargetMode::Recursive,
        &mut timings,
    )?;

    if !fail_fast && checker.lslint_fast_scopes.is_some() {
        let dir_snapshot = collect_directory_snapshot(
            report.checked_path.as_path(),
            report.checked_path.as_path(),
            &checker.exclude_patterns,
        )?;
        let file_snapshot = collect_file_snapshot(report.checked_path.as_path())?;
        if dir_snapshot != before_dir_snapshot || file_snapshot != before_file_snapshot {
            return Ok(report);
        }
        let cached_report = CachedCheckReport {
            schema_version: CACHE_SCHEMA_VERSION,
            assura_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash,
            config_fingerprint: SourceConfigFingerprint::from_path(&discovered_config_path).ok(),
            project_root,
            config_path: discovered_config_path,
            checked_path,
            exclude_patterns,
            dir_snapshot,
            file_snapshot,
            report: report.clone(),
        };
        write_cache(&cache_path, &cached_report);
        if let Some(path) = shared_cache_file_path(
            &cache_dir,
            &cached_report.project_root,
            &cached_report.checked_path,
            config_hash,
        ) {
            write_cache(&path, &cached_report);
        }
    }

    Ok(report)
}

fn read_cache(cache_path: &Path) -> Option<CachedCheckReport> {
    let content = fs::read(cache_path).ok()?;
    serde_json::from_slice(&content).ok()
}

fn fresh_cached_report(
    cached: Option<&CachedCheckReport>,
    config_hash: Option<u64>,
    project_root: &Path,
    config_path: &Path,
    checked_path: &Path,
) -> Option<StructureCheckReport> {
    let cached = cached?;
    if cached.schema_version != CACHE_SCHEMA_VERSION
        || cached.assura_version != env!("CARGO_PKG_VERSION")
        || cached.project_root != project_root
        || cached.config_path != config_path
        || cached.checked_path != checked_path
    {
        return None;
    }
    if !cached.config_is_fresh(config_path, config_hash) {
        return None;
    }

    let exclude_patterns = cached
        .exclude_patterns
        .iter()
        .map(|pattern| CompiledExclusion::new(pattern))
        .collect::<Vec<_>>();
    let fresh = if checked_path.is_file() {
        collect_file_snapshot(checked_path).ok()? == cached.file_snapshot
    } else {
        collect_directory_snapshot(checked_path, checked_path, &exclude_patterns).ok()?
            == cached.dir_snapshot
    };
    fresh.then_some(cached.report.clone())
}

impl CachedCheckReport {
    fn config_is_fresh(&self, config_path: &Path, config_hash: Option<u64>) -> bool {
        if self
            .config_fingerprint
            .as_ref()
            .is_some_and(|fingerprint| fingerprint.differs_from_path(config_path))
        {
            return false;
        }

        config_hash.is_some_and(|hash| self.config_hash == hash)
    }
}

fn write_cache(cache_path: &Path, cached: &CachedCheckReport) {
    let Some(parent) = cache_path.parent() else {
        return;
    };
    let Some(cache_root) = cache_root_for_entry(cache_path) else {
        return;
    };
    if !ensure_cache_root(cache_root) {
        return;
    }
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    set_private_permissions(parent, true);
    if let Ok(content) = serde_json::to_vec(cached) {
        let temporary = cache_path.with_extension(format!("tmp-{}", std::process::id()));
        if fs::write(&temporary, content).is_ok() {
            set_private_permissions(&temporary, false);
            #[cfg(windows)]
            if cache_path.exists() {
                let _ = fs::remove_file(cache_path);
            }
            let _ = fs::rename(temporary, cache_path);
        }
    }
}

fn worktree_cache_file_path(
    cache_dir: &Path,
    project_root: &Path,
    config_path: &Path,
    checked_path: &Path,
) -> PathBuf {
    let mut hasher = StableHasher::default();
    project_root.hash(&mut hasher);
    config_path.hash(&mut hasher);
    checked_path.hash(&mut hasher);
    cache_dir
        .join("worktrees")
        .join(digest_path(project_root))
        .join(format!("{:016x}.json", hasher.finish()))
}

fn shared_cache_file_path(
    cache_dir: &Path,
    project_root: &Path,
    checked_path: &Path,
    config_hash: u64,
) -> Option<PathBuf> {
    let namespace = shared_namespace(project_root)?;
    let relative = checked_path.strip_prefix(project_root).ok()?;
    let mut hasher = StableHasher::default();
    relative.hash(&mut hasher);
    config_hash.hash(&mut hasher);
    Some(
        cache_dir
            .join("shared")
            .join(namespace)
            .join(format!("{:016x}.json", hasher.finish())),
    )
}

fn shared_namespace(project_root: &Path) -> Option<String> {
    if !git_value(
        project_root,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignored=matching",
            "--",
            ".",
        ],
    )?
    .is_empty()
    {
        return None;
    }
    let common = PathBuf::from(git_value(project_root, &["rev-parse", "--git-common-dir"])?);
    let common = if common.is_absolute() {
        common
    } else {
        project_root.join(common)
    };
    let head = git_value(project_root, &["rev-parse", "HEAD"])?;
    Some(format!("{}-{head}", digest_path(&common)))
}

fn fresh_shared_report(
    cached: &CachedCheckReport,
    config_hash: u64,
    project_root: &Path,
    config_path: &Path,
    checked_path: &Path,
) -> Option<StructureCheckReport> {
    if cached.schema_version != CACHE_SCHEMA_VERSION
        || cached.assura_version != env!("CARGO_PKG_VERSION")
        || cached.config_hash != config_hash
    {
        return None;
    }
    let mut report = cached.report.clone();
    report.project_root = project_root.to_path_buf();
    report.config_path = config_path.to_path_buf();
    report.checked_path = checked_path.to_path_buf();
    for violation in &mut report.violations {
        if let Ok(relative) = violation.path.strip_prefix(&cached.project_root) {
            violation.path = project_root.join(relative);
        }
    }
    Some(report)
}

fn git_value(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn digest_path(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    format!(
        "{:016x}",
        stable_hash(canonical.to_string_lossy().as_bytes())
    )
}

#[cfg(test)]
#[path = "cache/tests.rs"]
mod tests;
