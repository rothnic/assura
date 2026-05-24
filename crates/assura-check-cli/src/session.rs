//! Persistent CLI session client for hot `assura-checkd` validation.

use lexopt::prelude::*;
use std::io::{BufRead, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::process;

struct Options {
    addr: String,
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("Error: {error}");
            print_usage();
            process::exit(2);
        }
    };

    if let Err(error) = run_session(&options.addr) {
        eprintln!("Error: {error}");
        process::exit(3);
    }
}

fn parse_options() -> Result<Options, lexopt::Error> {
    let mut addr = None;
    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("addr") => addr = Some(parser.value()?.string()?),
            Short('h') | Long("help") => {
                print_usage();
                process::exit(0);
            }
            Value(value) if addr.is_none() => addr = Some(value.string()?),
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Options {
        addr: addr.ok_or("missing daemon address")?,
    })
}

fn print_usage() {
    eprintln!("Usage: assura-check-session [--addr] <ADDR>");
    eprintln!("Commands on stdin: CHECK, PATH<TAB><PATH>, DIRTY-PROJECT-PATH<TAB><PATH>, QUIT");
}

fn run_session(addr: &str) -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    let mut daemon = connect_daemon_session(addr)?;
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("read session command: {error}"))?;
        let command = line.trim_end_matches('\r');
        if command == "QUIT" {
            daemon
                .write_all(b"QUIT\n")
                .map_err(|error| format!("write daemon quit: {error}"))?;
            return Ok(());
        }
        let request = match daemon_request(command) {
            Some(request) => request,
            None => {
                stdout
                    .write_all(b"ERR invalid command\n")
                    .map_err(|error| format!("write session response: {error}"))?;
                stdout
                    .flush()
                    .map_err(|error| format!("flush session response: {error}"))?;
                continue;
            }
        };
        let exit_code = request_over_stream(&mut *daemon, &request)?;
        writeln!(stdout, "OK {exit_code}")
            .map_err(|error| format!("write session response: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("flush session response: {error}"))?;
    }
    Ok(())
}

fn connect_daemon_session(addr: &str) -> Result<Box<dyn ReadWrite>, String> {
    let mut stream: Box<dyn ReadWrite> = {
        #[cfg(unix)]
        if let Some(socket_path) = addr.strip_prefix("unix:") {
            Box::new(
                UnixStream::connect(socket_path)
                    .map_err(|error| format!("connect unix socket: {error}"))?,
            )
        } else {
            Box::new(
                TcpStream::connect(addr).map_err(|error| format!("connect tcp socket: {error}"))?,
            )
        }

        #[cfg(not(unix))]
        {
            Box::new(
                TcpStream::connect(addr).map_err(|error| format!("connect tcp socket: {error}"))?,
            )
        }
    };
    stream
        .write_all(b"SESSION\n")
        .map_err(|error| format!("write session handshake: {error}"))?;
    let status = request_over_stream(&mut *stream, b"")?;
    if status != 0 {
        return Err(format!("daemon rejected session handshake with {status}"));
    }
    Ok(stream)
}

fn daemon_request(command: &str) -> Option<Vec<u8>> {
    if command == "CHECK" || command == "C" {
        return Some(b"C\n".to_vec());
    }
    if let Some(path) = command.strip_prefix("PATH\t") {
        if path.is_empty() {
            return None;
        }
        return Some(format!("CHECK-PATH\t{path}\n").into_bytes());
    }
    if let Some(path) = command
        .strip_prefix("DIRTY-PROJECT-PATH\t")
        .or_else(|| command.strip_prefix("D\t"))
    {
        if path.is_empty() {
            return None;
        }
        return Some(format!("D\t{path}\n").into_bytes());
    }
    None
}

fn request_over_stream(stream: &mut dyn ReadWrite, request: &[u8]) -> Result<i32, String> {
    if !request.is_empty() {
        stream
            .write_all(request)
            .map_err(|error| format!("write daemon request: {error}"))?;
    }
    let mut response = [0_u8; 32];
    let len = stream
        .read(&mut response)
        .map_err(|error| format!("read daemon response: {error}"))?;
    parse_hot_response(&response[..len])
}

fn parse_hot_response(response: &[u8]) -> Result<i32, String> {
    if response.len() == 1 && response[0].is_ascii_digit() {
        return Ok(i32::from(response[0] - b'0'));
    }

    let text = std::str::from_utf8(response).map_err(|_| "invalid UTF-8 response".to_string())?;
    let mut parts = text.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("OK"), Some(code)) => code
            .parse::<i32>()
            .map_err(|error| format!("invalid daemon status code: {error}")),
        _ => Err(format!("invalid daemon response: {}", text.trim())),
    }
}

trait ReadWrite: std::io::Read + Write {}

impl<T: std::io::Read + Write> ReadWrite for T {}
