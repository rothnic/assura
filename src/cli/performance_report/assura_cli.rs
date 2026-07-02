//! Assura CLI subprocess measurement for performance reports.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability, ASSURA_VERSION,
};
use crate::cli::performance_report::binary_profile::assura_binary_profile;
use crate::cli::performance_report::check_sources::latest_check_source_modified;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime};

pub(super) struct PreparedAssuraCli {
    pub(super) status: ToolAvailability,
    pub(super) binary_path: Option<PathBuf>,
    pub(super) binary_profile: Option<String>,
}

pub(super) fn prepare_assura_cli() -> PreparedAssuraCli {
    match std::env::current_exe() {
        Ok(binary_path) if binary_path.is_file() => {
            let primary_path = primary_assura_binary_path(&binary_path);
            if !primary_path.is_file() {
                return PreparedAssuraCli {
                    status: ToolAvailability {
                        available: false,
                        version: None,
                        blocker: Some(format!(
                            "primary assura binary was not found at {}",
                            primary_path.display()
                        )),
                    },
                    binary_path: None,
                    binary_profile: None,
                };
            }

            PreparedAssuraCli {
                binary_profile: Some(assura_binary_profile(&primary_path)),
                status: ToolAvailability {
                    available: true,
                    version: Some(ASSURA_VERSION.to_string()),
                    blocker: None,
                },
                binary_path: Some(primary_path),
            }
        }
        Ok(binary_path) => PreparedAssuraCli {
            status: ToolAvailability {
                available: false,
                version: None,
                blocker: Some(format!(
                    "current executable is not a file: {}",
                    binary_path.display()
                )),
            },
            binary_path: None,
            binary_profile: None,
        },
        Err(error) => PreparedAssuraCli {
            status: ToolAvailability {
                available: false,
                version: None,
                blocker: Some(format!("failed to resolve current executable: {error}")),
            },
            binary_path: None,
            binary_profile: None,
        },
    }
}

fn primary_assura_binary_path(current_exe: &Path) -> PathBuf {
    let full_companion = if cfg!(windows) {
        "assura-full.exe"
    } else {
        "assura-full"
    };

    if current_exe.file_name().and_then(|name| name.to_str()) == Some(full_companion) {
        let mut primary = current_exe.to_path_buf();
        primary.set_file_name(if cfg!(windows) {
            "assura.exe"
        } else {
            "assura"
        });
        primary
    } else {
        current_exe.to_path_buf()
    }
}

pub(super) fn prepare_assura_check_cli() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-check.exe"
    } else {
        "assura-check"
    })
}

pub(super) fn prepare_assura_check_client() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(unix) {
        "assura-check-unix-client"
    } else if cfg!(windows) {
        "assura-check-client.exe"
    } else {
        "assura-check-client"
    })
}

pub(super) fn prepare_assura_check_session() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-check-session.exe"
    } else {
        "assura-check-session"
    })
}

pub(super) fn prepare_assura_check_status() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-check-status.exe"
    } else {
        "assura-check-status"
    })
}

pub(super) fn prepare_assura_check_noop() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-check-noop.exe"
    } else {
        "assura-check-noop"
    })
}

pub(super) fn prepare_assura_check_compiled() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-check-compiled.exe"
    } else {
        "assura-check-compiled"
    })
}

pub(super) fn prepare_assura_check_compile_config() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-check-compile-config.exe"
    } else {
        "assura-check-compile-config"
    })
}

pub(super) fn prepare_assura_checkd() -> PreparedAssuraCli {
    prepare_sibling_binary(if cfg!(windows) {
        "assura-checkd.exe"
    } else {
        "assura-checkd"
    })
}

fn prepare_sibling_binary(binary_name: &str) -> PreparedAssuraCli {
    match std::env::current_exe() {
        Ok(current_exe) => {
            let binary_path = current_exe.with_file_name(binary_name);
            if binary_path.is_file() {
                if is_stale_sibling_binary(&binary_path, binary_name) {
                    return PreparedAssuraCli {
                        status: ToolAvailability {
                            available: false,
                            version: None,
                            blocker: Some(
                                "sibling Assura binary appears older than source files; rebuild release binaries before generating performance evidence"
                                    .to_string(),
                            ),
                        },
                        binary_path: None,
                        binary_profile: None,
                    };
                }

                return PreparedAssuraCli {
                    binary_profile: Some(assura_binary_profile(&binary_path)),
                    status: ToolAvailability {
                        available: true,
                        version: Some(ASSURA_VERSION.to_string()),
                        blocker: None,
                    },
                    binary_path: Some(binary_path),
                };
            }

            PreparedAssuraCli {
                status: ToolAvailability {
                    available: false,
                    version: None,
                    blocker: Some(format!(
                        "sibling Assura binary was not found next to current executable at {}",
                        binary_path.display()
                    )),
                },
                binary_path: None,
                binary_profile: None,
            }
        }
        Err(error) => PreparedAssuraCli {
            status: ToolAvailability {
                available: false,
                version: None,
                blocker: Some(format!("failed to resolve current executable: {error}")),
            },
            binary_path: None,
            binary_profile: None,
        },
    }
}

