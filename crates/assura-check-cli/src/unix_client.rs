//! Tiny Unix-socket client for hot `assura-checkd` project checks.

#![cfg_attr(all(unix, not(test)), no_main)]
#![cfg_attr(all(unix, not(debug_assertions), not(test)), no_std)]

#[cfg(all(unix, not(test)))]
use core::ffi::{c_char, c_int, c_void};
#[cfg(all(unix, not(debug_assertions), not(test)))]
use core::panic::PanicInfo;

#[cfg(not(unix))]
fn main() {
    eprintln!("assura-check-unix-client is only available on Unix platforms");
    std::process::exit(2);
}

#[cfg(all(unix, not(test)))]
const USAGE: &[u8] =
    b"Usage: assura-check-unix-client unix:<SOCKET> [PATH|--dirty-project-path PATH]\n";
#[cfg(all(unix, not(test)))]
const CONNECT_ERROR: &[u8] = b"Error: failed to connect to assura-checkd\n";
#[cfg(all(unix, not(test)))]
const IO_ERROR: &[u8] = b"Error: failed to communicate with assura-checkd\n";
#[cfg(all(unix, not(test)))]
const RESPONSE_ERROR: &[u8] = b"Error: invalid assura-checkd response\n";
#[cfg(all(unix, not(test)))]
const UNIX_PREFIX: &[u8] = b"unix:";
#[cfg(all(unix, not(test)))]
const DIRTY_PROJECT_PATH_FLAG: &[u8] = b"--dirty-project-path";
#[cfg(all(unix, not(test)))]
const AF_UNIX: c_int = 1;
#[cfg(all(unix, not(test)))]
const SOCK_STREAM: c_int = 1;
#[cfg(all(unix, not(test)))]
const STDERR_FILENO: c_int = 2;

#[cfg(all(target_vendor = "apple", unix, not(test)))]
#[repr(C)]
struct SockAddrUn {
    sun_len: u8,
    sun_family: u8,
    sun_path: [c_char; 104],
}

#[cfg(all(not(target_vendor = "apple"), unix, not(test)))]
#[repr(C)]
struct SockAddrUn {
    sun_family: u16,
    sun_path: [c_char; 108],
}

#[cfg(all(unix, not(test)))]
type SockLen = u32;

#[cfg(all(unix, not(test)))]
impl SockAddrUn {
    fn new() -> Self {
        Self {
            #[cfg(target_vendor = "apple")]
            sun_len: 0,
            #[cfg(target_vendor = "apple")]
            sun_family: AF_UNIX as u8,
            #[cfg(not(target_vendor = "apple"))]
            sun_family: AF_UNIX as u16,
            sun_path: [0; SOCKADDR_UN_PATH_LEN],
        }
    }

    fn path_len(&self) -> usize {
        self.sun_path.len()
    }

    fn set_path_byte(&mut self, index: usize, byte: c_char) {
        self.sun_path[index] = byte;
    }

    fn set_len(&mut self, len: SockLen) {
        #[cfg(target_vendor = "apple")]
        {
            self.sun_len = len as u8;
        }
        let _ = len;
    }
}

#[cfg(all(target_vendor = "apple", unix, not(test)))]
const SOCKADDR_UN_PATH_LEN: usize = 104;
#[cfg(all(not(target_vendor = "apple"), unix, not(test)))]
const SOCKADDR_UN_PATH_LEN: usize = 108;
#[cfg(all(target_vendor = "apple", unix, not(test)))]
const SOCKADDR_UN_HEADER_LEN: usize = 2;
#[cfg(all(not(target_vendor = "apple"), unix, not(test)))]
const SOCKADDR_UN_HEADER_LEN: usize = 2;

#[cfg(all(target_vendor = "apple", unix, not(test)))]
#[link(name = "System")]
extern "C" {
    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int;
    fn connect(fd: c_int, addr: *const c_void, len: SockLen) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    fn _exit(status: c_int) -> !;
}

#[cfg(all(not(target_vendor = "apple"), unix, not(test)))]
#[link(name = "c")]
extern "C" {
    fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int;
    fn connect(fd: c_int, addr: *const c_void, len: SockLen) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    fn _exit(status: c_int) -> !;
}

#[cfg(all(unix, not(test)))]
/// Raw Unix entrypoint for the hot daemon client.
///
/// # Safety
///
/// The platform C runtime must call this with the standard `argc`/`argv`
/// contract. `argv` must either be null when `argc` is invalid or point to at
/// least `argc` C-string pointers.
#[no_mangle]
pub unsafe extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    if !(argc == 2 || argc == 3 || argc == 4) || argv.is_null() {
        write_stderr(USAGE);
        exit_with(2);
    }

    let addr = *argv.add(1);
    if addr.is_null() || !starts_with(addr, UNIX_PREFIX) {
        write_stderr(USAGE);
        exit_with(2);
    }

    let socket_path = addr.add(UNIX_PREFIX.len());
    let request = if argc == 4 {
        let flag = *argv.add(2);
        let path = *argv.add(3);
        if flag.is_null() || path.is_null() || !equals(flag, DIRTY_PROJECT_PATH_FLAG) {
            write_stderr(USAGE);
            exit_with(2);
        }
        Request::DirtyProjectPath(path)
    } else if argc == 3 {
        let path = *argv.add(2);
        if path.is_null() {
            write_stderr(USAGE);
            exit_with(2);
        }
        Request::Path(path)
    } else {
        Request::Project
    };
    match request_check(socket_path, request) {
        Ok(code) => exit_with(code),
        Err(ClientError::Connect) => {
            write_stderr(CONNECT_ERROR);
            exit_with(3)
        }
        Err(ClientError::Io) => {
            write_stderr(IO_ERROR);
            exit_with(3)
        }
        Err(ClientError::Response) => {
            write_stderr(RESPONSE_ERROR);
            exit_with(3)
        }
    }
}

