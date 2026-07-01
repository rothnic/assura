//! Text rendering for daemon CLI response contracts.

use super::{DaemonCheckPathOutput, DaemonErrorOutput, DaemonTextRender};
use crate::daemon::{DaemonAffectedReferences, DaemonHealth, DaemonMovedTargetReferences};

impl DaemonTextRender for DaemonHealth {
    fn render_text(&self) -> String {
        format!(
            "Daemon health: {:?}\nreason={}\ngeneration={}\nproject_root={}\nconfig_path={}\nstatus_file={}\nlog_file={}\nfallback={}",
            self.state,
            self.reason,
            self.generation,
            self.project_root.display(),
            self.config_path.display(),
            self.runtime_paths.status_file.display(),
            self.runtime_paths.log_file.display(),
            self.fallback_command,
        )
    }
}

impl DaemonTextRender for DaemonCheckPathOutput {
    fn render_text(&self) -> String {
        format!(
            "{}\nchanged_path_success={}\nviolations={}",
            self.health.render_text(),
            self.report.success,
            self.report.violations.len()
        )
    }
}

impl DaemonTextRender for DaemonAffectedReferences {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Daemon references: {} {} ({}/{}, truncated={})",
            self.mode,
            self.path.display(),
            self.bounds.returned,
            self.bounds.limit,
            self.bounds.truncated
        )];
        lines.push(format!(
            "health={:?} reason={}",
            self.health.state, self.health.reason
        ));
        for reference in &self.references {
            lines.push(format!(
                "source={}:{}:{} target={} anchor={} lines={} exists={} rule={} kind={} confidence={}",
                reference.source_path.display(),
                optional_usize(reference.source_line),
                optional_usize(reference.source_column),
                reference.target_path.display(),
                optional_string(reference.target_anchor.as_deref()),
                target_lines(reference.target_line_start, reference.target_line_end),
                reference.target_exists,
                reference.rule,
                reference.reference_kind,
                reference.confidence,
            ));
        }
        lines.join("\n")
    }
}

impl DaemonTextRender for DaemonMovedTargetReferences {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Daemon moved-target references: {} -> {} ({}/{}, truncated={})",
            self.previous_path.display(),
            self.new_path.display(),
            self.bounds.returned,
            self.bounds.limit,
            self.bounds.truncated
        )];
        lines.push(format!(
            "health={:?} reason={}",
            self.health.state, self.health.reason
        ));
        for reference in &self.references {
            lines.push(format!(
                "source={}:{}:{} target={} anchor={} lines={} exists={} rule={} kind={} confidence={}",
                reference.source_path.display(),
                optional_usize(reference.source_line),
                optional_usize(reference.source_column),
                reference.target_path.display(),
                optional_string(reference.target_anchor.as_deref()),
                target_lines(reference.target_line_start, reference.target_line_end),
                reference.target_exists,
                reference.rule,
                reference.reference_kind,
                reference.confidence,
            ));
        }
        lines.join("\n")
    }
}

impl DaemonTextRender for DaemonErrorOutput {
    fn render_text(&self) -> String {
        format!("{}\nerror={}", self.health.render_text(), self.error)
    }
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn optional_string(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

fn target_lines(start: Option<usize>, end: Option<usize>) -> String {
    match (start, end) {
        (Some(start), Some(end)) if start != end => format!("{start}-{end}"),
        (Some(start), _) => start.to_string(),
        _ => "-".to_string(),
    }
}
