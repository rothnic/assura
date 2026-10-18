#![cfg(windows)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::Console::{
    CTRL_C_EVENT, GenerateConsoleCtrlEvent, GetConsoleProcessList, SetConsoleCtrlHandler,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{CREATE_NEW_CONSOLE, GetCurrentProcessId};

const READY_MARKER: &str = "NU17_HELPER_READY";
const HELPER_RECEIPT_PREFIX: &str = "NU17_HELPER_RECEIPT ";
const FINAL_RECEIPT_PREFIX: &str = "NU17_CTRL_C_RECEIPT ";
const EVENT_DEADLINE: Duration = Duration::from_secs(10);
const STARTUP_DEADLINE: Duration = Duration::from_secs(10);
const NO_SEND_OBSERVATION: Duration = Duration::from_millis(500);
const CLEANUP_DEADLINE: Duration = Duration::from_secs(5);
const MAX_STREAM_BYTES: usize = 64 * 1024;
const MAX_STREAM_LINE: usize = 8 * 1024;
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

#[derive(Debug)]
struct BoundedOutput {
    lines: Receiver<Vec<u8>>,
    overflow: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

fn spawn_bounded_line_reader<R: Read + Send + 'static>(reader: R) -> BoundedOutput {
    let (sender, lines) = mpsc::sync_channel(32);
    let overflow = Arc::new(AtomicBool::new(false));
    let thread_overflow = Arc::clone(&overflow);
    let thread = thread::spawn(move || {
        let mut reader = reader;
        let mut total = 0usize;
        let mut line = Vec::new();
        let mut chunk = [0u8; 2048];
        loop {
            let count = match reader.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(count) => count,
            };
            total = total.saturating_add(count);
            if total > MAX_STREAM_BYTES {
                thread_overflow.store(true, Ordering::SeqCst);
            }
            for byte in &chunk[..count] {
                if line.len() < MAX_STREAM_LINE {
                    line.push(*byte);
                } else {
                    thread_overflow.store(true, Ordering::SeqCst);
                }
                if *byte == b'\n' {
                    if !thread_overflow.load(Ordering::SeqCst) {
                        match sender.try_send(std::mem::take(&mut line)) {
                            Ok(()) => {}
                            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                                thread_overflow.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                    line.clear();
                }
            }
        }
    });
    BoundedOutput {
        lines,
        overflow,
        thread: Some(thread),
    }
}

fn spawn_bounded_drain<R: Read + Send + 'static>(
    reader: R,
) -> (Arc<AtomicBool>, JoinHandle<Vec<u8>>) {
    let overflow = Arc::new(AtomicBool::new(false));
    let thread_overflow = Arc::clone(&overflow);
    let thread = thread::spawn(move || {
        let mut reader = reader;
        let mut total = 0usize;
        let mut captured = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    total = total.saturating_add(count);
                    let remaining = MAX_STREAM_BYTES.saturating_sub(captured.len());
                    captured.extend_from_slice(&buffer[..count.min(remaining)]);
                    if total > MAX_STREAM_BYTES {
                        thread_overflow.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
        captured
    });
    (overflow, thread)
}

struct Job(HANDLE);

impl Job {
    fn new() -> Result<Self, String> {
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err("CreateJobObjectW failed".into());
        }
        let job = Self(handle);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) != 0
        };
        if configured {
            Ok(job)
        } else {
            Err("SetInformationJobObject(KILL_ON_JOB_CLOSE) failed".into())
        }
    }

    fn active_processes(&self) -> Result<u32, String> {
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        let queried = unsafe {
            QueryInformationJobObject(
                self.0,
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                std::ptr::null_mut(),
            ) != 0
        };
        if queried {
            Ok(info.ActiveProcesses)
        } else {
            Err("QueryInformationJobObject failed".into())
        }
    }

    fn wait_empty(&self) -> Result<u32, String> {
        let deadline = Instant::now() + CLEANUP_DEADLINE;
        loop {
            let active = self.active_processes()?;
            if active == 0 {
                return Ok(active);
            }
            if Instant::now() >= deadline {
                return Err(format!("job still owns {active} active process(es)"));
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

struct WatchChild {
    child: Child,
    stdout: BoundedOutput,
    stderr_overflow: Arc<AtomicBool>,
    stderr_thread: Option<JoinHandle<Vec<u8>>>,
    reaped: bool,
}

impl WatchChild {
    fn spawn(project: &TempDir) -> Result<Self, String> {
        let bin = PathBuf::from(env!("CARGO_BIN_EXE_assura-full"))
            .canonicalize()
            .map_err(|error| format!("canonicalize watch binary: {error}"))?;
        let mut command = Command::new(bin);
        command
            .arg("watch")
            .arg(project.path())
            .args(["--format", "json", "--debounce", "100"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|error| format!("spawn watch: {error}"))?;
        let stdout = spawn_bounded_line_reader(child.stdout.take().unwrap());
        let (stderr_overflow, stderr_thread) = spawn_bounded_drain(child.stderr.take().unwrap());
        Ok(Self {
            child,
            stdout,
            stderr_overflow,
            stderr_thread: Some(stderr_thread),
            reaped: false,
        })
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn wait_initial_event(&self) -> Result<(), String> {
        let line = self
            .stdout
            .lines
            .recv_timeout(STARTUP_DEADLINE)
            .map_err(|error| format!("watch initial event timeout/disconnect: {error}"))?;
        let event: Value = serde_json::from_slice(&line)
            .map_err(|error| format!("watch emitted invalid JSON before initial event: {error}"))?;
        if event["report"]["success"] == true {
            Ok(())
        } else {
            Err(format!("watch initial event was not successful: {event}"))
        }
    }

    fn is_live(&mut self) -> Result<bool, String> {
        self.child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|error| format!("watch try_wait: {error}"))
    }

    fn in_a_job(&self) -> Result<bool, String> {
        let mut in_job = 0;
        let ok = unsafe {
            windows_sys::Win32::System::JobObjects::IsProcessInJob(
                self.child.as_raw_handle() as HANDLE,
                std::ptr::null_mut(),
                &mut in_job,
            ) != 0
        };
        if ok {
            Ok(in_job != 0)
        } else {
            Err("IsProcessInJob(watch, current job) failed".into())
        }
    }

    fn wait_until_exit(&mut self, deadline: Instant) -> Result<ExitStatus, String> {
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| format!("watch try_wait: {error}"))?
            {
                self.reaped = true;
                return Ok(status);
            }
            if Instant::now() >= deadline {
                return Err("watch did not exit before the shared Ctrl-C deadline".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn terminate_and_reap(&mut self) -> Result<ExitStatus, String> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|error| format!("watch cleanup try_wait: {error}"))?
        {
            self.reaped = true;
            return Ok(status);
        }
        self.child
            .kill()
            .map_err(|error| format!("direct watch terminate failed: {error}"))?;
        let status = self
            .child
            .wait()
            .map_err(|error| format!("direct watch wait/reap failed: {error}"))?;
        self.reaped = true;
        Ok(status)
    }

    fn finish_readers(&mut self) -> Result<(), String> {
        if let Some(reader) = self.stdout.thread.take() {
            reader
                .join()
                .map_err(|_| "watch stdout reader panicked".to_string())?;
        }
        let stderr = if let Some(reader) = self.stderr_thread.take() {
            reader
                .join()
                .map_err(|_| "watch stderr reader panicked".to_string())?
        } else {
            Vec::new()
        };
        if self.stdout.overflow.load(Ordering::SeqCst)
            || self.stderr_overflow.load(Ordering::SeqCst)
        {
            Err(format!(
                "watch output exceeded the bounded capture limit; stderr={}",
                String::from_utf8_lossy(&stderr)
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for WatchChild {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.terminate_and_reap();
        }
        let _ = self.finish_readers();
    }
}

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

    let temp = TempDir::new().map_err(|error| format!("create watch fixture: {error}"))?;
    std::fs::write(temp.path().join("README.md"), "# NU17 fixture\n")
        .map_err(|error| format!("write watch fixture: {error}"))?;
    println!("{READY_MARKER}");
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

    let mut watch = WatchChild::spawn(&temp)?;
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
        watch.wait_initial_event()?;
        initial_event_ok = true;
        match mode {
            HelperMode::Positive => {
                let snapshot = console_snapshot()?;
                let watch_live = watch.is_live()?;
                watch_in_job = watch.in_a_job()?;
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
                watch_in_job = watch.in_a_job()?;
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
                let status = watch.terminate_and_reap()?;
                watch_exit_code = status.code().map(i64::from);
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        protocol_error = Some(error);
    }

    // Always terminate/reap the direct watcher before releasing its fixture.
    let cleanup_status = match watch.terminate_and_reap() {
        Ok(status) => Some(status.code().map(i64::from)),
        Err(error) => {
            if protocol_error.is_none() {
                protocol_error = Some(error);
            }
            None
        }
    };
    let reader_result = watch.finish_readers();
    if let Err(error) = reader_result {
        if protocol_error.is_none() {
            protocol_error = Some(error);
        }
    }
    let artifact_absent = !temp.path().join(".assura").join("watch").exists();
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

fn wait_for_job_empty(job: &Job) -> Result<u32, String> {
    match job.wait_empty() {
        Ok(active) => Ok(active),
        Err(first) => {
            unsafe { windows_sys::Win32::System::JobObjects::TerminateJobObject(job.0, 1) };
            job.wait_empty().map_err(|second| {
                format!("{first}; cleanup termination did not empty job: {second}")
            })
        }
    }
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
    let mut stdout = spawn_bounded_line_reader(child.stdout.take().unwrap());
    let (stderr_overflow, stderr_thread) = spawn_bounded_drain(child.stderr.take().unwrap());
    let mut protocol_error = None;
    let mut helper_proof = None;
    let mut ready = false;
    let mut ready_count = 0usize;
    let mut receipt_count = 0usize;
    let mut helper_status = None;
    let helper_deadline = Instant::now() + STARTUP_DEADLINE;

    let helper_handle = child.as_raw_handle() as HANDLE;
    if unsafe { AssignProcessToJobObject(job.0, helper_handle) } == 0 {
        protocol_error =
            Some("AssignProcessToJobObject(helper) failed; helper was not released".to_string());
    } else {
        let mut deadline = helper_deadline;
        let mut go_sent = false;
        while Instant::now() < deadline && helper_proof.is_none() && protocol_error.is_none() {
            match stdout
                .lines
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(line) => {
                    let text = String::from_utf8_lossy(&line);
                    let text = text.trim();
                    if text == READY_MARKER {
                        ready_count += 1;
                        if ready_count > 1 {
                            protocol_error = Some("helper emitted duplicate READY markers".into());
                            continue;
                        }
                        ready = true;
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
                            child.stdin.take();
                            go_sent = true;
                            deadline = Instant::now()
                                + STARTUP_DEADLINE
                                + EVENT_DEADLINE
                                + CLEANUP_DEADLINE;
                        }
                    } else if let Some(payload) = text.strip_prefix(HELPER_RECEIPT_PREFIX) {
                        receipt_count += 1;
                        protocol_error = Some(if !go_sent {
                            "helper receipt arrived before the parent released GO".to_string()
                        } else {
                            match serde_json::from_str::<Value>(payload) {
                                Ok(value) if value["mode"] == mode => {
                                    helper_proof = Some(value);
                                    continue;
                                }
                                Ok(_) => "helper receipt mode did not match request".to_string(),
                                Err(error) => format!("invalid helper receipt: {error}"),
                            }
                        });
                    }
                }
                Err(error) => {
                    protocol_error = Some(format!("waiting for helper protocol: {error}"));
                    break;
                }
            }
        }
        if !ready && protocol_error.is_none() {
            protocol_error = Some("helper did not become ready before deadline".into());
        }
        if go_sent && helper_proof.is_none() && protocol_error.is_none() {
            protocol_error = Some("helper did not emit its final receipt before deadline".into());
        }
    }

    if helper_proof.is_some() && protocol_error.is_none() {
        let exit_deadline = Instant::now() + CLEANUP_DEADLINE;
        while Instant::now() < exit_deadline {
            match child.try_wait() {
                Ok(Some(status)) => {
                    helper_status = Some(status);
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    protocol_error = Some(format!("wait for helper exit: {error}"));
                    break;
                }
            }
        }
        if helper_status.is_none() && protocol_error.is_none() {
            protocol_error = Some("helper did not exit after emitting its receipt".into());
        }
    }
    if helper_status.is_none() && child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }
    helper_status = helper_status.or_else(|| child.wait().ok());
    if let Some(reader) = stdout.thread.take() {
        if reader.join().is_err() {
            protocol_error.get_or_insert_with(|| "helper stdout reader panicked".into());
        }
    }
    let helper_stderr = stderr_thread.join().unwrap_or_default();
    while let Ok(line) = stdout.lines.try_recv() {
        let text = String::from_utf8_lossy(&line);
        let text = text.trim();
        if text == READY_MARKER {
            ready_count += 1;
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
    if ready_count != 1 || receipt_count != 1 {
        protocol_error.get_or_insert_with(|| {
            format!(
                "expected one READY and one final receipt, got {ready_count} and {receipt_count}"
            )
        });
    }
    if stdout.overflow.load(Ordering::SeqCst) || stderr_overflow.load(Ordering::SeqCst) {
        protocol_error.get_or_insert_with(|| "helper output exceeded bounded capture limit".into());
    }
    let job_active = match wait_for_job_empty(&job) {
        Ok(active) => Some(active),
        Err(error) => {
            protocol_error.get_or_insert(error);
            None
        }
    };
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
    if !helper_stderr.is_empty() {
        if let Some(error) = protocol_error.take() {
            protocol_error = Some(format!(
                "{error}; helper stderr: {}",
                String::from_utf8_lossy(&helper_stderr)
            ));
        }
    }
    if job_active != Some(0) {
        protocol_error
            .get_or_insert_with(|| format!("owned job is not proven empty: {job_active:?}"));
    }
    if let Some(error) = protocol_error {
        return Err(error);
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

#[rustfmt::skip]
fn snapshot(reported_count: u32, process_ids: &[u32]) -> ConsoleSnapshot { ConsoleSnapshot { reported_count, process_ids: process_ids.to_vec() } }

#[test]
#[rustfmt::skip]
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
    for (snapshot, ack, live, in_job) in cases { assert!(validate_then_send(&snapshot, 10, 11, 99, ack, live, in_job, || { sends += 1; true }).is_err()); }
    assert_eq!(sends, 0, "sender ran for rejected safety precondition");
}

#[test]
#[rustfmt::skip]
fn receipt_validator_rejects_missing_ack_event_exit_and_receipt() {
    let valid = json!({"schema":"assura.nu17.ctrl-c.v1", "positive":{"helper_pid":10,"watch_pid":11,"outer_pid":99,"console_process_ids":[10,11],"send_calls":1,"event_code":0,"ack":true,"watch_exit_code":0,"elapsed_ms":100,"watch_in_job":true,"watch_artifact_absent":true,"outer":{"helper_exit_code":0,"job_active_processes":0}}, "no_send":{"helper_pid":10,"watch_pid":11,"outer_pid":99,"console_process_ids":[10,11],"send_calls":0,"event_code":UNSEEN_EVENT,"ack":false,"watch_alive_before_cleanup":true,"watch_reaped":true,"watch_artifact_absent":true,"outer":{"helper_exit_code":0,"job_active_processes":0}}});
    assert!(final_receipt_is_valid(&valid));
    assert!(!final_receipt_is_valid(&Value::Null));
    for (field, value) in [("ack", json!(false)), ("event_code", json!(CTRL_C_EVENT + 1)), ("watch_exit_code", Value::Null)] {
        let mut invalid = valid.clone();
        invalid["positive"][field] = value;
        assert!(!final_receipt_is_valid(&invalid), "accepted invalid {field}");
    }
}
