//! Stable event schema and rendering for continuous validation.

use super::args::WatchOutputFormat;
use super::check::{CheckError, StructureCheckReport};
use serde::Serialize;
use std::io::{self, Write};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WatchTrigger {
    Initial,
    Filesystem,
    Config,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeMode {
    ColdFull,
    WarmIncremental,
    WarmFull,
}

impl RuntimeMode {
    fn report_scope(self) -> ReportScope {
        match self {
            Self::WarmIncremental => ReportScope::AffectedPath,
            Self::ColdFull | Self::WarmFull => ReportScope::RequestedPath,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ColdFull => "cold_full",
            Self::WarmIncremental => "warm_incremental",
            Self::WarmFull => "warm_full",
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReportScope {
    RequestedPath,
    AffectedPath,
}

impl ReportScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::RequestedPath => "requested_path",
            Self::AffectedPath => "affected_path",
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CacheState {
    Prepared,
    Reloaded,
    Degraded,
}

impl CacheState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Reloaded => "reloaded",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Serialize)]
pub(super) struct WatchEvent {
    schema: &'static str,
    sequence: u64,
    trigger: WatchTrigger,
    runtime_mode: RuntimeMode,
    report_scope: ReportScope,
    cache_state: CacheState,
    fallback_reason: Option<String>,
    changed_paths: Vec<String>,
    coalesced_events: usize,
    debounce_ms: u64,
    duration_ms: f64,
    pub(super) report: Option<StructureCheckReport>,
    error: Option<String>,
}

impl WatchEvent {
    // allow-reason: Constructor arguments mirror the serialized event contract fields.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn report(
        sequence: u64,
        trigger: WatchTrigger,
        runtime_mode: RuntimeMode,
        cache_state: CacheState,
        fallback_reason: Option<String>,
        changed_paths: Vec<String>,
        coalesced_events: usize,
        debounce_ms: u64,
        duration: Duration,
        report: StructureCheckReport,
    ) -> Self {
        Self {
            schema: "assura.watch.event.v1",
            sequence,
            trigger,
            runtime_mode,
            report_scope: runtime_mode.report_scope(),
            cache_state,
            fallback_reason,
            changed_paths,
            coalesced_events,
            debounce_ms,
            duration_ms: duration.as_secs_f64() * 1_000.0,
            report: Some(report),
            error: None,
        }
    }

    // allow-reason: Failure events preserve the same stable field ordering as reports.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn failure(
        sequence: u64,
        trigger: WatchTrigger,
        runtime_mode: RuntimeMode,
        cache_state: CacheState,
        fallback_reason: Option<String>,
        changed_paths: Vec<String>,
        coalesced_events: usize,
        debounce_ms: u64,
        duration: Duration,
        error: String,
    ) -> Self {
        Self {
            schema: "assura.watch.event.v1",
            sequence,
            trigger,
            runtime_mode,
            report_scope: runtime_mode.report_scope(),
            cache_state,
            fallback_reason,
            changed_paths,
            coalesced_events,
            debounce_ms,
            duration_ms: duration.as_secs_f64() * 1_000.0,
            report: None,
            error: Some(error),
        }
    }

    fn render_text(&self) -> String {
        let status = self
            .report
            .as_ref()
            .map(|report| if report.success { "pass" } else { "fail" })
            .unwrap_or("error");
        let violations = self
            .report
            .as_ref()
            .map(StructureCheckReport::violation_count)
            .unwrap_or(0);
        let mut lines = vec![format!(
            "Assura watch #{sequence}: {status} | {violations} violations | {duration:.2} ms",
            sequence = self.sequence,
            duration = self.duration_ms,
        )];
        lines.push(format!(
            "  mode: {} | scope: {} | cache: {} | coalesced: {}",
            self.runtime_mode.as_str(),
            self.report_scope.as_str(),
            self.cache_state.as_str(),
            self.coalesced_events,
        ));

        if !self.changed_paths.is_empty() {
            let shown = self
                .changed_paths
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let omitted = self.changed_paths.len().saturating_sub(5);
            lines.push(if omitted == 0 {
                format!("  changed: {shown}")
            } else {
                format!("  changed: {shown} (+{omitted} more)")
            });
        }
        if let Some(reason) = &self.fallback_reason {
            lines.push(format!("  fallback: {reason}"));
        }
        if let Some(error) = &self.error {
            lines.push(format!("  error: {error}"));
        }
        if let Some(report) = &self.report {
            for violation in report.violations.iter().take(5) {
                lines.push(format!(
                    "  {} {} [{}] {}",
                    if violation.blocking { "!" } else { "~" },
                    violation.path.display(),
                    violation.rule,
                    violation.message,
                ));
            }
            let omitted = report.violation_count().saturating_sub(5);
            if omitted > 0 {
                lines.push(format!("  ... {omitted} more violations"));
            }
        }
        lines.join("\n")
    }
}

