//! Changed-path hot daemon measurement for incremental validation evidence.

use super::assura_cli::PreparedAssuraCli;
use super::hot_cli::{start_hot_server, unavailable_row};
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_changed_path_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_client: &PreparedAssuraCli,
    assura_checkd: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new(
        "assura-check-changed-path-cli",
        "assura-check-changed-path-cli",
    );
    let Some(client_path) = assura_check_client.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-check-client",
            assura_check_client,
        );
    };
    let Some(server_path) = assura_checkd.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-checkd",
            assura_checkd,
        );
    };
    let changed_path = match changed_path_candidate(fixture) {
        Ok(path) => path,
        Err(error) => {
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            )
        }
    };

    let mut server = match start_hot_server(server_path, fixture, None) {
        Ok(server) => server,
        Err(error) => {
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            )
        }
    };

    if let Err(error) = mark_changed_path(&fixture.root.join(&changed_path)) {
        let _ = server.child.kill();
        let _ = server.child.wait();
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_status.version.as_deref().unwrap_or("unavailable"),
            measurement,
            Vec::new(),
            Some(error),
            baseline_id,
        );
    }

    let expected_status = 0;
    let mut samples = Vec::with_capacity(iterations);
    let mut failure =
        run_hot_client_for_path(client_path, &server.addr, &changed_path, expected_status)
            .err()
            .map(|error| format!("changed-path daemon warmup failed: {error}"));

    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        match run_timed_hot_client_for_path(
            client_path,
            &server.addr,
            &changed_path,
            expected_status,
        ) {
            Ok(runtime_ms) => samples.push(runtime_ms),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    let _ = server.child.kill();
    let _ = server.child.wait();

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement.with_assura_binary(client_path, assura_check_client.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_dirty_project_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_client: &PreparedAssuraCli,
    assura_checkd: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new(
        "assura-check-dirty-project-cli",
        "assura-check-dirty-project-cli",
    );
    let Some(client_path) = assura_check_client.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-check-client",
            assura_check_client,
        );
    };
    let Some(server_path) = assura_checkd.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-checkd",
            assura_checkd,
        );
    };
    let changed_path = match changed_path_candidate(fixture) {
        Ok(path) => path,
        Err(error) => {
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            )
        }
    };

    let mut server = match start_hot_server(server_path, fixture, None) {
        Ok(server) => server,
        Err(error) => {
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            )
        }
    };

    let expected_status = 0;
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = run_hot_project_client(client_path, &server.addr, expected_status)
        .err()
        .map(|error| format!("dirty-project daemon warmup failed: {error}"));

    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        if let Err(error) = mark_changed_path(&fixture.root.join(&changed_path)) {
            failure = Some(error);
            break;
        }
        match run_timed_hot_dirty_project_client(
            client_path,
            &server.addr,
            &changed_path,
            expected_status,
        ) {
            Ok(runtime_ms) => samples.push(runtime_ms),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    let _ = server.child.kill();
    let _ = server.child.wait();

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement.with_assura_binary(client_path, assura_check_client.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}

fn run_hot_project_client(
    client_path: &Path,
    addr: &str,
    expected_status: i32,
) -> Result<(), String> {
    run_timed_hot_project_client(client_path, addr, expected_status).map(|_| ())
}

fn run_timed_hot_project_client(
    client_path: &Path,
    addr: &str,
    expected_status: i32,
) -> Result<f64, String> {
    let mut command = Command::new(client_path);
    command
        .arg(addr)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let started = Instant::now();
    let status = command
        .status()
        .map_err(|error| format!("failed to run assura-check-client for project: {error}"))?;

    if status.code() == Some(expected_status) {
        return Ok(started.elapsed().as_secs_f64() * 1000.0);
    }

    Err(format!(
        "expected exit {expected_status}, got {:?}",
        status.code()
    ))
}

fn run_timed_hot_dirty_project_client(
    client_path: &Path,
    addr: &str,
    changed_path: &Path,
    expected_status: i32,
) -> Result<f64, String> {
    let mut command = Command::new(client_path);
    command
        .arg(addr)
        .arg("--dirty-project-path")
        .arg(changed_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let started = Instant::now();
    let status = command.status().map_err(|error| {
        format!("failed to run assura-check-client for dirty project path: {error}")
    })?;

    if status.code() == Some(expected_status) {
        return Ok(started.elapsed().as_secs_f64() * 1000.0);
    }

    Err(format!(
        "expected exit {expected_status}, got {:?}",
        status.code()
    ))
}

fn run_hot_client_for_path(
    client_path: &Path,
    addr: &str,
    changed_path: &Path,
    expected_status: i32,
) -> Result<(), String> {
    run_timed_hot_client_for_path(client_path, addr, changed_path, expected_status).map(|_| ())
}

fn run_timed_hot_client_for_path(
    client_path: &Path,
    addr: &str,
    changed_path: &Path,
    expected_status: i32,
) -> Result<f64, String> {
    let mut command = Command::new(client_path);
    command
        .arg(addr)
        .arg(changed_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let started = Instant::now();
    let status = command
        .status()
        .map_err(|error| format!("failed to run assura-check-client for changed path: {error}"))?;

    if status.code() == Some(expected_status) {
        return Ok(started.elapsed().as_secs_f64() * 1000.0);
    }

    Err(format!(
        "expected exit {expected_status}, got {:?}",
        status.code()
    ))
}

pub(super) fn changed_path_candidate(fixture: &MaterializedFixture) -> Result<PathBuf, String> {
    changed_path_candidate_from_root(&fixture.root)
}

fn changed_path_candidate_from_root(root: &Path) -> Result<PathBuf, String> {
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let abs = root.join(&rel);
        let entries = std::fs::read_dir(&abs).map_err(|error| {
            format!("read changed-path candidate dir {}: {error}", abs.display())
        })?;
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let child_rel = if rel.as_os_str().is_empty() {
                PathBuf::from(&name)
            } else {
                rel.join(&name)
            };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if is_ignored_changed_path_dir(&name_str) {
                    continue;
                }
                stack.push(child_rel);
            } else if file_type.is_file() && !is_ignored_changed_path_file(&name_str) {
                return Ok(child_rel);
            }
        }
    }

    Err("no changed-path candidate found for fixture".to_string())
}

pub(super) fn mark_changed_path(path: &Path) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| format!("open changed-path candidate {}: {error}", path.display()))?;
    file.write_all(b"\n")
        .map_err(|error| format!("write changed-path candidate {}: {error}", path.display()))
}

fn is_ignored_changed_path_dir(name: &str) -> bool {
    matches!(
        name,
        ".assura" | ".git" | "node_modules" | "dist" | "build" | "coverage" | "target"
    )
}

fn is_ignored_changed_path_file(name: &str) -> bool {
    matches!(name, ".ls-lint.yml" | "assura-check.status")
}

#[cfg(test)]
mod tests {
    use super::changed_path_candidate_from_root;
    use std::fs;

    #[test]
    fn changed_path_candidate_is_deterministic_and_skips_generated_dirs() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("node_modules")).unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("z-file.ts"), "").unwrap();
        fs::write(temp.path().join("src").join("a-file.ts"), "").unwrap();
        fs::write(temp.path().join("node_modules").join("ignored.ts"), "").unwrap();

        let candidate = changed_path_candidate_from_root(temp.path()).unwrap();

        assert_eq!(candidate, std::path::PathBuf::from("z-file.ts"));
    }
}
