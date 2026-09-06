//! Continuous structure validation backed by a prepared check plan.

use super::args::{ExitCode, WatchOutputFormat};
use super::check::{CheckError, PreparedStructureCheck};
use super::doctor::exit_code_for_check_error;
use super::watch_event::{
    emit_event, event_from_result, CacheState, RuntimeMode, WatchEvent, WatchTrigger,
};
use super::watch_signal::shutdown_signal;
use super::watch_state::{
    display_paths, normalize_config_event, normalize_known_file_removal, DirtyProject, DirtyState,
    DirtyTake,
};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[cfg(test)]
use std::cell::RefCell;

const DEFAULT_DEBOUNCE_MS: u64 = 300;
const WATCH_CHANNEL_CAPACITY: usize = 256;
const NORMALIZATION_DEBUG_ENV: &str = "ASSURA_WATCH_NORMALIZATION_DEBUG";
const NORMALIZATION_DIAGNOSTIC_PREFIX: &str = "assura.watch.normalization.v1 ";

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
struct NormalizationCapture {
    paths: Vec<PathBuf>,
    kind: String,
    needs_rescan: bool,
    config_changed: bool,
    invalidated: bool,
}

#[cfg(test)]
thread_local! {
    static NORMALIZATION_CAPTURE: RefCell<Option<NormalizationCapture>> = const { RefCell::new(None) };
}

#[cfg(test)]
fn take_normalization_capture() -> Option<NormalizationCapture> {
    NORMALIZATION_CAPTURE.with(|capture| capture.borrow_mut().take())
}

#[cfg(test)]
fn record_normalization_capture(event: &Event, dirty: &DirtyState, invalidated: bool) {
    NORMALIZATION_CAPTURE.with(|capture| {
        *capture.borrow_mut() = Some(NormalizationCapture {
            paths: event.paths.clone(),
            kind: format!("{:?}", event.kind),
            needs_rescan: event.need_rescan(),
            config_changed: dirty.config_changed(),
            invalidated,
        });
    });
}

#[cfg(not(test))]
fn record_normalization_capture(_event: &Event, _dirty: &DirtyState, _invalidated: bool) {}

/// Continuously validate filesystem changes until interrupted.
pub async fn watch_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    debounce: Option<u64>,
    format: WatchOutputFormat,
    no_git: bool,
) -> ExitCode {
    let debounce_ms = debounce.unwrap_or(DEFAULT_DEBOUNCE_MS).max(1);
    match run_watch(path, config, debounce_ms, format, no_git).await {
        Ok(()) => ExitCode::Success,
        Err(WatchError::Check(error)) => {
            eprintln!("Error: {error}");
            exit_code_for_check_error(&error)
        }
        Err(WatchError::Runtime(error)) => {
            eprintln!("Error: {error}");
            ExitCode::RuntimeError
        }
    }
}

