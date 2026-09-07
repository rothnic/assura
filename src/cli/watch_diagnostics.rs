//! Watch normalization diagnostics and bounded callback-arrival investigation.

use super::{DirtyState, Event, WatchContext, WatchMessage};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static ORDINAL: AtomicU64 = AtomicU64::new(0);
static PHASE: AtomicUsize = AtomicUsize::new(0);
static PROCESSING_ORDINAL: AtomicU64 = AtomicU64::new(0);
const MAX_CALLBACKS: u64 = 64;
const MAX_PATHS: usize = 8;
const MAX_PATH_UNITS: usize = 256;
const NORMALIZATION_DEBUG_ENV: &str = "ASSURA_WATCH_NORMALIZATION_DEBUG";
const NORMALIZATION_DIAGNOSTIC_PREFIX: &str = "assura.watch.normalization.v1 ";
const PHASES: &[&str] = &[
    "scope_subscription",
    "external_subscription",
    "initial_scan",
    "after_initial_report",
];

pub(super) struct Trace {
    ordinal: u64,
    callback_phase: &'static str,
}

pub(super) fn emit_normalization_diagnostic(
    event: &Event,
    context: &WatchContext,
    dirty: &DirtyState,
    invalidated: bool,
) {
    if !cfg!(debug_assertions) || std::env::var_os(NORMALIZATION_DEBUG_ENV).is_none() {
        return;
    }
    let diagnostic = serde_json::json!({
        "paths": super::display_paths(&context.root, &event.paths),
        "event_kind": format!("{:?}", event.kind),
        "need_rescan": event.need_rescan(),
        "config_changed": dirty.config_changed(),
        "invalidated": invalidated,
    });
    eprintln!("{NORMALIZATION_DIAGNOSTIC_PREFIX}{diagnostic}");
}

fn enabled() -> bool {
    cfg!(debug_assertions) && std::env::var_os("ASSURA_WATCH_TRACE_DEBUG").is_some()
}

fn timestamp_micros() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

fn path_record(path: &Path) -> serde_json::Value {
    #[cfg(unix)]
    let (encoding, total_units, units): (&str, usize, Vec<u32>) = {
        use std::os::unix::ffi::OsStrExt;
        (
            "unix_bytes_hex4",
            path.as_os_str().as_bytes().len(),
            path.as_os_str()
                .as_bytes()
                .iter()
                .take(MAX_PATH_UNITS)
                .map(|&byte| u32::from(byte))
                .collect(),
        )
    };
    #[cfg(windows)]
    let (encoding, total_units, units): (&str, usize, Vec<u32>) = {
        use std::os::windows::ffi::OsStrExt;
        (
            "windows_utf16_hex4",
            path.as_os_str().encode_wide().count(),
            path.as_os_str()
                .encode_wide()
                .take(MAX_PATH_UNITS)
                .map(u32::from)
                .collect(),
        )
    };
    let raw_hex: Vec<String> = units
        .iter()
        .take(MAX_PATH_UNITS)
        .map(|unit| format!("{unit:04x}"))
        .collect();
    let display = path.to_string_lossy();
    serde_json::json!({
        "display": display.chars().take(MAX_PATH_UNITS).collect::<String>(),
        "lossy": matches!(display, std::borrow::Cow::Owned(_)),
        "encoding": encoding, "raw_hex": raw_hex.concat(),
        "total_units": total_units, "truncated": total_units > MAX_PATH_UNITS
    })
}

fn paths_record(paths: &[PathBuf], root: Option<&Path>) -> serde_json::Value {
    let records: Vec<_> = paths
        .iter()
        .take(MAX_PATHS)
        .map(|path| {
            path_record(
                root.and_then(|root| path.strip_prefix(root).ok())
                    .unwrap_or(path),
            )
        })
        .collect();
    let incomplete =
        paths.len() > MAX_PATHS || records.iter().any(|record| record["truncated"] == true);
    serde_json::json!({"items": records, "total_paths": paths.len(), "incomplete": incomplete})
}

