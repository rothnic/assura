//! Filesystem fingerprints used to prove cached check results are fresh.

use super::*;

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(super) struct DirectoryFingerprint {
    pub(super) rel: String,
    modified_ns: u128,
    child_count: usize,
    child_hash: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct FileFingerprint {
    modified_ns: u128,
    size: u64,
    content_hash: u64,
}

pub(super) fn collect_directory_snapshot(
    root: &Path,
    checked_path: &Path,
    exclude_patterns: &[CompiledExclusion],
) -> Result<Vec<DirectoryFingerprint>, CheckError> {
    if !checked_path.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    collect_directory_snapshot_inner(
        root,
        checked_path,
        Path::new(""),
        exclude_patterns,
        &mut dirs,
    )?;
    dirs.sort();
    Ok(dirs)
}

pub(super) fn collect_file_snapshot(path: &Path) -> Result<Option<FileFingerprint>, CheckError> {
    if !path.is_file() {
        return Ok(None);
    }
    let metadata = fs::metadata(path)?;
    let bytes = fs::read(path)?;
    Ok(Some(FileFingerprint {
        modified_ns: modified_ns(metadata.modified().ok()),
        size: metadata.len(),
        content_hash: stable_hash(&bytes),
    }))
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

pub(super) fn collect_child_fingerprint(
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
