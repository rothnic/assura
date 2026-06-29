//! Safe Markdown fix operations backed by structure rule scopes.

use super::markdown::{blank_line_trailing_spaces, fix_blank_line_trailing_spaces};
use super::markdown_fix_report::{
    MarkdownFixFailure, MarkdownFixFileReport, MarkdownFixFileStatus, MarkdownFixMode,
    MarkdownFixRecord, MarkdownFixReport, MarkdownFixRollback, MarkdownFixSkip, MarkdownFixStatus,
};
use super::{discover_project, CheckError, CompiledStructureConfig, StructureChecker};
use crate::config::loader::ConfigLoader;
use crate::stable_hash::stable_hash;
use std::path::{Path, PathBuf};

/// Markdown fix rule supported by the first safe-fix slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFixRule {
    /// Remove spaces and tabs from otherwise blank Markdown lines.
    TrailingSpaces,
}

/// Apply a safe Markdown fix for configured Markdown scopes.
pub fn run_markdown_fix(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    rule: MarkdownFixRule,
    dry_run: bool,
) -> Result<MarkdownFixReport, CheckError> {
    let checked_path = match path {
        Some(path) => {
            if !path.exists() {
                return Err(CheckError::MissingPath(path));
            }
            path.canonicalize()?
        }
        None => std::env::current_dir()?,
    };
    let (project_root, config_path) = discover_project(&checked_path, config_path)?;
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let config = ConfigLoader::load(&config_path)?;
    let compiled = CompiledStructureConfig::new_for_check(config, false);
    let mut checker = StructureChecker::from_compiled_owned(project_root.clone(), compiled, false);
    let mut report = MarkdownFixReport {
        schema: "assura.safe-fix.markdown.v1",
        project_root,
        checked_path: checked_path.clone(),
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

    checker.fix_markdown_path(&checked_path, rule, dry_run, &mut report)?;
    Ok(report)
}

impl StructureChecker {
    fn fix_markdown_path(
        &mut self,
        checked_path: &Path,
        rule: MarkdownFixRule,
        dry_run: bool,
        report: &mut MarkdownFixReport,
    ) -> Result<(), CheckError> {
        if checked_path.is_file() {
            self.fix_markdown_file(checked_path, rule, dry_run, report, true)?;
            return Ok(());
        }

        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let walker = walkdir::WalkDir::new(checked_path)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(move |entry| {
                let path = entry.path();
                if path == checked_path {
                    return true;
                }
                let rel = path.strip_prefix(&project_root).unwrap_or(path);
                !super::rules::is_excluded_rel_with(&exclude_patterns, rel)
            });

        for entry in walker {
            let entry = entry?;
            if entry.file_type().is_file() {
                self.fix_markdown_file(entry.path(), rule, dry_run, report, false)?;
            }
        }

        Ok(())
    }

    fn fix_markdown_file(
        &mut self,
        path: &Path,
        rule: MarkdownFixRule,
        dry_run: bool,
        report: &mut MarkdownFixReport,
        explicit_target: bool,
    ) -> Result<(), CheckError> {
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            if explicit_target {
                let rel = self.relative_path(path);
                report.push_skip(
                    rel,
                    rule,
                    "not_markdown",
                    "Target file is not a Markdown file.",
                );
            }
            return Ok(());
        }

        let rel = self.relative_path(path);
        if self.is_excluded_rel(&rel) {
            if explicit_target {
                report.push_skip(
                    rel,
                    rule,
                    "excluded",
                    "Target Markdown file is excluded by project policy.",
                );
            }
            return Ok(());
        }

        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let rules = self.resolve_rules(parent_rel);
        let Some(markdown) = rules.markdown else {
            self.push_markdown_skip_if_fixable(
                path,
                rel,
                rule,
                "not_configured",
                "Markdown safe fixes only run in configured Markdown scopes.",
                report,
            )?;
            return Ok(());
        };

        match rule {
            MarkdownFixRule::TrailingSpaces => {
                if markdown.lint_trailing_spaces != Some(true) {
                    self.push_markdown_skip_if_fixable(
                        path,
                        rel,
                        rule,
                        "rule_disabled",
                        "markdown.lint_trailing_spaces is not enabled for this scope.",
                        report,
                    )?;
                    return Ok(());
                }

                report.files_checked += 1;
                let content = match std::fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(error) => {
                        report.failures.push(MarkdownFixFailure {
                            path: rel,
                            operation: operation_name(rule),
                            fix_ids: Vec::new(),
                            reason: error.to_string(),
                        });
                        return Ok(());
                    }
                };
                let violations = blank_line_trailing_spaces(&content);
                let (fixed, fixes) = fix_blank_line_trailing_spaces(&content);
                let fix_ids = violations
                    .iter()
                    .map(|violation| markdown_fix_id(&rel, *violation))
                    .collect::<Vec<_>>();
                report.fixes_before += violations.len();

                if fixes > 0 {
                    report.files_would_change += 1;
                    report.fixes_would_apply += fixes;
                    if !dry_run {
                        if let Err(error) = std::fs::write(path, fixed) {
                            report.fixes_after += violations.len();
                            report.failures.push(MarkdownFixFailure {
                                path: rel.clone(),
                                operation: operation_name(rule),
                                fix_ids: fix_ids.clone(),
                                reason: error.to_string(),
                            });
                            report.files.push(MarkdownFixFileReport {
                                path: rel.clone(),
                                status: MarkdownFixFileStatus::Failed,
                                fixes_before: violations.len(),
                                fixes_after: violations.len(),
                                fixes_planned: fixes,
                                fixes_applied: 0,
                                fix_ids,
                            });
                            report.fixes.extend(violations.iter().map(|violation| {
                                markdown_fix_record(&rel, *violation, MarkdownFixStatus::Failed)
                            }));
                            return Ok(());
                        }
                        let after_content = match std::fs::read_to_string(path) {
                            Ok(content) => content,
                            Err(error) => {
                                report.failures.push(MarkdownFixFailure {
                                    path: rel.clone(),
                                    operation: operation_name(rule),
                                    fix_ids: fix_ids.clone(),
                                    reason: error.to_string(),
                                });
                                return Ok(());
                            }
                        };
                        let after = blank_line_trailing_spaces(&after_content).len();
                        report.files_changed += 1;
                        report.fixes_applied += fixes;
                        report.fixes_after += after;
                        report.changed_paths.push(rel.clone());
                        report.applied_fix_ids.extend(fix_ids.clone());
                        report.files.push(MarkdownFixFileReport {
                            path: rel.clone(),
                            status: MarkdownFixFileStatus::Changed,
                            fixes_before: violations.len(),
                            fixes_after: after,
                            fixes_planned: fixes,
                            fixes_applied: fixes,
                            fix_ids,
                        });
                        report.fixes.extend(violations.iter().map(|violation| {
                            markdown_fix_record(&rel, *violation, MarkdownFixStatus::Applied)
                        }));
                    } else {
                        report.fixes_after += violations.len();
                        report.files.push(MarkdownFixFileReport {
                            path: rel.clone(),
                            status: MarkdownFixFileStatus::Planned,
                            fixes_before: violations.len(),
                            fixes_after: violations.len(),
                            fixes_planned: fixes,
                            fixes_applied: 0,
                            fix_ids,
                        });
                        report.fixes.extend(violations.iter().map(|violation| {
                            markdown_fix_record(&rel, *violation, MarkdownFixStatus::Planned)
                        }));
                    }
                } else {
                    report.files.push(MarkdownFixFileReport {
                        path: rel.clone(),
                        status: MarkdownFixFileStatus::Unchanged,
                        fixes_before: 0,
                        fixes_after: 0,
                        fixes_planned: 0,
                        fixes_applied: 0,
                        fix_ids,
                    });
                }
            }
        }

        Ok(())
    }

    fn push_markdown_skip_if_fixable(
        &self,
        path: &Path,
        rel: PathBuf,
        rule: MarkdownFixRule,
        reason: &'static str,
        message: &'static str,
        report: &mut MarkdownFixReport,
    ) -> Result<(), CheckError> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if !blank_line_trailing_spaces(&content).is_empty() {
                    report.push_skip(rel, rule, reason, message);
                }
            }
            Err(error) => report.failures.push(MarkdownFixFailure {
                path: rel,
                operation: operation_name(rule),
                fix_ids: Vec::new(),
                reason: error.to_string(),
            }),
        }
        Ok(())
    }
}

