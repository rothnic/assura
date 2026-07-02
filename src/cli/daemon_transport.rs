//! Local daemon socket transport helpers.

#[cfg(unix)]
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::path::PathBuf;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(750);
const READ_TIMEOUT: Duration = Duration::from_millis(2_000);

pub(super) enum Listener {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix {
        listener: UnixListener,
        path: PathBuf,
    },
}

impl Listener {
    pub(super) fn bind(listen_addr: &str) -> Result<Self, String> {
        #[cfg(unix)]
        if let Some(path) = listen_addr.strip_prefix("unix:") {
            let path = PathBuf::from(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "create daemon socket directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
            let _ = fs::remove_file(&path);
            let listener = UnixListener::bind(&path)
                .map_err(|error| format!("bind unix socket {}: {error}", path.display()))?;
            return Ok(Self::Unix { listener, path });
        }

        TcpListener::bind(listen_addr)
            .map(Self::Tcp)
            .map_err(|error| format!("bind tcp socket {listen_addr}: {error}"))
    }

    pub(super) fn addr(&self) -> String {
        match self {
            Self::Tcp(listener) => listener
                .local_addr()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|_| "127.0.0.1:0".to_string()),
            #[cfg(unix)]
            Self::Unix { path, .. } => format!("unix:{}", path.display()),
        }
    }

    pub(super) fn accept(&self) -> Result<Option<ClientStream>, String> {
        match self {
            Self::Tcp(listener) => listener
                .accept()
                .map(|(stream, _)| Some(ClientStream::Tcp(stream)))
                .map_err(|error| error.to_string()),
            #[cfg(unix)]
            Self::Unix { listener, .. } => listener
                .accept()
                .map(|(stream, _)| Some(ClientStream::Unix(stream)))
                .map_err(|error| error.to_string()),
        }
    }
}

pub(super) enum ClientStream {
    Tcp(TcpStream),
    #[cfg(unix)]
    Unix(UnixStream),
}

impl ClientStream {
    pub(super) fn connect(listen_addr: &str) -> Result<Self, String> {
        #[cfg(unix)]
        if let Some(path) = listen_addr.strip_prefix("unix:") {
            let stream = UnixStream::connect(path)
                .map_err(|error| format!("connect unix socket {path}: {error}"))?;
            stream
                .set_read_timeout(Some(READ_TIMEOUT))
                .map_err(|error| error.to_string())?;
            stream
                .set_write_timeout(Some(READ_TIMEOUT))
                .map_err(|error| error.to_string())?;
            return Ok(Self::Unix(stream));
        }

        let addr = listen_addr
            .parse()
            .map_err(|error| format!("parse tcp daemon address {listen_addr}: {error}"))?;
        let stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
            .map_err(|error| format!("connect tcp socket {listen_addr}: {error}"))?;
        stream
            .set_read_timeout(Some(READ_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(READ_TIMEOUT))
            .map_err(|error| error.to_string())?;
        Ok(Self::Tcp(stream))
    }

    pub(super) fn read_line(&mut self) -> Result<String, String> {
        let mut response = Vec::new();
        let mut byte = [0_u8; 1];
        loop {
            let read = self.read(&mut byte).map_err(|error| error.to_string())?;
            if read == 0 || byte[0] == b'\n' {
                break;
            }
            response.push(byte[0]);
            if response.len() > 8 * 1024 * 1024 {
                return Err("daemon response exceeded maximum size".to_string());
            }
        }
        String::from_utf8(response)
            .map_err(|error| format!("daemon response was not UTF-8: {error}"))
    }
}

impl Read for ClientStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            #[cfg(unix)]
            Self::Unix(stream) => stream.read(buf),
        }
    }
}

impl Write for ClientStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buf),
            #[cfg(unix)]
            Self::Unix(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            #[cfg(unix)]
            Self::Unix(stream) => stream.flush(),
        }
    }
}
