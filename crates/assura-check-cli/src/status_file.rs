//! Binary status-file helpers shared by the hot daemon and tiny status client.

// Binary test harnesses compile this shared module per executable, so one side
// can appear unused even though another binary uses the same helpers.
#![allow(dead_code)]

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"AS2\0";
const STATUS_LEN: usize = 4 + 8 + 1 + 1;
const CLEAN: u8 = 0;
const DIRTY: u8 = 1;
const VERSION_HASH: u64 = stable_hash(env!("CARGO_PKG_VERSION").as_bytes());

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

    let temp_path = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temp_path, bytes)?;
    fs::rename(temp_path, path)
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
    path == status_file || path == temp_status_path(status_file)
}

fn temp_status_path(path: &Path) -> PathBuf {
    path.with_extension(format!("tmp-{}", std::process::id()))
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

const fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }
    hash
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
