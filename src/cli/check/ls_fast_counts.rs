//! Direct count helpers for the LS-Lint-compatible fast path.

use super::ls_fast::join_rel;
use super::ls_fast_plan::FastRules;
use super::patterns::{
    best_lslint_suffix_pair, is_lslint_extension_pattern, matches_single_compiled_pattern,
};
use super::rules::{
    count_satisfies, display_rel, is_excluded_rel_with, severity_for_bundle,
    severity_for_directory_bundle,
};
use super::{CheckError, StructureCheckReport, StructureChecker};
use std::ffi::OsString;
use std::fs::FileType;
use std::path::{Path, PathBuf};

pub(super) struct FastDirEntry {
    pub(super) name: OsString,
    pub(super) rel: PathBuf,
    pub(super) file_type: FileType,
}

pub(super) struct FastDirChildren {
    pub(super) entries: Vec<FastDirEntry>,
}

impl StructureChecker {
    pub(super) fn validate_fast_file_target_counts(
        &self,
        index_rel: &Path,
        filename: &str,
        report: &mut StructureCheckReport,
        rules: &FastRules,
        entries: &[FastDirEntry],
    ) {
        let Some(files) = rules.effective.files.as_ref() else {
            return;
        };
        let Some(exists) = files.exists.as_ref() else {
            return;
        };

        let lslint_patterns = exists
            .iter()
            .filter(|(pattern, _)| is_lslint_extension_pattern(pattern))
            .map(|(pattern, expected)| (pattern.clone(), expected.clone()))
            .collect::<Vec<_>>();
        let Some((pattern, expected)) = best_lslint_suffix_pair(&lslint_patterns, filename) else {
            return;
        };

        let count = entries
            .iter()
            .filter(|entry| {
                entry.file_type.is_file()
                    && entry.name.to_str().is_some_and(|name| {
                        matches_single_compiled_pattern(pattern, name, &self.glob_patterns)
                    })
            })
            .count();
        if !count_satisfies(count, expected) {
            self.push_violation(
                report,
                index_rel.to_path_buf(),
                "exists_count",
                format!(
                    "Directory '{}' has {} files matching '{}', expected {}",
                    display_rel(index_rel),
                    count,
                    pattern,
                    expected
                ),
                severity_for_bundle(files),
            );
        }
    }

    pub(super) fn validate_fast_directory_target_counts(
        &self,
        index_rel: &Path,
        target_rel: &Path,
        report: &mut StructureCheckReport,
        rules: &FastRules,
    ) {
        let Some(directory) = rules.effective.self_directory.as_ref() else {
            return;
        };
        let Some(exists) = directory.exists.as_ref() else {
            return;
        };

        let count = usize::from(target_rel == index_rel);
        for expected in exists.values() {
            if count_satisfies(count, expected) {
                continue;
            }
            self.push_violation(
                report,
                index_rel.to_path_buf(),
                "exists_count",
                format!(
                    "Directory '{}' exists {} times, expected {}",
                    display_rel(index_rel),
                    count,
                    expected
                ),
                severity_for_directory_bundle(directory),
            );
        }
    }

    pub(super) fn validate_fast_directory_counts(
        &self,
        rel: &Path,
        report: &mut StructureCheckReport,
        rules: &FastRules,
        entries: &[FastDirEntry],
    ) {
        if let Some(files) = rules.effective.files.as_ref() {
            if let Some(exists) = files.exists.as_ref() {
                for (pattern, expected) in exists {
                    let count = entries
                        .iter()
                        .filter(|entry| {
                            entry.file_type.is_file()
                                && entry.name.to_str().is_some_and(|name| {
                                    matches_single_compiled_pattern(
                                        pattern,
                                        name,
                                        &self.glob_patterns,
                                    )
                                })
                        })
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
        }

        if let Some(directories) = rules.effective.directories.as_ref() {
            if let Some(exists) = directories.exists.as_ref() {
                for (pattern, expected) in exists {
                    let count = entries
                        .iter()
                        .filter(|entry| {
                            entry.file_type.is_dir()
                                && entry.name.to_str().is_some_and(|name| {
                                    matches_single_compiled_pattern(
                                        pattern,
                                        name,
                                        &self.glob_patterns,
                                    )
                                })
                        })
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
        }

        self.validate_fast_self_directory_count(rel, report, rules);
    }

    pub(super) fn validate_fast_self_directory_count(
        &self,
        rel: &Path,
        report: &mut StructureCheckReport,
        rules: &FastRules,
    ) {
        if let Some(directory) = rules.effective.self_directory.as_ref() {
            if let Some(exists) = directory.exists.as_ref() {
                if rel.as_os_str().is_empty() {
                    return;
                }
                for expected in exists.values() {
                    if count_satisfies(1, expected) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "exists_count",
                        format!(
                            "Directory '{}' exists 1 times, expected {}",
                            display_rel(rel),
                            expected
                        ),
                        severity_for_directory_bundle(directory),
                    );
                }
            }
        }
    }

    pub(super) fn collect_lslint_fast_children(
        &self,
        dir: &Path,
        dir_rel: &Path,
    ) -> Result<FastDirChildren, CheckError> {
        let mut children = FastDirChildren {
            entries: Vec::new(),
        };

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let rel = join_rel(dir_rel, &name);
            if is_excluded_rel_with(&self.exclude_patterns, &rel) {
                continue;
            }

            let file_type = entry.file_type()?;
            children.entries.push(FastDirEntry {
                name,
                rel,
                file_type,
            });
        }

        Ok(children)
    }
}

pub(super) fn fast_rules_have_direct_counts(rules: Option<&FastRules>) -> bool {
    rules.is_some_and(|rules| {
        rules
            .effective
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .is_some()
            || rules
                .effective
                .directories
                .as_ref()
                .and_then(|directories| directories.exists.as_ref())
                .is_some()
            || rules
                .effective
                .self_directory
                .as_ref()
                .and_then(|directory| directory.exists.as_ref())
                .is_some()
    })
}

pub(super) fn fast_rules_have_child_counts(rules: Option<&FastRules>) -> bool {
    rules.is_some_and(|rules| {
        rules
            .effective
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .is_some()
            || rules
                .effective
                .directories
                .as_ref()
                .and_then(|directories| directories.exists.as_ref())
                .is_some()
    })
}
