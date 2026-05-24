//! Tiny CLI that exits from an `assura-checkd` binary status file.

#![cfg_attr(all(unix, not(test)), no_main)]
#![cfg_attr(all(unix, not(debug_assertions), not(test)), no_std)]

#[cfg(all(unix, not(test)))]
use core::ffi::{c_char, c_int, c_void};
#[cfg(all(unix, not(debug_assertions), not(test)))]
use core::panic::PanicInfo;

#[cfg(not(unix))]
mod status_file;

#[cfg(not(unix))]
use std::env;
#[cfg(not(unix))]
use std::path::PathBuf;
#[cfg(not(unix))]
use std::process;

#[cfg(not(unix))]
fn main() {
    let mut args = env::args_os().skip(1);
    let status_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assura-check.status"));

    if args.next().is_some() {
        eprintln!("Usage: assura-check-status [STATUS_FILE]");
        process::exit(2);
    }

    match status_file::read_status(&status_path) {
        Ok(status) if !status.dirty => process::exit(status.exit_code),
        Ok(_) => {
            eprintln!("Error: Assura status is dirty; ask assura-checkd to refresh validation");
            process::exit(3);
        }
        Err(error) => {
            eprintln!("Error: failed to read {}: {error}", status_path.display());
            process::exit(3);
        }
    }
}

#[cfg(all(unix, not(test)))]
const MAGIC: &[u8; 4] = b"AS2\0";
#[cfg(all(unix, not(test)))]
const STATUS_LEN: usize = 4 + 8 + 1 + 1;
#[cfg(all(unix, not(test)))]
const DEFAULT_STATUS_PATH: &[u8] = b"assura-check.status\0";
#[cfg(all(unix, not(test)))]
const VERSION_HASH: u64 = stable_hash(env!("CARGO_PKG_VERSION").as_bytes());
#[cfg(all(unix, not(test)))]
const CLEAN: u8 = 0;
#[cfg(all(unix, not(test)))]
const DIRTY: u8 = 1;
#[cfg(all(unix, not(test)))]
const USAGE: &[u8] = b"Usage: assura-check-status [STATUS_FILE]\n";
#[cfg(all(unix, not(test)))]
const DIRTY_ERROR: &[u8] =
    b"Error: Assura status is dirty; ask assura-checkd to refresh validation\n";
#[cfg(all(unix, not(test)))]
const READ_ERROR: &[u8] = b"Error: failed to read Assura status file\n";
#[cfg(all(unix, not(test)))]
const INVALID_ERROR: &[u8] = b"Error: invalid Assura status file\n";
#[cfg(all(unix, not(test)))]
const O_RDONLY: c_int = 0;
#[cfg(all(unix, not(test)))]
const STDERR_FILENO: c_int = 2;

#[cfg(all(target_vendor = "apple", unix, not(test)))]
#[link(name = "System")]
extern "C" {
    fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    fn _exit(status: c_int) -> !;
}

#[cfg(all(not(target_vendor = "apple"), unix, not(test)))]
#[link(name = "c")]
extern "C" {
    fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    fn _exit(status: c_int) -> !;
}

#[cfg(all(unix, not(test)))]
/// Raw Unix entrypoint for the status-file client.
///
/// # Safety
///
/// The platform C runtime must call this with the standard `argc`/`argv`
/// contract. `argv` must either be null when `argc` is invalid or point to at
/// least `argc` C-string pointers.
#[no_mangle]
pub unsafe extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    if !(argc == 1 || argc == 2) || argv.is_null() {
        write_stderr(USAGE);
        exit_with(2);
    }

    let status_path = if argc == 1 {
        DEFAULT_STATUS_PATH.as_ptr().cast()
    } else {
        *argv.add(1)
    };
    if status_path.is_null() {
        write_stderr(USAGE);
        exit_with(2);
    }

    match read_status(status_path) {
        Ok(Status {
            exit_code,
            dirty: false,
        }) => exit_with(exit_code),
        Ok(Status { dirty: true, .. }) => {
            write_stderr(DIRTY_ERROR);
            exit_with(3)
        }
        Err(ReadStatusError::Read) => {
            write_stderr(READ_ERROR);
            exit_with(3)
        }
        Err(ReadStatusError::Invalid) => {
            write_stderr(INVALID_ERROR);
            exit_with(3)
        }
    }
}

#[cfg(all(unix, not(test)))]
unsafe fn exit_with(code: c_int) -> ! {
    _exit(code);
}

#[cfg(all(unix, not(test)))]
struct Status {
    exit_code: c_int,
    dirty: bool,
}

#[cfg(all(unix, not(test)))]
enum ReadStatusError {
    Read,
    Invalid,
}

#[cfg(all(unix, not(debug_assertions), not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    unsafe {
        _exit(3);
    }
}

#[cfg(all(unix, not(test)))]
unsafe fn read_status(path: *const c_char) -> Result<Status, ReadStatusError> {
    let fd = open(path, O_RDONLY);
    if fd < 0 {
        return Err(ReadStatusError::Read);
    }

    let mut bytes = [0_u8; STATUS_LEN];
    let mut offset = 0;
    while offset < STATUS_LEN {
        let read_len = read(fd, bytes[offset..].as_mut_ptr().cast(), STATUS_LEN - offset);
        if read_len <= 0 {
            return Err(ReadStatusError::Read);
        }
        offset += read_len as usize;
    }

    if &bytes[..4] != MAGIC {
        return Err(ReadStatusError::Invalid);
    }

    let version = u64::from_le_bytes([
        bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11],
    ]);
    if version != VERSION_HASH {
        return Err(ReadStatusError::Invalid);
    }

    let dirty = match bytes[12] {
        CLEAN => false,
        DIRTY => true,
        _ => return Err(ReadStatusError::Invalid),
    };
    let exit_code = i32::from(bytes[13]);

    Ok(Status { exit_code, dirty })
}

#[cfg(all(unix, not(test)))]
unsafe fn write_stderr(message: &[u8]) {
    let _ = write(STDERR_FILENO, message.as_ptr().cast(), message.len());
}

#[cfg(all(unix, not(test)))]
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
