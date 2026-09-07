//! Watch subprocess transport and deterministic diagnostic-association tests.

use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;

pub(super) const EVENT_TIMEOUT: Duration = Duration::from_secs(10);
const NORMALIZATION_DIAGNOSTIC_PREFIX: &str = "assura.watch.normalization.v1 ";

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

pub(super) struct WatchProcess {
    child: Child,
    pub(super) events: Receiver<Value>,
    pub(super) diagnostics: Receiver<Value>,
}

impl WatchProcess {
    pub(super) fn spawn(project: &TempDir, debounce_ms: u64) -> Self {
        Self::spawn_path(project.path(), None, debounce_ms)
    }

    pub(super) fn spawn_path(
        path: &std::path::Path,
        config: Option<&std::path::Path>,
        debounce_ms: u64,
    ) -> Self {
        Self::spawn_path_with_normalization_diagnostics(path, config, debounce_ms, false)
    }

    pub(super) fn spawn_path_with_normalization_diagnostics(
        path: &std::path::Path,
        config: Option<&std::path::Path>,
        debounce_ms: u64,
        normalization_diagnostics: bool,
    ) -> Self {
        let mut command = Command::new(assura_full_bin());
        if let Some(config) = config {
            command.arg("--config").arg(config);
        }
        command
            .arg("watch")
            .arg(path)
            .args(["--format", "json", "--debounce", &debounce_ms.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if normalization_diagnostics {
            command.env("ASSURA_WATCH_NORMALIZATION_DEBUG", "1");
        }
        #[cfg(windows)]
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
        let mut child = command.spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, events) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = line.unwrap();
                if line.trim().is_empty() {
                    continue;
                }
                sender.send(serde_json::from_str(&line).unwrap()).unwrap();
            }
        });
        let stderr = child.stderr.take().unwrap();
        let (diagnostic_sender, diagnostics) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                let line = line.unwrap();
                if let Some(payload) = line.strip_prefix(NORMALIZATION_DIAGNOSTIC_PREFIX) {
                    diagnostic_sender
                        .send(serde_json::from_str(payload).unwrap())
                        .unwrap();
                }
            }
        });
        Self {
            child,
            events,
            diagnostics,
        }
    }

    pub(super) fn next_event(&self) -> Value {
        self.events.recv_timeout(EVENT_TIMEOUT).unwrap()
    }

    pub(super) fn next_config_event(&self, expected_predecessor_path: &str) -> (Value, u64) {
        let deadline = Instant::now() + EVENT_TIMEOUT;
        let mut preceding_filesystem_events = 0;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let event = self.events.recv_timeout(remaining).unwrap();
            if event["trigger"] == "config" {
                return (event, preceding_filesystem_events);
            }
            eprintln!("event before expected config reload: {event}");
            assert_eq!(event["trigger"], "filesystem");
            preceding_filesystem_events += 1;
            assert_eq!(
                preceding_filesystem_events, 1,
                "watch emitted more than one filesystem event before config reload"
            );
            assert_eq!(event["report"]["success"], true);
            match event["runtime_mode"].as_str() {
                Some("warm_incremental") => {
                    assert_eq!(event["report_scope"], "affected_path");
                    assert!(event["changed_paths"].as_array().is_some_and(|paths| {
                        paths.iter().any(|path| path == expected_predecessor_path)
                    }));
                }
                Some("warm_full") => {
                    assert_eq!(event["report_scope"], "requested_path");
                    assert_eq!(event["fallback_reason"], "full_rescan_event");
                }
                runtime_mode => {
                    panic!("unexpected filesystem event before config reload: {runtime_mode:?}")
                }
            }
        }
    }

    pub(super) fn next_normalization_diagnostic(&self) -> Value {
        self.diagnostics.recv_timeout(EVENT_TIMEOUT).unwrap()
    }

    pub(super) fn assert_no_event(&self, duration: Duration) {
        match self.events.recv_timeout(duration) {
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Ok(event) => panic!("watch emitted an extra event after the debounce window: {event}"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("watch event reader disconnected before the debounce window elapsed")
            }
        }
    }

    pub(super) fn assert_no_event_or_scoped_rescan(
        &self,
        duration: Duration,
        diagnostic_scope: Option<(&str, &std::path::Path)>,
    ) -> u64 {
        match self.events.recv_timeout(duration) {
            Err(mpsc::RecvTimeoutError::Timeout) => 2,
            Ok(event) => {
                assert_event(&event, 2, "filesystem", "warm_full");
                assert_eq!(event["fallback_reason"], "full_rescan_event");
                assert_eq!(event["report_scope"], "requested_path");
                assert_eq!(event["changed_paths"], serde_json::json!([]));
                assert_eq!(event["report"]["success"], true);
                if let Some((scope, checked_path)) = diagnostic_scope {
                    assert_eq!(
                        event["report"]["checked_path"],
                        checked_path.to_string_lossy().replace('\\', "/")
                    );
                    read_report_diagnostics(&self.diagnostics, &event, scope, EVENT_TIMEOUT);
                }
                3
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("watch event reader disconnected before the debounce window elapsed")
            }
        }
    }

    pub(super) fn interrupt(&mut self) {
        #[cfg(unix)]
        let status = Command::new("kill")
            .args(["-INT", &self.child.id().to_string()])
            .status()
            .unwrap();
        #[cfg(unix)]
        assert!(status.success(), "failed to send SIGINT");

        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Console::{GenerateConsoleCtrlEvent, CTRL_BREAK_EVENT};
            let sent = unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, self.child.id()) };
            assert_ne!(sent, 0, "failed to send Ctrl-Break");
        }

        let deadline = Instant::now() + EVENT_TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success(), "watch exited with {status}");
                return;
            }
            assert!(Instant::now() < deadline, "watch did not stop after SIGINT");
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}