async fn run_watch(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    debounce_ms: u64,
    format: WatchOutputFormat,
    no_git: bool,
) -> Result<(), WatchError> {
    let requested_path = match path {
        Some(path) => path,
        None => std::env::current_dir().map_err(CheckError::Io)?,
    };
    if !requested_path.exists() {
        return Err(CheckError::MissingPath(requested_path).into());
    }
    let watch_scope = requested_path.canonicalize().map_err(CheckError::Io)?;
    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(watch_scope.clone()), config, false)?;
    let root = prepared.project_root().to_path_buf();
    let config_path = prepared.config_path().to_path_buf();
    let config_watch_parent = separate_config_watch_parent(&watch_scope, &config_path);
    let context = WatchContext {
        root,
        watch_scope_is_file: watch_scope.is_file(),
        watch_scope,
        config_path,
        config_watch_parent,
        no_git,
    };
    let dirty = Arc::new(DirtyState::new());
    let overflowed = Arc::new(AtomicBool::new(false));
    let (sender, mut receiver) = mpsc::channel(WATCH_CHANNEL_CAPACITY);
    let _watcher = create_watcher(
        &context.watch_scope,
        context.config_watch_parent.as_deref(),
        sender,
        Arc::clone(&overflowed),
    )?;

    dirty.take();
    let started = Instant::now();
    let report = prepared.check_path(context.watch_scope.clone())?;
    let mut project_clean = report.success;
    emit_event(
        format,
        &WatchEvent::report(
            1,
            WatchTrigger::Initial,
            RuntimeMode::ColdFull,
            CacheState::Prepared,
            None,
            Vec::new(),
            0,
            debounce_ms,
            started.elapsed(),
            report,
        ),
    )
    .map_err(WatchError::Runtime)?;

    let mut sequence = 2;
    let debounce = Duration::from_millis(debounce_ms);
    let max_batch_window = Duration::from_millis(debounce_ms.saturating_mul(4).max(1_000));
    loop {
        let first = tokio::select! {
            signal = shutdown_signal() => {
                signal.map_err(|error| WatchError::Runtime(format!("listen for interrupt: {error}")))?;
                return Ok(());
            }
            message = receiver.recv() => match message {
                Some(message) => message,
                None => return Err(WatchError::Runtime("filesystem watcher stopped".into())),
            },
        };

        let mut batch = WatchBatch::default();
        record_message(first, &context, &prepared, &dirty, &mut batch);
        let batch_started = Instant::now();
        loop {
            let remaining = max_batch_window.saturating_sub(batch_started.elapsed());
            if remaining.is_zero() {
                batch.max_window_reached = true;
                break;
            }
            let quiet_window = debounce.min(remaining);
            let received = tokio::select! {
                signal = shutdown_signal() => {
                    signal.map_err(|error| WatchError::Runtime(format!("listen for interrupt: {error}")))?;
                    return Ok(());
                }
                received = tokio::time::timeout(quiet_window, receiver.recv()) => received,
            };
            match received {
                Ok(Some(message)) => {
                    record_message(message, &context, &prepared, &dirty, &mut batch)
                }
                Ok(None) => return Err(WatchError::Runtime("filesystem watcher stopped".into())),
                Err(_) => {
                    batch.max_window_reached = batch_started.elapsed() >= max_batch_window;
                    break;
                }
            }
        }

        if overflowed.swap(false, Ordering::AcqRel) {
            dirty.record_event(&Event::new(EventKind::Any), &context.config_path);
            batch.invalidating_events += 1;
            batch.watcher_error = Some("event_channel_overflow".into());
        }
        if batch.invalidating_events == 0 {
            continue;
        }

        let taken = dirty.take();
        let terminal_watcher_error = batch.watcher_failed.then(|| {
            batch
                .watcher_error
                .clone()
                .unwrap_or_else(|| "unknown filesystem watcher failure".into())
        });
        let event = validate_batch(
            sequence,
            debounce_ms,
            &context,
            &mut prepared,
            taken,
            batch,
            project_clean,
        );
        project_clean = event.report.as_ref().is_some_and(|report| report.success);
        emit_event(format, &event).map_err(WatchError::Runtime)?;
        if let Some(error) = terminal_watcher_error {
            return Err(WatchError::Runtime(format!(
                "filesystem watcher failed: {error}"
            )));
        }
        sequence += 1;
    }
}

fn create_watcher(
    watch_scope: &Path,
    config_watch_parent: Option<&Path>,
    sender: mpsc::Sender<WatchMessage>,
    overflowed: Arc<AtomicBool>,
) -> Result<RecommendedWatcher, WatchError> {
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| {
            let message = match result {
                Ok(event) => WatchMessage::Event(event),
                Err(error) => WatchMessage::Error(error.to_string()),
            };
            if sender.try_send(message).is_err() {
                overflowed.store(true, Ordering::Release);
            }
        },
        Config::default(),
    )
    .map_err(|error| WatchError::Runtime(format!("create filesystem watcher: {error}")))?;
    let (subscription, recursive_mode) = if watch_scope.is_dir() {
        (watch_scope, RecursiveMode::Recursive)
    } else {
        (
            watch_scope.parent().ok_or_else(|| {
                WatchError::Runtime(format!(
                    "watch file has no parent directory: {}",
                    watch_scope.display()
                ))
            })?,
            RecursiveMode::NonRecursive,
        )
    };
    watcher
        .watch(subscription, recursive_mode)
        .map_err(|error| {
            WatchError::Runtime(format!("watch {}: {error}", subscription.display()))
        })?;
    if let Some(parent) = config_watch_parent.filter(|parent| *parent != subscription) {
        watcher
            .watch(parent, RecursiveMode::NonRecursive)
            .map_err(|error| {
                WatchError::Runtime(format!(
                    "watch config directory {}: {error}",
                    parent.display()
                ))
            })?;
    }
    Ok(watcher)
}

