//! Human text rendering for compact project review.

use super::report::{ProjectReviewFinding, ProjectReviewReport};
use std::io::IsTerminal;

const LABEL_WIDTH: usize = 10;
const FINDING_LIMIT: usize = 3;

pub(super) fn render_project_review_text(report: &ProjectReviewReport, verbose: bool) -> String {
    let style = TextStyle::detect();
    let mut lines = vec![
        style.title("Assura review"),
        String::new(),
        row(&style, "Status", style.review_status(report.status)),
        row(&style, "Scope", render_scope(report)),
        row(&style, "Findings", render_finding_summary(report, &style)),
        row(&style, "Branch", render_branch_signal(report, &style)),
        row(&style, "Worktree", render_worktree_summary(report, &style)),
    ];

    if !report.heatmap.risk_flags.is_empty() {
        lines.push(row(&style, "Watch", render_watch(report, &style)));
    }
    if let Some(hot_path) = render_hot_path(report, &style) {
        lines.push(row(&style, "Hot path", hot_path));
    }
    if let Some(finding) = primary_finding(report) {
        lines.push(row(&style, "Fix first", style.danger(&finding.detail)));
    }
    if let Some(action) = report.next_actions.first() {
        lines.push(row(&style, "Next", action.action.clone()));
        lines.push(row(&style, "Run", style.command(&action.command)));
    }

    if verbose {
        lines.extend(verbose_diagnostics(report, &style));
    }
    lines.join("\n")
}

fn verbose_diagnostics(report: &ProjectReviewReport, style: &TextStyle) -> Vec<String> {
    vec![
        String::new(),
        style.title("Diagnostics"),
        String::new(),
        row(
            style,
            "Check",
            format!(
                "{}  files={} dirs={} violations={}",
                style.status(report.structure.status),
                report.structure.files_checked,
                report.structure.dirs_checked,
                style.issue_count(report.structure.violations)
            ),
        ),
        row(style, "Heat", render_heat(report, style)),
        row(style, "Thresholds", render_thresholds(report)),
        row(style, "Hot dirs", render_hot_dirs(report, style)),
        row(
            style,
            "Content",
            format!(
                "diag={} missing={} refs={} fixes={}",
                style.issue_count(report.content_gaps.diagnostics),
                style.issue_count(report.content_gaps.missing_relations),
                report.content_gaps.unresolved_repository_references,
                report.content_gaps.safe_fixes
            ),
        ),
        row(
            style,
            "Findings",
            format!(
                "fix={} config={} inspect={} info={} omitted={}",
                count_action(report, "fix-now"),
                count_action(report, "configure-intentionally"),
                count_action(report, "inspect-before-changing"),
                report.summary.informational,
                report.summary.omitted_noise
            ),
        ),
        action_row(style, "Fix now", report.findings_by_action("fix-now")),
        action_row(
            style,
            "Configure",
            report.findings_by_action("configure-intentionally"),
        ),
        action_row(
            style,
            "Inspect",
            report.findings_by_action("inspect-before-changing"),
        ),
        row(
            style,
            "Policy",
            "inspect shape first; edit .assura/config.yml only intentionally".to_string(),
        ),
        row(
            style,
            "Details",
            report
                .lower_level_commands
                .iter()
                .map(|command| style.command(command))
                .collect::<Vec<_>>()
                .join(" | "),
        ),
    ]
}

fn render_scope(report: &ProjectReviewReport) -> String {
    if !report.heatmap.git_available {
        return "whole project (git unavailable)".to_string();
    }
    let branch = report.heatmap.branch.name.as_deref().unwrap_or("detached");
    report
        .heatmap
        .branch
        .base
        .as_deref()
        .map(|base| format!("{branch} -> {base}"))
        .unwrap_or_else(|| branch.to_string())
}

fn render_finding_summary(report: &ProjectReviewReport, style: &TextStyle) -> String {
    let mut parts = vec![
        format!("blocking={}", style.issue_count(report.summary.blocking)),
        format!("advisory={}", report.summary.advisory),
    ];
    for (state, count) in [
        ("new", actionable_state_count(report, "new")),
        ("worsened", actionable_state_count(report, "worsened")),
        ("resolved", report.finding_history.resolved),
    ] {
        if count > 0 {
            parts.push(format!("{state}={}", style.change_count(count)));
        }
    }
    parts.join(" ")
}