// stdout and stderr have separate reader threads. Associate in FIFO order by
// the report's invalidating-event count, never by searching for an expected path.
pub(super) fn read_report_diagnostics(
    diagnostics: &Receiver<Value>,
    event: &Value,
    scope: &str,
    timeout: Duration,
) -> Vec<Value> {
    let expected = event["coalesced_events"].as_u64().unwrap();
    assert!(
        expected > 0,
        "filesystem report must contain invalidating events"
    );
    let deadline = Instant::now() + timeout;
    let mut records = Vec::new();
    while records.len() < expected as usize {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "normalization diagnostic deadline elapsed"
        );
        let diagnostic = diagnostics
            .recv_timeout(remaining)
            .expect("missing normalization diagnostic for report");
        eprintln!(
            "normalization diagnostic for sequence {}: {diagnostic}",
            event["sequence"]
        );
        assert_eq!(diagnostic["config_changed"], false);
        let kind = diagnostic["event_kind"].as_str().unwrap();
        assert!(!kind.is_empty());
        let paths = diagnostic["paths"].as_array().unwrap();
        assert!(
            paths.iter().all(|path| path.as_str().is_some_and(|path| {
                // display_paths represents the root as "", not as a missing
                // path. Every other normalized path must remain relative.
                let relative = path.is_empty()
                    || path.split('/').all(|part| {
                        !part.is_empty()
                            && part != "."
                            && part != ".."
                            && !part.contains(['\\', ':'])
                    });
                relative
                    && (scope.is_empty() || path == scope || path.starts_with(&format!("{scope}/")))
            })),
            "diagnostic outside requested scope: {diagnostic}"
        );
        let rescan = diagnostic["need_rescan"].as_bool().unwrap();
        if diagnostic["invalidated"].as_bool().unwrap() {
            assert!(
                !paths.is_empty() || rescan,
                "pathless diagnostic must request rescan"
            );
            records.push(diagnostic);
        } else {
            assert!(
                kind.starts_with("Access(") && kind != "Access(Close(Write))",
                "unexpected non-invalidating diagnostic: {diagnostic}"
            );
            assert!(
                !rescan && !paths.is_empty(),
                "invalid ignored access diagnostic"
            );
        }
    }
    records
}

