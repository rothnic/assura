//! Bounded Git subprocess execution for trajectory collection.

#[cfg(unix)]
use std::io;
use std::io::Read;
use std::path::Path;
#[cfg(windows)]
use std::process::ChildStdout;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::thread;
#[cfg(not(windows))]
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) enum GitOutput {
    Text(String),
    Failed,
    Truncated,
    TimedOut,
}

const MAX_RUNTIME: Duration = Duration::from_secs(2);
const CLEANUP_GRACE: Duration = Duration::from_millis(250);
#[cfg(windows)]
const MAX_WINDOWS_FALLBACK_PROCESSES: usize = 4096;
static REFRESH_CANCELLED: AtomicBool = AtomicBool::new(false);

#[cfg(all(windows, test))]
thread_local! {
    static FORCE_WINDOWS_PROCESS_TREE_FALLBACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FORCE_WINDOWS_PROCESS_TREE_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn reset_refresh_cancellation() {
    REFRESH_CANCELLED.store(false, Ordering::Release);
}

pub(super) fn cancel_refresh_worker() {
    REFRESH_CANCELLED.store(true, Ordering::Release);
}

fn refresh_cancelled() -> bool {
    REFRESH_CANCELLED.load(Ordering::Acquire)
}

pub(super) fn isolate_process_tree(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        // A refresh worker owns its process group so timeout cleanup cannot
        // leave a shell or Git descendant holding the output pipe open.
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP);
    }
}

#[cfg(windows)]
fn force_windows_process_tree_fallback() -> bool {
    #[cfg(test)]
    {
        FORCE_WINDOWS_PROCESS_TREE_FALLBACK.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        false
    }
}

#[cfg(windows)]
fn terminate_windows_process(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if handle.is_null() {
        return false;
    }
    let terminated = unsafe { TerminateProcess(handle, 1) != 0 };
    unsafe {
        let _ = CloseHandle(handle);
    }
    terminated
}

#[cfg(windows)]
fn terminate_windows_process_tree_fallback(root_pid: u32) -> bool {
    use std::collections::HashMap;
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    #[cfg(test)]
    if FORCE_WINDOWS_PROCESS_TREE_FAILURE.with(std::cell::Cell::get) {
        return false;
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }
    let mut children = HashMap::<u32, Vec<u32>>::new();
    let mut process_count = 0;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) != 0 };
    let mut enumeration_ok = has_entry || unsafe { GetLastError() == ERROR_NO_MORE_FILES };
    while has_entry && process_count < MAX_WINDOWS_FALLBACK_PROCESSES {
        children
            .entry(entry.th32ParentProcessID)
            .or_default()
            .push(entry.th32ProcessID);
        process_count += 1;
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) != 0 };
        if !has_entry && unsafe { GetLastError() != ERROR_NO_MORE_FILES } {
            enumeration_ok = false;
        }
    }
    unsafe {
        let _ = CloseHandle(snapshot);
    }

    if !children.values().any(|pids| pids.contains(&root_pid)) {
        return false;
    }

    let mut pending = vec![(root_pid, 0usize)];
    let mut seen = vec![root_pid];
    let mut descendants = Vec::new();
    let mut complete = enumeration_ok && !has_entry;
    while let Some((parent_pid, depth)) = pending.pop() {
        let Some(child_pids) = children.get(&parent_pid) else {
            continue;
        };
        for &pid in child_pids {
            if pid == 0 || seen.contains(&pid) {
                continue;
            }
            seen.push(pid);
            descendants.push((pid, depth + 1));
            pending.push((pid, depth + 1));
            if seen.len() >= MAX_WINDOWS_FALLBACK_PROCESSES {
                complete = false;
                break;
            }
        }
    }
    // ponytail: one rare cleanup uses a bounded linear scan; use an indexed
    // process graph only if this ever becomes a hot path.
    descendants.sort_unstable_by_key(|(_, depth)| std::cmp::Reverse(*depth));
    let mut terminated = true;
    for (pid, _) in descendants {
        terminated &= terminate_windows_process(pid);
    }
    terminated &= terminate_windows_process(root_pid);
    complete && terminated
}

