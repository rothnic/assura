#![cfg(windows)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use windows_sys::Win32::System::Console::{
    GenerateConsoleCtrlEvent, GetConsoleProcessList, SetConsoleCtrlHandler, CTRL_C_EVENT,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, CREATE_NEW_CONSOLE};

const READY_MARKER: &str = "NU17_HELPER_READY";
const HELPER_RECEIPT_PREFIX: &str = "NU17_HELPER_RECEIPT ";
const WATCH_READY_PREFIX: &str = "NU17_HELPER_WATCH_READY ";
const FINAL_RECEIPT_PREFIX: &str = "NU17_CTRL_C_RECEIPT ";
const EVENT_DEADLINE: Duration = Duration::from_secs(10);
const STARTUP_DEADLINE: Duration = Duration::from_secs(10);
const NO_SEND_OBSERVATION: Duration = Duration::from_millis(500);
const CLEANUP_DEADLINE: Duration = Duration::from_secs(5);
const MAX_CONSOLE_PROCESSES: usize = 16;
const UNSEEN_EVENT: u32 = u32::MAX;

static CTRL_ACK: AtomicBool = AtomicBool::new(false);
static CTRL_EVENT: AtomicU32 = AtomicU32::new(UNSEEN_EVENT);

#[derive(Clone, Debug)]
struct ConsoleSnapshot {
    reported_count: u32,
    process_ids: Vec<u32>,
}

#[derive(Debug, PartialEq, Eq)]
enum SafetyError {
    IncompleteConsoleList,
    WrongConsoleMembers,
    HelperAlreadyAcknowledged,
    WatchNotLive,
    WatchNotInJob,
    SendFailed,
}

fn validate_then_send(
    snapshot: &ConsoleSnapshot,
    helper_pid: u32,
    watch_pid: u32,
    outer_pid: u32,
    helper_ack: bool,
    watch_live: bool,
    watch_in_job: bool,
    sender: impl FnOnce() -> bool,
) -> Result<(), SafetyError> {
    validate_safety(
        snapshot,
        helper_pid,
        watch_pid,
        outer_pid,
        helper_ack,
        watch_live,
        watch_in_job,
    )?;
    if sender() {
        Ok(())
    } else {
        Err(SafetyError::SendFailed)
    }
}

fn validate_safety(
    snapshot: &ConsoleSnapshot,
    helper_pid: u32,
    watch_pid: u32,
    outer_pid: u32,
    helper_ack: bool,
    watch_live: bool,
    watch_in_job: bool,
) -> Result<(), SafetyError> {
    if snapshot.reported_count as usize != snapshot.process_ids.len()
        || snapshot.reported_count == 0
    {
        return Err(SafetyError::IncompleteConsoleList);
    }
    let mut actual = snapshot.process_ids.clone();
    actual.sort_unstable();
    let mut expected = vec![helper_pid, watch_pid];
    expected.sort_unstable();
    if actual != expected || snapshot.process_ids.contains(&outer_pid) {
        return Err(SafetyError::WrongConsoleMembers);
    }
    if helper_ack {
        return Err(SafetyError::HelperAlreadyAcknowledged);
    }
    if !watch_live {
        return Err(SafetyError::WatchNotLive);
    }
    if !watch_in_job {
        return Err(SafetyError::WatchNotInJob);
    }
    Ok(())
}