pub(super) fn phase(value: usize) {
    if enabled() {
        PHASE.store(value, Ordering::SeqCst);
        eprintln!(
            "assura.watch.trace.v1 {}",
            serde_json::json!({
                "stage": "phase", "phase": PHASES[value], "unix_micros": timestamp_micros()
            })
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptureBudget {
    Record,
    MarkLimit,
    Omit,
}

fn capture_budget(ordinal: u64) -> CaptureBudget {
    if ordinal <= MAX_CALLBACKS {
        CaptureBudget::Record
    } else if ordinal == MAX_CALLBACKS + 1 {
        CaptureBudget::MarkLimit
    } else {
        CaptureBudget::Omit
    }
}

pub(super) fn callback(event: Event) -> WatchMessage {
    if !enabled() {
        return WatchMessage::Event(event);
    }
    let ordinal = ORDINAL.fetch_add(1, Ordering::SeqCst) + 1;
    let budget = capture_budget(ordinal);
    if budget != CaptureBudget::Record {
        if budget == CaptureBudget::MarkLimit {
            eprintln!(
                "assura.watch.trace.v1 {}",
                serde_json::json!({
                    "stage": "capture_limit", "incomplete": true,
                    "first_omitted_callback": ordinal, "max_callbacks": MAX_CALLBACKS
                })
            );
        }
        return WatchMessage::Event(event);
    }
    let callback_phase = PHASES[PHASE.load(Ordering::SeqCst)];
    eprintln!(
        "assura.watch.trace.v1 {}",
        serde_json::json!({
            "stage": "callback", "ordinal": ordinal, "phase": callback_phase,
            "unix_micros": timestamp_micros(), "paths": paths_record(&event.paths, None),
            "event_kind": format!("{:?}", event.kind), "need_rescan": event.need_rescan()
        })
    );
    WatchMessage::TracedEvent(
        event,
        Trace {
            ordinal,
            callback_phase,
        },
    )
}

pub(super) fn filtered(
    trace: Option<&Trace>,
    event: &Event,
    context: &WatchContext,
    dirty: &DirtyState,
    decision: &str,
) {
    let Some(trace) = trace else { return };
    let processing_ordinal = PROCESSING_ORDINAL.fetch_add(1, Ordering::SeqCst) + 1;
    eprintln!(
        "assura.watch.trace.v1 {}",
        serde_json::json!({
            "stage": "filter", "ordinal": trace.ordinal, "callback_phase": trace.callback_phase,
            "processing_ordinal": processing_ordinal,
            "unix_micros": timestamp_micros(), "decision": decision,
            "paths": paths_record(&event.paths, None),
            "normalized_paths": paths_record(&event.paths, Some(&context.root)),
            "event_kind": format!("{:?}", event.kind), "need_rescan": event.need_rescan(),
            "config_changed": dirty.config_changed()
        })
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_budget_records_exactly_the_cap_and_marks_only_the_first_omission() {
        let decisions: Vec<_> = (1..=MAX_CALLBACKS * 2).map(capture_budget).collect();
        assert_eq!(
            decisions
                .iter()
                .filter(|&&decision| decision == CaptureBudget::Record)
                .count(),
            MAX_CALLBACKS as usize
        );
        assert_eq!(
            decisions
                .iter()
                .filter(|&&decision| decision == CaptureBudget::MarkLimit)
                .count(),
            1
        );
        assert_eq!(capture_budget(MAX_CALLBACKS), CaptureBudget::Record);
        assert_eq!(capture_budget(MAX_CALLBACKS + 1), CaptureBudget::MarkLimit);
        assert_eq!(capture_budget(MAX_CALLBACKS + 2), CaptureBudget::Omit);
    }

    #[test]
    fn complete_capture_preserves_empty_root_path_distinct_from_no_paths() {
        let root = PathBuf::from("fixture");
        let record = paths_record(std::slice::from_ref(&root), Some(&root));
        assert_eq!(record["incomplete"], false);
        assert_eq!(record["items"][0]["display"], "");
        assert_eq!(record["items"][0]["raw_hex"], "");
        assert_eq!(record["total_paths"], 1);
        let pathless = paths_record(&[], Some(&root));
        assert_eq!(pathless["total_paths"], 0);
        assert_eq!(pathless["items"], serde_json::json!([]));
    }

    #[test]
    fn trace_paths_are_bounded_and_mark_incomplete_capture() {
        let long_path = PathBuf::from("x".repeat(MAX_PATH_UNITS + 1));
        let record = paths_record(&vec![long_path; MAX_PATHS + 1], None);
        assert_eq!(record["incomplete"], true);
        assert_eq!(record["total_paths"], MAX_PATHS + 1);
        assert_eq!(record["items"].as_array().unwrap().len(), MAX_PATHS);
        assert_eq!(record["items"][0]["truncated"], true);
        assert_eq!(
            record["items"][0]["raw_hex"].as_str().unwrap().len(),
            MAX_PATH_UNITS * 4
        );
    }

    #[cfg(unix)]
    #[test]
    fn trace_preserves_invalid_utf8_bytes_without_serialization_panic() {
        use std::os::unix::ffi::OsStrExt;
        let record = path_record(Path::new(std::ffi::OsStr::from_bytes(b"a\xff")));
        assert_eq!(record["lossy"], true);
        assert_eq!(record["raw_hex"], "006100ff");
        assert_eq!(record["truncated"], false);
        assert!(serde_json::to_string(&record).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn trace_preserves_unpaired_utf16_units_without_serialization_panic() {
        use std::os::windows::ffi::OsStringExt;
        let path = PathBuf::from(std::ffi::OsString::from_wide(&[0x61, 0xd800]));
        let record = path_record(&path);
        assert_eq!(record["lossy"], true);
        assert_eq!(record["raw_hex"], "0061d800");
        assert_eq!(record["truncated"], false);
        assert!(serde_json::to_string(&record).is_ok());
    }
}
