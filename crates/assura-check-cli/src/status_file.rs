//! Binary status-file helpers shared by the hot daemon and tiny status client.

// Binary test harnesses compile this shared module per executable, so one side
// can appear unused even though another binary uses the same helpers.
// allow-reason: shared across companion binaries whose test harnesses compile
// only a subset of callers.
#![allow(dead_code)]

use assura_stable_hash::stable_hash_const;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAGIC: &[u8; 4] = b"AS2\0";
const STATUS_LEN: usize = 4 + 8 + 1 + 1;
const CLEAN: u8 = 0;
const DIRTY: u8 = 1;
const VERSION_HASH: u64 = stable_hash_const(env!("CARGO_PKG_VERSION").as_bytes());
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CheckStatus {
    pub(crate) exit_code: i32,
    pub(crate) dirty: bool,
}

pub(crate) fn write_status(path: &Path, status: CheckStatus) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut bytes = Vec::with_capacity(STATUS_LEN);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION_HASH.to_le_bytes());
    bytes.push(if status.dirty { DIRTY } else { CLEAN });
    bytes.push(status.exit_code.clamp(0, u8::MAX as i32) as u8);

    let temp_path = temp_status_path(path);
    fs::write(&temp_path, bytes)?;
    if let Err(error) = replace_status_file(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    Ok(())
}

pub(crate) fn read_status(path: &Path) -> io::Result<CheckStatus> {
    let mut file = fs::File::open(path)?;
    let mut bytes = [0_u8; STATUS_LEN];
    file.read_exact(&mut bytes)?;
    if &bytes[..4] != MAGIC {
        return Err(invalid_data("invalid Assura status file header"));
    }

    let version = u64::from_le_bytes(bytes[4..12].try_into().expect("version slice length"));
    if version != VERSION_HASH {
        return Err(invalid_data(
            "Assura status file was written by another version",
        ));
    }

    let dirty = match bytes[12] {
        CLEAN => false,
        DIRTY => true,
        _ => return Err(invalid_data("invalid Assura status dirty flag")),
    };
    let exit_code = i32::from(bytes[13]);

    Ok(CheckStatus { exit_code, dirty })
}

pub(crate) fn is_status_artifact(path: &Path, status_file: &Path) -> bool {
    if path == status_file || path.parent() != status_file.parent() {
        return path == status_file;
    }
    let Some(file_name) = path.file_name() else {
        return false;
    };
    file_name
        .to_string_lossy()
        .starts_with(&temp_status_prefix(status_file))
}

fn temp_status_path(path: &Path) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!("{}{sequence}", temp_status_prefix(path)))
}

fn temp_status_prefix(path: &Path) -> String {
    format!(
        ".{}.tmp-{}-",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    )
}

#[cfg(not(windows))]
fn replace_status_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn replace_status_file(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::{is_status_artifact, read_status, write_status, CheckStatus};

    #[test]
    fn status_file_round_trips_clean_and_dirty_status() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("assura.status");

        write_status(
            &path,
            CheckStatus {
                exit_code: 0,
                dirty: false,
            },
        )
        .unwrap();
        assert_eq!(
            read_status(&path).unwrap(),
            CheckStatus {
                exit_code: 0,
                dirty: false
            }
        );

        write_status(
            &path,
            CheckStatus {
                exit_code: 3,
                dirty: true,
            },
        )
        .unwrap();
        assert_eq!(
            read_status(&path).unwrap(),
            CheckStatus {
                exit_code: 3,
                dirty: true
            }
        );
    }

    #[test]
    fn status_artifact_identifies_binary_and_temp_paths() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("assura.status");

        assert!(is_status_artifact(&path, &path));
        assert!(is_status_artifact(&super::temp_status_path(&path), &path));
        assert!(!is_status_artifact(&temp.path().join("other"), &path));
    }
}