fn final_receipt_is_valid(receipt: &Value) -> bool {
    let Some(positive) = receipt.get("positive") else {
        return false;
    };
    let Some(no_send) = receipt.get("no_send") else {
        return false;
    };
    receipt["schema"] == "assura.nu17.ctrl-c.v1"
        && positive["send_calls"] == 1
        && positive["event_code"].as_u64() == Some(CTRL_C_EVENT as u64)
        && positive["ack"] == true
        && positive["watch_exit_code"].as_i64() == Some(0)
        && positive["elapsed_ms"]
            .as_u64()
            .is_some_and(|elapsed| elapsed <= EVENT_DEADLINE.as_millis() as u64)
        && positive["initial_event_success"] == true
        && positive["watch_in_job"] == true
        && positive["watch_artifact_absent"] == true
        && exact_console_members(positive)
        && positive["outer"]["helper_exit_code"].as_i64() == Some(0)
        && positive["outer"]["job_active_processes"].as_u64() == Some(0)
        && no_send["send_calls"] == 0
        && no_send["ack"] == false
        && no_send["event_code"].as_u64() == Some(UNSEEN_EVENT as u64)
        && no_send["watch_alive_before_cleanup"] == true
        && no_send["watch_reaped"] == true
        && no_send["watch_artifact_absent"] == true
        && no_send["initial_event_success"] == true
        && no_send["watch_in_job"] == true
        && exact_console_members(no_send)
        && no_send["outer"]["helper_exit_code"].as_i64() == Some(0)
        && no_send["outer"]["job_active_processes"].as_u64() == Some(0)
}

fn exact_console_members(proof: &Value) -> bool {
    let Some(pids) = proof["console_process_ids"].as_array() else {
        return false;
    };
    let Some(helper) = proof["helper_pid"].as_u64() else {
        return false;
    };
    let Some(watch) = proof["watch_pid"].as_u64() else {
        return false;
    };
    let Some(outer) = proof["outer_pid"].as_u64() else {
        return false;
    };
    let mut actual = pids.iter().filter_map(Value::as_u64).collect::<Vec<_>>();
    if actual.len() != pids.len() {
        return false;
    }
    actual.sort_unstable();
    let mut expected = vec![helper, watch];
    expected.sort_unstable();
    actual == expected && !actual.contains(&outer)
}

#[path = "watch_cli_windows_ctrl_c/owned_process.rs"]
mod owned_process;

use owned_process::{
    join_bounded, spawn_bounded_drain, spawn_bounded_line_reader, terminate_child_and_reap,
    wait_child_until, Job, WatchChild,
};

