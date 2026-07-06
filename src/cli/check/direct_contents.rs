//! Direct child file and directory policy validation.

use super::patterns::matches_single_compiled_pattern;
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

        if !needs_file_counts && !needs_directory_counts {
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
            let count = filenames
                .iter()
                .filter(|name| matches_single_compiled_pattern(pattern, name, &self.glob_patterns))
                .count();
            if !count_satisfies(count, expected) {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "exists_count",
                    format!(
                        "Directory '{}' has {} files matching '{}', expected {}",
                        display_rel(rel),
                        count,
                        pattern,
                        expected
                    ),
                    severity_for_bundle(files),
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
            let count = names
                .iter()
                .filter(|name| matches_single_compiled_pattern(pattern, name, &self.glob_patterns))
                .count();
            if !count_satisfies(count, expected) {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "exists_count",
                    format!(
                        "Directory '{}' has {} directories matching '{}', expected {}",
                        display_rel(rel),
                        count,
                        pattern,
                        expected
                    ),
                    severity_for_directory_bundle(directories),
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
