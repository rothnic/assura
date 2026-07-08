//! Human text rendering for compact project review.

use super::report::{ProjectReviewFinding, ProjectReviewReport};
use std::io::IsTerminal;

const LABEL_WIDTH: usize = 10;
const FINDING_LIMIT: usize = 3;

pub(super) fn render_project_review_text(report: &ProjectReviewReport) -> String {
    let style = TextStyle::detect();
    let mut lines = vec![
        format!(
            "{}  {}",
            style.title("Assura review"),
            style.status(report.status)
        ),
        row(
            &style,
            "Check",
            format!(
                "{}  files={} dirs={} violations={}",
                style.status(report.structure.status),
                report.structure.files_checked,
                report.structure.dirs_checked,
                style.issue_count(report.structure.violations)
            ),
        ),
        row(&style, "Heat", render_heat(report, &style)),
        row(&style, "Branch", render_branch_signal(report, &style)),
        row(&style, "Worktree", render_worktree_signal(report, &style)),
        row(&style, "Hot dirs", render_hot_dirs(report, &style)),
        row(
            &style,
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
            &style,
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
        action_row(&style, "Fix now", report.findings_by_action("fix-now")),
        action_row(
            &style,
            "Configure",
            report.findings_by_action("configure-intentionally"),
        ),
        action_row(
            &style,
            "Inspect",
            report.findings_by_action("inspect-before-changing"),
        ),
        row(
            &style,
            "Policy",
            "inspect shape first; edit .assura/config.yml only intentionally".to_string(),
        ),
    ];

    if let Some(action) = report.next_actions.first() {
        lines.push(row(&style, "Next", action.action.clone()));
        lines.push(row(&style, "Run", style.command(&action.command)));
    }
    lines.push(row(
        &style,
        "Details",
        report
            .lower_level_commands
            .iter()
            .map(|command| style.command(command))
            .collect::<Vec<_>>()
            .join(" | "),
    ));
    lines.join("\n")
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

fn render_worktree_signal(report: &ProjectReviewReport, style: &TextStyle) -> String {
    let totals = &report.heatmap.totals;
    format!(
        "staged={} unstaged={} modified={} untracked={} deleted={} conflicts={} lines={}",
        style.change_count(totals.staged_files),
        style.change_count(totals.unstaged_files),
        style.change_count(totals.modified_files),
        style.change_count(totals.untracked_files),
        style.change_count(totals.deleted_files),
        style.change_count(totals.conflicted_files),
        line_delta(
            totals.worktree_line_additions,
            totals.worktree_line_deletions
        )
    )
}

fn render_hot_dirs(report: &ProjectReviewReport, style: &TextStyle) -> String {
    if report.heatmap.hot_dirs.is_empty() {
        return "none".to_string();
    }
    report
        .heatmap
        .hot_dirs
        .iter()
        .take(3)
        .map(|dir| {
            let mut text = format!(
                "{} !{} modified={} untracked={} branch={}",
                dir.path,
                style.issue_count(dir.validation_violations),
                style.change_count(dir.modified_files),
                style.change_count(dir.untracked_files),
                style.change_count(dir.branch_changed_files)
            );
            if dir.branch_line_additions > 0 || dir.branch_line_deletions > 0 {
                text.push_str(&format!(
                    " branch_lines={}",
                    line_delta(dir.branch_line_additions, dir.branch_line_deletions)
                ));
            }
            if dir.worktree_line_additions > 0 || dir.worktree_line_deletions > 0 {
                text.push_str(&format!(
                    " worktree_lines={}",
                    line_delta(dir.worktree_line_additions, dir.worktree_line_deletions)
                ));
            }
            text
        })
        .collect::<Vec<_>>()
        .join(" | ")
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
