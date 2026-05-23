//! Direct count helpers for the LS-Lint-compatible fast path.

use super::ls_fast::join_rel;
use super::ls_fast_plan::FastRules;
use super::patterns::matches_single_compiled_pattern;
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
    })
}