pub(super) fn assert_invalid_edit_feedback(
    watch: &WatchProcess,
    event: &Value,
    root: &std::path::Path,
    expected_path: &str,
) {
    eprintln!("invalid edit report: {event}");
    let mode = event["runtime_mode"].as_str().unwrap();
    assert!(matches!(mode, "warm_incremental" | "warm_full"));
    assert_event(event, 2, "filesystem", mode);
    assert_eq!(event["cache_state"], "prepared");
    assert_eq!(event["report"]["success"], false);
    let violations = event["report"]["violations"].as_array().unwrap();
    assert!(violations
        .iter()
        .any(|violation| violation["rule"] == "file_naming" && violation["path"] == expected_path));
    if mode == "warm_incremental" {
        assert_eq!(event["report_scope"], "affected_path");
        assert_eq!(event["changed_paths"], serde_json::json!([expected_path]));
        assert_eq!(
            event["report"]["checked_path"],
            root.join(expected_path)
                .to_string_lossy()
                .replace('\\', "/")
        );
    } else {
        assert_eq!(event["report_scope"], "requested_path");
        assert_eq!(event["fallback_reason"], "full_rescan_event");
        assert_eq!(event["changed_paths"], serde_json::json!([]));
        assert_eq!(
            event["report"]["checked_path"],
            root.to_string_lossy().replace('\\', "/")
        );
    }
    let diagnostics = read_report_diagnostics(&watch.diagnostics, event, "", EVENT_TIMEOUT);
    if mode == "warm_full" {
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic["need_rescan"] == true
                    || diagnostic["paths"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|path| path != expected_path)
                    || matches!(
                        diagnostic["event_kind"].as_str(),
                        Some(
                            "Any"
                                | "Other"
                                | "Create(Folder)"
                                | "Remove(Any)"
                                | "Remove(Folder)"
                                | "Remove(Other)"
                        )
                    )
            }),
            "full report lacks a full-scope diagnostic cause"
        );
    }
    for diagnostic in &diagnostics {
        if mode == "warm_incremental" {
            assert_eq!(diagnostic["paths"], serde_json::json!([expected_path]));
            assert_eq!(diagnostic["need_rescan"], false);
        } else {
            assert!(
                diagnostic["paths"].as_array().unwrap().iter().all(|path| {
                    let path = path.as_str().unwrap();
                    path.is_empty()
                        || path == expected_path
                        || expected_path.starts_with(&format!("{path}/"))
                }),
                "unrelated diagnostic in invalid-edit fixture: {diagnostic}"
            );
        }
    }
}

impl Drop for WatchProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[test]
#[should_panic(expected = "watch event reader disconnected")]
fn assert_no_event_rejects_a_disconnected_event_reader() {
    let child = Command::new(assura_full_bin())
        .arg("--version")
        .spawn()
        .unwrap();
    let (sender, events) = mpsc::channel();
    drop(sender);
    let (_diagnostic_sender, diagnostics) = mpsc::channel();
    let watch = WatchProcess {
        child,
        events,
        diagnostics,
    };

    watch.assert_no_event(Duration::from_millis(1));
}

#[test]
fn scoped_rescan_consumes_its_diagnostic_before_the_incremental_edit() {
    let child = Command::new(assura_full_bin())
        .arg("--version")
        .spawn()
        .unwrap();
    let (sender, events) = mpsc::channel();
    let (diagnostic_sender, diagnostics) = mpsc::channel();
    sender
        .send(serde_json::json!({
            "schema": "assura.watch.event.v1", "sequence": 2,
            "trigger": "filesystem", "runtime_mode": "warm_full", "debounce_ms": 100,
            "fallback_reason": "full_rescan_event", "report_scope": "requested_path",
            "changed_paths": [], "coalesced_events": 1,
            "report": {"success": true, "checked_path": "/fixture/src"}
        }))
        .unwrap();
    for (event_kind, path) in [
        ("Create(Folder)", "src"),
        ("Create(File)", "src/BadName.ts"),
    ] {
        diagnostic_sender
            .send(serde_json::json!({
                "event_kind": event_kind, "paths": [path], "need_rescan": false,
                "config_changed": false, "invalidated": true
            }))
            .unwrap();
    }
    let watch = WatchProcess {
        child,
        events,
        diagnostics,
    };
    assert_eq!(
        watch.assert_no_event_or_scoped_rescan(
            Duration::from_millis(20),
            Some(("src", std::path::Path::new("/fixture/src")))
        ),
        3
    );
    assert_eq!(
        watch.next_normalization_diagnostic()["paths"],
        serde_json::json!(["src/BadName.ts"])
    );
}

