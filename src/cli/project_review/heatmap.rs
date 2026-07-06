//! Directory-level project review heat map signals.

mod git;

use crate::cli::check::{StructureCheckReport, StructureViolation};
use crate::cli::content_query::AgentQueryGapsOutput;
use git::collect_git_heat;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

const HOT_DIR_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectReviewHeatmap {
    pub(super) git_available: bool,
    pub(super) branch: HeatBranch,
    pub(super) totals: HeatTotals,
    pub(super) hot_dirs: Vec<HeatDirectory>,
    pub(super) risk_flags: Vec<HeatRiskFlag>,
    pub(super) legend: Vec<&'static str>,
}

impl ProjectReviewHeatmap {
    pub(super) fn render_text_lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "heat: !{} chg={} ?{} branch_files={} commits={}",
            self.totals.validation_violations,
            self.totals.modified_files,
            self.totals.untracked_files,
            self.totals.branch_changed_files,
            optional_usize(self.branch.commits_on_branch)
        )];
        if self.hot_dirs.is_empty() {
            lines.push("hot: none".to_string());
        } else {
            lines.push(format!(
                "hot: {}",
                self.hot_dirs
                    .iter()
                    .take(3)
                    .map(HeatDirectory::compact_text)
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
        if !self.risk_flags.is_empty() {
            lines.push(format!(
                "risk: {}",
                self.risk_flags
                    .iter()
                    .take(3)
                    .map(|flag| flag.id)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        lines
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub(super) struct HeatBranch {
    name: Option<String>,
    base: Option<String>,
    upstream: Option<String>,
    commits_on_branch: Option<usize>,
    ahead: Option<usize>,
    behind: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(super) struct HeatTotals {
    validation_violations: usize,
    blocking_violations: usize,
    naming_violations: usize,
    line_limit_violations: usize,
    content_diagnostics: usize,
    unresolved_repository_references: usize,
    staged_files: usize,
    unstaged_files: usize,
    modified_files: usize,
    untracked_files: usize,
    deleted_files: usize,
    conflicted_files: usize,
    branch_changed_files: usize,
    line_additions: usize,
    line_deletions: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(super) struct HeatDirectory {
    path: String,
    score: usize,
    validation_violations: usize,
    blocking_violations: usize,
    naming_violations: usize,
    line_limit_violations: usize,
    staged_files: usize,
    unstaged_files: usize,
    modified_files: usize,
    untracked_files: usize,
    deleted_files: usize,
    conflicted_files: usize,
    branch_changed_files: usize,
    line_additions: usize,
    line_deletions: usize,
}

impl HeatDirectory {
    fn compact_text(&self) -> String {
        format!(
            "{} !{} chg={} ?{} +/-{}/{}",
            self.path,
            self.validation_violations,
            self.modified_files,
            self.untracked_files,
            self.line_additions,
            self.line_deletions
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct HeatRiskFlag {
    id: &'static str,
    severity: &'static str,
    detail: String,
}

pub(super) fn build_project_review_heatmap(
    report: &StructureCheckReport,
    content_gaps: &AgentQueryGapsOutput,
) -> ProjectReviewHeatmap {
    let mut totals = HeatTotals {
        content_diagnostics: content_gaps.diagnostics,
        unresolved_repository_references: content_gaps.unresolved_repository_references,
        ..HeatTotals::default()
    };
    let mut dirs = BTreeMap::<String, HeatDirectory>::new();

    for violation in &report.violations {
        add_violation(&mut totals, &mut dirs, &report.project_root, violation);
    }

    let git = collect_git_heat(&report.project_root, &mut totals, &mut dirs);
    let hot_dirs = hot_directories(dirs);
    let risk_flags = risk_flags(&totals, git.branch.commits_on_branch);

    ProjectReviewHeatmap {
        git_available: git.available,
        branch: git.branch,
        totals,
        hot_dirs,
        risk_flags,
        legend: vec![
            "! validation violations",
            "chg tracked changed files",
            "? untracked files",
            "+/- line churn",
            "branch_files files changed since branch base",
        ],
    }
}

fn add_violation(
    totals: &mut HeatTotals,
    dirs: &mut BTreeMap<String, HeatDirectory>,
    project_root: &Path,
    violation: &StructureViolation,
) {
    totals.validation_violations += 1;
    totals.blocking_violations += usize::from(violation.blocking);
    totals.naming_violations += usize::from(is_naming_violation(&violation.rule));
    totals.line_limit_violations += usize::from(violation.rule == "max_lines");

    let path = relative_or_original(project_root, &violation.path);
    for dir in violation_rollup_dirs(&path, violation) {
        let entry = dir_entry(dirs, &dir);
        entry.validation_violations += 1;
        entry.blocking_violations += usize::from(violation.blocking);
        entry.naming_violations += usize::from(is_naming_violation(&violation.rule));
        entry.line_limit_violations += usize::from(violation.rule == "max_lines");
    }
}

fn hot_directories(dirs: BTreeMap<String, HeatDirectory>) -> Vec<HeatDirectory> {
    let mut dirs = dirs
        .into_values()
        .filter(|dir| dir.path != ".")
        .map(|mut dir| {
            dir.score = heat_score(&dir);
            dir
        })
        .filter(|dir| dir.score > 0)
        .collect::<Vec<_>>();
    dirs.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then(left.path.cmp(&right.path))
    });
    dirs.truncate(HOT_DIR_LIMIT);
    dirs
}

fn heat_score(dir: &HeatDirectory) -> usize {
    (dir.blocking_violations * 20)
        + (dir.validation_violations * 10)
        + (dir.line_limit_violations * 8)
        + (dir.naming_violations * 6)
        + (dir.untracked_files * 4)
        + (dir.modified_files * 3)
        + (dir.deleted_files * 3)
        + (dir.conflicted_files * 10)
        + dir.branch_changed_files
        + ((dir.line_additions + dir.line_deletions) / 100)
}

fn risk_flags(totals: &HeatTotals, commits_on_branch: Option<usize>) -> Vec<HeatRiskFlag> {
    let mut flags = Vec::new();
    if totals.blocking_violations > 0 {
        flags.push(risk(
            "blocking-validation",
            "blocking",
            format!(
                "{} blocking validation signal(s)",
                totals.blocking_violations
            ),
        ));
    }
    let worktree_files = totals.modified_files + totals.untracked_files + totals.deleted_files;
    if worktree_files >= 10 {
        flags.push(risk(
            "large-worktree",
            "advisory",
            format!("{worktree_files} changed/untracked/deleted file(s) in the worktree"),
        ));
    }
    if totals.untracked_files >= 5 {
        flags.push(risk(
            "untracked-growth",
            "advisory",
            format!("{} untracked file(s)", totals.untracked_files),
        ));
    }
    if totals.line_additions + totals.line_deletions >= 1_000 {
        flags.push(risk(
            "large-churn",
            "advisory",
            format!(
                "{} added and {} deleted line(s)",
                totals.line_additions, totals.line_deletions
            ),
        ));
    }
    if commits_on_branch.unwrap_or_default() >= 10 {
        flags.push(risk(
            "long-branch",
            "advisory",
            format!(
                "{} commit(s) since branch base",
                commits_on_branch.unwrap_or_default()
            ),
        ));
    }
    flags
}

fn risk(id: &'static str, severity: &'static str, detail: String) -> HeatRiskFlag {
    HeatRiskFlag {
        id,
        severity,
        detail,
    }
}

fn dir_entry<'a>(
    dirs: &'a mut BTreeMap<String, HeatDirectory>,
    path: &str,
) -> &'a mut HeatDirectory {
    dirs.entry(path.to_string())
        .or_insert_with(|| HeatDirectory {
            path: path.to_string(),
            ..HeatDirectory::default()
        })
}

fn rollup_dirs(path: &str) -> Vec<String> {
    rollup_dirs_for_path(path, false)
}

fn violation_rollup_dirs(path: &str, violation: &StructureViolation) -> Vec<String> {
    rollup_dirs_for_path(path, is_directory_violation(&violation.rule))
}

fn rollup_dirs_for_path(path: &str, target_is_directory: bool) -> Vec<String> {
    let trimmed = path.trim().trim_matches('"');
    let has_trailing_separator = trimmed.ends_with('/');
    let normalized = trimmed.trim_end_matches('/');
    if normalized.is_empty() || normalized == "." {
        return vec![".".to_string()];
    }
    let parts = normalized
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return vec![".".to_string()];
    }
    let dir_count = if target_is_directory || has_trailing_separator {
        parts.len()
    } else {
        parts.len().saturating_sub(1)
    };
    let mut dirs = vec![".".to_string()];
    for index in 0..dir_count {
        dirs.push(parts[..=index].join("/"));
    }
    dirs
}

fn relative_or_original(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_naming_violation(rule: &str) -> bool {
    rule.contains("naming") || matches!(rule, "file_name" | "directory_name")
}

fn is_directory_violation(rule: &str) -> bool {
    rule.contains("directory")
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}