fn actionable_state_count(report: &ProjectReviewReport, state: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.state == state && matches!(finding.severity, "blocking" | "advisory")
        })
        .count()
}

fn render_worktree_summary(report: &ProjectReviewReport, style: &TextStyle) -> String {
    let totals = &report.heatmap.totals;
    let files = totals.modified_files + totals.untracked_files + totals.deleted_files;
    format!(
        "files={} modified={} untracked={} lines={}",
        style.change_count(files),
        style.change_count(totals.modified_files),
        style.change_count(totals.untracked_files),
        line_delta(
            totals.worktree_line_additions,
            totals.worktree_line_deletions
        )
    )
}

fn render_watch(report: &ProjectReviewReport, style: &TextStyle) -> String {
    report
        .heatmap
        .risk_flags
        .iter()
        .map(|flag| {
            let value = format!("{}/{}", flag.value, flag.threshold);
            if flag.severity == "blocking" {
                format!("{}={}", flag.id, style.danger(&value))
            } else {
                format!("{}={}", flag.id, style.warning(&value))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_hot_path(report: &ProjectReviewReport, style: &TextStyle) -> Option<String> {
    let dir = report.heatmap.hot_dirs.iter().max_by_key(|dir| {
        (
            dir.blocking_violations,
            dir.validation_violations,
            dir.path.matches('/').count(),
        )
    })?;
    let changed =
        dir.branch_changed_files + dir.modified_files + dir.untracked_files + dir.deleted_files;
    Some(format!(
        "{} violations={} changed={} lines={}",
        dir.path,
        style.issue_count(dir.validation_violations),
        style.change_count(changed),
        line_delta(
            dir.branch_line_additions + dir.worktree_line_additions,
            dir.branch_line_deletions + dir.worktree_line_deletions
        )
    ))
}

fn primary_finding(report: &ProjectReviewReport) -> Option<&ProjectReviewFinding> {
    report
        .findings
        .iter()
        .find(|finding| finding.severity == "blocking" && finding.state != "resolved")
        .or_else(|| {
            report.findings.iter().find(|finding| {
                finding.action_kind == "fix-now" && matches!(finding.state, "new" | "worsened")
            })
        })
}

fn render_heat(report: &ProjectReviewReport, style: &TextStyle) -> String {
    format!(
        "!{} hot_dirs={} risks={}",
        style.issue_count(report.heatmap.totals.validation_violations),
        style.change_count(report.heatmap.hot_dirs.len()),
        style.change_count(report.heatmap.risk_flags.len())
    )
}

fn render_branch_signal(report: &ProjectReviewReport, style: &TextStyle) -> String {
    let branch = report.heatmap.branch.name.as_deref().unwrap_or("n/a");
    format!(
        "{branch} files={} lines={} commits={}",
        style.change_count(report.heatmap.totals.branch_changed_files),
        line_delta(
            report.heatmap.totals.branch_line_additions,
            report.heatmap.totals.branch_line_deletions
        ),
        optional_usize(report.heatmap.branch.commits_on_branch)
    )
}

fn render_hot_dirs(report: &ProjectReviewReport, style: &TextStyle) -> String {
    if report.heatmap.hot_dirs.is_empty() {
        return "none".to_string();
    }
    let mut dirs = report.heatmap.hot_dirs.iter().take(3).collect::<Vec<_>>();
    dirs.sort_by(|left, right| left.path.cmp(&right.path));
    dirs.iter()
        .enumerate()
        .map(|(index, dir)| {
            let ancestors = dirs[..index]
                .iter()
                .filter(|candidate| dir.path.starts_with(&format!("{}/", candidate.path)))
                .count();
            let marker = if index + 1 == dirs.len() { "`-" } else { "|-" };
            let label = if ancestors > 0 {
                dir.path.rsplit('/').next().unwrap_or(&dir.path)
            } else {
                &dir.path
            };
            let mut text = format!(
                "{}{marker} {} v={} files=b{}/m{}/u{} lines=b{},w{}",
                "|  ".repeat(ancestors),
                label,
                style.issue_count(dir.validation_violations),
                style.change_count(dir.branch_changed_files),
                style.change_count(dir.modified_files),
                style.change_count(dir.untracked_files),
                line_delta(dir.branch_line_additions, dir.branch_line_deletions),
                line_delta(dir.worktree_line_additions, dir.worktree_line_deletions)
            );
            if dir.blocking_violations > 0 {
                text.push_str(&format!(" blocking={}", dir.blocking_violations));
            }
            text
        })
        .collect::<Vec<_>>()
        .join(&format!("\n{}", " ".repeat(LABEL_WIDTH + 1)))
}

fn render_thresholds(report: &ProjectReviewReport) -> String {
    let totals = &report.heatmap.totals;
    let thresholds = &report.heatmap.thresholds;
    let worktree_files = totals.modified_files + totals.untracked_files + totals.deleted_files;
    let churn = totals.branch_line_additions
        + totals.branch_line_deletions
        + totals.worktree_line_additions
        + totals.worktree_line_deletions;
    format!(
        "blocking={}/{} worktree={}/{} untracked={}/{} churn={}/{} commits={}/{}",
        totals.blocking_violations,
        thresholds.blocking_violations,
        worktree_files,
        thresholds.worktree_files,
        totals.untracked_files,
        thresholds.untracked_files,
        churn,
        thresholds.line_churn,
        report.heatmap.branch.commits_on_branch.unwrap_or_default(),
        thresholds.commits_on_branch,
    )
}

fn action_row(
    style: &TextStyle,
    label: &'static str,
    findings: Vec<&ProjectReviewFinding>,
) -> String {
    row(style, label, finding_ids(findings))
}

fn finding_ids(findings: Vec<&ProjectReviewFinding>) -> String {
    if findings.is_empty() {
        return "none".to_string();
    }
    let total = findings.len();
    let mut ids = findings
        .iter()
        .take(FINDING_LIMIT)
        .map(|finding| finding.id.as_str())
        .collect::<Vec<_>>();
    if total > FINDING_LIMIT {
        ids.push("+more");
    }
    ids.join(", ")
}

fn count_action(report: &ProjectReviewReport, action_kind: &'static str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| finding.action_kind == action_kind)
        .count()
}

fn row(style: &TextStyle, label: &'static str, value: String) -> String {
    format!("{} {}", style.label(label), value)
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn line_delta(additions: usize, deletions: usize) -> String {
    format!("+{additions}/-{deletions}")
}

struct TextStyle {
    color: bool,
}

impl TextStyle {
    fn detect() -> Self {
        Self {
            color: colors_enabled(),
        }
    }

    fn title(&self, value: &str) -> String {
        self.paint("1;36", value)
    }

    fn label(&self, value: &str) -> String {
        self.paint("2;36", &format!("{value:<width$}", width = LABEL_WIDTH))
    }

    fn command(&self, value: &str) -> String {
        self.paint("36", value)
    }

    fn danger(&self, value: &str) -> String {
        self.paint("31;1", value)
    }

    fn warning(&self, value: &str) -> String {
        self.paint("33;1", value)
    }

    fn issue_count(&self, value: usize) -> String {
        if value == 0 {
            self.paint("32", "0")
        } else {
            self.paint("31;1", &value.to_string())
        }
    }

    fn change_count(&self, value: usize) -> String {
        if value == 0 {
            self.paint("32", "0")
        } else {
            self.paint("33;1", &value.to_string())
        }
    }

    fn status(&self, value: &str) -> String {
        match value {
            "pass" => self.paint("32;1", value),
            "fail" => self.paint("31;1", value),
            "needs-review" => self.paint("33;1", value),
            _ => value.to_string(),
        }
    }

    fn review_status(&self, value: &str) -> String {
        match value {
            "pass" => self.paint("32;1", "clear"),
            "fail" | "needs-review" => self.paint("33;1", "needs attention"),
            _ => value.to_string(),
        }
    }

    fn paint(&self, code: &str, value: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{value}\x1b[0m")
        } else {
            value.to_string()
        }
    }
}

fn colors_enabled() -> bool {
    if env_flag("ASSURA_FORCE_COLOR") || env_flag("CLICOLOR_FORCE") {
        return true;
    }
    if std::env::var_os("NO_COLOR").is_some()
        || std::env::var("CLICOLOR").is_ok_and(|value| value == "0")
    {
        return false;
    }
    std::io::stdout().is_terminal()
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value != "0" && !value.is_empty())
}
