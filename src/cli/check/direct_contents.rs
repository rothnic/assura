//! Direct child file and directory policy validation.

use super::patterns::{best_file_pattern_match, matches_single_compiled_pattern};
use super::rules::{
    count_satisfies, display_rel, file_matches_any_extension, severity_for_bundle,
    severity_for_directory_bundle,
};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::{DirectoryBundle, FileBundle};
use glob::Pattern;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct DirectFilePolicy<'a> {
    pub(super) filename: &'a str,
    pub(super) allowed_by_name: bool,
    pub(super) allowed_by_pattern: bool,
    pub(super) forbidden_by_pattern: bool,
}

struct DirectChildNames {
    files: Vec<String>,
    directories: Vec<String>,
}

pub(super) fn exists_patterns_allow_name(
    exists: Option<&HashMap<String, String>>,
    name: &str,
    glob_patterns: &HashMap<String, Pattern>,
) -> bool {
    exists
        .map(|exists| {
            exists.iter().any(|(pattern, expected)| {
                count_rule_allows_child(expected)
                    && matches_single_compiled_pattern(pattern, name, glob_patterns)
            })
        })
        .unwrap_or(false)
}

impl StructureChecker {
    pub(super) fn validate_directory_contents(
        &mut self,
        path: &Path,
        report: &mut StructureCheckReport,
    ) {
        if !self.has_direct_count_constraints {
            return;
        }

        let rel = self.relative_path(path);
        let rules = self.resolve_rules(&rel);
        let needs_file_counts = rules
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .is_some();
        let needs_directory_counts = rules
            .directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .is_some();
        let needs_child_limit = rules.limit_children.is_some();

        if !needs_file_counts && !needs_directory_counts && !needs_child_limit {
            return;
        }

        let Some(children) = self.collect_direct_child_names(path, &rel) else {
            return;
        };

        if let Some(files) = rules.files.as_ref() {
            self.validate_file_count_constraints(&rel, files, &children.files, report);
        }

        if let Some(directories) = rules.directories.as_ref() {
            self.validate_directory_count_constraints(
                &rel,
                directories,
                &children.directories,
                report,
            );
        }

        if let Some(limit) = rules.limit_children.as_ref() {
            self.validate_aggregate_child_limit(&rel, limit, &children, report);
        }
    }

    fn validate_aggregate_child_limit(
        &self,
        rel: &Path,
        limit: &crate::config::types::ChildrenLimitConfig,
        children: &DirectChildNames,
        report: &mut StructureCheckReport,
    ) {
        let count = children.files.len() + children.directories.len();
        let Some(max) = limit.max else {
            return;
        };
        if count <= max {
            return;
        }
        let message = limit.message.clone().unwrap_or_else(|| {
            format!(
                "Directory '{}' has {} direct children, exceeding limit {}",
                display_rel(rel),
                count,
                max
            )
        });
        self.push_violation(
            report,
            rel.to_path_buf(),
            "limit_children",
            message,
            child_limit_severity(limit),
        );
    }

    pub(super) fn validate_direct_file_policy(
        &self,
        rel: &Path,
        files: &FileBundle,
        policy: DirectFilePolicy<'_>,
        report: &mut StructureCheckReport,
    ) {
        if policy.forbidden_by_pattern {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "forbidden_file",
                format!("File '{}' is forbidden by policy", display_rel(rel)),
                severity_for_bundle(files),
            );
        }

        if files.allow_extra == Some(false)
            && !policy.allowed_by_name
            && !policy.allowed_by_pattern
            && !file_matches_any_extension(policy.filename, files.extensions.as_deref())
        {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "unexpected_file",
                format!("File '{}' is not allowed here", display_rel(rel)),
                severity_for_bundle(files),
            );
        }
    }

    fn validate_file_count_constraints(
        &self,
        rel: &Path,
        files: &FileBundle,
        filenames: &[String],
        report: &mut StructureCheckReport,
    ) {
        let Some(exists) = files.exists.as_ref() else {
            return;
        };

        for (pattern, expected) in exists {
            let matches = filenames
                .iter()
                .filter(|name| {
                    matches_count_target(
                        exists,
                        files.allowed_patterns.as_deref(),
                        pattern,
                        expected,
                        name,
                        &self.glob_patterns,
                    )
                })
                .collect::<Vec<_>>();
            let count = matches.len();
            if !count_satisfies(count, expected) {
                let target = join_pattern_rel(rel, pattern);
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "exists_count",
                    append_repair_message(
                        format!(
                            "Directory '{}' has {} files matching '{}', expected {}",
                            display_rel(rel),
                            count,
                            pattern,
                            expected
                        ),
                        pattern_repair_message(
                            files.message_patterns.as_ref(),
                            pattern,
                            &target,
                            &self.glob_patterns,
                        ),
                    ),
                    pattern_severity(
                        files.severity_patterns.as_ref(),
                        pattern,
                        &target,
                        &self.glob_patterns,
                        || severity_for_bundle(files),
                    ),
                );
            }
        }
    }

    fn validate_directory_count_constraints(
        &self,
        rel: &Path,
        directories: &DirectoryBundle,
        names: &[String],
        report: &mut StructureCheckReport,
    ) {
        let Some(exists) = directories.exists.as_ref() else {
            return;
        };

        for (pattern, expected) in exists {
            let matches = names
                .iter()
                .filter(|name| {
                    matches_count_target(
                        exists,
                        directories.allowed_patterns.as_deref(),
                        pattern,
                        expected,
                        name,
                        &self.glob_patterns,
                    )
                })
                .collect::<Vec<_>>();
            if expected == "0" && !matches.is_empty() {
                for name in matches {
                    let child = rel.join(name);
                    self.push_violation(
                        report,
                        child.clone(),
                        "exists_count",
                        append_repair_message(
                            format!(
                                "Directory '{}' exists 1 times, expected 0",
                                display_rel(&child)
                            ),
                            pattern_repair_message(
                                directories.message_patterns.as_ref(),
                                pattern,
                                &child,
                                &self.glob_patterns,
                            ),
                        ),
                        pattern_severity(
                            directories.severity_patterns.as_ref(),
                            pattern,
                            &child,
                            &self.glob_patterns,
                            || severity_for_directory_bundle(directories),
                        ),
                    );
                }
                continue;
            }
            let count = matches.len();
            if !count_satisfies(count, expected) {
                let target = join_pattern_rel(rel, pattern);
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "exists_count",
                    append_repair_message(
                        format!(
                            "Directory '{}' has {} directories matching '{}', expected {}",
                            display_rel(rel),
                            count,
                            pattern,
                            expected
                        ),
                        pattern_repair_message(
                            directories.message_patterns.as_ref(),
                            pattern,
                            &target,
                            &self.glob_patterns,
                        ),
                    ),
                    pattern_severity(
                        directories.severity_patterns.as_ref(),
                        pattern,
                        &target,
                        &self.glob_patterns,
                        || severity_for_directory_bundle(directories),
                    ),
                );
            }
        }
    }

    fn collect_direct_child_names(&self, directory: &Path, rel: &Path) -> Option<DirectChildNames> {
        let entries = fs::read_dir(directory).ok()?;
        let mut children = DirectChildNames {
            files: Vec::new(),
            directories: Vec::new(),
        };

        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name();
            if self.is_excluded_rel(&join_child_rel(rel, &name)) {
                continue;
            }
            let Some(name) = name.to_str().map(ToOwned::to_owned) else {
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_file() {
                children.files.push(name);
            } else if file_type.is_dir() {
                children.directories.push(name);
            }
        }

        Some(children)
    }
}

