use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

#[cfg(unix)]
use std::fs;

pub(crate) enum Listener {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix {
        listener: UnixListener,
        path: PathBuf,
    },
}

pub(crate) enum ClientStream {
    Tcp(TcpStream),
    #[cfg(unix)]
    Unix(UnixStream),
}

impl Listener {
    pub(crate) fn bind(listen: &str) -> Result<Self, String> {
        #[cfg(unix)]
        if let Some(path) = listen.strip_prefix("unix:") {
            let path = PathBuf::from(path);
            let _ = fs::remove_file(&path);
            let listener = UnixListener::bind(&path)
                .map_err(|error| format!("bind unix socket {}: {error}", path.display()))?;
            return Ok(Self::Unix { listener, path });
        }

        TcpListener::bind(listen)
            .map(Self::Tcp)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn addr(&self) -> String {
        match self {
            Self::Tcp(listener) => listener
                .local_addr()
                .map(|addr| addr.to_string())
                .unwrap_or_else(|_| "127.0.0.1:0".to_string()),
            #[cfg(unix)]
            Self::Unix { path, .. } => format!("unix:{}", path.display()),
        }
    }

    pub(crate) fn accept(&self) -> Result<Option<ClientStream>, String> {
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

impl Read for ClientStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            #[cfg(unix)]
            Self::Unix(stream) => stream.read(buf),
        }
    }
}
