//! Alternating headline measurements for fair Assura and LS-Lint comparison.

use super::assura_cli::{measure_assura_cli, PreparedAssuraCli};
use super::ls_lint::PreparedLsLint;
use super::{
    measure_ls_lint, row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow,
    RowMeasurement,
};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

struct Samples {
    assura: Vec<f64>,
    ls_lint: Vec<f64>,
    assura_failure: Option<String>,
    ls_lint_failure: Option<String>,
}

// allow-reason: headline rows carry the full benchmark provenance explicitly.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_headline_pair(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    assura_cli: &PreparedAssuraCli,
    ls_lint: &PreparedLsLint,
) -> [PerformanceResultRow; 2] {
    let (Some(assura_path), Some(ls_lint_path)) = (
        assura_cli.binary_path.as_ref(),
        ls_lint.binary_path.as_ref(),
    ) else {
        return [
            measure_assura_cli(
                fixture,
                iterations,
                timestamp,
                commit_sha,
                branch,
                environment,
                baseline_id,
                &ls_lint.status,
                assura_cli,
            ),
            measure_ls_lint(
                fixture,
                iterations,
                timestamp,
                commit_sha,
                branch,
                environment,
                baseline_id,
                ls_lint,
            ),
        ];
    };

    let samples = measure_alternating(
        iterations,
        || {
            measure_command(
                assura_path,
                &fixture.root,
                &["check", "--quiet"],
                fixture.metadata.expected_assura_exit_status,
                true,
            )
        },
        || {
            measure_command(
                ls_lint_path,
                &fixture.root,
                &[],
                fixture.metadata.expected_ls_lint_exit_status,
                false,
            )
        },
    );
    let measurement_order = measurement_order(iterations);

    let ls_lint_version = ls_lint.status.version.as_deref().unwrap_or("unavailable");
    [
        row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_version,
            RowMeasurement::new("assura-cli", "assura-cli")
                .with_assura_binary(assura_path, assura_cli.binary_profile.as_deref())
                .with_measurement_order(measurement_order),
            samples.assura,
            samples.assura_failure,
            baseline_id,
        ),
        row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_version,
            RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli")
                .with_ls_lint_binary(ls_lint_path, ls_lint.execution_mode)
                .with_measurement_order(measurement_order),
            samples.ls_lint,
            samples.ls_lint_failure,
            baseline_id,
        ),
    ]
}

fn measurement_order(iterations: usize) -> &'static str {
    if iterations % 2 == 0 {
        "alternating-per-iteration-balanced"
    } else {
        "alternating-per-iteration-assura-first-extra"
    }
}

fn measure_command(
    binary: &Path,
    root: &Path,
    args: &[&str],
    expected_status: i32,
    clear_assura_traversal: bool,
) -> Result<f64, String> {
    let mut command = Command::new(binary);
    command
        .current_dir(root)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if clear_assura_traversal {
        command.env_remove("ASSURA_CHECK_TRAVERSAL");
    }
    let started = Instant::now();
    match command.status() {
        Ok(status) if status.code() == Some(expected_status) => {
            Ok(started.elapsed().as_secs_f64() * 1000.0)
        }
        Ok(status) => Err(format!(
            "expected exit {expected_status}, got {:?}",
            status.code()
        )),
        Err(error) => Err(error.to_string()),
    }
}

fn measure_alternating<Assura, LsLint>(
    iterations: usize,
    mut measure_assura: Assura,
    mut measure_ls_lint: LsLint,
) -> Samples
where
    Assura: FnMut() -> Result<f64, String>,
    LsLint: FnMut() -> Result<f64, String>,
{
    let mut samples = Samples {
        assura: Vec::with_capacity(iterations),
        ls_lint: Vec::with_capacity(iterations),
        assura_failure: None,
        ls_lint_failure: None,
    };

    for iteration in 0..iterations {
        let result = if iteration % 2 == 0 {
            measure_assura()
                .map_err(|error| (true, error))
                .and_then(|assura| {
                    measure_ls_lint()
                        .map(|ls_lint| (assura, ls_lint))
                        .map_err(|error| (false, error))
                })
        } else {
            measure_ls_lint()
                .map_err(|error| (false, error))
                .and_then(|ls_lint| {
                    measure_assura()
                        .map(|assura| (assura, ls_lint))
                        .map_err(|error| (true, error))
                })
        };
        match result {
            Ok((assura, ls_lint)) => {
                samples.assura.push(assura);
                samples.ls_lint.push(ls_lint);
            }
            Err((assura_failed, error)) => {
                if assura_failed {
                    samples.assura_failure = Some(error);
                    samples.ls_lint_failure =
                        Some("paired measurement stopped after Assura failed".into());
                } else {
                    samples.ls_lint_failure = Some(error);
                    samples.assura_failure =
                        Some("paired measurement stopped after LS-Lint failed".into());
                }
                break;
            }
        }
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::{measure_alternating, measurement_order};
    use std::cell::RefCell;

    #[test]
    fn alternates_tool_order_for_each_iteration() {
        let order = RefCell::new(Vec::new());
        let samples = measure_alternating(
            4,
            || {
                order.borrow_mut().push("assura");
                Ok(1.0)
            },
            || {
                order.borrow_mut().push("ls-lint");
                Ok(2.0)
            },
        );

        assert_eq!(
            order.into_inner(),
            ["assura", "ls-lint", "ls-lint", "assura", "assura", "ls-lint", "ls-lint", "assura",]
        );
        assert_eq!(samples.assura, [1.0; 4]);
        assert_eq!(samples.ls_lint, [2.0; 4]);
        assert!(samples.assura_failure.is_none());
        assert!(samples.ls_lint_failure.is_none());
    }

    #[test]
    fn stops_pair_when_assura_fails() {
        let samples = measure_alternating(2, || Err("assura failed".into()), || Ok(2.0));

        assert!(samples.assura.is_empty());
        assert!(samples.ls_lint.is_empty());
        assert_eq!(samples.assura_failure.as_deref(), Some("assura failed"));
        assert_eq!(
            samples.ls_lint_failure.as_deref(),
            Some("paired measurement stopped after Assura failed")
        );
    }

    #[test]
    fn preserves_complete_pairs_when_ls_lint_fails() {
        let mut ls_lint_calls = 0;
        let samples = measure_alternating(
            3,
            || Ok(1.0),
            || {
                ls_lint_calls += 1;
                if ls_lint_calls == 2 {
                    Err("LS-Lint failed".into())
                } else {
                    Ok(2.0)
                }
            },
        );

        assert_eq!(samples.assura, [1.0]);
        assert_eq!(samples.ls_lint, [2.0]);
        assert_eq!(samples.ls_lint_failure.as_deref(), Some("LS-Lint failed"));
        assert_eq!(
            samples.assura_failure.as_deref(),
            Some("paired measurement stopped after LS-Lint failed")
        );
    }

    #[test]
    fn reports_whether_alternating_order_is_balanced() {
        assert_eq!(measurement_order(16), "alternating-per-iteration-balanced");
        assert_eq!(
            measurement_order(3),
            "alternating-per-iteration-assura-first-extra"
        );
    }
}