fn record_message(
    message: WatchMessage,
    context: &WatchContext,
    prepared: &PreparedStructureCheck,
    dirty: &DirtyState,
    batch: &mut WatchBatch,
) {
    match message {
        WatchMessage::Event(mut event) => {
            if !normalize_config_event(&mut event, &context.config_path, || {
                prepared.config_content_changed().unwrap_or(true)
            }) {
                return;
            }
            let had_only_irrelevant_paths = !event.paths.is_empty()
                && event.paths.iter().all(|path| {
                    path != &context.config_path
                        && (!path.starts_with(&context.watch_scope)
                            || ignored_runtime_path(&context.root, path, context.no_git))
                });
            event.paths.retain(|path| {
                path == &context.config_path
                    || (path.starts_with(&context.watch_scope)
                        && !ignored_runtime_path(&context.root, path, context.no_git)
                        && !prepared.is_excluded_path(path))
            });
            normalize_known_file_removal(
                &mut event,
                &context.watch_scope,
                context.watch_scope_is_file,
            );
            if event.paths.is_empty() && (had_only_irrelevant_paths || !event.need_rescan()) {
                return;
            }
            let invalidated = dirty.record_event(&event, &context.config_path);
            record_normalization_capture(&event, dirty, invalidated);
            emit_normalization_diagnostic(&event, context, dirty, invalidated);
            if invalidated {
                batch.invalidating_events += 1;
            }
        }
        WatchMessage::Error(error) => {
            dirty.record_event(&Event::new(EventKind::Any), &context.config_path);
            batch.invalidating_events += 1;
            batch.watcher_error = Some(error);
            batch.watcher_failed = true;
        }
    }
}

fn emit_normalization_diagnostic(
    event: &Event,
    context: &WatchContext,
    dirty: &DirtyState,
    invalidated: bool,
) {
    if !cfg!(debug_assertions) || std::env::var_os(NORMALIZATION_DEBUG_ENV).is_none() {
        return;
    }
    let diagnostic = serde_json::json!({
        "paths": display_paths(&context.root, &event.paths),
        "event_kind": format!("{:?}", event.kind),
        "need_rescan": event.need_rescan(),
        "config_changed": dirty.config_changed(),
        "invalidated": invalidated,
    });
    eprintln!("{NORMALIZATION_DIAGNOSTIC_PREFIX}{diagnostic}");
}

