mod server_dirty;
mod server_io;
mod status_file;

use crate::server_dirty::{dirty_project_paths, DirtyProject, DirtyState};
use crate::server_io::{ClientStream, Listener};
use assura::cli::{CheckError, PreparedStructureCheck};
use lexopt::prelude::*;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{mpsc, Arc};

struct Options {
    listen: String,
    root: PathBuf,
    config: Option<PathBuf>,
    status_file: Option<PathBuf>,
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!(
                "Usage: assura-checkd --listen <ADDR> --root <PATH> [--config <PATH>] [--status-file <PATH>]"
            );
            process::exit(2);
        }
    };

    if let Err(error) = serve(options) {
        eprintln!("Error: {error}");
        process::exit(3);
    }
}

fn parse_options() -> Result<Options, lexopt::Error> {
    let mut listen = None;
    let mut root = None;
    let mut config = None;
    let mut status_file = None;
    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("listen") => listen = Some(PathBuf::from(parser.value()?).display().to_string()),
            Long("root") => root = Some(PathBuf::from(parser.value()?)),
            Long("config") => config = Some(PathBuf::from(parser.value()?)),
            Long("status-file") => status_file = Some(PathBuf::from(parser.value()?)),
            Short('h') | Long("help") => {
                println!(
                    "Usage: assura-checkd --listen <ADDR> --root <PATH> [--config <PATH>] [--status-file <PATH>]"
                );
                process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Options {
        listen: listen.unwrap_or_else(|| "127.0.0.1:0".to_string()),
        root: root.ok_or("missing --root")?,
        config,
        status_file,
    })
}

fn serve(options: Options) -> Result<(), String> {
    let root = options
        .root
        .canonicalize()
        .map_err(|error| format!("canonicalize root: {error}"))?;
    let config = match options.config {
        Some(config) => Some(
            config
                .canonicalize()
                .map_err(|error| format!("canonicalize config: {error}"))?,
        ),
        None => None,
    };

    let listener = Listener::bind(&options.listen)?;

    let mut prepared_check =
        PreparedStructureCheck::load_for_path(Some(root.clone()), config, false)
            .map_err(|error| error.to_string())?;
    let dirty = Arc::new(DirtyState::new());
    let _watcher = match watch_root(
        &root,
        prepared_check.config_path(),
        Arc::clone(&dirty),
        options.status_file.clone(),
    ) {
        Ok(watcher) => Some(watcher),
        Err(error) => {
            eprintln!("Warning: file watcher disabled: {error}");
            None
        }
    };

    let mut cached_exit = None;
    if let Some(status_file) = &options.status_file {
        let exit_code = run_check(&mut prepared_check, &root, false);
        dirty.mark_clean_after_initial_check();
        write_status(status_file, exit_code, false);
        cached_exit = Some(exit_code);
    }

    println!("{}", listener.addr());

    while let Some(stream) = listener.accept()? {
        let ClientRequestWithStream {
            request,
            mut stream,
        } = read_client_request(stream)?;
        if matches!(request, ClientRequest::Session) {
            write_client_response(&mut stream, 0, false)?;
            stream.flush().map_err(|error| error.to_string())?;
            serve_client_session(
                stream,
                &mut prepared_check,
                &root,
                &dirty,
                options.status_file.as_deref(),
                &mut cached_exit,
            )?;
            continue;
        }

        let (exit_code, compact) = handle_client_request(
            request,
            &mut prepared_check,
            &root,
            &dirty,
            options.status_file.as_deref(),
            &mut cached_exit,
        );
        write_client_response(&mut stream, exit_code, compact)?;
    }
    Ok(())
}

