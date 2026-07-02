//! Combined Markdown safe-fix orchestration.

use super::markdown_fix::{run_single_markdown_fix, MarkdownFixRule};
use super::markdown_fix_report::{MarkdownFixMode, MarkdownFixReport, MarkdownFixRollback};
use super::CheckError;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(super) fn run_all_markdown_fixes(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    dry_run: bool,
) -> Result<MarkdownFixReport, CheckError> {
    if !dry_run {
        let planned_trailing = run_single_markdown_fix(
            path.clone(),
            config_path.clone(),
            MarkdownFixRule::TrailingSpaces,
            true,
        )?;
        let planned_required = run_single_markdown_fix(
            path.clone(),
            config_path.clone(),
            MarkdownFixRule::RequiredSections,
            true,
        )?;
        let planned = merge_markdown_fix_reports(
            MarkdownFixRule::All,
            true,
            vec![planned_trailing, planned_required],
        );
        let applied_trailing = run_single_markdown_fix(
            path.clone(),
            config_path.clone(),
            MarkdownFixRule::TrailingSpaces,
            false,
        )?;
        let applied_required =
            run_single_markdown_fix(path, config_path, MarkdownFixRule::RequiredSections, false)?;
        let mut applied = merge_markdown_fix_reports(
            MarkdownFixRule::All,
            false,
            vec![applied_trailing, applied_required],
        );
        applied.files_would_change = planned.files_would_change;
        applied.fixes_would_apply = planned.fixes_would_apply;
        applied.fixes_before = planned.fixes_before;
        return Ok(applied);
    }

    let trailing = run_single_markdown_fix(
        path.clone(),
        config_path.clone(),
        MarkdownFixRule::TrailingSpaces,
        true,
    )?;
    let required =
        run_single_markdown_fix(path, config_path, MarkdownFixRule::RequiredSections, true)?;
    Ok(merge_markdown_fix_reports(
        MarkdownFixRule::All,
        true,
        vec![trailing, required],
    ))
}

fn merge_markdown_fix_reports(
    rule: MarkdownFixRule,
    dry_run: bool,
    reports: Vec<MarkdownFixReport>,
) -> MarkdownFixReport {
    let Some(first) = reports.first() else {
        unreachable!("all markdown fixes always has at least one child report")
    };
    let mut merged = MarkdownFixReport {
        schema: "assura.safe-fix.markdown.v1",
        project_root: first.project_root.clone(),
        checked_path: first.checked_path.clone(),
        dry_run,
        mode: if dry_run {
            MarkdownFixMode::DryRun
        } else {
            MarkdownFixMode::Apply
        },
        rule: rule.into(),
        files_checked: 0,
        files_changed: 0,
        fixes_applied: 0,
        files_would_change: 0,
        fixes_would_apply: 0,
        fixes_before: 0,
        fixes_after: 0,
        changed_paths: Vec::new(),
        applied_fix_ids: Vec::new(),
        files: Vec::new(),
        fixes: Vec::new(),
        skipped_fixes: Vec::new(),
        failures: Vec::new(),
        rollback: MarkdownFixRollback {
            backup_created: false,
            guidance: "Use version control to inspect or revert applied safe fixes.",
        },
    };

    let mut checked_paths = BTreeSet::new();
    let mut changed_paths = BTreeSet::new();
    let mut would_change_paths = BTreeSet::new();

    for report in reports {
        merged.fixes_applied += report.fixes_applied;
        merged.fixes_would_apply += report.fixes_would_apply;
        merged.fixes_before += report.fixes_before;
        merged.fixes_after += report.fixes_after;
        merged.applied_fix_ids.extend(report.applied_fix_ids);
        for path in report.changed_paths {
            if changed_paths.insert(path.clone()) {
                merged.changed_paths.push(path);
            }
        }
        for file in &report.files {
            checked_paths.insert(file.path.clone());
            if file.fixes_planned > 0 {
                would_change_paths.insert(file.path.clone());
            }
        }
        merged.files.extend(report.files);
        merged.fixes.extend(report.fixes);
        merged.skipped_fixes.extend(report.skipped_fixes);
        merged.failures.extend(report.failures);
    }

    merged.files_checked = checked_paths.len();
    merged.files_changed = changed_paths.len();
    merged.files_would_change = would_change_paths.len();
    merged
}