fn validate_batch(
    sequence: u64,
    debounce_ms: u64,
    context: &WatchContext,
    prepared: &mut PreparedStructureCheck,
    taken: DirtyTake,
    batch: WatchBatch,
    project_clean: bool,
) -> WatchEvent {
    let started = Instant::now();
    let changed_paths = match &taken.project {
        DirtyProject::Paths(paths) => display_paths(&context.root, paths),
        DirtyProject::Clean | DirtyProject::Full => Vec::new(),
    };

    let config_reloaded = match prepared.reload_if_config_changed() {
        Ok(changed) => changed,
        Err(error) => {
            return WatchEvent::failure(
                sequence,
                WatchTrigger::Config,
                RuntimeMode::WarmFull,
                CacheState::Degraded,
                Some("config_reload_failed".into()),
                changed_paths,
                batch.invalidating_events,
                debounce_ms,
                started.elapsed(),
                error.to_string(),
            )
        }
    };
    if taken.config_changed || config_reloaded {
        return event_from_result(
            sequence,
            WatchTrigger::Config,
            RuntimeMode::WarmFull,
            if config_reloaded {
                CacheState::Reloaded
            } else {
                CacheState::Prepared
            },
            batch.watcher_error,
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started,
            prepared.check_path(context.watch_scope.clone()),
        );
    }

    if batch.max_window_reached {
        return event_from_result(
            sequence,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Prepared,
            Some("max_batch_window".into()),
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started,
            prepared.check_path(context.watch_scope.clone()),
        );
    }

    match taken.project {
        DirtyProject::Paths(paths)
            if paths.len() == 1 && project_clean && prepared.supports_incremental_path_checks() =>
        {
            event_from_result(
                sequence,
                WatchTrigger::Filesystem,
                RuntimeMode::WarmIncremental,
                CacheState::Prepared,
                batch.watcher_error,
                changed_paths,
                batch.invalidating_events,
                debounce_ms,
                started,
                prepared.check_changed_path(paths.into_iter().next().unwrap()),
            )
        }
        DirtyProject::Paths(paths) if paths.len() == 1 && project_clean => event_from_result(
            sequence,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Prepared,
            Some("project_wide_policy".into()),
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started,
            prepared.check_path(context.watch_scope.clone()),
        ),
        DirtyProject::Paths(paths) if paths.len() == 1 => event_from_result(
            sequence,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Prepared,
            Some("project_not_clean".into()),
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started,
            prepared.check_path(context.watch_scope.clone()),
        ),
        DirtyProject::Paths(_) => event_from_result(
            sequence,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Prepared,
            Some("multiple_changed_paths".into()),
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started,
            prepared.check_path(context.watch_scope.clone()),
        ),
        DirtyProject::Full => event_from_result(
            sequence,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Prepared,
            batch
                .watcher_error
                .or_else(|| Some("full_rescan_event".into())),
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started,
            prepared.check_path(context.watch_scope.clone()),
        ),
        DirtyProject::Clean => WatchEvent::failure(
            sequence,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Degraded,
            Some("empty_invalidating_batch".into()),
            changed_paths,
            batch.invalidating_events,
            debounce_ms,
            started.elapsed(),
            "filesystem batch did not identify a validation scope".into(),
        ),
    }
}

fn separate_config_watch_parent(watch_scope: &Path, config_path: &Path) -> Option<PathBuf> {
    let scope_covers_config = if watch_scope.is_dir() {
        config_path.starts_with(watch_scope)
    } else {
        config_path == watch_scope
    };
    (!scope_covers_config)
        .then(|| config_path.parent().map(Path::to_path_buf))
        .flatten()
}

fn ignored_runtime_path(root: &Path, path: &Path, no_git: bool) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let mut components = relative.components();
    let first = components.next().and_then(|part| part.as_os_str().to_str());
    if no_git && first == Some(".git") {
        return true;
    }
    if first != Some(".assura") {
        return false;
    }
    match components.next().and_then(|part| part.as_os_str().to_str()) {
        None | Some("agent-sessions" | "cache" | "daemon" | "watch") => true,
        Some(_) => false,
    }
}

#[derive(Default)]
struct WatchBatch {
    invalidating_events: usize,
    watcher_error: Option<String>,
    watcher_failed: bool,
    max_window_reached: bool,
}

struct WatchContext {
    root: PathBuf,
    watch_scope: PathBuf,
    watch_scope_is_file: bool,
    config_path: PathBuf,
    config_watch_parent: Option<PathBuf>,
    no_git: bool,
}

enum WatchMessage {
    Event(Event),
    Error(String),
}

enum WatchError {
    Check(CheckError),
    Runtime(String),
}

impl From<CheckError> for WatchError {
    fn from(error: CheckError) -> Self {
        Self::Check(error)
    }
}

#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "watch_rescan_tests.rs"]
mod rescan_tests;