impl MarkdownFixReport {
    fn push_skip(
        &mut self,
        path: PathBuf,
        rule: MarkdownFixRule,
        reason: &'static str,
        message: &'static str,
    ) {
        self.skipped_fixes.push(MarkdownFixSkip {
            id: skipped_fix_id(&path, rule, reason),
            path,
            operation: operation_name(rule),
            reason,
            message: message.to_string(),
        });
    }
}

fn markdown_fix_record(
    rel: &Path,
    violation: super::markdown::MarkdownTrailingSpaces,
    status: MarkdownFixStatus,
) -> MarkdownFixRecord {
    MarkdownFixRecord {
        id: markdown_fix_id(rel, violation),
        path: rel.to_path_buf(),
        operation: operation_name(MarkdownFixRule::TrailingSpaces),
        status,
        line: violation.line_number,
        column: 1,
        before_trailing_whitespace: violation.trailing_count,
        after_trailing_whitespace: match status {
            MarkdownFixStatus::Planned | MarkdownFixStatus::Failed => violation.trailing_count,
            MarkdownFixStatus::Applied => 0,
        },
    }
}

fn markdown_fix_id(rel: &Path, violation: super::markdown::MarkdownTrailingSpaces) -> String {
    let key = format!(
        "{}:{}:{}",
        operation_name(MarkdownFixRule::TrailingSpaces),
        rel.display(),
        violation.line_number
    );
    format!("markdown.safe_fix.{:016x}", stable_hash(key.as_bytes()))
}

fn skipped_fix_id(rel: &Path, rule: MarkdownFixRule, reason: &str) -> String {
    let key = format!("skip:{}:{}:{reason}", operation_name(rule), rel.display());
    format!(
        "markdown.safe_fix.skip.{:016x}",
        stable_hash(key.as_bytes())
    )
}

fn operation_name(rule: MarkdownFixRule) -> &'static str {
    match rule {
        MarkdownFixRule::TrailingSpaces => "remove_blank_line_trailing_spaces",
    }
}
