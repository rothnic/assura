//! Bounded process and output ownership for the native Windows watch test.

use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
    JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

const CLEANUP_DEADLINE: Duration = Duration::from_secs(5);
const MAX_STREAM_BYTES: usize = 64 * 1024;
const MAX_STREAM_LINE: usize = 8 * 1024;

#[derive(Debug)]
pub(crate) struct BoundedOutput {
    pub(crate) lines: Receiver<Vec<u8>>,
    pub(crate) overflow: Arc<AtomicBool>,
    reader: Option<JoinHandle<()>>,
}

impl BoundedOutput {
    pub(crate) fn finish(&mut self) -> Result<(), String> {
        if let Some(reader) = self.reader.take() {
            join_bounded(reader, "output reader")?;
        }
        if self.overflow.load(Ordering::SeqCst) {
            return Err("child output exceeded the bounded capture limit".into());
        }
        Ok(())
    }
}

pub(crate) fn spawn_bounded_line_reader<R: Read + Send + 'static>(reader: R) -> BoundedOutput {
    let (sender, lines) = mpsc::sync_channel(32);
    let overflow = Arc::new(AtomicBool::new(false));
    let thread_overflow = Arc::clone(&overflow);
    let reader = thread::spawn(move || {
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
        reader: Some(reader),
    }
}

pub(crate) fn spawn_bounded_drain<R: Read + Send + 'static>(
    reader: R,
) -> (Arc<AtomicBool>, JoinHandle<Vec<u8>>) {
    let overflow = Arc::new(AtomicBool::new(false));
    let thread_overflow = Arc::clone(&overflow);
    let reader = thread::spawn(move || {
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
    (overflow, reader)
}

pub(crate) fn join_bounded<T>(reader: JoinHandle<T>, description: &str) -> Result<T, String> {
    let deadline = Instant::now() + CLEANUP_DEADLINE;
    while !reader.is_finished() {
        if Instant::now() >= deadline {
            drop(reader);
            return Err(format!(
                "{description} did not finish before cleanup deadline"
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    reader.join().map_err(|_| format!("{description} panicked"))
}

pub(crate) struct Job(HANDLE);

impl Job {
    pub(crate) fn new() -> Result<Self, String> {
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

    pub(crate) fn assign(&self, child: &Child) -> Result<(), String> {
        let assigned =
            unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as HANDLE) != 0 };
        if assigned {
            Ok(())
        } else {
            Err("AssignProcessToJobObject(helper) failed".into())
        }
    }

    pub(crate) fn contains_pid(&self, pid: u32) -> Result<bool, String> {
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if process.is_null() {
            return Err(format!("OpenProcess({pid}) for job proof failed"));
        }
        let mut belongs = 0;
        let queried = unsafe { IsProcessInJob(process, self.0, &mut belongs) != 0 };
        unsafe { CloseHandle(process) };
        if queried {
            Ok(belongs != 0)
        } else {
            Err(format!("IsProcessInJob({pid}, owned job) failed"))
        }
    }

    pub(crate) fn active_processes(&self) -> Result<u32, String> {
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

    pub(crate) fn terminate(&self) -> Result<(), String> {
        if unsafe { TerminateJobObject(self.0, 1) } != 0 {
            Ok(())
        } else {
            Err("TerminateJobObject(owned job) failed".into())
        }
    }

    pub(crate) fn wait_empty(&self) -> Result<u32, String> {
        let deadline = Instant::now() + CLEANUP_DEADLINE;
        loop {
            let active = self.active_processes()?;
            if active == 0 {
                return Ok(active);
            }
            if Instant::now() >= deadline {
                return Err(format!("owned job still has {active} active process(es)"));
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

pub(crate) struct WatchChild {
    child: Child,
    pub(crate) stdout: BoundedOutput,
    stderr_overflow: Arc<AtomicBool>,
    stderr_reader: Option<JoinHandle<Vec<u8>>>,
    reaped: bool,
}

impl WatchChild {
    pub(crate) fn spawn(project: &TempDir, bin: &std::path::Path) -> Result<Self, String> {
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
        let (stderr_overflow, stderr_reader) = spawn_bounded_drain(child.stderr.take().unwrap());
        Ok(Self {
            child,
            stdout,
            stderr_overflow,
            stderr_reader: Some(stderr_reader),
            reaped: false,
        })
    }

    pub(crate) fn pid(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn wait_initial_event(&self, timeout: Duration) -> Result<(), String> {
        let line = self
            .stdout
            .lines
            .recv_timeout(timeout)
            .map_err(|error| format!("watch initial event timeout/disconnect: {error}"))?;
        let event: Value = serde_json::from_slice(&line)
            .map_err(|error| format!("watch emitted invalid JSON before initial event: {error}"))?;
        if event["report"]["success"] == true {
            Ok(())
        } else {
            Err(format!("watch initial event was not successful: {event}"))
        }
    }

    pub(crate) fn is_live(&mut self) -> Result<bool, String> {
        match self
            .child
            .try_wait()
            .map_err(|error| format!("watch try_wait: {error}"))?
        {
            Some(_) => {
                self.reaped = true;
                Ok(false)
            }
            None => Ok(true),
        }
    }

    pub(crate) fn wait_until_exit(&mut self, deadline: Instant) -> Result<ExitStatus, String> {
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

    pub(crate) fn terminate_and_reap(&mut self, timeout: Duration) -> Result<ExitStatus, String> {
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
        let status = wait_child_until(&mut self.child, Instant::now() + timeout)?;
        self.reaped = true;
        Ok(status)
    }

    pub(crate) fn is_reaped(&self) -> bool {
        self.reaped
    }

    pub(crate) fn finish_readers(&mut self) -> Result<Vec<u8>, String> {
        if !self.reaped {
            return Err(
                "watch readers cannot be joined before watcher termination is proven".into(),
            );
        }
        self.stdout.finish()?;
        let stderr = if let Some(reader) = self.stderr_reader.take() {
            join_bounded(reader, "watch stderr reader")?
        } else {
            Vec::new()
        };
        if self.stderr_overflow.load(Ordering::SeqCst) {
            Err(format!(
                "watch stderr exceeded the bounded capture limit: {}",
                String::from_utf8_lossy(&stderr)
            ))
        } else {
            Ok(stderr)
        }
    }
}

impl Drop for WatchChild {
    fn drop(&mut self) {
        if !self.reaped && self.terminate_and_reap(CLEANUP_DEADLINE).is_err() {
            return;
        }
        let _ = self.finish_readers();
    }
}

pub(crate) fn wait_child_until(child: &mut Child, deadline: Instant) -> Result<ExitStatus, String> {
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("child try_wait: {error}"))?
        {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err("child did not exit before cleanup deadline".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub(crate) fn terminate_child_and_reap(
    child: &mut Child,
    timeout: Duration,
) -> Result<ExitStatus, String> {
    if let Some(status) = child
        .try_wait()
        .map_err(|error| format!("child cleanup try_wait: {error}"))?
    {
        return Ok(status);
    }
    child
        .kill()
        .map_err(|error| format!("direct child terminate failed: {error}"))?;
    wait_child_until(child, Instant::now() + timeout)
}