pub(super) fn terminate_process_tree(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(unix)]
    {
        let group = -(pid as i32);
        // SIGTERM gives cooperative descendants a chance to close resources;
        // SIGKILL is the bounded fallback for a stuck setup or Git process.
        unsafe {
            let _ = libc::kill(group, libc::SIGTERM);
        }
        thread::sleep(Duration::from_millis(25));
        unsafe {
            let _ = libc::kill(group, libc::SIGKILL);
            let _ = libc::kill(pid as i32, libc::SIGKILL);
        }
        true
    }
    #[cfg(windows)]
    {
        if force_windows_process_tree_fallback() {
            return terminate_windows_process_tree_fallback(pid);
        }
        let Ok(mut killer) = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return terminate_windows_process_tree_fallback(pid);
        };
        let mut taskkill_succeeded = false;
        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            match killer.try_wait() {
                Ok(Some(status)) => {
                    taskkill_succeeded = status.success();
                    break;
                }
                Err(_) => break,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = killer.kill();
                    let _ = killer.try_wait();
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(5)),
            }
        }
        if !taskkill_succeeded {
            return terminate_windows_process_tree_fallback(pid);
        }
        true
    }
}

#[cfg(windows)]
struct WindowsOutputReader {
    stdout: ChildStdout,
    output: Vec<u8>,
    limit: usize,
    done: bool,
    read_ok: bool,
}

#[cfg(windows)]
impl WindowsOutputReader {
    fn new(stdout: ChildStdout, limit: usize) -> Self {
        Self {
            stdout,
            output: Vec::with_capacity(limit.min(64 * 1024)),
            limit,
            done: false,
            read_ok: true,
        }
    }

    fn poll(&mut self) {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Pipes::PeekNamedPipe;

        if self.done {
            return;
        }
        let mut available = 0u32;
        let peeked = unsafe {
            PeekNamedPipe(
                self.stdout.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        };
        if peeked == 0 {
            self.done = true;
            return;
        }
        if available == 0 {
            return;
        }
        let remaining = self.limit.saturating_add(1) - self.output.len();
        let read_len = remaining.min(available as usize).min(64 * 1024);
        if read_len == 0 {
            self.done = true;
            return;
        }
        let mut chunk = vec![0; read_len];
        match self.stdout.read(&mut chunk) {
            Ok(0) => self.done = true,
            Ok(read) => {
                self.output.extend_from_slice(&chunk[..read]);
                if self.output.len() > self.limit {
                    self.done = true;
                }
            }
            Err(_) => {
                self.read_ok = false;
                self.done = true;
            }
        }
    }

    fn finish(self) -> (Vec<u8>, bool) {
        (self.output, self.read_ok)
    }
}

#[cfg(windows)]
type OutputReader = WindowsOutputReader;

#[cfg(not(windows))]
type OutputReader = JoinHandle<(Vec<u8>, bool)>;

#[cfg(windows)]
struct WindowsProcessJob {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl WindowsProcessJob {
    fn assign(child: &std::process::Child) -> Option<Self> {
        use std::mem::size_of;
        use std::os::windows::io::AsRawHandle;
        use std::ptr::null;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let handle = unsafe { CreateJobObjectW(null(), null()) };
        if handle.is_null() {
            return None;
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) != 0
        };
        let assigned =
            configured && unsafe { AssignProcessToJobObject(handle, child.as_raw_handle()) != 0 };
        if !assigned {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(handle);
            }
            return None;
        }
        Some(Self { handle })
    }

    fn terminate(&self) -> bool {
        unsafe { windows_sys::Win32::System::JobObjects::TerminateJobObject(self.handle, 1) != 0 }
    }
}

