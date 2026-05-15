//! Environment metadata collection for performance evidence.

use serde::Serialize;
use std::process::Command;

/// Environment and toolchain metadata attached to performance evidence.
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceEnvironment {
    /// Operating system identifier reported by the Rust target.
    pub os: String,
    /// CPU architecture identifier reported by the Rust target.
    pub arch: String,
    /// Rust compiler version used to build or run Assura.
    pub rust_version: String,
    /// Node.js version used for LS-Lint execution.
    pub node_version: String,
    /// npm version used for LS-Lint execution.
    pub npm_version: String,
}

pub(super) fn collect_environment() -> PerformanceEnvironment {
    PerformanceEnvironment {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rust_version: command_value("rustc", ["--version"]),
        node_version: command_value("node", ["--version"]),
        npm_version: command_value("npm", ["--version"]),
    }
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
