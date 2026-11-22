//! Pattern-specific and directory-wide file limit checks.

#[cfg(unix)]
use super::discover_project;
use super::patterns::best_file_pattern_match;
use super::rules::{display_rel, parse_size, severity_for_bundle};
use super::{StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::FileBundle;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_file_size(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        report: &mut StructureCheckReport,
    ) {
        let matched = files.max_size_patterns.as_ref().and_then(|patterns| {
            best_file_pattern_match(patterns, filename, rel, &self.glob_patterns)
        });
        let matched_pattern = matched.map(|(pattern, _)| pattern);
        let max_size = matched.map(|(_, value)| value).or(files.max_size.as_ref());
        let (Some(max_size), Ok(metadata)) = (max_size, fs::metadata(path)) else {
            return;
        };
        let Some(max_bytes) = parse_size(max_size) else {
            return;
        };
        if metadata.len() > max_bytes {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "max_size",
                append_repair_message(
                    format!(
                        "File '{}' is {} bytes, exceeding limit {}",
                        display_rel(rel),
                        metadata.len(),
                        max_size
                    ),
                    repair_message(files, matched_pattern, filename, rel, &self.glob_patterns),
                ),
                pattern_severity(files, matched_pattern, filename, rel, &self.glob_patterns),
            );
        }
    }

    pub(super) fn validate_file_line_count(
        &self,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        let matched = files.max_lines_patterns.as_ref().and_then(|patterns| {
            best_file_pattern_match(patterns, filename, rel, &self.glob_patterns)
        });
        let matched_pattern = matched.map(|(pattern, _)| pattern);
        let max_lines = matched.map(|(_, value)| *value).or(files.max_lines);
        if let (Some(max_lines), Some(content)) = (max_lines, content) {
            let line_count = content.lines().count();
            if line_count > max_lines && !self.is_excluded_rel(rel) {
                let overage = line_count - max_lines;
                let mut violation = StructureViolation::new(
                    rel.to_path_buf(),
                    "max_lines",
                    append_repair_message(
                        format!(
                            "File '{}' has {} lines, exceeding limit {}",
                            display_rel(rel),
                            line_count,
                            max_lines
                        ),
                        repair_message(files, matched_pattern, filename, rel, &self.glob_patterns),
                    ),
                    pattern_severity(files, matched_pattern, filename, rel, &self.glob_patterns),
                )
                .with_metadata(Some(json!({
                    "actual": line_count,
                    "limit": max_lines,
                    "overage": overage,
                    "actions": line_budget_actions(report, rel).unwrap_or_default(),
                })));
                violation.corrective_context = format!(
                    "Lines to cut or move: {overage}. {}",
                    violation.corrective_context
                );
                report.violations.push(violation);
            }
        }
    }
}

#[cfg(unix)]
fn line_budget_actions(report: &StructureCheckReport, rel: &Path) -> Option<Vec<Value>> {
    let target = report.project_root.join(rel);
    let cwd = report.project_root.to_str()?;
    let config = shell_path(&report.config_path)?;
    let path = shell_path(&Path::new(".").join(rel))?;
    if cwd.chars().any(char::is_control) || target.canonicalize().ok()? != target {
        return None;
    }
    // An explicit config outside .assura can select a different root for a file target.
    let (root, _) = discover_project(&target, Some(report.config_path.clone())).ok()?;
    if root != report.project_root {
        return None;
    }
    Some(vec![
        json!({
            "id": "explain",
            "cwd": cwd,
            "command": format!("assura explain --config {config} {path}"),
        }),
        json!({
            "id": "recheck",
            "cwd": cwd,
            "command": format!("assura check --config {config} {path}"),
        }),
    ])
}

#[cfg(unix)]
fn shell_path(path: &Path) -> Option<String> {
    let value = path.to_str()?;
    (!value.chars().any(char::is_control)).then(|| format!("'{}'", value.replace('\'', "'\\''")))
}

#[cfg(not(unix))]
fn line_budget_actions(_report: &StructureCheckReport, _rel: &Path) -> Option<Vec<Value>> {
    None
}

fn pattern_severity(
    files: &FileBundle,
    matched_pattern: Option<&str>,
    filename: &str,
    rel: &Path,
    compiled: &std::collections::HashMap<String, glob::Pattern>,
) -> String {
    matched_pattern
        .and_then(|pattern| files.severity_patterns.as_ref()?.get(pattern))
        .or_else(|| {
            files.severity_patterns.as_ref().and_then(|patterns| {
                best_file_pattern_match(patterns, filename, rel, compiled).map(|(_, value)| value)
            })
        })
        .cloned()
        .unwrap_or_else(|| severity_for_bundle(files))
}

fn repair_message<'a>(
    files: &'a FileBundle,
    matched_pattern: Option<&str>,
    filename: &str,
    rel: &Path,
    compiled: &std::collections::HashMap<String, glob::Pattern>,
) -> Option<&'a str> {
    matched_pattern
        .and_then(|pattern| files.message_patterns.as_ref()?.get(pattern))
        .or_else(|| {
            files.message_patterns.as_ref().and_then(|patterns| {
                best_file_pattern_match(patterns, filename, rel, compiled).map(|(_, value)| value)
            })
        })
        .map(String::as_str)
}

fn append_repair_message(mut message: String, repair: Option<&str>) -> String {
    if let Some(repair) = repair {
        message.push_str(". ");
        message.push_str(repair.trim_end_matches('.'));
        message.push('.');
    }
    message
}