#[test]
fn root_report_diagnostics_include_root_folder_and_coalesced_file() {
    let (sender, diagnostics) = mpsc::channel();
    for (kind, path) in [("Create(Folder)", ""), ("Create(File)", "src/BadName.ts")] {
        sender
            .send(serde_json::json!({
                "event_kind": kind, "paths": [path], "need_rescan": false,
                "config_changed": false, "invalidated": true
            }))
            .unwrap();
    }
    let records = read_report_diagnostics(
        &diagnostics,
        &serde_json::json!({"sequence": 2, "coalesced_events": 2}),
        "",
        EVENT_TIMEOUT,
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["paths"], serde_json::json!([""]));
    assert_eq!(records[1]["paths"], serde_json::json!(["src/BadName.ts"]));
}

#[test]
fn root_diagnostics_distinguish_named_root_from_pathless_rescan() {
    for (paths, rescan) in [
        (serde_json::json!([""]), false),
        (serde_json::json!([]), true),
    ] {
        let (sender, diagnostics) = mpsc::channel();
        sender
            .send(serde_json::json!({
                "event_kind": "Create(Folder)", "paths": paths, "need_rescan": rescan,
                "config_changed": false, "invalidated": true
            }))
            .unwrap();
        let records = read_report_diagnostics(
            &diagnostics,
            &serde_json::json!({"coalesced_events": 1}),
            "",
            EVENT_TIMEOUT,
        );
        assert_eq!(records[0]["paths"], paths);
        assert_eq!(records[0]["need_rescan"], rescan);
    }
}

#[test]
fn root_diagnostics_reject_non_normalized_paths_and_unflagged_pathless_events() {
    for paths in [
        serde_json::json!([]),
        serde_json::json!(["/outside"]),
        serde_json::json!(["../outside"]),
        serde_json::json!(["src/../BadName.ts"]),
        serde_json::json!(["src//BadName.ts"]),
        serde_json::json!(["./src/BadName.ts"]),
        serde_json::json!(["C:/outside"]),
        serde_json::json!(["src\\BadName.ts"]),
        serde_json::json!([null]),
    ] {
        let (sender, diagnostics) = mpsc::channel();
        sender
            .send(serde_json::json!({
                "event_kind": "Create(Folder)", "paths": paths, "need_rescan": false,
                "config_changed": false, "invalidated": true
            }))
            .unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            read_report_diagnostics(
                &diagnostics,
                &serde_json::json!({"coalesced_events": 1}),
                "",
                EVENT_TIMEOUT,
            );
        }));
        assert_rejection(
            result,
            if paths == serde_json::json!([]) {
                "pathless diagnostic must request rescan"
            } else {
                "diagnostic outside requested scope"
            },
        );
    }
}

fn queued_edit_feedback(full: bool, paths: &[&str]) -> (WatchProcess, Value) {
    let child = Command::new(assura_full_bin())
        .arg("--version")
        .spawn()
        .unwrap();
    let (_sender, events) = mpsc::channel();
    let (sender, diagnostics) = mpsc::channel();
    for path in paths {
        sender
            .send(serde_json::json!({
                "event_kind": if path.is_empty() { "Create(Folder)" } else { "Create(File)" },
                "paths": [path], "need_rescan": false, "config_changed": false, "invalidated": true
            }))
            .unwrap();
    }
    let checked_path = if full {
        std::path::PathBuf::from("/fixture")
    } else {
        std::path::Path::new("/fixture").join("src/BadName.ts")
    };
    let event = serde_json::json!({
        "schema": "assura.watch.event.v1", "sequence": 2, "trigger": "filesystem",
        "runtime_mode": if full { "warm_full" } else { "warm_incremental" },
        "cache_state": "prepared", "debounce_ms": 100, "coalesced_events": paths.len(),
        "fallback_reason": if full { Some("full_rescan_event") } else { None },
        "report_scope": if full { "requested_path" } else { "affected_path" },
        "changed_paths": if full { vec![] } else { vec!["src/BadName.ts"] },
        "report": {"success": false,
            "checked_path": checked_path.to_string_lossy().replace('\\', "/"),
            "violations": [{"rule": "file_naming", "path": "src/BadName.ts"}]}
    });
    (
        WatchProcess {
            child,
            events,
            diagnostics,
        },
        event,
    )
}