fn console_snapshot() -> Result<ConsoleSnapshot, String> {
    let mut process_ids = [0u32; MAX_CONSOLE_PROCESSES];
    let count =
        unsafe { GetConsoleProcessList(process_ids.as_mut_ptr(), process_ids.len() as u32) };
    if count == 0 || count as usize > process_ids.len() {
        return Err(format!(
            "GetConsoleProcessList did not return a complete list (reported {count})"
        ));
    }
    Ok(ConsoleSnapshot {
        reported_count: count,
        process_ids: process_ids[..count as usize].to_vec(),
    })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open binary for SHA-256: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read binary for SHA-256: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> i32 {
    if ctrl_type == CTRL_C_EVENT {
        CTRL_EVENT.store(ctrl_type, Ordering::SeqCst);
        CTRL_ACK.store(true, Ordering::SeqCst);
        1
    } else {
        0
    }
}

#[derive(Clone, Copy)]
enum HelperMode {
    Positive,
    NoSend,
}

fn parse_helper_mode() -> Result<HelperMode, String> {
    match std::env::var("NU17_HELPER_MODE").as_deref() {
        Ok("positive") => Ok(HelperMode::Positive),
        Ok("no-send") => Ok(HelperMode::NoSend),
        _ => Err("NU17_HELPER_MODE must be positive or no-send".into()),
    }
}

fn helper_protocol() -> Result<Value, String> {
    let mode = parse_helper_mode()?;
    let outer_pid = std::env::var("NU17_OUTER_PID")
        .map_err(|error| format!("read outer PID: {error}"))?
        .parse::<u32>()
        .map_err(|error| format!("parse outer PID: {error}"))?;
    let expected_bin =
        PathBuf::from(std::env::var_os("NU17_EXPECTED_BIN").ok_or("NU17_EXPECTED_BIN missing")?);
    let expected_hash = std::env::var("NU17_EXPECTED_SHA256")
        .map_err(|error| format!("read expected binary hash: {error}"))?;
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_assura-full"))
        .canonicalize()
        .map_err(|error| format!("canonicalize helper binary: {error}"))?;
    let binary_hash = sha256_file(&bin)?;
    if bin != expected_bin || binary_hash != expected_hash {
        return Err("helper and outer process disagree on exact watch binary identity".into());
    }

    CTRL_ACK.store(false, Ordering::SeqCst);
    CTRL_EVENT.store(UNSEEN_EVENT, Ordering::SeqCst);
    if unsafe { SetConsoleCtrlHandler(None, 1) } == 0 {
        return Err("could not set inherited Ctrl-C ignore state".into());
    }
    if unsafe { SetConsoleCtrlHandler(None, 0) } == 0 {
        return Err("could not clear inherited Ctrl-C ignore state".into());
    }
    if unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), 1) } == 0 {
        return Err("could not register helper Ctrl-C acknowledgment handler".into());
    }

    let mut temp = Some(TempDir::new().map_err(|error| format!("create watch fixture: {error}"))?);
    std::fs::write(
        temp.as_ref().unwrap().path().join("README.md"),
        "# NU17 fixture\n",
    )
    .map_err(|error| format!("write watch fixture: {error}"))?;
    // libtest pretty output leaves its test-name prefix on the current line.
    // Start the protocol on its own line without relaxing exact marker matching.
    println!("\n{READY_MARKER}");
    io::stdout()
        .flush()
        .map_err(|error| format!("flush helper readiness: {error}"))?;
    let mut go = String::new();
    io::stdin()
        .read_line(&mut go)
        .map_err(|error| format!("read parent GO token: {error}"))?;
    if go.trim() != "GO" {
        return Err("parent did not release helper with exact GO token".into());
    }

    let mut watch = WatchChild::spawn(temp.as_ref().unwrap(), &bin)?;
    let helper_pid = unsafe { GetCurrentProcessId() };
    let watch_pid = watch.pid();
    let mut initial_event_ok = false;
    let mut send_calls = 0u32;
    let mut elapsed_ms = 0u128;
    let mut watch_exit_code = None;
    let mut watch_alive_before_cleanup = false;
    let mut watch_in_job = false;
    let mut console_process_ids = Vec::new();
    let mut protocol_error = None;

    let result = (|| -> Result<(), String> {
        watch.wait_initial_event(STARTUP_DEADLINE)?;
        initial_event_ok = true;
        println!(
            "{WATCH_READY_PREFIX}{}",
            json!({
                "watch_pid": watch_pid,
                "initial_event_success": initial_event_ok
            })
        );
        io::stdout()
            .flush()
            .map_err(|error| format!("flush watcher membership handshake: {error}"))?;
        let mut membership = String::new();
        io::stdin()
            .read_line(&mut membership)
            .map_err(|error| format!("read exact-job membership token: {error}"))?;
        if membership.trim() != "WATCH_IN_JOB" {
            return Err("parent did not confirm watcher in the exact owned job".into());
        }
        watch_in_job = true;

        match mode {
            HelperMode::Positive => {
                let snapshot = console_snapshot()?;
                let watch_live = watch.is_live()?;
                console_process_ids = snapshot.process_ids.clone();
                let mut started = None;
                let mut deadline = None;
                validate_then_send(
                    &snapshot,
                    helper_pid,
                    watch_pid,
                    outer_pid,
                    CTRL_ACK.load(Ordering::SeqCst),
                    watch_live,
                    watch_in_job,
                    || {
                        send_calls += 1;
                        let event_start = Instant::now();
                        started = Some(event_start);
                        deadline = Some(event_start + EVENT_DEADLINE);
                        unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) != 0 }
                    },
                )
                .map_err(|error| {
                    format!("pre-send safety gate rejected or send failed: {error:?}")
                })?;
                let event_deadline = deadline.expect("sender establishes the shared deadline");
                let event_start = started.expect("sender records the event start");
                loop {
                    if CTRL_ACK.load(Ordering::SeqCst) {
                        break;
                    }
                    if Instant::now() >= event_deadline {
                        return Err(
                            "helper did not acknowledge Ctrl-C before event deadline".into()
                        );
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                let event_code = CTRL_EVENT.load(Ordering::SeqCst);
                if event_code != CTRL_C_EVENT {
                    return Err(format!(
                        "handler acknowledged unexpected event {event_code}"
                    ));
                }
                let status = watch.wait_until_exit(event_deadline)?;
                elapsed_ms = event_start.elapsed().as_millis();
                watch_exit_code = status.code().map(i64::from);
                if status.code() != Some(0) {
                    return Err(format!(
                        "watch exited unsuccessfully after Ctrl-C: {status}"
                    ));
                }
            }
            HelperMode::NoSend => {
                let snapshot = console_snapshot()?;
                let watch_live = watch.is_live()?;
                console_process_ids = snapshot.process_ids.clone();
                watch_alive_before_cleanup = watch_live;
                validate_safety(
                    &snapshot,
                    helper_pid,
                    watch_pid,
                    outer_pid,
                    CTRL_ACK.load(Ordering::SeqCst),
                    watch_live,
                    watch_in_job,
                )
                .map_err(|error| format!("no-send control ownership check failed: {error:?}"))?;
                thread::sleep(NO_SEND_OBSERVATION);
                if CTRL_ACK.load(Ordering::SeqCst)
                    || CTRL_EVENT.load(Ordering::SeqCst) != UNSEEN_EVENT
                {
                    return Err("no-send control observed a Ctrl-C acknowledgment".into());
                }
                if !watch.is_live()? {
                    return Err("no-send control watch exited during observation".into());
                }
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        protocol_error = Some(error);
    }

    let cleanup_status = match watch.terminate_and_reap(CLEANUP_DEADLINE) {
        Ok(status) => Some(status.code().map(i64::from)),
        Err(error) => {
            if protocol_error.is_none() {
                protocol_error = Some(error);
            }
            None
        }
    };
    if watch.is_reaped() {
        if let Err(error) = watch.finish_readers() {
            protocol_error.get_or_insert(error);
        }
    } else {
        // Keep the fixture available until the outer owner terminates the exact job.
        std::mem::forget(temp.take().unwrap());
        protocol_error
            .get_or_insert_with(|| "watcher termination was not proven; fixture retained".into());
    }
    let artifact_absent = temp
        .as_ref()
        .is_some_and(|fixture| !fixture.path().join(".assura").join("watch").exists());
    if !artifact_absent && protocol_error.is_none() {
        protocol_error = Some("watch artifact exists after child termination".into());
    }
    if unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), 0) } == 0 && protocol_error.is_none() {
        protocol_error = Some("could not unregister helper Ctrl-C handler".into());
    }
    if let Some(error) = protocol_error {
        return Err(error);
    }
    Ok(json!({
        "mode": match mode { HelperMode::Positive => "positive", HelperMode::NoSend => "no-send" },
        "helper_pid": helper_pid,
        "watch_pid": watch_pid,
        "outer_pid": outer_pid,
        "console_process_ids": console_process_ids,
        "watch_binary": bin.display().to_string(),
        "watch_binary_sha256": binary_hash,
        "initial_event_success": initial_event_ok,
        "send_calls": send_calls,
        "event_code": CTRL_EVENT.load(Ordering::SeqCst),
        "ack": CTRL_ACK.load(Ordering::SeqCst),
        "elapsed_ms": elapsed_ms,
        "watch_exit_code": watch_exit_code,
        "watch_alive_before_cleanup": watch_alive_before_cleanup,
        "watch_reaped": cleanup_status.is_some(),
        "watch_artifact_absent": artifact_absent,
        "watch_in_job": watch_in_job
    }))
}