fn is_stale_sibling_binary(sibling_binary: &Path, binary_name: &str) -> bool {
    let Some(build_modified) = latest_sibling_build_modified(sibling_binary) else {
        return false;
    };

    latest_check_source_modified(binary_name)
        .map(|latest_source_modified| build_modified < latest_source_modified)
        .unwrap_or(false)
}

fn latest_sibling_build_modified(sibling_binary: &Path) -> Option<SystemTime> {
    let mut latest = sibling_binary
        .metadata()
        .and_then(|metadata| metadata.modified())
        .ok()?;
    for depinfo_path in depinfo_candidates(sibling_binary) {
        if let Ok(modified) = depinfo_path
            .metadata()
            .and_then(|metadata| metadata.modified())
        {
            latest = latest.max(modified);
        }
    }
    Some(latest)
}

fn depinfo_candidates(sibling_binary: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![sibling_binary.with_extension("d")];
    if let Some(file_name) = sibling_binary.file_name() {
        let mut depinfo_name = file_name.to_os_string();
        depinfo_name.push(".d");
        let candidate = sibling_binary.with_file_name(depinfo_name);
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

#[cfg(test)]
#[path = "assura_cli_tests.rs"]
mod tests;

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
) -> PerformanceResultRow {
    measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_cli,
        RowMeasurement::new("assura-cli", "assura-cli"),
        None,
        AssuraInvocation::FullCli,
    )
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_cli: &PreparedAssuraCli,
) -> PerformanceResultRow {
    measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_check_cli,
        RowMeasurement::new("assura-check-cli", "assura-check-cli"),
        None,
        AssuraInvocation::CheckOnly,
    )
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_strategy(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
    strategy_name: &'static str,
    traversal_env: Option<&'static str>,
) -> PerformanceResultRow {
    measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_cli,
        RowMeasurement::new(strategy_name, strategy_name),
        traversal_env,
        AssuraInvocation::FullCli,
    )
}

#[derive(Debug, Clone)]
pub(super) enum AssuraInvocation {
    FullCli,
    CheckOnly,
    CheckOnlyCached { cache_dir: PathBuf },
    CheckOnlyCompiled,
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_cli_row(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
    measurement: RowMeasurement<'_>,
    traversal_env: Option<&str>,
    invocation: AssuraInvocation,
) -> PerformanceResultRow {
    let Some(binary_path) = assura_cli.binary_path.as_ref() else {
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_status.version.as_deref().unwrap_or("unavailable"),
            measurement,
            Vec::new(),
            Some(format!(
                "skipped because Assura CLI is unavailable: {}",
                assura_cli
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
    let expected_status = fixture.metadata.expected_assura_exit_status;
    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        if let AssuraInvocation::CheckOnlyCached { cache_dir } = &invocation {
            let mut warmup = Command::new(binary_path);
            let warmup_output = warmup
                .current_dir(&fixture.root)
                .arg("--cache-dir")
                .arg(cache_dir)
                .arg("--quiet")
                .env_remove("ASSURA_CHECK_TRAVERSAL")
                .output();
            match warmup_output {
                Ok(output) if output.status.code() == Some(expected_status) => {}
                Ok(output) => {
                    failure = Some(format!(
                        "warmup expected exit {expected_status}, got {:?}; stdout: {}; stderr: {}",
                        output.status.code(),
                        String::from_utf8_lossy(&output.stdout).trim(),
                        String::from_utf8_lossy(&output.stderr).trim()
                    ));
                    break;
                }
                Err(error) => {
                    failure = Some(format!("warmup failed: {error}"));
                    break;
                }
            }
        }
        let mut command = Command::new(binary_path);
        match &invocation {
            AssuraInvocation::FullCli => {
                command
                    .current_dir(&fixture.root)
                    .arg("check")
                    .arg("--quiet");
            }
            AssuraInvocation::CheckOnly => {
                command.current_dir(&fixture.root).arg("--quiet");
            }
            AssuraInvocation::CheckOnlyCached { cache_dir } => {
                command
                    .current_dir(&fixture.root)
                    .arg("--cache-dir")
                    .arg(cache_dir)
                    .arg("--quiet");
            }
            AssuraInvocation::CheckOnlyCompiled => {
                command.current_dir(&fixture.root).arg("--quiet");
            }
        }
        command
            .env_remove("ASSURA_CHECK_TRAVERSAL")
            .envs(traversal_env.map(|value| ("ASSURA_CHECK_TRAVERSAL", value)))
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let started = Instant::now();
        let status = command.status();
        match status {
            Ok(status) if status.code() == Some(expected_status) => {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(status) => {
                failure = Some(format!(
                    "expected exit {expected_status}, got {:?}",
                    status.code()
                ));
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
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement.with_assura_binary(binary_path, assura_cli.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}
