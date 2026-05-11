//! Direct child file and directory policy validation.

use super::patterns::matches_single_compiled_pattern;
use super::rules::{
    count_satisfies, display_rel, file_matches_any_extension, severity_for_bundle,
    severity_for_directory_bundle,
};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::{DirectoryBundle, FileBundle};
use std::fs;
use std::path::Path;

pub(super) struct DirectFilePolicy<'a> {
    pub(super) filename: &'a str,
    pub(super) allowed_by_name: bool,
    pub(super) allowed_by_pattern: bool,
    pub(super) forbidden_by_pattern: bool,
}

impl StructureChecker {
    pub(super) fn validate_directory_contents(
        &mut self,
        path: &Path,
        report: &mut StructureCheckReport,
    ) {
        let rel = self.relative_path(path);
        let rules = self.resolve_rules(&rel);

        if let Some(files) = rules.files.as_ref() {
            self.validate_file_count_constraints(path, &rel, files, report);
        }

        if let Some(directories) = rules.directories.as_ref() {
            self.validate_directory_count_constraints(path, &rel, directories, report);
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
        directory: &Path,
        rel: &Path,
        files: &FileBundle,
        report: &mut StructureCheckReport,
    ) {
        let Some(exists) = files.exists.as_ref() else {
            return;
        };

        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let filenames: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().to_str().map(ToOwned::to_owned))
            .collect();

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
        directory: &Path,
        rel: &Path,
        directories: &DirectoryBundle,
        report: &mut StructureCheckReport,
    ) {
        let Some(exists) = directories.exists.as_ref() else {
            return;
        };

        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().to_str().map(ToOwned::to_owned))
            .collect();

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
}
