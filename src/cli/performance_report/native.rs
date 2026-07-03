//! Native Assura performance suite rows.
//!
//! The native suite measures full-CLI capability surfaces through the sibling
//! `assura-full` binary so the lightweight LS-Lint comparison binary remains
//! focused on structure parity.

use super::assura_cli::{prepare_assura_full_cli, PreparedAssuraCli};
use super::fixtures::materialize_fixture;
use super::native_fixtures::native_scenarios;
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use std::process::{Command, Output, Stdio};
use std::time::Instant;

struct NativeCommand {
    row_family: &'static str,
    tool_name: &'static str,
    args: &'static [&'static str],
}

const NATIVE_COMMANDS: &[NativeCommand] = &[
    NativeCommand {
        row_family: "native:content-check-cli",
        tool_name: "assura-full check --format json",
        args: &["check", ".", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:content-collections-cli",
        tool_name: "assura-full content collections",
        args: &["content", "collections", ".", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:content-instances-cli",
        tool_name: "assura-full content instances",
        args: &["content", "instances", "goals", ".", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:content-show-cli",
        tool_name: "assura-full content show",
        args: &[
            "content",
            "show",
            "goals",
            "goal-0000",
            ".",
            "--format",
            "json",
        ],
    },
    NativeCommand {
        row_family: "native:content-expand-cli",
        tool_name: "assura-full content expand",
        args: &[
            "content",
            "expand",
            "goals",
            "goal-0000",
            ".",
            "--format",
            "json",
        ],
    },
    NativeCommand {
        row_family: "native:content-search-cli",
        tool_name: "assura-full content search",
        args: &["content", "search", "runtime", ".", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:content-missing-relations-cli",
        tool_name: "assura-full content missing-relations",
        args: &["content", "missing-relations", ".", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:content-references-cli",
        tool_name: "assura-full content references",
        args: &["content", "references", ".", "--all", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:context-pack-cli",
        tool_name: "assura-full content context-pack",
        args: &[
            "content",
            "context-pack",
            ".",
            "--collection",
            "goals",
            "--id",
            "goal-0000",
            "--format",
            "json",
        ],
    },
    NativeCommand {
        row_family: "native:agent-query-keyword-search-cli",
        tool_name: "assura-full content agent-query keyword-search",
        args: &[
            "content",
            "agent-query",
            "keyword-search",
            ".",
            "--text",
            "runtime",
            "--format",
            "json",
        ],
    },
    NativeCommand {
        row_family: "native:agent-query-missing-relations-cli",
        tool_name: "assura-full content agent-query missing-relations",
        args: &[
            "content",
            "agent-query",
            "missing-relations",
            ".",
            "--format",
            "json",
        ],
    },
    NativeCommand {
        row_family: "native:markdown-safe-fix-dry-run-cli",
        tool_name: "assura-full fix markdown --dry-run",
        args: &["fix", "markdown", ".", "--dry-run", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:session-agent-context-cli",
        tool_name: "assura-full content agent-context",
        args: &["content", "agent-context", ".", "--format", "json"],
    },
    NativeCommand {
        row_family: "native:daemon-status-cli",
        tool_name: "assura-full daemon status",
        args: &["daemon", "status", ".", "--format", "json"],
    },
];

pub(super) fn measure_native_rows(
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
) -> Result<(ToolAvailability, Vec<PerformanceResultRow>), String> {
    let assura_full = prepare_assura_full_cli();
    let suite_status = ToolAvailability {
        available: assura_full.status.available,
        version: assura_full.status.version.clone(),
        blocker: assura_full.status.blocker.clone(),
    };
    let mut rows = Vec::new();
    for scenario in native_scenarios() {
        let fixture = materialize_fixture(scenario)?;
        for command in NATIVE_COMMANDS {
            rows.push(measure_native_command(
                &fixture,
                iterations,
                timestamp,
                commit_sha,
                branch,
                environment,
                baseline_id,
                &suite_status,
                &assura_full,
                command,
            ));
        }
        let _ = std::fs::remove_dir_all(&fixture.root);
    }
    Ok((suite_status, rows))
}

// allow-reason: native performance rows share common immutable run metadata
// with the LS-Lint row factory shape for comparable checked evidence.
#[allow(clippy::too_many_arguments)]
fn measure_native_command(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    suite_status: &ToolAvailability,
    assura_full: &PreparedAssuraCli,
    native: &NativeCommand,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new(native.tool_name, native.row_family);
    let Some(binary_path) = assura_full.binary_path.as_ref() else {
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            "not-applicable",
            measurement,
            Vec::new(),
            Some(format!(
                "skipped because assura-full is unavailable: {}",
                assura_full
                    .status
                    .blocker
                    .as_deref()
                    .unwrap_or("unknown blocker")
            )),
            baseline_id,
        );
    };

    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;
    let expected_status = expected_status(fixture, native.row_family);
    for _ in 0..iterations {
        let mut command = Command::new(binary_path);
        command
            .current_dir(&fixture.root)
            .args(native.args)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let started = Instant::now();
        match command.output() {
            Ok(output) if output.status.code() == Some(expected_status) => {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(output) => {
                failure = Some(native_failure_detail(expected_status, &output));
                break;
            }
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        }
    }

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        suite_status.version.as_deref().unwrap_or("not-applicable"),
        measurement
            .with_assura_binary(binary_path, assura_full.binary_profile.as_deref())
            .with_expected_assura_exit_status(expected_status),
        samples,
        failure,
        baseline_id,
    )
}

fn expected_status(fixture: &MaterializedFixture, row_family: &str) -> i32 {
    let diagnostic_fixture = matches!(
        fixture.scenario.id,
        "native_reference_heavy" | "native_real_project"
    );
    if diagnostic_fixture && matches!(row_family, "native:content-check-cli") {
        1
    } else {
        0
    }
}

fn native_failure_detail(expected_status: i32, output: &Output) -> String {
    let stdout = concise_output(&output.stdout);
    let stderr = concise_output(&output.stderr);
    format!(
        "expected exit {expected_status}, got {:?}; stdout: {}; stderr: {}",
        output.status.code(),
        stdout,
        stderr
    )
}

fn concise_output(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        "<empty>".to_string()
    } else {
        compact.chars().take(500).collect()
    }
}