#[cfg(windows)]
impl Drop for WindowsProcessJob {
    fn drop(&mut self) {
        unsafe {
            let _ = windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

#[cfg(windows)]
type ProcessJob = WindowsProcessJob;

#[cfg(not(windows))]
type ProcessJob = ();

fn process_job(child: &std::process::Child) -> Option<ProcessJob> {
    #[cfg(windows)]
    {
        if force_windows_process_tree_fallback() {
            return None;
        }
        WindowsProcessJob::assign(child)
    }
    #[cfg(not(windows))]
    {
        let _ = child;
        None
    }
}

fn terminate_child_process(pid: u32, job: Option<&ProcessJob>) -> bool {
    #[cfg(windows)]
    if let Some(job) = job {
        if job.terminate() {
            true
        } else {
            terminate_process_tree(pid)
        }
    } else {
        terminate_process_tree(pid)
    }
    #[cfg(not(windows))]
    {
        let _ = job;
        terminate_process_tree(pid)
    }
}

fn cleanup_after_termination(mut child: std::process::Child, reader: Option<OutputReader>) {
    let deadline = Instant::now() + CLEANUP_GRACE;
    let mut exited = false;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => {
                exited = true;
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => break,
        }
    }
    if !exited {
        let _ = child.kill();
    }
    #[cfg(not(windows))]
    if let Some(reader) = reader {
        while !reader.is_finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        if reader.is_finished() {
            let _ = reader.join();
        }
    }
    #[cfg(windows)]
    drop(reader);
}

pub(super) fn run_git(
    repo_root: &Path,
    args: &[&str],
    limit: usize,
    deadline: Option<Instant>,
) -> GitOutput {
    let mut command = Command::new("git");
    command
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    run_bounded_until_internal(
        command,
        limit,
        refresh_timeout(),
        deadline,
        std::env::var_os("ASSURA_FEEDBACK_REFRESH_LOCK").is_none(),
        true,
    )
}

fn refresh_timeout() -> Duration {
    let configured = refresh_timeout_from(
        std::env::var("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS")
            .ok()
            .as_deref(),
    );
    let Some(deadline) = std::env::var("ASSURA_FEEDBACK_REFRESH_DEADLINE_MS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
    else {
        return configured;
    };
    let remaining = deadline.saturating_sub(now_millis());
    configured.min(Duration::from_millis(remaining.max(1) as u64))
}

fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn refresh_timeout_from(value: Option<&str>) -> Duration {
    value
        .and_then(|value| value.parse::<u64>().ok())
        .map(|millis| Duration::from_millis(millis.clamp(1, 10_000)))
        .unwrap_or(MAX_RUNTIME)
}

#[cfg(test)]
fn run_bounded(command: Command, limit: usize, timeout: Duration) -> GitOutput {
    run_bounded_until_internal(command, limit, timeout, None, true, true)
}

#[cfg(test)]
fn run_bounded_until(
    command: Command,
    limit: usize,
    timeout: Duration,
    deadline: Option<Instant>,
) -> GitOutput {
    run_bounded_until_internal(command, limit, timeout, deadline, true, true)
}

fn run_bounded_until_internal(
    mut command: Command,
    limit: usize,
    timeout: Duration,
    deadline: Option<Instant>,
    isolate: bool,
    honor_cancellation: bool,
) -> GitOutput {
    if isolate {
        isolate_process_tree(&mut command);
    }
    if honor_cancellation && refresh_cancelled() {
        return GitOutput::TimedOut;
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return GitOutput::Failed,
    };
    let process_job = process_job(&child);
    #[cfg(windows)]
    let mut reader = child
        .stdout
        .take()
        .map(|stdout| WindowsOutputReader::new(stdout, limit));
    #[cfg(not(windows))]
    let reader = child.stdout.take().map(|stdout| {
        thread::spawn(move || {
            let mut output = Vec::new();
            let result = stdout.take((limit + 1) as u64).read_to_end(&mut output);
            (output, result.is_ok())
        })
    });
    let timeout_deadline = Instant::now() + timeout;
    let deadline = deadline.map_or(timeout_deadline, |deadline| deadline.min(timeout_deadline));
    let status = loop {
        #[cfg(windows)]
        if let Some(reader) = reader.as_mut() {
            reader.poll();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if honor_cancellation && refresh_cancelled() => {
                let terminated = terminate_child_process(child.id(), process_job.as_ref());
                cleanup_after_termination(child, reader);
                return if terminated {
                    GitOutput::TimedOut
                } else {
                    GitOutput::Failed
                };
            }
            Ok(None) if Instant::now() >= deadline => {
                let terminated = terminate_child_process(child.id(), process_job.as_ref());
                cleanup_after_termination(child, reader);
                return if terminated {
                    GitOutput::TimedOut
                } else {
                    GitOutput::Failed
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => {
                let _ = terminate_child_process(child.id(), process_job.as_ref());
                cleanup_after_termination(child, reader);
                return GitOutput::Failed;
            }
        }
    };
    #[cfg(windows)]
    let (output, read_ok) = if let Some(mut reader) = reader {
        let reader_deadline = Instant::now() + CLEANUP_GRACE;
        while !reader.done && Instant::now() < reader_deadline {
            reader.poll();
            if !reader.done {
                thread::sleep(Duration::from_millis(5));
            }
        }
        if !reader.done {
            let terminated = terminate_child_process(child.id(), process_job.as_ref());
            cleanup_after_termination(child, Some(reader));
            return if terminated {
                GitOutput::TimedOut
            } else {
                GitOutput::Failed
            };
        }
        reader.finish()
    } else {
        (Vec::new(), true)
    };
    #[cfg(not(windows))]
    let (output, read_ok) = if let Some(reader) = reader {
        let reader_deadline = Instant::now() + CLEANUP_GRACE;
        while !reader.is_finished() && Instant::now() < reader_deadline {
            thread::sleep(Duration::from_millis(5));
        }
        if !reader.is_finished() {
            let terminated = terminate_child_process(child.id(), process_job.as_ref());
            cleanup_after_termination(child, Some(reader));
            return if terminated {
                GitOutput::TimedOut
            } else {
                GitOutput::Failed
            };
        }
        reader.join().unwrap_or_default()
    } else {
        (Vec::new(), true)
    };
    if !read_ok {
        return GitOutput::Failed;
    }
    if output.len() > limit {
        return GitOutput::Truncated;
    }
    match status {
        status if status.success() => GitOutput::Text(String::from_utf8_lossy(&output).into()),
        _ => GitOutput::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cancel_refresh_worker, refresh_timeout_from, reset_refresh_cancellation, run_bounded,
        run_bounded_until, run_bounded_until_internal, GitOutput,
    };
    #[cfg(windows)]
    use super::{FORCE_WINDOWS_PROCESS_TREE_FAILURE, FORCE_WINDOWS_PROCESS_TREE_FALLBACK};
    use std::fs;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    #[cfg(unix)]
    #[test]
    fn hanging_git_like_process_is_terminated_by_the_timeout() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "while :; do :; done"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        assert!(matches!(
            run_bounded(command, 1024, Duration::from_millis(20)),
            GitOutput::TimedOut
        ));
    }

    #[cfg(unix)]
    #[test]
    fn aggregate_deadline_caps_the_worker_timeout() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        assert!(matches!(
            run_bounded_until(
                command,
                1024,
                Duration::from_secs(2),
                Some(Instant::now() + Duration::from_millis(20)),
            ),
            GitOutput::TimedOut
        ));
    }

    #[cfg(unix)]
    #[test]
    fn refresh_cancellation_terminates_a_running_git_child() {
        reset_refresh_cancellation();
        let mut command = Command::new("sh");
        command
            .args(["-c", "while :; do :; done"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let canceller = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(20));
            cancel_refresh_worker();
        });
        assert!(matches!(
            run_bounded(command, 1024, Duration::from_secs(2)),
            GitOutput::TimedOut
        ));
        canceller.join().expect("cancellation thread joins");
        reset_refresh_cancellation();
    }

    #[cfg(windows)]
    fn windows_test_shell() -> &'static str {
        ["powershell.exe", "pwsh.exe"]
            .into_iter()
            .find(|candidate| {
                Command::new(candidate)
                    .args(["-NoProfile", "-Command", "exit 0"])
                    .status()
                    .is_ok_and(|status| status.success())
            })
            .expect("PowerShell is required for the Windows process-tree fixture")
    }

    #[cfg(windows)]
    fn assert_windows_timeout_kills_descendant(force_fallback: bool) {
        FORCE_WINDOWS_PROCESS_TREE_FALLBACK.with(|flag| flag.set(force_fallback));
        let shell = windows_test_shell();
        let directory = tempfile::tempdir().expect("process fixture");
        let pid_file = directory.path().join("child.pid");
        let escaped_pid_file = pid_file.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$child = Start-Process -FilePath $env:ComSpec -ArgumentList '/c','ping','-n','31','127.0.0.1' -PassThru; Set-Content -LiteralPath '{escaped_pid_file}' -Value $child.Id; Wait-Process -Id $child.Id"
        );
        let mut command = Command::new(shell);
        command
            .args(["-NoProfile", "-Command", &script])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let started = Instant::now();
        // Use a native descendant so the fixture can publish its PID before
        // the bounded timeout without another PowerShell startup.
        assert!(matches!(
            run_bounded(command, 1024, Duration::from_secs(2)),
            GitOutput::TimedOut
        ));
        let child_pid = (0..100).find_map(|_| {
            fs::read_to_string(&pid_file)
                .ok()
                .and_then(|pid| pid.trim().parse::<u32>().ok())
                .or_else(|| {
                    thread::sleep(Duration::from_millis(10));
                    None
                })
        });
        let child_pid = child_pid.expect("descendant pid is written before timeout");
        for _ in 0..100 {
            let alive = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {child_pid}"), "/FO", "CSV", "/NH"])
                .output()
                .expect("process probe");
            let listing = String::from_utf8_lossy(&alive.stdout);
            if !listing.contains(&format!("{child_pid}")) {
                assert!(started.elapsed() < Duration::from_secs(4));
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("timeout left descendant {child_pid} alive");
    }

    #[cfg(windows)]
    #[test]
    fn windows_timeout_kills_descendants_without_blocking_on_inherited_pipes() {
        assert_windows_timeout_kills_descendant(false);
        assert_windows_timeout_kills_descendant(true);
        FORCE_WINDOWS_PROCESS_TREE_FALLBACK.with(|flag| flag.set(false));
    }

    #[cfg(windows)]
    #[test]
    fn windows_cleanup_failure_is_reported_without_blocking() {
        FORCE_WINDOWS_PROCESS_TREE_FALLBACK.with(|flag| flag.set(true));
        FORCE_WINDOWS_PROCESS_TREE_FAILURE.with(|flag| flag.set(true));
        let mut command = Command::new(windows_test_shell());
        command
            .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let started = Instant::now();
        assert!(matches!(
            run_bounded(command, 1024, Duration::from_millis(500)),
            GitOutput::Failed
        ));
        assert!(started.elapsed() < Duration::from_secs(2));
        FORCE_WINDOWS_PROCESS_TREE_FAILURE.with(|flag| flag.set(false));
        FORCE_WINDOWS_PROCESS_TREE_FALLBACK.with(|flag| flag.set(false));
    }

    #[cfg(unix)]
    #[test]
    fn timeout_terminates_descendants_and_closes_the_pipe() {
        let directory = tempfile::tempdir().expect("process fixture");
        let pid_file = directory.path().join("child.pid");
        let mut command = Command::new("sh");
        command
            .args([
                "-c",
                "sleep 30 & echo $! > \"$ASSURA_TEST_CHILD_PID\"; wait",
            ])
            .env("ASSURA_TEST_CHILD_PID", &pid_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let runner = thread::spawn(move || {
            run_bounded_until_internal(command, 1024, Duration::from_secs(1), None, true, false)
        });
        let child_pid = (0..400).find_map(|_| {
            fs::read_to_string(&pid_file)
                .ok()
                .and_then(|pid| pid.trim().parse::<u32>().ok())
                .or_else(|| {
                    thread::sleep(Duration::from_millis(5));
                    None
                })
        });
        assert!(matches!(
            runner.join().expect("bounded runner joins"),
            GitOutput::TimedOut
        ));
        let child_pid = child_pid.expect("descendant pid is written before the bounded timeout");
        for _ in 0..50 {
            let alive = Command::new("kill")
                .args(["-0", &child_pid.to_string()])
                .stderr(Stdio::null())
                .status()
                .expect("process probe");
            if !alive.success() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("timeout left descendant {child_pid} alive");
    }

    #[test]
    fn refresh_timeout_is_bounded_and_configurable() {
        assert_eq!(refresh_timeout_from(Some("20")), Duration::from_millis(20));
        assert_eq!(refresh_timeout_from(Some("0")), Duration::from_millis(1));
        assert_eq!(refresh_timeout_from(Some("99999")), Duration::from_secs(10));
        assert_eq!(
            refresh_timeout_from(Some("invalid")),
            Duration::from_secs(2)
        );
    }
}