fn run_helper_case(mode: &str, expected_bin: &Path, expected_hash: &str) -> Result<Value, String> {
    let job = Job::new()?;
    let exe =
        std::env::current_exe().map_err(|error| format!("current test executable: {error}"))?;
    let mut child = Command::new(exe)
        .args([
            "--exact",
            "fixture_helper_entry",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("NU17_HELPER_MODE", mode)
        .env(
            "NU17_OUTER_PID",
            unsafe { GetCurrentProcessId() }.to_string(),
        )
        .env("NU17_EXPECTED_BIN", expected_bin)
        .env("NU17_EXPECTED_SHA256", expected_hash)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .map_err(|error| format!("spawn private-console helper: {error}"))?;
    if let Err(error) = job.assign(&child) {
        let cleanup = terminate_child_and_reap(&mut child, CLEANUP_DEADLINE);
        return Err(match cleanup {
            Ok(_) => error,
            Err(cleanup_error) => {
                format!("{error}; helper cleanup after assignment failure: {cleanup_error}")
            }
        });
    }
    let mut stdout = spawn_bounded_line_reader(child.stdout.take().unwrap());
    let (stderr_overflow, stderr_reader) = spawn_bounded_drain(child.stderr.take().unwrap());
    let mut protocol_error = None;
    let mut diagnostic_stdout = Vec::new();
    let mut diagnostic_stderr = Vec::new();
    let mut helper_proof = None;
    let mut ready_count = 0usize;
    let mut watch_ready_count = 0usize;
    let mut receipt_count = 0usize;
    let mut helper_status = None;
    let mut membership_confirmed = false;
    let mut confirmed_watch_pid = None;
    let mut deadline = Instant::now() + STARTUP_DEADLINE;

    while Instant::now() < deadline && helper_proof.is_none() && protocol_error.is_none() {
        match stdout
            .lines
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        {
            Ok(line) => {
                let remaining = (64 * 1024usize).saturating_sub(diagnostic_stdout.len());
                diagnostic_stdout.extend_from_slice(&line[..line.len().min(remaining)]);
                let text = String::from_utf8_lossy(&line);
                let text = text.trim();
                if text == READY_MARKER {
                    ready_count += 1;
                    if ready_count != 1 {
                        protocol_error = Some("helper emitted duplicate READY markers".into());
                        continue;
                    }
                    let sent = child
                        .stdin
                        .as_mut()
                        .ok_or_else(|| "helper stdin unavailable".to_string())
                        .and_then(|stdin| {
                            stdin
                                .write_all(b"GO\n")
                                .and_then(|_| stdin.flush())
                                .map_err(|error| format!("release helper GO: {error}"))
                        });
                    if let Err(error) = sent {
                        protocol_error = Some(error);
                    } else {
                        deadline =
                            Instant::now() + STARTUP_DEADLINE + EVENT_DEADLINE + CLEANUP_DEADLINE;
                    }
                } else if let Some(payload) = text.strip_prefix(WATCH_READY_PREFIX) {
                    watch_ready_count += 1;
                    if watch_ready_count != 1 || ready_count != 1 {
                        protocol_error =
                            Some("watch membership handshake was duplicated or premature".into());
                        continue;
                    }
                    let readiness: Value = match serde_json::from_str(payload) {
                        Ok(value) => value,
                        Err(error) => {
                            protocol_error = Some(format!("invalid watcher readiness: {error}"));
                            continue;
                        }
                    };
                    let Some(watch_pid) = readiness["watch_pid"]
                        .as_u64()
                        .and_then(|pid| u32::try_from(pid).ok())
                    else {
                        protocol_error = Some("watch readiness omitted a valid process ID".into());
                        continue;
                    };
                    if readiness["initial_event_success"] != true {
                        protocol_error = Some("watch initial event was not successful".into());
                        continue;
                    }
                    let belongs = match job.contains_pid(watch_pid) {
                        Ok(belongs) => belongs,
                        Err(error) => {
                            protocol_error = Some(error);
                            continue;
                        }
                    };
                    if !belongs {
                        protocol_error = Some("watcher is not in the exact owned job".into());
                        continue;
                    }
                    let sent = child
                        .stdin
                        .as_mut()
                        .ok_or_else(|| {
                            "helper stdin unavailable before membership token".to_string()
                        })
                        .and_then(|stdin| {
                            stdin
                                .write_all(b"WATCH_IN_JOB\n")
                                .and_then(|_| stdin.flush())
                                .map_err(|error| {
                                    format!("release exact-job membership token: {error}")
                                })
                        });
                    if let Err(error) = sent {
                        protocol_error = Some(error);
                    } else {
                        membership_confirmed = true;
                        confirmed_watch_pid = Some(watch_pid);
                        child.stdin.take();
                    }
                } else if let Some(payload) = text.strip_prefix(HELPER_RECEIPT_PREFIX) {
                    receipt_count += 1;
                    if !membership_confirmed {
                        protocol_error =
                            Some("helper receipt arrived before exact-job membership proof".into());
                        continue;
                    }
                    match serde_json::from_str::<Value>(payload) {
                        Ok(value)
                            if value["mode"] == mode
                                && value["watch_pid"].as_u64()
                                    == confirmed_watch_pid.map(u64::from)
                                && value["initial_event_success"] == true =>
                        {
                            helper_proof = Some(value)
                        }
                        Ok(_) => {
                            protocol_error = Some(
                                "helper receipt did not match confirmed watcher readiness".into(),
                            )
                        }
                        Err(error) => {
                            protocol_error = Some(format!("invalid helper receipt: {error}"))
                        }
                    }
                }
            }
            Err(error) => {
                protocol_error = Some(format!("waiting for helper protocol: {error}"));
            }
        }
    }
    if ready_count != 1 && protocol_error.is_none() {
        protocol_error = Some(format!(
            "expected one helper READY marker, got {ready_count}"
        ));
    }
    if watch_ready_count != 1 && protocol_error.is_none() {
        protocol_error = Some(format!(
            "expected one watcher membership handshake, got {watch_ready_count}"
        ));
    }
    if helper_proof.is_none() && protocol_error.is_none() {
        protocol_error = Some("helper did not emit its final receipt before deadline".into());
    }

    let direct_deadline = Instant::now() + CLEANUP_DEADLINE;
    while helper_status.is_none() && Instant::now() < direct_deadline {
        match child.try_wait() {
            Ok(Some(status)) => helper_status = Some(status),
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                protocol_error.get_or_insert_with(|| format!("helper try_wait: {error}"));
                break;
            }
        }
    }
    if helper_status.is_none() {
        match terminate_child_and_reap(&mut child, CLEANUP_DEADLINE) {
            Ok(status) => helper_status = Some(status),
            Err(direct_error) => {
                protocol_error
                    .get_or_insert_with(|| format!("bounded helper cleanup: {direct_error}"));
                if let Err(job_error) = job.terminate() {
                    protocol_error
                        .get_or_insert_with(|| format!("exact owned-job fallback: {job_error}"));
                }
                match wait_child_until(&mut child, Instant::now() + CLEANUP_DEADLINE) {
                    Ok(status) => helper_status = Some(status),
                    Err(error) => {
                        protocol_error.get_or_insert_with(|| {
                            format!("helper reap after owned-job termination: {error}")
                        });
                    }
                };
            }
        }
    }
    let job_active = match job.wait_empty() {
        Ok(active) => Some(active),
        Err(first) => {
            if let Err(error) = job.terminate() {
                protocol_error.get_or_insert_with(|| {
                    format!("exact owned-job termination fallback: {error}")
                });
            }
            match job.wait_empty() {
                Ok(active) => Some(active),
                Err(second) => {
                    protocol_error.get_or_insert_with(|| {
                        format!("{first}; owned job remains active after termination: {second}")
                    });
                    None
                }
            }
        }
    };

    if helper_status.is_some() && job_active == Some(0) {
        if let Err(error) = stdout.finish() {
            protocol_error.get_or_insert(error);
        }
        match join_bounded(stderr_reader, "helper stderr reader") {
            Ok(stderr) => {
                if !stderr.is_empty() {
                    protocol_error.get_or_insert_with(|| "helper emitted stderr".into());
                }
                diagnostic_stderr = stderr;
            }
            Err(error) => {
                protocol_error.get_or_insert(error);
            }
        }
    } else {
        protocol_error.get_or_insert_with(|| {
            "helper/job termination was not proven; reader joins skipped".into()
        });
    }
    while let Ok(line) = stdout.lines.try_recv() {
        let remaining = (64 * 1024usize).saturating_sub(diagnostic_stdout.len());
        diagnostic_stdout.extend_from_slice(&line[..line.len().min(remaining)]);
        let text = String::from_utf8_lossy(&line);
        let text = text.trim();
        if text == READY_MARKER {
            ready_count += 1;
        } else if text.starts_with(WATCH_READY_PREFIX) {
            watch_ready_count += 1;
        } else if let Some(payload) = text.strip_prefix(HELPER_RECEIPT_PREFIX) {
            receipt_count += 1;
            match serde_json::from_str::<Value>(payload) {
                Ok(value) if helper_proof.is_none() => helper_proof = Some(value),
                Ok(_) => {
                    protocol_error
                        .get_or_insert_with(|| "helper emitted duplicate receipts".into());
                }
                Err(error) => {
                    protocol_error
                        .get_or_insert_with(|| format!("invalid helper receipt: {error}"));
                }
            };
        }
    }
    if ready_count != 1 || watch_ready_count != 1 || receipt_count != 1 {
        protocol_error.get_or_insert_with(|| {
            format!("expected one READY, one watcher proof and one final receipt, got {ready_count}, {watch_ready_count}, {receipt_count}")
        });
    }
    if stdout.overflow.load(Ordering::SeqCst) || stderr_overflow.load(Ordering::SeqCst) {
        protocol_error.get_or_insert_with(|| "helper output exceeded bounded capture limit".into());
    }
    let helper_exit = helper_status
        .and_then(|status| status.code())
        .map(i64::from);
    if helper_exit != Some(0) {
        protocol_error
            .get_or_insert_with(|| format!("helper did not exit successfully: {helper_exit:?}"));
    }
    if helper_proof.is_none() {
        protocol_error.get_or_insert_with(|| "helper receipt missing".into());
    }
    if job_active != Some(0) {
        protocol_error
            .get_or_insert_with(|| format!("owned job is not proven empty: {job_active:?}"));
    }
    if let Some(error) = protocol_error {
        return Err(format!(
            "{error}; helper_pid={}, helper_exit={helper_exit:?}, job_active={job_active:?}, \
             ready_count={ready_count}, watch_ready_count={watch_ready_count}, receipt_count={receipt_count}; \
             bounded helper stdout={:?}; bounded helper stderr={:?}",
            child.id(),
            String::from_utf8_lossy(&diagnostic_stdout),
            String::from_utf8_lossy(&diagnostic_stderr)
        ));
    }
    let mut proof = helper_proof.unwrap();
    proof["outer"] = json!({
        "helper_exit_code": helper_exit,
        "job_active_processes": job_active.unwrap()
    });
    Ok(proof)
}

#[test]
#[ignore = "requires an isolated Windows console and owned job object"]
fn ctrl_c_stops_watch_in_owned_console() {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_assura-full"))
        .canonicalize()
        .expect("canonicalize assura-full binary");
    let hash = sha256_file(&bin).expect("hash assura-full binary");
    let positive = run_helper_case("positive", &bin, &hash)
        .unwrap_or_else(|error| panic!("positive Ctrl-C case failed: {error}"));
    let no_send = run_helper_case("no-send", &bin, &hash).expect("no-send control failed");
    let receipt = json!({
        "schema": "assura.nu17.ctrl-c.v1",
        "positive": positive,
        "no_send": no_send
    });
    assert!(
        final_receipt_is_valid(&receipt),
        "receipt validator rejected actual proof: {receipt}"
    );
    println!("{FINAL_RECEIPT_PREFIX}{receipt}");
}

#[test]
#[ignore = "invoked only by the owned outer test"]
fn fixture_helper_entry() {
    match helper_protocol() {
        Ok(receipt) => println!("{HELPER_RECEIPT_PREFIX}{receipt}"),
        Err(error) => panic!("NU17 helper protocol failed after cleanup: {error}"),
    }
}

fn snapshot(reported_count: u32, process_ids: &[u32]) -> ConsoleSnapshot {
    ConsoleSnapshot {
        reported_count,
        process_ids: process_ids.to_vec(),
    }
}

#[test]
fn safety_gate_rejects_foreign_and_unsafe_membership_before_sender() {
    let mut sends = 0;
    let cases = [
        (snapshot(2, &[10, 12]), false, true, true),
        (snapshot(3, &[10, 11]), false, true, true),
        (snapshot(1, &[10, 11]), false, true, true),
        (snapshot(3, &[10, 11, 99]), false, true, true),
        (snapshot(2, &[10, 11]), true, true, true),
        (snapshot(2, &[10, 11]), false, false, true),
        (snapshot(2, &[10, 11]), false, true, false),
    ];
    for (snapshot, ack, live, in_job) in cases {
        assert!(
            validate_then_send(&snapshot, 10, 11, 99, ack, live, in_job, || {
                sends += 1;
                true
            })
            .is_err()
        );
    }
    assert_eq!(sends, 0, "sender ran for rejected safety precondition");
}

#[test]
fn receipt_validator_rejects_missing_ack_event_exit_and_receipt() {
    let valid = json!({"schema":"assura.nu17.ctrl-c.v1", "positive":{"helper_pid":10,"watch_pid":11,"outer_pid":99,"console_process_ids":[10,11],"initial_event_success":true,"send_calls":1,"event_code":0,"ack":true,"watch_exit_code":0,"elapsed_ms":100,"watch_in_job":true,"watch_artifact_absent":true,"outer":{"helper_exit_code":0,"job_active_processes":0}}, "no_send":{"helper_pid":10,"watch_pid":11,"outer_pid":99,"console_process_ids":[10,11],"initial_event_success":true,"send_calls":0,"event_code":UNSEEN_EVENT,"ack":false,"watch_alive_before_cleanup":true,"watch_reaped":true,"watch_artifact_absent":true,"watch_in_job":true,"outer":{"helper_exit_code":0,"job_active_processes":0}}});
    assert!(final_receipt_is_valid(&valid));
    assert!(!final_receipt_is_valid(&Value::Null));
    for (field, value) in [
        ("ack", json!(false)),
        ("event_code", json!(CTRL_C_EVENT + 1)),
        ("watch_exit_code", Value::Null),
        ("initial_event_success", json!(false)),
        ("watch_in_job", json!(false)),
    ] {
        let mut invalid = valid.clone();
        invalid["positive"][field] = value;
        assert!(
            !final_receipt_is_valid(&invalid),
            "accepted invalid {field}"
        );
    }
    let mut missing_readiness = valid.clone();
    missing_readiness["no_send"]
        .as_object_mut()
        .unwrap()
        .remove("initial_event_success");
    assert!(!final_receipt_is_valid(&missing_readiness));
    let mut missing_job_proof = valid.clone();
    missing_job_proof["no_send"]
        .as_object_mut()
        .unwrap()
        .remove("watch_in_job");
    assert!(!final_receipt_is_valid(&missing_job_proof));
}

#[test]
fn helper_ready_marker_has_a_standalone_protocol_line() {
    let transcript = format!("running 1 test\ntest fixture_helper_entry ... \n{READY_MARKER}\n");
    let mut output = spawn_bounded_line_reader(std::io::Cursor::new(transcript.into_bytes()));
    output
        .finish()
        .expect("bounded transcript reader completes");
    let lines = output.lines.try_iter().collect::<Vec<_>>();
    assert_eq!(
        lines
            .iter()
            .filter(|line| String::from_utf8_lossy(line).trim() == READY_MARKER)
            .count(),
        1
    );
    let old_same_line = format!("test fixture_helper_entry ... {READY_MARKER}");
    assert_ne!(old_same_line.trim(), READY_MARKER);
}