#[cfg(all(unix, not(test)))]
unsafe fn exit_with(code: c_int) -> ! {
    _exit(code);
}

#[cfg(all(unix, not(test)))]
enum ClientError {
    Connect,
    Io,
    Response,
}

#[cfg(all(unix, not(test)))]
enum Request {
    Project,
    Path(*const c_char),
    DirtyProjectPath(*const c_char),
}

#[cfg(all(unix, not(debug_assertions), not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    unsafe {
        _exit(3);
    }
}

#[cfg(all(unix, not(test)))]
unsafe fn request_check(
    socket_path: *const c_char,
    request: Request,
) -> Result<c_int, ClientError> {
    let fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if fd < 0 {
        return Err(ClientError::Connect);
    }

    let mut addr = SockAddrUn::new();
    let mut index = 0;
    while index + 1 < addr.path_len() {
        let byte = *socket_path.add(index);
        addr.set_path_byte(index, byte);
        if byte == 0 {
            break;
        }
        index += 1;
    }
    if index + 1 >= addr.path_len() {
        return Err(ClientError::Connect);
    }

    let len = (SOCKADDR_UN_HEADER_LEN + index + 1) as SockLen;
    addr.set_len(len);
    let connected = connect(fd, (&addr as *const SockAddrUn).cast(), len);
    if connected != 0 {
        return Err(ClientError::Connect);
    }

    match request {
        Request::Project => {
            write_all(fd, b"C\n")?;
        }
        Request::Path(path) => {
            write_all(fd, b"CHECK-PATH\t")?;
            write_c_string(fd, path)?;
            write_all(fd, b"\n")?;
        }
        Request::DirtyProjectPath(path) => {
            write_all(fd, b"D\t")?;
            write_c_string(fd, path)?;
            write_all(fd, b"\n")?;
        }
    }
    let mut response = [0_u8; 16];
    let read_len = read(fd, response.as_mut_ptr().cast(), response.len());
    if read_len <= 0 {
        return Err(ClientError::Io);
    }

    parse_response(&response[..read_len as usize]).ok_or(ClientError::Response)
}

#[cfg(all(unix, not(test)))]
unsafe fn write_all(fd: c_int, bytes: &[u8]) -> Result<(), ClientError> {
    let mut written = 0;
    while written < bytes.len() {
        let count = write(fd, bytes[written..].as_ptr().cast(), bytes.len() - written);
        if count <= 0 {
            return Err(ClientError::Io);
        }
        written += count as usize;
    }
    Ok(())
}

#[cfg(all(unix, not(test)))]
unsafe fn write_c_string(fd: c_int, value: *const c_char) -> Result<(), ClientError> {
    let mut start = value;
    let mut len = 0_usize;
    loop {
        let byte = *start.add(len);
        if byte == 0 {
            if len > 0 {
                write_all(fd, core::slice::from_raw_parts(start.cast(), len))?;
            }
            return Ok(());
        }
        len += 1;
        if len == 256 {
            write_all(fd, core::slice::from_raw_parts(start.cast(), len))?;
            start = start.add(len);
            len = 0;
        }
    }
}

#[cfg(all(unix, not(test)))]
fn parse_response(response: &[u8]) -> Option<c_int> {
    if response.len() == 1 && response[0].is_ascii_digit() {
        return Some(i32::from(response[0] - b'0'));
    }

    if response.len() < 5 || &response[..3] != b"OK " {
        return None;
    }

    let mut code = 0_i32;
    let mut saw_digit = false;
    for byte in &response[3..] {
        match byte {
            b'0'..=b'9' => {
                saw_digit = true;
                code = code
                    .saturating_mul(10)
                    .saturating_add(i32::from(byte - b'0'));
            }
            b'\n' | b'\r' | b' ' | b'\t' => break,
            _ => return None,
        }
    }

    saw_digit.then_some(code)
}

#[cfg(all(unix, not(test)))]
unsafe fn starts_with(value: *const c_char, prefix: &[u8]) -> bool {
    for (index, expected) in prefix.iter().enumerate() {
        if *value.add(index) as u8 != *expected {
            return false;
        }
    }
    true
}

#[cfg(all(unix, not(test)))]
unsafe fn equals(value: *const c_char, expected: &[u8]) -> bool {
    for (index, byte) in expected.iter().enumerate() {
        if *value.add(index) as u8 != *byte {
            return false;
        }
    }
    *value.add(expected.len()) == 0
}

#[cfg(all(unix, not(test)))]
unsafe fn write_stderr(message: &[u8]) {
    let _ = write(STDERR_FILENO, message.as_ptr().cast(), message.len());
}
