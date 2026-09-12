//! Bounded Git subprocess execution for trajectory collection.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub(super) enum GitOutput {
    Text(String),
    Failed,
    Truncated,
}

pub(super) fn run_git(repo_root: &Path, args: &[&str], limit: usize) -> GitOutput {
    let mut child = match Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return GitOutput::Failed,
    };
    let mut output = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let mut bounded = stdout.take((limit + 1) as u64);
        if bounded.read_to_end(&mut output).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return GitOutput::Failed;
        }
    }
    if output.len() > limit {
        let _ = child.kill();
        let _ = child.wait();
        return GitOutput::Truncated;
    }
    match child.wait() {
        Ok(status) if status.success() => GitOutput::Text(String::from_utf8_lossy(&output).into()),
        _ => GitOutput::Failed,
    }
}
