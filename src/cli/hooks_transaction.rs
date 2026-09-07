//! Transactional two-file replacement for managed Git hook pairs.

use super::{HookError, HookResult};
use std::fs::{self, OpenOptions, Permissions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct FileSnapshot {
    contents: Option<Vec<u8>>,
    permissions: Option<Permissions>,
}

pub(super) fn replace_pair(
    sidecar_path: &Path,
    sidecar_content: &[u8],
    wrapper_path: &Path,
    wrapper_content: &[u8],
) -> HookResult<()> {
    replace_pair_with(
        sidecar_path,
        sidecar_content,
        wrapper_path,
        wrapper_content,
        atomic_replace,
    )
}

fn replace_pair_with<F>(
    sidecar_path: &Path,
    sidecar_content: &[u8],
    wrapper_path: &Path,
    wrapper_content: &[u8],
    mut replace: F,
) -> HookResult<()>
where
    F: FnMut(&Path, &[u8], Option<Permissions>) -> HookResult<()>,
{
    let sidecar_snapshot = capture_regular_file(sidecar_path)?;
    let _wrapper_snapshot = capture_regular_file(wrapper_path)?;

    replace(sidecar_path, sidecar_content, None)?;
    if let Err(operation_error) = replace(wrapper_path, wrapper_content, None) {
        return match restore_with(sidecar_path, sidecar_snapshot, &mut replace) {
            Ok(()) => Err(operation_error),
            Err(rollback_error) => Err(HookError::RollbackFailed {
                operation: operation_error.to_string(),
                rollback: rollback_error.to_string(),
            }),
        };
    }

    Ok(())
}

fn capture_regular_file(path: &Path) -> HookResult<FileSnapshot> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Some(metadata),
        Ok(_) => return Err(HookError::UnsafePath(path.to_path_buf())),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let contents = metadata.as_ref().map(|_| fs::read(path)).transpose()?;
    let permissions = metadata.map(|metadata| metadata.permissions());
    Ok(FileSnapshot {
        contents,
        permissions,
    })
}

fn restore_with<F>(path: &Path, snapshot: FileSnapshot, replace: &mut F) -> HookResult<()>
where
    F: FnMut(&Path, &[u8], Option<Permissions>) -> HookResult<()>,
{
    match snapshot.contents {
        Some(contents) => replace(path, &contents, snapshot.permissions),
        None => match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_file() => {
                fs::remove_file(path)?;
                Ok(())
            }
            Ok(_) => Err(HookError::UnsafePath(path.to_path_buf())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        },
    }
}

fn atomic_replace(path: &Path, content: &[u8], permissions: Option<Permissions>) -> HookResult<()> {
    validate_regular_or_absent(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| HookError::UnsafePath(path.to_path_buf()))?;
    let parent_metadata = fs::symlink_metadata(parent)?;
    if !parent_metadata.file_type().is_dir() {
        return Err(HookError::UnsafePath(parent.to_path_buf()));
    }

    let temporary = write_temporary(parent, path, content, permissions)?;
    if let Err(error) = validate_regular_or_absent(path)
        .and_then(|_| fs::rename(&temporary, path).map_err(HookError::from))
    {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

fn validate_regular_or_absent(path: &Path) -> HookResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(HookError::UnsafePath(path.to_path_buf())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn write_temporary(
    parent: &Path,
    destination: &Path,
    content: &[u8],
    permissions: Option<Permissions>,
) -> HookResult<PathBuf> {
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("hook");
    for _ in 0..128 {
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{file_name}.{}.{}.assura-tmp",
            std::process::id(),
            sequence
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        };
        if let Err(error) = file
            .write_all(content)
            .and_then(|_| file.sync_all())
            .and_then(|_| set_permissions(&file, permissions))
        {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        return Ok(temporary);
    }
    Err(HookError::Io(std::io::Error::new(
        ErrorKind::AlreadyExists,
        format!(
            "failed to allocate a temporary hook beside {}",
            destination.display()
        ),
    )))
}

fn set_permissions(file: &fs::File, permissions: Option<Permissions>) -> std::io::Result<()> {
    if let Some(permissions) = permissions {
        return file.set_permissions(permissions);
    }
    set_executable(file)
}

#[cfg(unix)]
fn set_executable(file: &fs::File) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn set_executable(_file: &fs::File) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_failure_restores_sidecar_and_leaves_wrapper_unchanged() {
        let directory = tempfile::TempDir::new().unwrap();
        let sidecar = directory.path().join("assura-hook");
        let wrapper = directory.path().join("git-hook");
        let original_sidecar = b"original sidecar bytes\n";
        let original_wrapper = b"original wrapper bytes\n";
        let new_wrapper = b"new wrapper bytes\n";
        fs::write(&sidecar, original_sidecar).unwrap();
        fs::write(&wrapper, original_wrapper).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&sidecar, Permissions::from_mode(0o600)).unwrap();
        }

        let result = replace_pair_with(
            &sidecar,
            b"new sidecar bytes\n",
            &wrapper,
            new_wrapper,
            |path, content, permissions| {
                if path == wrapper && content == new_wrapper {
                    return Err(HookError::Io(std::io::Error::other(
                        "injected wrapper publication failure",
                    )));
                }
                atomic_replace(path, content, permissions)
            },
        );

        assert!(matches!(
            result,
            Err(HookError::Io(ref error))
                if error.to_string() == "injected wrapper publication failure"
        ));
        assert_eq!(fs::read(&sidecar).unwrap(), original_sidecar);
        assert_eq!(fs::read(&wrapper).unwrap(), original_wrapper);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&sidecar).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
