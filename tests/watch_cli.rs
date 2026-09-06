use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;

const EVENT_TIMEOUT: Duration = Duration::from_secs(10);
const NORMALIZATION_DIAGNOSTIC_PREFIX: &str = "assura.watch.normalization.v1 ";

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

struct WatchProcess {
    child: Child,
    events: Receiver<Value>,
    diagnostics: Receiver<Value>,
}

impl WatchProcess {
    fn spawn(project: &TempDir, debounce_ms: u64) -> Self {
        Self::spawn_path(project.path(), None, debounce_ms)
    }

    fn spawn_path(
        path: &std::path::Path,
        config: Option<&std::path::Path>,
        debounce_ms: u64,
    ) -> Self {
        Self::spawn_path_with_normalization_diagnostics(path, config, debounce_ms, false)
    }

    fn spawn_path_with_normalization_diagnostics(
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

    fn next_event(&self) -> Value {
        self.events.recv_timeout(EVENT_TIMEOUT).unwrap()
    }

    fn next_config_event(&self, expected_predecessor_path: &str) -> (Value, u64) {
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

    fn next_normalization_diagnostic(&self) -> Value {
        self.diagnostics.recv_timeout(EVENT_TIMEOUT).unwrap()
    }

    fn assert_no_event(&self, duration: Duration) {
        match self.events.recv_timeout(duration) {
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Ok(event) => panic!("watch emitted an extra event after the debounce window: {event}"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("watch event reader disconnected before the debounce window elapsed")
            }
        }
    }

    fn interrupt(&mut self) {
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
fn watch_debug_diagnostic_reports_normalized_event_inputs() {
    let project = watch_project("kebab-case");
    fs::create_dir(project.path().join("src")).unwrap();
    let watch =
        WatchProcess::spawn_path_with_normalization_diagnostics(project.path(), None, 100, true);
    assert_eq!(watch.next_event()["report"]["success"], true);

    let source = project.path().join("src/BadName.ts");
    fs::write(&source, "export {};\n").unwrap();
    let changed = watch.next_event();
    assert_eq!(changed["report"]["success"], false);

    let diagnostic = watch.next_normalization_diagnostic();
    eprintln!("normalization diagnostic: {diagnostic}");
    assert_eq!(diagnostic["paths"], serde_json::json!(["src/BadName.ts"]));
    assert!(diagnostic["event_kind"]
        .as_str()
        .is_some_and(|kind| !kind.is_empty()));
    assert_eq!(diagnostic["need_rescan"], false);
    assert_eq!(diagnostic["config_changed"], false);
    assert_eq!(diagnostic["invalidated"], true);
}

#[test]
fn watch_emits_initial_and_warm_edit_reports() {
    let project = watch_project("kebab-case");
    let watch =
        WatchProcess::spawn_path_with_normalization_diagnostics(project.path(), None, 100, true);

    let initial = watch.next_event();
    assert_event(&initial, 1, "initial", "cold_full");
    assert_eq!(initial["report"]["success"], true);

    fs::write(project.path().join("BadName.ts"), "export {};\n").unwrap();

    let changed = watch.next_event();
    eprintln!("warm edit report: {changed}");
    let runtime_mode = changed["runtime_mode"].as_str().unwrap();
    assert!(matches!(runtime_mode, "warm_incremental" | "warm_full"));
    assert_event(&changed, 2, "filesystem", runtime_mode);
    assert_eq!(changed["cache_state"], "prepared");
    assert_eq!(changed["report"]["success"], false);
    assert_eq!(changed["report"]["violations"][0]["rule"], "file_naming");
    if runtime_mode == "warm_incremental" {
        assert_eq!(changed["report_scope"], "affected_path");
        assert!(changed["changed_paths"]
            .as_array()
            .is_some_and(|paths| paths.iter().any(|path| path == "BadName.ts")));
    } else {
        assert_eq!(changed["report_scope"], "requested_path");
        assert_eq!(changed["fallback_reason"], "full_rescan_event");
    }
    eprintln!(
        "warm edit diagnostic: {}",
        watch.next_normalization_diagnostic()
    );
}

#[test]
fn watch_uses_full_reports_while_project_is_already_failing() {
    let project = watch_project("kebab-case");
    fs::write(project.path().join("BadName.ts"), "export {};\n").unwrap();
    let watch = WatchProcess::spawn(&project, 100);
    assert_eq!(watch.next_event()["report"]["success"], false);

    fs::write(
        project.path().join("BadName.ts"),
        "export const changed = true;\n",
    )
    .unwrap();

    let changed = watch.next_event();
    assert_event(&changed, 2, "filesystem", "warm_full");
    assert!(matches!(
        changed["fallback_reason"].as_str(),
        Some("project_not_clean" | "full_rescan_event")
    ));
    assert_eq!(changed["report_scope"], "requested_path");
    assert_eq!(changed["report"]["success"], false);
}

#[test]
fn watch_coalesces_bursts_into_one_full_report() {
    let project = watch_project("kebab-case");
    let watch = WatchProcess::spawn(&project, 120);
    watch.next_event();

    for file_name in ["one.ts", "two.ts", "three.ts"] {
        fs::write(project.path().join(file_name), "export {};\n").unwrap();
    }

    let changed = watch.next_event();
    assert_event(&changed, 2, "filesystem", "warm_full");
    assert!(changed["coalesced_events"]
        .as_u64()
        .is_some_and(|count| count >= 1));
    assert!(matches!(
        changed["fallback_reason"].as_str(),
        Some("multiple_changed_paths" | "full_rescan_event")
    ));
    watch.assert_no_event(Duration::from_millis(450));
}

#[test]
fn watch_reloads_changed_configuration_before_reporting() {
    let project = watch_project("kebab-case");
    fs::write(project.path().join("good-name.ts"), "export {};\n").unwrap();
    let watch = WatchProcess::spawn(&project, 100);
    assert_eq!(watch.next_event()["report"]["success"], true);

    write_config(&project, "snake_case");

    let changed = watch.next_event();
    assert_event(&changed, 2, "config", "warm_full");
    assert_eq!(changed["cache_state"], "reloaded");
    assert_eq!(changed["report"]["success"], false);
    assert!(changed["report"]["violations"]
        .as_array()
        .is_some_and(|violations| violations.iter().any(|violation| {
            violation["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("good-name.ts"))
        })));
}

#[test]
fn watch_observes_an_explicit_config_outside_the_project() {
    let project = TempDir::new().unwrap();
    let config_home = TempDir::new().unwrap();
    let config = config_home.path().join("assura.yml");
    fs::write(&config, config_with_naming("kebab-case")).unwrap();
    fs::write(project.path().join("good-name.ts"), "export {};\n").unwrap();
    let watch = WatchProcess::spawn_path_with_normalization_diagnostics(
        project.path(),
        Some(&config),
        100,
        true,
    );
    assert_eq!(watch.next_event()["report"]["success"], true);

    fs::write(config_home.path().join("unrelated.yml"), "ignored: true\n").unwrap();
    watch.assert_no_event(Duration::from_millis(350));
    fs::write(&config, config_with_naming("snake_case")).unwrap();

    let (changed, preceding_filesystem_events) = watch.next_config_event("good-name.ts");
    eprintln!("external config report: {changed}");
    assert_event(
        &changed,
        2 + preceding_filesystem_events,
        "config",
        "warm_full",
    );
    assert_eq!(changed["cache_state"], "reloaded");
    assert_eq!(changed["report"]["success"], false);
}

#[test]
fn watch_uses_a_full_report_when_project_wide_policy_is_configured() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::create_dir_all(project.path().join("tests")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
extensions:
  custom_constraints:
    - id: source_test_pair
      type: paired_file_exists
      source: "src/*.ts"
      target: "tests/{stem}_test.rs"
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
"#,
    )
    .unwrap();
    let watch = WatchProcess::spawn(&project, 100);
    assert_eq!(watch.next_event()["report"]["success"], true);

    fs::write(project.path().join("src/new-source.ts"), "export {};\n").unwrap();

    let changed = watch.next_event();
    assert_event(&changed, 2, "filesystem", "warm_full");
    assert!(matches!(
        changed["fallback_reason"].as_str(),
        Some("project_wide_policy" | "full_rescan_event")
    ));
    assert_eq!(changed["report_scope"], "requested_path");
    assert_eq!(changed["report"]["success"], false);
    assert!(changed["report"]["violations"]
        .as_array()
        .is_some_and(|violations| violations
            .iter()
            .any(|violation| { violation["rule"] == "custom:source_test_pair" })));
}

#[test]
fn watch_honors_the_requested_directory_scope() {
    let project = watch_project("kebab-case");
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    let watch = WatchProcess::spawn_path(&project.path().join("src"), None, 100);
    let initial = watch.next_event();
    assert!(initial["report"]["checked_path"]
        .as_str()
        .is_some_and(|path| path.replace('\\', "/").ends_with("/src")));

    fs::write(project.path().join("docs/BadName.ts"), "export {};\n").unwrap();
    watch.assert_no_event(Duration::from_millis(450));
    fs::write(project.path().join("src/BadName.ts"), "export {};\n").unwrap();

    let changed = watch.next_event();
    assert_event(&changed, 2, "filesystem", "warm_incremental");
    assert_eq!(changed["report"]["success"], false);
}

#[test]
fn watch_file_scope_survives_atomic_replacement() {
    let project = watch_project("kebab-case");
    let source = project.path().join("entry.ts");
    fs::write(&source, "one\n").unwrap();
    let watch = WatchProcess::spawn_path(&source, None, 100);
    assert_eq!(watch.next_event()["report"]["success"], true);

    let replacement = project.path().join("entry.tmp");
    fs::write(&replacement, "replacement\n").unwrap();
    replace_file(&replacement, &source);
    let replaced = watch.next_event();
    assert_event(&replaced, 2, "filesystem", "warm_incremental");

    fs::write(&source, "edited after replacement\n").unwrap();
    let edited = watch.next_event();
    assert_event(&edited, 3, "filesystem", "warm_incremental");
}

#[test]
fn watch_ignores_paths_excluded_by_project_policy() {
    let project = watch_project("kebab-case");
    fs::create_dir(project.path().join("generated")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
exclude:
  - "generated/**"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": kebab-case
"#,
    )
    .unwrap();
    let watch = WatchProcess::spawn(&project, 100);
    watch.next_event();

    fs::write(project.path().join("generated/BadName.ts"), "export {};\n").unwrap();

    watch.assert_no_event(Duration::from_millis(450));
}

#[test]
fn watch_ignores_assura_runtime_output() {
    let project = watch_project("kebab-case");
    let watch = WatchProcess::spawn(&project, 100);
    watch.next_event();

    fs::create_dir_all(project.path().join(".assura/cache/worktree")).unwrap();
    fs::write(
        project.path().join(".assura/cache/worktree/result.json"),
        "{}\n",
    )
    .unwrap();

    watch.assert_no_event(Duration::from_millis(450));
}

#[test]
fn watch_emits_feedback_during_sustained_edits() {
    let project = watch_project("kebab-case");
    let watch = WatchProcess::spawn(&project, 100);
    watch.next_event();
    let target = project.path().join("working-file.ts");
    let finished = Arc::new(AtomicBool::new(false));
    let writer_finished = Arc::clone(&finished);
    let writer = std::thread::spawn(move || {
        let started = Instant::now();
        let mut revision = 0;
        while started.elapsed() < Duration::from_millis(2_500) {
            fs::write(&target, format!("export const revision = {revision};\n")).unwrap();
            revision += 1;
            std::thread::sleep(Duration::from_millis(20));
        }
        writer_finished.store(true, Ordering::Release);
    });

    let changed = watch.next_event();
    assert!(
        !finished.load(Ordering::Acquire),
        "watch waited for sustained edits to stop before reporting"
    );
    let runtime_mode = changed["runtime_mode"].as_str().unwrap();
    assert!(
        matches!(runtime_mode, "warm_incremental" | "warm_full"),
        "unexpected warm runtime mode: {runtime_mode}"
    );
    assert_event(&changed, 2, "filesystem", runtime_mode);
    if runtime_mode == "warm_full" {
        assert_eq!(changed["fallback_reason"], "max_batch_window");
    }
    writer.join().unwrap();
}

#[test]
fn watch_stops_cleanly_without_runtime_artifacts() {
    let project = watch_project("kebab-case");
    let mut watch = WatchProcess::spawn(&project, 100);
    watch.next_event();

    let keep_writing = Arc::new(AtomicBool::new(true));
    let writer_flag = Arc::clone(&keep_writing);
    let target = project.path().join("active-edit.ts");
    let writer = std::thread::spawn(move || {
        let mut revision = 0;
        while writer_flag.load(Ordering::Acquire) {
            fs::write(&target, format!("export const revision = {revision};\n")).unwrap();
            revision += 1;
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    std::thread::sleep(Duration::from_millis(150));

    watch.interrupt();
    keep_writing.store(false, Ordering::Release);
    writer.join().unwrap();

    assert!(!project.path().join(".assura/watch").exists());
}

fn assert_event(event: &Value, sequence: u64, trigger: &str, runtime_mode: &str) {
    assert_eq!(event["schema"], "assura.watch.event.v1");
    assert_eq!(event["sequence"], sequence);
    assert_eq!(event["trigger"], trigger);
    assert_eq!(event["runtime_mode"], runtime_mode);
    assert!(event["report_scope"].is_string());
    assert_eq!(event["debounce_ms"], event["debounce_ms"].as_u64().unwrap());
}

#[cfg(not(windows))]
fn replace_file(source: &std::path::Path, destination: &std::path::Path) {
    fs::rename(source, destination).unwrap();
}

#[cfg(windows)]
fn replace_file(source: &std::path::Path, destination: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING,
        )
    };
    assert_ne!(replaced, 0, "failed to atomically replace watched file");
}

fn watch_project(naming: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    write_config(&project, naming);
    project
}

fn write_config(project: &TempDir, naming: &str) {
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": {naming}
"#
        ),
    )
    .unwrap();
}

fn config_with_naming(naming: &str) -> String {
    format!(
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": {naming}
"#
    )
}