#[test]
fn checked_path_assertions_accept_serialized_windows_paths() {
    for (native, serialized) in [
        (r"C:\fixture", "C:/fixture"),
        (r"\\?\C:\fixture", "//?/C:/fixture"),
        (r"\\server\share\fixture", "//server/share/fixture"),
    ] {
        for full in [false, true] {
            let (watch, mut event) =
                queued_edit_feedback(full, if full { &[""] } else { &["src/BadName.ts"] });
            event["report"]["checked_path"] = Value::String(if full {
                serialized.to_owned()
            } else {
                format!("{serialized}/src/BadName.ts")
            });
            assert_invalid_edit_feedback(
                &watch,
                &event,
                std::path::Path::new(native),
                "src/BadName.ts",
            );
        }
    }
}

#[test]
fn root_full_feedback_accepts_folder_only_and_folder_file_but_incremental_requires_file() {
    for (full, paths) in [
        (true, vec![""]),
        (true, vec!["", "src/BadName.ts"]),
        (false, vec!["src/BadName.ts"]),
    ] {
        let (watch, event) = queued_edit_feedback(full, &paths);
        assert_invalid_edit_feedback(
            &watch,
            &event,
            std::path::Path::new("/fixture"),
            "src/BadName.ts",
        );
    }
}

#[test]
fn root_feedback_rejects_unrelated_diagnostics_and_unjustified_full_reports() {
    for (paths, expected) in [
        (vec!["", "docs/BadName.ts"], "unrelated diagnostic"),
        (
            vec!["src/BadName.ts"],
            "full report lacks a full-scope diagnostic cause",
        ),
    ] {
        let (watch, event) = queued_edit_feedback(true, &paths);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_invalid_edit_feedback(
                &watch,
                &event,
                std::path::Path::new("/fixture"),
                "src/BadName.ts",
            );
        }));
        assert_rejection(result, expected);
    }
}

#[test]
#[should_panic(expected = "src/BadName.ts")]
fn incremental_feedback_rejects_a_root_folder_diagnostic() {
    let (watch, event) = queued_edit_feedback(false, &[""]);
    assert_invalid_edit_feedback(
        &watch,
        &event,
        std::path::Path::new("/fixture"),
        "src/BadName.ts",
    );
}

#[test]
fn report_diagnostics_count_invalidations_not_access_notifications() {
    let (sender, diagnostics) = mpsc::channel();
    for (kind, invalidated) in [
        ("Access(Read)", false),
        ("Create(File)", true),
        ("Access(Open(Read))", false),
        ("Modify(Data(Content))", true),
    ] {
        sender
            .send(serde_json::json!({
                "event_kind": kind, "paths": ["src/BadName.ts"], "need_rescan": false,
                "config_changed": false, "invalidated": invalidated
            }))
            .unwrap();
    }
    let records = read_report_diagnostics(
        &diagnostics,
        &serde_json::json!({"sequence": 3, "coalesced_events": 2}),
        "src",
        EVENT_TIMEOUT,
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["event_kind"], "Create(File)");
    assert_eq!(records[1]["event_kind"], "Modify(Data(Content))");
}

fn assert_rejection(result: std::thread::Result<()>, expected: &str) {
    let panic = result.expect_err("invalid diagnostic was accepted");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap();
    assert!(
        message.contains(expected),
        "unexpected rejection: {message}"
    );
}

#[test]
fn report_diagnostics_reject_missing_and_disconnected_records() {
    for disconnected in [false, true] {
        let (sender, diagnostics) = mpsc::channel::<Value>();
        let sender = if disconnected {
            drop(sender);
            None
        } else {
            Some(sender)
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            read_report_diagnostics(
                &diagnostics,
                &serde_json::json!({"coalesced_events": 1}),
                "src",
                if disconnected {
                    EVENT_TIMEOUT
                } else {
                    Duration::from_millis(5)
                },
            );
        }));
        assert_rejection(
            result,
            if disconnected {
                "missing normalization diagnostic for report: Disconnected"
            } else {
                "normalization diagnostic"
            },
        );
        drop(sender);
    }
}