// allow-reason: This adapter forwards one complete event context into either result shape.
#[allow(clippy::too_many_arguments)]
pub(super) fn event_from_result(
    sequence: u64,
    trigger: WatchTrigger,
    runtime_mode: RuntimeMode,
    cache_state: CacheState,
    fallback_reason: Option<String>,
    changed_paths: Vec<String>,
    coalesced_events: usize,
    debounce_ms: u64,
    started: Instant,
    result: Result<StructureCheckReport, CheckError>,
) -> WatchEvent {
    match result {
        Ok(report) => WatchEvent::report(
            sequence,
            trigger,
            runtime_mode,
            cache_state,
            fallback_reason,
            changed_paths,
            coalesced_events,
            debounce_ms,
            started.elapsed(),
            report,
        ),
        Err(error) => WatchEvent::failure(
            sequence,
            trigger,
            runtime_mode,
            CacheState::Degraded,
            fallback_reason.or_else(|| Some("validation_error".into())),
            changed_paths,
            coalesced_events,
            debounce_ms,
            started.elapsed(),
            error.to_string(),
        ),
    }
}

pub(super) fn emit_event(format: WatchOutputFormat, event: &WatchEvent) -> Result<(), String> {
    let rendered = match format {
        WatchOutputFormat::Json => serde_json::to_string(event)
            .map_err(|error| format!("serialize watch event: {error}"))?,
        WatchOutputFormat::Text => event.render_text(),
    };
    let stdout = io::stdout();
    let mut output = stdout.lock();
    writeln!(output, "{rendered}")
        .and_then(|()| output.flush())
        .map_err(|error| format!("write watch event: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::check::StructureViolation;
    use std::path::PathBuf;

    #[test]
    fn text_events_include_scope_changes_fallback_and_actionable_findings() {
        let report = StructureCheckReport {
            success: false,
            project_root: PathBuf::from("/project"),
            config_path: PathBuf::from("/project/.assura/config.yml"),
            checked_path: PathBuf::from("/project"),
            files_checked: 1,
            dirs_checked: 1,
            violations: vec![StructureViolation {
                path: PathBuf::from("src/BadName.ts"),
                rule: "file_naming".into(),
                message: "expected kebab-case".into(),
                severity: "high".into(),
                severity_label: "High".into(),
                blocking: true,
                corrective_context: "Rename the file.".into(),
                metadata: None,
            }],
        };
        let event = WatchEvent::report(
            2,
            WatchTrigger::Filesystem,
            RuntimeMode::WarmFull,
            CacheState::Prepared,
            Some("project_wide_policy".into()),
            vec!["src/BadName.ts".into()],
            3,
            100,
            Duration::from_millis(2),
            report,
        );

        let text = event.render_text();
        assert!(text.contains("mode: warm_full | scope: requested_path | cache: prepared"));
        assert!(text.contains("changed: src/BadName.ts"));
        assert!(text.contains("fallback: project_wide_policy"));
        assert!(text.contains("! src/BadName.ts [file_naming] expected kebab-case"));
    }

    #[test]
    fn text_failure_events_include_the_runtime_error() {
        let event = WatchEvent::failure(
            3,
            WatchTrigger::Config,
            RuntimeMode::WarmFull,
            CacheState::Degraded,
            Some("config_reload_failed".into()),
            Vec::new(),
            1,
            100,
            Duration::from_millis(1),
            "invalid configuration".into(),
        );

        let text = event.render_text();
        assert!(text.contains("Assura watch #3: error"));
        assert!(text.contains("fallback: config_reload_failed"));
        assert!(text.contains("error: invalid configuration"));
    }
}