fn serve_client_session(
    mut stream: ClientStream,
    prepared_check: &mut PreparedStructureCheck,
    root: &Path,
    dirty: &DirtyState,
    status_file: Option<&Path>,
    cached_exit: &mut Option<i32>,
) -> Result<(), String> {
    while let Some(request) = read_client_request_from_stream(&mut stream)? {
        if matches!(request, ClientRequest::Quit) {
            return Ok(());
        }
        let (exit_code, compact) = handle_client_request(
            request,
            prepared_check,
            root,
            dirty,
            status_file,
            cached_exit,
        );
        write_client_response(&mut stream, exit_code, compact)?;
        stream.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn handle_client_request(
    request: ClientRequest,
    prepared_check: &mut PreparedStructureCheck,
    root: &Path,
    dirty: &DirtyState,
    status_file: Option<&Path>,
    cached_exit: &mut Option<i32>,
) -> (i32, bool) {
    let request_compact_response = request.compact_response();
    let config_changed_by_fingerprint = match prepared_check.reload_if_config_changed() {
        Ok(changed) => changed,
        Err(error) => {
            let exit_code = exit_code_for_check_error(&error);
            if let Some(status_file) = status_file {
                write_status(status_file, exit_code, true);
            }
            *cached_exit = Some(exit_code);
            return (exit_code, request_compact_response);
        }
    };
    if config_changed_by_fingerprint {
        *cached_exit = None;
        if let Some(status_file) = status_file {
            write_status(status_file, 3, true);
        }
    }

    match request {
        ClientRequest::Project { compact_response } => {
            let dirty_state = dirty.take();
            let config_changed = dirty_state.config_changed || config_changed_by_fingerprint;
            if config_changed
                || matches!(dirty_state.project, DirtyProject::Full)
                || cached_exit.is_none()
            {
                let exit_code = run_check(prepared_check, root, dirty_state.config_changed);
                if let Some(status_file) = status_file {
                    write_status(status_file, exit_code, false);
                }
                *cached_exit = Some(exit_code);
            } else if let DirtyProject::Paths(paths) = dirty_state.project {
                let exit_code = if *cached_exit == Some(0) {
                    run_incremental_project_check(prepared_check, root, paths)
                } else {
                    run_check(prepared_check, root, false)
                };
                if let Some(status_file) = status_file {
                    write_status(status_file, exit_code, false);
                }
                *cached_exit = Some(exit_code);
            }
            (cached_exit.unwrap_or(3), compact_response)
        }
        ClientRequest::Path(path) => {
            let exit_code = run_path_check(prepared_check, root, path, dirty.config_changed());
            (exit_code, false)
        }
        ClientRequest::DirtyProjectPath {
            path,
            compact_response,
        } => {
            let dirty_state = dirty.take();
            let config_changed = dirty_state.config_changed || config_changed_by_fingerprint;
            let exit_code = match (config_changed, dirty_state.project) {
                (true, _) | (_, DirtyProject::Full) => {
                    run_check(prepared_check, root, dirty_state.config_changed)
                }
                (_, _) if *cached_exit != Some(0) => run_check(prepared_check, root, false),
                (false, DirtyProject::Clean) => run_path_check(prepared_check, root, path, false),
                (false, project) => run_incremental_project_check(
                    prepared_check,
                    root,
                    dirty_project_paths(project, path).expect("full project handled above"),
                ),
            };
            if let Some(status_file) = status_file {
                write_status(status_file, exit_code, false);
            }
            *cached_exit = Some(exit_code);
            (exit_code, compact_response)
        }
        ClientRequest::Session | ClientRequest::Quit => (2, false),
    }
}

fn watch_root(
    root: &Path,
    config_path: &Path,
    dirty: Arc<DirtyState>,
    status_file: Option<PathBuf>,
) -> Result<RecommendedWatcher, String> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|error| format!("create watcher: {error}"))?;
    watcher
        .watch(root, RecursiveMode::Recursive)
        .map_err(|error| format!("watch root: {error}"))?;

    let config_path = config_path.to_path_buf();
    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            if let Ok(event) = event {
                if status_file
                    .as_deref()
                    .is_some_and(|path| event_touches_status_file(&event, path))
                {
                    continue;
                }
                dirty.record_event(&event, &config_path);
                if let Some(status_file) = &status_file {
                    write_status(status_file, 3, true);
                }
            }
        }
    });

    Ok(watcher)
}

fn event_touches_status_file(event: &notify::Event, status_file: &Path) -> bool {
    event
        .paths
        .iter()
        .any(|path| status_file::is_status_artifact(path, status_file))
}

fn write_status(status_file: &Path, exit_code: i32, dirty: bool) {
    let _ = status_file::write_status(status_file, status_file::CheckStatus { exit_code, dirty });
}

fn run_check(prepared_check: &mut PreparedStructureCheck, root: &Path, reload_config: bool) -> i32 {
    if reload_config {
        if let Err(error) = prepared_check.reload_if_config_changed() {
            return exit_code_for_check_error(&error);
        }
    }

    match prepared_check.check_path(root.to_path_buf()) {
        Ok(report) if report.success => 0,
        Ok(_) => 1,
        Err(error) => exit_code_for_check_error(&error),
    }
}

fn run_path_check(
    prepared_check: &mut PreparedStructureCheck,
    root: &Path,
    path: PathBuf,
    reload_config: bool,
) -> i32 {
    if reload_config {
        if let Err(error) = prepared_check.reload_if_config_changed() {
            return exit_code_for_check_error(&error);
        }
    }

    let checked_path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    match prepared_check.check_changed_path(checked_path) {
        Ok(report) if report.success => 0,
        Ok(_) => 1,
        Err(error) => exit_code_for_check_error(&error),
    }
}

