//! Bounded Git subprocess execution for trajectory collection.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
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

pub(super) fn run_git(repo_root: &Path, args: &[&str], limit: usize) -> GitOutput {
    let mut command = Command::new("git");
    command
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    run_bounded(command, limit, refresh_timeout())
}

fn refresh_timeout() -> Duration {
    refresh_timeout_from(
        std::env::var("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS")
            .ok()
            .as_deref(),
    )
}

fn refresh_timeout_from(value: Option<&str>) -> Duration {
    value
        .and_then(|value| value.parse::<u64>().ok())
        .map(|millis| Duration::from_millis(millis.clamp(1, 10_000)))
        .unwrap_or(MAX_RUNTIME)
}

fn run_bounded(mut command: Command, limit: usize, timeout: Duration) -> GitOutput {
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
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                if let Some(reader) = reader {
                    let _ = reader.join();
                }
                return GitOutput::TimedOut;
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => {
                let _ = child.kill();
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
    use super::{run_bounded, GitOutput};
    use std::process::{Command, Stdio};
    use std::time::Duration;

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
