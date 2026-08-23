use super::*;

/// Observable state for one check-cache root.
#[derive(Debug, Serialize)]
#[cfg_attr(not(feature = "full-cli"), allow(dead_code))]
pub struct CheckCacheStatus {
    /// Stable diagnostic schema.
    pub schema: &'static str,
    /// Cache root inspected by this command.
    pub cache_root: String,
    /// Current worktree-local namespace.
    pub worktree_namespace: String,
    /// Shared immutable namespace, when Git can prove one.
    pub shared_namespace: Option<String>,
    /// Why shared immutable reuse is unavailable.
    pub fallback_reason: Option<String>,
    /// Number of cache records under the root.
    pub entries: usize,
    /// Total serialized cache bytes under the root.
    pub bytes: u64,
}

/// Resolve the default check-cache root for a project or worktree path.
pub fn default_check_cache_dir(path: &Path) -> PathBuf {
    let base = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let git_base = if base.is_file() {
        base.parent().unwrap_or(&base)
    } else {
        &base
    };
    git_value(git_base, &["rev-parse", "--git-common-dir"])
        .map(PathBuf::from)
        .map(|git_dir| {
            if git_dir.is_absolute() {
                git_dir
            } else {
                git_base.join(git_dir)
            }
        })
        .map(|git_dir| git_dir.join("assura/check-cache"))
        .unwrap_or_else(|| git_base.join(".assura/cache/check"))
}

/// Inspect cache namespaces and serialized size without loading reports.
#[cfg_attr(not(feature = "full-cli"), allow(dead_code))]
pub fn inspect_check_cache(path: &Path, cache_dir: Option<&Path>) -> CheckCacheStatus {
    let cache_root = cache_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_check_cache_dir(path));
    let worktree_key = digest_path(path);
    let shared_key = shared_namespace(path);
    let (entries, bytes) = cache_size(&cache_root);
    CheckCacheStatus {
        schema: "assura.check-cache-status.v1",
        cache_root: normalized_path(&cache_root),
        worktree_namespace: format!("worktrees/{worktree_key}"),
        shared_namespace: shared_key.as_ref().map(|key| format!("shared/{key}")),
        fallback_reason: shared_key
            .is_none()
            .then(|| "Git HEAD/common-dir or a clean worktree is unavailable".to_string()),
        entries,
        bytes,
    }
}

/// Remove one complete check-cache root and return the state that existed.
#[cfg_attr(not(feature = "full-cli"), allow(dead_code))]
pub fn clean_check_cache(
    path: &Path,
    cache_dir: Option<&Path>,
) -> std::io::Result<CheckCacheStatus> {
    let status = inspect_check_cache(path, cache_dir);
    let cache_root = PathBuf::from(&status.cache_root);
    if !cache_root.exists() {
        return Ok(status);
    }
    if dangerous_cache_root(path, &cache_root) || !valid_cache_root(&cache_root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to remove unrecognized or unsafe cache root {}",
                cache_root.display()
            ),
        ));
    }
    fs::remove_dir_all(cache_root)?;
    Ok(status)
}

#[cfg_attr(not(feature = "full-cli"), allow(dead_code))]
fn cache_size(root: &Path) -> (usize, u64) {
    let mut entries = 0;
    let mut bytes = 0;
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() && entry.file_name() != CACHE_ROOT_MARKER {
            entries += 1;
            bytes += entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        }
    }
    (entries, bytes)
}

pub(super) fn cache_root_for_entry(path: &Path) -> Option<&Path> {
    let mut current = path;
    while let Some(parent) = current.parent() {
        if matches!(
            current.file_name().and_then(|name| name.to_str()),
            Some("worktrees" | "shared")
        ) {
            return Some(parent);
        }
        current = parent;
    }
    None
}

pub(super) fn ensure_cache_root(root: &Path) -> bool {
    if root.exists() {
        if valid_cache_root(root) {
            return true;
        }
        if !directory_is_empty(root) {
            return false;
        }
    } else if fs::create_dir_all(root).is_err() {
        return false;
    }
    set_private_permissions(root, true);
    let marker = root.join(CACHE_ROOT_MARKER);
    if marker.exists() {
        return valid_cache_root(root);
    }
    let body = format!("{{\"schema\":\"{CACHE_ROOT_SCHEMA}\"}}\n");
    if fs::write(&marker, body).is_err() {
        return false;
    }
    set_private_permissions(&marker, false);
    true
}

fn directory_is_empty(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .and_then(|mut entries| entries.next().transpose().ok())
        .flatten()
        .is_none()
}

fn valid_cache_root(root: &Path) -> bool {
    fs::read_to_string(root.join(CACHE_ROOT_MARKER))
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
        .and_then(|value| {
            value
                .get("schema")
                .and_then(|schema| schema.as_str())
                .map(str::to_string)
        })
        .as_deref()
        == Some(CACHE_ROOT_SCHEMA)
}

fn dangerous_cache_root(project: &Path, root: &Path) -> bool {
    let project = project
        .canonicalize()
        .unwrap_or_else(|_| project.to_path_buf());
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if project.starts_with(&root) {
        return true;
    }
    for argument in ["--git-dir", "--git-common-dir"] {
        if let Some(value) = git_value(&project, &["rev-parse", argument]) {
            let metadata = PathBuf::from(value);
            let metadata = if metadata.is_absolute() {
                metadata
            } else {
                project.join(metadata)
            };
            let metadata = metadata.canonicalize().unwrap_or(metadata);
            if metadata.starts_with(&root) {
                return true;
            }
        }
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .and_then(|home| home.canonicalize().ok())
        .is_some_and(|home| root == home)
        || root.parent().is_none()
}

#[cfg_attr(not(feature = "full-cli"), allow(dead_code))]
fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(unix)]
pub(super) fn set_private_permissions(path: &Path, directory: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if directory { 0o700 } else { 0o600 };
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
pub(super) fn set_private_permissions(_path: &Path, _directory: bool) {}