fn run_incremental_project_check(
    prepared_check: &mut PreparedStructureCheck,
    root: &Path,
    paths: Vec<PathBuf>,
) -> i32 {
    for path in paths {
        let exit_code = run_path_check(prepared_check, root, path, false);
        if exit_code != 0 {
            return exit_code;
        }
    }
    0
}

struct ClientRequestWithStream {
    request: ClientRequest,
    stream: ClientStream,
}

enum ClientRequest {
    Session,
    Quit,
    Project {
        compact_response: bool,
    },
    Path(PathBuf),
    DirtyProjectPath {
        path: PathBuf,
        compact_response: bool,
    },
}

impl ClientRequest {
    fn compact_response(&self) -> bool {
        matches!(
            self,
            Self::Project {
                compact_response: true
            } | Self::DirtyProjectPath {
                compact_response: true,
                ..
            }
        )
    }
}

fn read_client_request(mut stream: ClientStream) -> Result<ClientRequestWithStream, String> {
    let request =
        read_client_request_from_stream(&mut stream)?.ok_or_else(|| "empty request".to_string())?;
    Ok(ClientRequestWithStream { request, stream })
}

fn read_client_request_from_stream(
    stream: &mut ClientStream,
) -> Result<Option<ClientRequest>, String> {
    let mut request = [0_u8; 4096];
    let mut len = 0;
    while len < request.len() {
        let read_len = stream
            .read(&mut request[len..])
            .map_err(|error| error.to_string())?;
        if read_len == 0 {
            if len == 0 {
                return Ok(None);
            }
            break;
        }
        len += read_len;
        if request[..len].contains(&b'\n') {
            break;
        }
    }

    parse_client_request(&request[..len]).map(Some)
}

fn parse_client_request(mut request: &[u8]) -> Result<ClientRequest, String> {
    while matches!(request.last(), Some(b'\n' | b'\r')) {
        request = &request[..request.len() - 1];
    }

    if request == b"C" {
        return Ok(ClientRequest::Project {
            compact_response: true,
        });
    }
    if request == b"SESSION" {
        return Ok(ClientRequest::Session);
    }
    if request == b"QUIT" {
        return Ok(ClientRequest::Quit);
    }
    if request == b"CHECK" {
        return Ok(ClientRequest::Project {
            compact_response: false,
        });
    }
    if let Some(path) = request.strip_prefix(b"CHECK-PATH\t") {
        if path.is_empty() {
            return Err("invalid empty CHECK-PATH request".to_string());
        }
        let path = std::str::from_utf8(path).map_err(|_| "invalid UTF-8 path".to_string())?;
        return Ok(ClientRequest::Path(PathBuf::from(path)));
    }
    if let Some(path) = request.strip_prefix(b"D\t") {
        if path.is_empty() {
            return Err("invalid empty D request".to_string());
        }
        let path = std::str::from_utf8(path).map_err(|_| "invalid UTF-8 path".to_string())?;
        return Ok(ClientRequest::DirtyProjectPath {
            path: PathBuf::from(path),
            compact_response: true,
        });
    }
    if let Some(path) = request.strip_prefix(b"CHECK-DIRTY-PROJECT-PATH\t") {
        if path.is_empty() {
            return Err("invalid empty CHECK-DIRTY-PROJECT-PATH request".to_string());
        }
        let path = std::str::from_utf8(path).map_err(|_| "invalid UTF-8 path".to_string())?;
        return Ok(ClientRequest::DirtyProjectPath {
            path: PathBuf::from(path),
            compact_response: false,
        });
    }
    Err("invalid request".to_string())
}

fn write_client_response(
    stream: &mut ClientStream,
    exit_code: i32,
    compact: bool,
) -> Result<(), String> {
    if compact && (0..=9).contains(&exit_code) {
        return stream
            .write_all(&[b'0' + exit_code as u8])
            .map_err(|error| error.to_string());
    }

    if (0..=9).contains(&exit_code) {
        return stream
            .write_all(&[b'O', b'K', b' ', b'0' + exit_code as u8, b'\n'])
            .map_err(|error| error.to_string());
    }

    stream
        .write_all(format!("OK {exit_code}\n").as_bytes())
        .map_err(|error| error.to_string())
}

fn exit_code_for_check_error(error: &CheckError) -> i32 {
    match error {
        CheckError::NoConfig(_) => 4,
        CheckError::Config(_) => 2,
        _ => 3,
    }
}
