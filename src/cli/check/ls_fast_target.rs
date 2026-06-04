//! Explicit target helpers for the LS-Lint-compatible fast path.

use super::ls_fast::file_stem;
use super::ls_fast_counts::fast_rules_have_direct_counts;
use super::ls_fast_plan::{fast_target_scope_for_dir, FastScope};
use super::rules::is_excluded_rel_with;
use super::{CheckError, StructureCheckReport, StructureChecker};
use std::path::Path;

impl StructureChecker {
    pub(in crate::cli::check) fn try_check_lslint_explicit_target(
        &self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
        scopes: &[FastScope],
    ) -> Result<bool, CheckError> {
        let metadata = std::fs::symlink_metadata(checked_path)?;
        if metadata.file_type().is_symlink() {
            return Ok(false);
        }

        let rel = checked_path
            .strip_prefix(&self.project_root)
            .unwrap_or(checked_path);
        if is_excluded_rel_with(&self.exclude_patterns, rel) {
            return Ok(true);
        }

        if metadata.is_dir() {
            report.dirs_checked += 1;
            let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
            let parent_rules = self.fast_rules_for_dir(parent_rel, scopes);
            let Some(scope_match) = fast_target_scope_for_dir(rel, scopes) else {
                return Ok(true);
            };
            let name = checked_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            self.validate_fast_directory(rel, name, parent_rules, Some(scope_match.rules), report);

            if fast_rules_have_direct_counts(Some(scope_match.exact_rules)) {
                self.validate_fast_directory_target_counts(
                    &scope_match.index_dir,
                    rel,
                    report,
                    scope_match.exact_rules,
                );
            }
            return Ok(true);
        }

        if !metadata.is_file() {
            return Ok(false);
        }

        report.files_checked += 1;
        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let Some(scope_match) = fast_target_scope_for_dir(parent_rel, scopes) else {
            return Ok(true);
        };
        let filename = checked_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        self.validate_fast_file(
            scope_match.rules,
            rel,
            filename,
            file_stem(filename),
            report,
        );

        if fast_rules_have_direct_counts(Some(scope_match.exact_rules)) {
            let parent = self.project_root.join(&scope_match.index_dir);
            let children = self.collect_lslint_fast_children(&parent, &scope_match.index_dir)?;
            self.validate_fast_file_target_counts(
                &scope_match.index_dir,
                filename,
                report,
                scope_match.exact_rules,
                &children.entries,
            );
        }

        Ok(true)
    }
}
