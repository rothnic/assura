//! Environment metadata collection for performance evidence.

use serde::Serialize;
use std::fs;
use std::process::Command;

/// Environment and toolchain metadata attached to performance evidence.
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceEnvironment {
    /// Operating system identifier reported by the Rust target.
    pub os: String,
    /// CPU architecture identifier reported by the Rust target.
    pub arch: String,
    /// Best-effort CPU model or host CPU description.
    pub cpu_model: String,
    /// Logical CPU count available to this process.
    pub logical_cpu_count: usize,
    /// Best-effort total system memory in bytes.
    pub total_memory_bytes: Option<u64>,
    /// Rust compiler version used to build or run Assura.
    pub rust_version: String,
    /// Node.js version available when installing the pinned LS-Lint package.
    pub node_version: String,
    /// npm version used to install the pinned LS-Lint package.
    pub npm_version: String,
}

pub(super) fn collect_environment() -> PerformanceEnvironment {
    PerformanceEnvironment {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_model: cpu_model(),
        logical_cpu_count: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        total_memory_bytes: total_memory_bytes(),
        rust_version: command_value("rustc", ["--version"]),
        node_version: command_value("node", ["--version"]),
        npm_version: command_value("npm", ["--version"]),
    }
}

fn cpu_model() -> String {
    let sysctl = command_value("sysctl", ["-n", "machdep.cpu.brand_string"]);
    if !sysctl.starts_with("unavailable:") && sysctl != "unknown" {
        return sysctl;
    }

    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|value| value.split_once(':').map(|(_, model)| model.trim()))
                    .filter(|model| !model.is_empty())
                    .map(str::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn total_memory_bytes() -> Option<u64> {
    if let Ok(output) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(bytes) = value.parse::<u64>() {
                return Some(bytes);
            }
        }
    }

    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("MemTotal:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|kb| kb.parse::<u64>().ok())
                    .map(|kb| kb * 1024)
            })
        })
}

fn command_value<const N: usize>(program: &str, args: [&str; N]) -> String {
    let output = Command::new(program).args(args).output();
    match output {
        Ok(output) if output.status.success() => {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() {
                "unknown".to_string()
            } else {
                value
            }
        }
        Ok(output) => format!(
            "unavailable: exit {:?}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ),
        Err(error) => format!("unavailable: {error}"),
    }
}
