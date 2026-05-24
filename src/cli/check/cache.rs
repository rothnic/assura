//! Opt-in hot result cache for repeated LS-Lint-compatible checks.

use super::{
    compiled_fingerprint::SourceConfigFingerprint, discover_project, rules::is_excluded_rel_with,
    rules::CompiledExclusion, CheckError, StructureCheckReport, StructureCheckTimings,
    StructureChecker,
};
use crate::config::loader::ConfigLoader;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_SCHEMA_VERSION: u32 = 4;

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
    dir_snapshot: Vec<DirectoryFingerprint>,
    report: StructureCheckReport,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct DirectoryFingerprint {
    rel: String,
    modified_ns: u128,
    child_count: usize,
    child_hash: u64,
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

    let cache_path = cache_file_path(
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

    let config = ConfigLoader::parse_validated(&config_content)?;
    let exclude_patterns = config.exclude.clone();
    let mut checker = StructureChecker::new(project_root.clone(), config, fail_fast);
    let mut timings = StructureCheckTimings::default();
    let report = checker.check(
        checked_path.clone(),
        discovered_config_path.clone(),
        &mut timings,
    )?;

    if !fail_fast && checked_path.is_dir() && checker.lslint_fast_scopes.is_some() {
        write_cache(
            &cache_path,
            CachedCheckReport {
                schema_version: CACHE_SCHEMA_VERSION,
                assura_version: env!("CARGO_PKG_VERSION").to_string(),
                config_hash,
                config_fingerprint: SourceConfigFingerprint::from_path(&discovered_config_path)
                    .ok(),
                project_root,
                config_path: discovered_config_path,
                checked_path,
                exclude_patterns,
                dir_snapshot: collect_directory_snapshot(
                    report.checked_path.as_path(),
                    &checker.exclude_patterns,
                )?,
                report: report.clone(),
            },
        );
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
    let current_snapshot = collect_directory_snapshot(checked_path, &exclude_patterns).ok()?;
    (current_snapshot == cached.dir_snapshot).then_some(cached.report.clone())
}

impl CachedCheckReport {
    fn config_is_fresh(&self, config_path: &Path, config_hash: Option<u64>) -> bool {
        self.config_fingerprint
            .as_ref()
            .is_some_and(|fingerprint| fingerprint.matches_path(config_path))
            || config_hash.is_some_and(|hash| self.config_hash == hash)
    }
}

fn write_cache(cache_path: &Path, cached: CachedCheckReport) {
    let Some(parent) = cache_path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    if let Ok(content) = serde_json::to_vec(&cached) {
        let _ = fs::write(cache_path, content);
    }
}

fn collect_directory_snapshot(
    root: &Path,
    exclude_patterns: &[CompiledExclusion],
) -> Result<Vec<DirectoryFingerprint>, CheckError> {
    let mut dirs = Vec::new();
    collect_directory_snapshot_inner(root, root, Path::new(""), exclude_patterns, &mut dirs)?;
    dirs.sort();
    Ok(dirs)
}

fn collect_directory_snapshot_inner(
    root: &Path,
    dir: &Path,
    dir_rel: &Path,
    exclude_patterns: &[CompiledExclusion],
    dirs: &mut Vec<DirectoryFingerprint>,
) -> Result<(), CheckError> {
    let metadata = fs::metadata(dir)?;
    let (child_count, child_hash, child_dirs) =
        collect_child_fingerprint(dir, dir_rel, exclude_patterns)?;
    dirs.push(DirectoryFingerprint {
        rel: rel_string(root, dir),
        modified_ns: modified_ns(metadata.modified().ok()),
        child_count,
        child_hash,
    });

    for (child_rel, child_dir) in child_dirs {
        collect_directory_snapshot_inner(root, &child_dir, &child_rel, exclude_patterns, dirs)?;
    }

    Ok(())
}

type ChildFingerprint = (usize, u64, Vec<(PathBuf, PathBuf)>);

fn collect_child_fingerprint(
    dir: &Path,
    dir_rel: &Path,
    exclude_patterns: &[CompiledExclusion],
) -> Result<ChildFingerprint, CheckError> {
    let mut children = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let child_rel = join_rel(dir_rel, &name);
        if is_excluded_rel_with(exclude_patterns, &child_rel) {
            continue;
        }

        let file_type = entry.file_type()?;
        let kind = child_kind(&file_type);
        children.push((
            name.to_string_lossy().into_owned(),
            kind,
            child_rel,
            file_type.is_dir(),
            entry.path(),
        ));
    }

    children.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

    let mut hasher = StableHasher::default();
    for (name, kind, _, _, _) in &children {
        hasher.write(&(name.len() as u64).to_le_bytes());
        hasher.write(name.as_bytes());
        hasher.write(&[*kind]);
    }
    let child_count = children.len();
    let child_hash = hasher.finish();
    let child_dirs = children
        .into_iter()
        .filter_map(|(_, _, child_rel, is_dir, path)| is_dir.then_some((child_rel, path)))
        .collect();
    Ok((child_count, child_hash, child_dirs))
}

fn join_rel(parent: &Path, name: &std::ffi::OsStr) -> PathBuf {
    if parent.as_os_str().is_empty() {
        PathBuf::from(name)
    } else {
        parent.join(name)
    }
}

fn child_kind(file_type: &fs::FileType) -> u8 {
    if file_type.is_file() {
        b'f'
    } else if file_type.is_dir() {
        b'd'
    } else if file_type.is_symlink() {
        b'l'
    } else {
        b'o'
    }
}

fn rel_string(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn modified_ns(modified: Option<SystemTime>) -> u128 {
    modified
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn cache_file_path(
    cache_dir: &Path,
    project_root: &Path,
    config_path: &Path,
    checked_path: &Path,
) -> PathBuf {
    let mut hasher = StableHasher::default();
    project_root.hash(&mut hasher);
    config_path.hash(&mut hasher);
    checked_path.hash(&mut hasher);
    cache_dir.join(format!("{:016x}.json", hasher.finish()))
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Default)]
struct StableHasher(u64);

impl Hasher for StableHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0 = stable_hash_with_seed(self.0, bytes);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

fn stable_hash_with_seed(seed: u64, bytes: &[u8]) -> u64 {
    let mut hash = if seed == 0 { 0xcbf29ce484222325 } else { seed };
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_report_accepts_matching_config_fingerprint_without_hash() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join(".assura")).unwrap();
        fs::write(temp.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
        fs::write(temp.path().join("valid-file.ts"), "").unwrap();
        let config_path = temp.path().join(".assura/config.yml");
        let checked_path = temp.path();
        let cached = CachedCheckReport {
            schema_version: CACHE_SCHEMA_VERSION,
            assura_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash: 123,
            config_fingerprint: SourceConfigFingerprint::from_path(&config_path).ok(),
            project_root: temp.path().to_path_buf(),
            config_path: config_path.clone(),
            checked_path: checked_path.to_path_buf(),
            exclude_patterns: Vec::new(),
            dir_snapshot: collect_directory_snapshot(checked_path, &[]).unwrap(),
            report: StructureCheckReport {
                success: true,
                project_root: temp.path().to_path_buf(),
                config_path: config_path.clone(),
                checked_path: checked_path.to_path_buf(),
                files_checked: 1,
                dirs_checked: 1,
                violations: Vec::new(),
            },
        };

        let report =
            fresh_cached_report(Some(&cached), None, temp.path(), &config_path, checked_path)
                .unwrap();

        assert!(report.success);
        assert_eq!(report.files_checked, 1);
    }

    #[test]
    fn cached_report_rejects_stale_config_without_hash() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join(".assura")).unwrap();
        fs::write(temp.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
        let config_path = temp.path().join(".assura/config.yml");
        let cached = CachedCheckReport {
            schema_version: CACHE_SCHEMA_VERSION,
            assura_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash: stable_hash(b"structure: {}\n"),
            config_fingerprint: SourceConfigFingerprint::from_path(&config_path).ok(),
            project_root: temp.path().to_path_buf(),
            config_path: config_path.clone(),
            checked_path: temp.path().to_path_buf(),
            exclude_patterns: Vec::new(),
            dir_snapshot: collect_directory_snapshot(temp.path(), &[]).unwrap(),
            report: StructureCheckReport {
                success: true,
                project_root: temp.path().to_path_buf(),
                config_path: config_path.clone(),
                checked_path: temp.path().to_path_buf(),
                files_checked: 0,
                dirs_checked: 1,
                violations: Vec::new(),
            },
        };

        fs::write(&config_path, "structure:\n  src/: {}\n").unwrap();

        assert!(
            fresh_cached_report(Some(&cached), None, temp.path(), &config_path, temp.path(),)
                .is_none()
        );
    }

    #[test]
    fn child_fingerprint_changes_when_child_name_changes() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("valid-file.ts"), "").unwrap();
        let (_, before_hash, _) =
            collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();

        fs::rename(
            temp.path().join("valid-file.ts"),
            temp.path().join("bad_name.ts"),
        )
        .unwrap();
        let (_, after_hash, _) =
            collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();

        assert_ne!(before_hash, after_hash);
    }

    #[test]
    fn child_fingerprint_changes_when_child_type_changes() {
        let temp = tempfile::tempdir().unwrap();
        let child = temp.path().join("child");
        fs::write(&child, "").unwrap();
        let (_, before_hash, _) =
            collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();

        fs::remove_file(&child).unwrap();
        fs::create_dir(&child).unwrap();
        let (_, after_hash, child_dirs) =
            collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();

        assert_ne!(before_hash, after_hash);
        assert_eq!(child_dirs, vec![(PathBuf::from("child"), child)]);
    }

    #[test]
    fn directory_snapshot_prunes_excluded_children() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("dist")).unwrap();
        fs::create_dir(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("dist").join("one.ts"), "").unwrap();
        fs::write(temp.path().join("src").join("one.ts"), "").unwrap();
        let exclude = vec![CompiledExclusion::new("dist/**")];

        let before = collect_directory_snapshot(temp.path(), &exclude).unwrap();
        fs::write(temp.path().join("dist").join("two.ts"), "").unwrap();
        let after = collect_directory_snapshot(temp.path(), &exclude).unwrap();

        assert_eq!(before, after);
        assert!(before.iter().all(|fingerprint| fingerprint.rel != "dist"));
    }
}
