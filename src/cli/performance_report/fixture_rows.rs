//! Per-fixture performance row orchestration.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use super::assura_cli::{measure_assura_check_cli, measure_assura_strategy, PreparedAssuraCli};
use super::cached_cli::measure_assura_check_cached_cli;
use super::changed_path_cli::{
    measure_assura_check_changed_path_cli, measure_assura_check_dirty_project_cli,
};
use super::compiled_cli::measure_assura_check_compiled_cli;
use super::dirty_project_socket::measure_assura_check_dirty_project_socket;
use super::fixtures::FixtureScenario;
use super::headline_pair::measure_headline_pair;
use super::hot_cli::{measure_assura_check_hot_cli, measure_assura_check_status_cli};
use super::ls_lint::PreparedLsLint;
use super::prepared_rows::{measure_prepared_five_changed_paths, measure_prepared_full_check};
use super::process_floor::{measure_assura_rust_cli_floor, measure_process_floor};
use super::session_cli::measure_assura_check_dirty_project_session_cli;
use super::traversal::{
    measure_parallel_jwalk_traversal, measure_serial_jwalk_traversal, measure_walkdir_traversal,
};
use super::{
    materialize_fixture, measure_assura, PerformanceEnvironment, PerformanceResultRow,
    ToolAvailability,
};
use std::fs;

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_scenario_rows(
    scenario: FixtureScenario,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
    assura_check_cli: &PreparedAssuraCli,
    assura_check_compiled: &PreparedAssuraCli,
    assura_check_compile_config: &PreparedAssuraCli,
    assura_check_client: &PreparedAssuraCli,
    assura_check_session: &PreparedAssuraCli,
    assura_check_status: &PreparedAssuraCli,
    assura_check_noop: &PreparedAssuraCli,
    assura_checkd: &PreparedAssuraCli,
    ls_lint: &PreparedLsLint,
) -> Result<Vec<PerformanceResultRow>, String> {
    let fixture = materialize_fixture(scenario)?;
    remove_benchmark_compiled_artifacts(&fixture);
    let [assura_headline, ls_lint_headline] = measure_headline_pair(
        &fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        assura_cli,
        ls_lint,
    );
    let mut results = vec![
        assura_headline,
        ls_lint_headline,
        measure_process_floor(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
        ),
        measure_assura_rust_cli_floor(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_noop,
        ),
        measure_assura_check_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_cli,
        ),
        measure_prepared_full_check(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
        ),
        measure_prepared_five_changed_paths(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
        ),
        measure_assura_check_cached_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_cli,
        ),
        measure_assura_check_compiled_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_compiled,
            assura_check_compile_config,
        ),
        measure_assura_check_hot_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_client,
            assura_checkd,
        ),
        measure_assura_check_changed_path_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_client,
            assura_checkd,
        ),
        measure_assura_check_dirty_project_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_client,
            assura_checkd,
        ),
        measure_assura_check_dirty_project_session_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_session,
            assura_checkd,
        ),
        measure_assura_check_dirty_project_socket(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_checkd,
        ),
        measure_assura_check_status_cli(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_check_status,
            assura_checkd,
        ),
        measure_assura_strategy(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_cli,
            "strategy:jwalk-serial-cli",
            None,
        ),
        measure_assura_strategy(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_cli,
            "strategy:walkdir-cli",
            Some("walkdir"),
        ),
        measure_assura_strategy(
            &fixture,
            iterations,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            assura_cli,
            "strategy:jwalk-parallel-cli",
            Some("parallel-jwalk"),
        ),
    ];
    results.extend(measure_assura(
        &fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
    ));
    results.push(measure_walkdir_traversal(
        &fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
    ));
    results.push(measure_serial_jwalk_traversal(
        &fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
    ));
    results.push(measure_parallel_jwalk_traversal(
        &fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
    ));

    let _ = fs::remove_dir_all(&fixture.root);
    Ok(results)
}

fn remove_benchmark_compiled_artifacts(fixture: &super::MaterializedFixture) {
    let assura_dir = fixture.root.join(".assura");
    let _ = fs::remove_file(assura_dir.join("check-config.bin"));
    let _ = fs::remove_file(assura_dir.join("performance-check-config.bin"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::performance_report::fixtures::{
        FixtureKind, FixtureMetadata, FixtureScenario, MaterializedFixture,
    };

    #[test]
    fn benchmark_compiled_artifact_cleanup_preserves_source_config() {
        let temp = tempfile::tempdir().unwrap();
        let assura_dir = temp.path().join(".assura");
        fs::create_dir(&assura_dir).unwrap();
        fs::write(assura_dir.join("config.yml"), "structure: {}\n").unwrap();
        fs::write(assura_dir.join("check-config.bin"), b"default artifact").unwrap();
        fs::write(
            assura_dir.join("performance-check-config.bin"),
            b"performance artifact",
        )
        .unwrap();

        let fixture = MaterializedFixture {
            root: temp.path().to_path_buf(),
            scenario: FixtureScenario {
                id: "cleanup-test",
                source_revision: "test",
                rule_cohort: "test",
                dirs: 0,
                files_per_dir: 0,
                kind: FixtureKind::SimpleLibrary,
            },
            metadata: FixtureMetadata {
                source_type: "generated",
                source_revision: "test".to_string(),
                cohort: "realistic-equivalent",
                checked_file_count: 0,
                ignored_file_count: 0,
                directory_count: 0,
                rule_count: 0,
                rule_surface_summary: "test",
                native_ls_lint_parity: true,
                assura_config_path: ".assura/config.yml",
                ls_lint_config_path: ".ls-lint.yml",
                config_generation_method: "test",
                shared_config_id: "test:cleanup-test".to_string(),
                expected_assura_exit_status: 0,
                expected_ls_lint_exit_status: 0,
            },
        };

        remove_benchmark_compiled_artifacts(&fixture);

        assert!(assura_dir.join("config.yml").exists());
        assert!(!assura_dir.join("check-config.bin").exists());
        assert!(!assura_dir.join("performance-check-config.bin").exists());
    }
}
