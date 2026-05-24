//! Tiny client for the hot `assura-checkd` validation process.

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process;

enum Request {
    Project,
    Path(PathBuf),
    DirtyProjectPath(PathBuf),
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(addr) = args.next() else {
        print_usage();
        process::exit(2);
    };
    let request = match parse_request(args.collect()) {
        Some(request) => request,
        None => {
            print_usage();
            process::exit(2);
        }
    };

    let exit_code = match check(&addr, &request) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Error: {error}");
            3
        }
    };
    process::exit(exit_code);
}

fn parse_request(args: Vec<String>) -> Option<Request> {
    match args.as_slice() {
        [] => Some(Request::Project),
        [path] => Some(Request::Path(PathBuf::from(path))),
        [flag, path] if flag == "--dirty-project-path" => {
            Some(Request::DirtyProjectPath(PathBuf::from(path)))
        }
        _ => None,
    }
}

fn print_usage() {
    eprintln!("Usage: assura-check-client <ADDR> [PATH]");
    eprintln!("       assura-check-client <ADDR> --dirty-project-path <PATH>");
}

fn check(addr: &str, request: &Request) -> Result<i32, String> {
    #[cfg(unix)]
    if let Some(socket_path) = addr.strip_prefix("unix:") {
        let mut stream = UnixStream::connect(socket_path).map_err(|error| error.to_string())?;
        return request_check(&mut stream, request);
    }

    let mut stream = TcpStream::connect(addr).map_err(|error| error.to_string())?;
    request_check(&mut stream, request)
}

fn request_check(stream: &mut impl ReadWrite, request: &Request) -> Result<i32, String> {
    match request {
        Request::Path(path) => stream
            .write_all(format!("CHECK-PATH\t{}\n", path.display()).as_bytes())
            .map_err(|error| error.to_string())?,
        Request::DirtyProjectPath(path) => stream
            .write_all(format!("CHECK-DIRTY-PROJECT-PATH\t{}\n", path.display()).as_bytes())
            .map_err(|error| error.to_string())?,
        Request::Project => stream
            .write_all(b"CHECK\n")
            .map_err(|error| error.to_string())?,
    }

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| error.to_string())?;

    let mut parts = response.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("OK"), Some(code)) => code
            .parse::<i32>()
            .map_err(|error| format!("invalid daemon status code: {error}")),
        _ => Err(format!("invalid daemon response: {}", response.trim())),
    }
}

trait ReadWrite: Read + Write {}

impl<T: Read + Write> ReadWrite for T {}