fn matches_count_target(
    exists: &HashMap<String, String>,
    allowed_patterns: Option<&[String]>,
    pattern: &str,
    expected: &str,
    name: &str,
    compiled: &HashMap<String, Pattern>,
) -> bool {
    if !matches_single_compiled_pattern(pattern, name, compiled) {
        return false;
    }
    if expected != "0" {
        return true;
    }

    let refined_by_count = exists.iter().any(|(refinement, refinement_count)| {
        refinement != pattern
            && count_rule_allows_child(refinement_count)
            && pattern_specificity(refinement) > pattern_specificity(pattern)
            && matches_single_compiled_pattern(refinement, name, compiled)
    });
    let refined_by_declaration = allowed_patterns.is_some_and(|refinements| {
        refinements.iter().any(|refinement| {
            pattern_specificity(refinement) >= pattern_specificity(pattern)
                && matches_single_compiled_pattern(refinement, name, compiled)
        })
    });

    !refined_by_count && !refined_by_declaration
}

fn pattern_specificity(pattern: &str) -> usize {
    pattern
        .chars()
        .filter(|character| !matches!(character, '*' | '?' | '[' | ']' | '{' | '}'))
        .count()
}

fn join_pattern_rel(parent: &Path, pattern: &str) -> PathBuf {
    if parent.as_os_str().is_empty() {
        PathBuf::from(pattern)
    } else {
        parent.join(pattern)
    }
}

fn pattern_repair_message<'a>(
    messages: Option<&'a HashMap<String, String>>,
    pattern: &str,
    target: &Path,
    compiled: &HashMap<String, Pattern>,
) -> Option<&'a str> {
    let messages = messages?;
    best_file_pattern_match(messages, pattern, target, compiled)
        .map(|(_, message)| message.as_str())
}

fn pattern_severity(
    severities: Option<&HashMap<String, String>>,
    pattern: &str,
    _target: &Path,
    _compiled: &HashMap<String, Pattern>,
    fallback: impl FnOnce() -> String,
) -> String {
    severities
        .and_then(|severities| severities.get(pattern))
        .cloned()
        .unwrap_or_else(fallback)
}

fn append_repair_message(mut message: String, repair: Option<&str>) -> String {
    if let Some(repair) = repair {
        message.push_str(". ");
        message.push_str(repair.trim_end_matches('.'));
        message.push('.');
    }
    message
}

fn child_limit_severity(limit: &crate::config::types::ChildrenLimitConfig) -> &'static str {
    use crate::config::types::Severity;
    match limit.severity.unwrap_or(Severity::Medium) {
        Severity::Critical => "critical",
        Severity::High => "high",
        Severity::Medium => "medium",
        Severity::Low => "low",
        Severity::Off => "low",
    }
}

fn count_rule_allows_child(expected: &str) -> bool {
    let expected = expected.trim();
    if expected == "exists" {
        return true;
    }

    if let Some((_, max)) = expected.split_once('-') {
        return max
            .trim()
            .parse::<usize>()
            .map(|max| max > 0)
            .unwrap_or(false);
    }

    if let Some((_, max)) = expected.split_once("..") {
        return max
            .trim()
            .parse::<usize>()
            .map(|max| max > 0)
            .unwrap_or(false);
    }

    expected
        .parse::<usize>()
        .map(|required| required > 0)
        .unwrap_or(false)
}

fn join_child_rel(parent: &Path, name: &OsStr) -> PathBuf {
    if parent.as_os_str().is_empty() {
        PathBuf::from(name)
    } else {
        parent.join(name)
    }
}
