//! Assura phase row collection for performance reports.

use super::{row, FixtureScenario, PerformanceEnvironment, PerformanceResultRow, ToolAvailability};
use crate::cli::check::StructureCheckTimings;

pub(super) struct AssuraPhaseSamples {
    config_discovery: Vec<f64>,
    config_load: Vec<f64>,
    checker_init: Vec<f64>,
    configured_structure: Vec<f64>,
    walk_and_validate: Vec<f64>,
    report_sort: Vec<f64>,
}

impl AssuraPhaseSamples {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            config_discovery: Vec::with_capacity(capacity),
            config_load: Vec::with_capacity(capacity),
            checker_init: Vec::with_capacity(capacity),
            configured_structure: Vec::with_capacity(capacity),
            walk_and_validate: Vec::with_capacity(capacity),
            report_sort: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn push(&mut self, timings: StructureCheckTimings) {
        self.config_discovery.push(timings.config_discovery_ms);
        self.config_load.push(timings.config_load_ms);
        self.checker_init.push(timings.checker_init_ms);
        self.configured_structure
            .push(timings.configured_structure_ms);
        self.walk_and_validate.push(timings.walk_and_validate_ms);
        self.report_sort.push(timings.report_sort_ms);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn into_rows(
        self,
        scenario: FixtureScenario,
        timestamp: &str,
        commit_sha: &str,
        branch: &str,
        environment: &PerformanceEnvironment,
        ls_lint_status: &ToolAvailability,
        failure: Option<&str>,
        baseline_id: &str,
    ) -> Vec<PerformanceResultRow> {
        let phase_failure = failure.map(str::to_string);
        [
            ("assura:config-discovery", self.config_discovery),
            ("assura:config-load", self.config_load),
            ("assura:checker-init", self.checker_init),
            ("assura:configured-structure", self.configured_structure),
            ("assura:walk-and-validate", self.walk_and_validate),
            ("assura:report-sort", self.report_sort),
        ]
        .into_iter()
        .map(|(tool_name, samples)| {
            row(
                scenario,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                tool_name,
                samples,
                phase_failure.clone(),
                baseline_id,
            )
        })
        .collect()
    }
}
