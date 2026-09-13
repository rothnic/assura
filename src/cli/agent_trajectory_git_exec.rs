//! Bounded Git subprocess execution for trajectory collection.

#[cfg(unix)]
use std::io;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) enum GitOutput {
    Text(String),
    Failed,
    Truncated,
    TimedOut,
}

const MAX_RUNTIME: Duration = Duration::from_secs(2);
static REFRESH_CANCELLED: AtomicBool = AtomicBool::new(false);

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

pub(super) fn terminate_process_tree(pid: u32) {
    if pid == 0 {
        return;
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
    }
    #[cfg(windows)]
    {
        let Ok(mut killer) = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            match killer.try_wait() {
                Ok(Some(_)) | Err(_) => break,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = killer.kill();
                    let _ = killer.wait();
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(5)),
            }
        }
    }
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
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if honor_cancellation && refresh_cancelled() => {
                terminate_process_tree(child.id());
                let _ = child.wait();
                if let Some(reader) = reader {
                    let _ = reader.join();
                }
                return GitOutput::TimedOut;
            }
            Ok(None) if Instant::now() >= deadline => {
                terminate_process_tree(child.id());
                let _ = child.wait();
                if let Some(reader) = reader {
                    let _ = reader.join();
                }
                return GitOutput::TimedOut;
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => {
                terminate_process_tree(child.id());
                let _ = child.wait();
                if let Some(reader) = reader {
                    let _ = reader.join();
                }
                return GitOutput::Failed;
            }
        }
    };
    let (output, read_ok) = reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default();
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