#[test]
fn report_diagnostics_reject_wrong_scope_and_invalid_ignored_records() {
    for (path, kind, invalidated, rescan) in [
        ("docs/BadName.ts", "Create(File)", true, false),
        ("src-other/BadName.ts", "Create(File)", true, false),
        ("src/BadName.ts", "Create(File)", false, false),
        ("src/BadName.ts", "Access(Close(Write))", false, false),
        ("src/BadName.ts", "Access(Read)", false, true),
    ] {
        let (sender, diagnostics) = mpsc::channel();
        sender
            .send(serde_json::json!({
                "event_kind": kind, "paths": [path], "need_rescan": rescan,
                "config_changed": false, "invalidated": invalidated
            }))
            .unwrap();
        // A valid record follows so silently skipping the bad input would pass
        // the reader, not merely fail later from an unrelated timeout.
        sender
            .send(serde_json::json!({
                "event_kind": "Create(File)", "paths": ["src/BadName.ts"],
                "need_rescan": false, "config_changed": false, "invalidated": true
            }))
            .unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            read_report_diagnostics(
                &diagnostics,
                &serde_json::json!({"coalesced_events": 1}),
                "src",
                EVENT_TIMEOUT,
            );
        }));
        let expected = if invalidated {
            "diagnostic outside requested scope"
        } else if rescan {
            "invalid ignored access diagnostic"
        } else {
            "unexpected non-invalidating diagnostic"
        };
        assert_rejection(result, expected);
    }
}

#[test]
#[should_panic(expected = "warm_full")]
fn scoped_rescan_does_not_hide_a_second_predecessor() {
    let child = Command::new(assura_full_bin())
        .arg("--version")
        .spawn()
        .unwrap();
    let (sender, events) = mpsc::channel();
    let (_diagnostic_sender, diagnostics) = mpsc::channel();
    for sequence in [2, 3] {
        sender
            .send(serde_json::json!({
                "schema": "assura.watch.event.v1", "sequence": sequence,
                "trigger": "filesystem", "runtime_mode": "warm_full", "debounce_ms": 100,
                "fallback_reason": "full_rescan_event", "report_scope": "requested_path",
                "changed_paths": [], "coalesced_events": 1, "report": {"success": true}
            }))
            .unwrap();
    }
    let watch = WatchProcess {
        child,
        events,
        diagnostics,
    };
    let expected = watch.assert_no_event_or_scoped_rescan(Duration::from_millis(20), None);
    assert_event(
        &watch.next_event(),
        expected,
        "filesystem",
        "warm_incremental",
    );
}

#[test]
#[should_panic(expected = "/fixture/docs")]
fn scoped_rescan_rejects_a_report_checked_outside_the_requested_directory() {
    let child = Command::new(assura_full_bin())
        .arg("--version")
        .spawn()
        .unwrap();
    let (sender, events) = mpsc::channel();
    let (_diagnostic_sender, diagnostics) = mpsc::channel();
    sender
        .send(serde_json::json!({
            "schema": "assura.watch.event.v1", "sequence": 2,
            "trigger": "filesystem", "runtime_mode": "warm_full", "debounce_ms": 100,
            "fallback_reason": "full_rescan_event", "report_scope": "requested_path",
            "changed_paths": [], "coalesced_events": 1,
            "report": {"success": true, "checked_path": "/fixture/docs"}
        }))
        .unwrap();
    let watch = WatchProcess {
        child,
        events,
        diagnostics,
    };
    watch.assert_no_event_or_scoped_rescan(
        Duration::from_millis(20),
        Some(("src", std::path::Path::new("/fixture/src"))),
    );
}

pub(super) fn assert_event(event: &Value, sequence: u64, trigger: &str, runtime_mode: &str) {
    assert_eq!(event["schema"], "assura.watch.event.v1");
    assert_eq!(event["sequence"], sequence);
    assert_eq!(event["trigger"], trigger);
    assert_eq!(event["runtime_mode"], runtime_mode);
    assert!(event["report_scope"].is_string());
    assert_eq!(event["debounce_ms"], event["debounce_ms"].as_u64().unwrap());
}
