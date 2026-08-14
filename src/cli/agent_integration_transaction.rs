//! Rollback support for multi-file agent integration lifecycle mutations.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

struct FileSnapshot {
    path: PathBuf,
    contents: Option<Vec<u8>>,
    permissions: Option<fs::Permissions>,
}

pub(super) fn run<T>(
    paths: impl IntoIterator<Item = PathBuf>,
    project_root: &Path,
    dry_run: bool,
    operation: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    if dry_run {
        return operation();
    }

    let snapshots = capture(paths, project_root)?;
    match operation() {
        Ok(value) => Ok(value),
        Err(error) => match restore(snapshots, project_root) {
            Ok(()) => Err(error),
            Err(rollback_error) => Err(format!("{error}; rollback also failed: {rollback_error}")),
        },
    }
}

fn capture(
    paths: impl IntoIterator<Item = PathBuf>,
    project_root: &Path,
) -> Result<Vec<FileSnapshot>, String> {
    let paths = paths.into_iter().collect::<BTreeSet<_>>();
    paths
        .into_iter()
        .map(|path| {
            validate_managed_path(&path, project_root)?;
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_file() => Some(metadata),
                Ok(_) => {
                    return Err(format!(
                        "refusing lifecycle mutation because {} is not a regular file",
                        path.display()
                    ))
                }
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                Err(error) => return Err(error.to_string()),
            };
            let contents = metadata
                .as_ref()
                .map(|_| fs::read(&path).map_err(|error| error.to_string()))
                .transpose()?;
            let permissions = metadata.map(|metadata| metadata.permissions());
            Ok(FileSnapshot {
                path,
                contents,
                permissions,
            })
        })
        .collect()
}

fn validate_managed_path(path: &Path, project_root: &Path) -> Result<(), String> {
    let relative = path.strip_prefix(project_root).map_err(|_| {
        format!(
            "refusing lifecycle mutation outside project root: {}",
            path.display()
        )
    })?;
    let mut cursor = project_root.to_path_buf();
    for component in relative.components() {
        cursor.push(component);
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing lifecycle mutation through symbolic link: {}",
                    cursor.display()
                ))
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn restore(snapshots: Vec<FileSnapshot>, project_root: &Path) -> Result<(), String> {
    let mut failures = Vec::new();
    for snapshot in snapshots.into_iter().rev() {
        let result = match snapshot.contents {
            Some(contents) => restore_file(&snapshot.path, &contents, snapshot.permissions),
            None => remove_created_file(&snapshot.path, project_root),
        };
        if let Err(error) = result {
            failures.push(format!("{}: {error}", snapshot.path.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn restore_file(
    path: &Path,
    contents: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, contents).map_err(|error| error.to_string())?;
    if let Some(permissions) = permissions {
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn remove_created_file(path: &Path, project_root: &Path) -> Result<(), String> {
    if path.is_file() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    let mut cursor = path.parent();
    while let Some(directory) = cursor {
        if directory == project_root || !directory.starts_with(project_root) {
            break;
        }
        if fs::remove_dir(directory).is_err() {
            break;
        }
        cursor = directory.parent();
    }
    Ok(())
}
