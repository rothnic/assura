//! Narrow LS-Lint-compatible validation path.

use super::ls_fast_counts::fast_rules_have_direct_counts;
use super::ls_fast_naming::{validate_fast_file_stem, validate_fast_name};
use super::ls_fast_plan::{fast_rules_for_dir, FastRules, FastScope};
use super::patterns::matches_any_compiled_pattern;
use super::rules::{
    display_rel, file_matches_any_extension, is_excluded_rel_with, severity_for_bundle,
    severity_for_directory_bundle,
};
use super::{CheckError, StructureCheckReport, StructureCheckTimings, StructureChecker};
use std::ffi::OsStr;
use std::path::Path;
use std::time::Instant;

impl StructureChecker {
    pub(in crate::cli::check) fn try_check_lslint_fast(
        &mut self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
        timings: &mut StructureCheckTimings,
    ) -> Result<bool, CheckError> {
        if self.fail_fast {
            return Ok(false);
        }

        let Some(scopes) = self.lslint_fast_scopes.as_deref() else {
            return Ok(false);
        };

        let configured_started = Instant::now();
        self.validate_fast_configured_structure(report);
        timings.configured_structure_ms = configured_started.elapsed().as_secs_f64() * 1000.0;

        if self.fail_fast && !report.violations.is_empty() {
            return Ok(true);
        }

        let walk_started = Instant::now();
        self.walk_lslint_fast(checked_path, report, scopes)?;
        timings.walk_and_validate_ms = walk_started.elapsed().as_secs_f64() * 1000.0;

        let sort_started = Instant::now();
        report
            .violations
            .sort_by(|left, right| left.path.cmp(&right.path).then(left.rule.cmp(&right.rule)));
        timings.report_sort_ms = sort_started.elapsed().as_secs_f64() * 1000.0;
        Ok(true)
    }

    fn validate_fast_configured_structure(&self, report: &mut StructureCheckReport) {
        for node_rel in &self.required_dirs {
            if self.project_root.join(node_rel).is_dir() {
                continue;
            }
            self.push_violation(
                report,
                node_rel.to_path_buf(),
                "required_directory",
                format!("Required directory '{}' is missing", display_rel(node_rel)),
                "medium",
            );
        }
    }

    fn walk_lslint_fast(
        &self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
        scopes: &[FastScope],
    ) -> Result<(), CheckError> {
        let metadata = std::fs::symlink_metadata(checked_path)?;
        if metadata.file_type().is_symlink() {
            return Ok(());
        }

        let rel = checked_path
            .strip_prefix(&self.project_root)
            .unwrap_or(checked_path);
        if metadata.is_dir() {
            self.walk_lslint_fast_dir(checked_path, rel, report, scopes)?;
        } else if metadata.is_file() {
            report.files_checked += 1;
            let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
            let Some(rules) = fast_rules_for_dir(parent_rel, scopes) else {
                return Ok(());
            };
            let filename = checked_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            self.validate_fast_file(rules, rel, filename, file_stem(filename), report);
        }

        Ok(())
    }

    fn walk_lslint_fast_dir(
        &self,
        dir: &Path,
        dir_rel: &Path,
        report: &mut StructureCheckReport,
        scopes: &[FastScope],
    ) -> Result<(), CheckError> {
        let dir_rules = fast_rules_for_dir(dir_rel, scopes);
        let collect_counts =
            self.has_direct_count_constraints && fast_rules_have_direct_counts(dir_rules);
        if !collect_counts {
            return self.walk_lslint_fast_dir_streaming(dir, dir_rel, report, scopes, dir_rules);
        }

        let children = self.collect_lslint_fast_children(dir, dir_rel)?;

        if let Some(rules) = dir_rules {
            self.validate_fast_directory_counts(dir_rel, report, rules, &children.entries);
        }

        for entry in children.entries {
            if entry.file_type.is_symlink() {
                continue;
            } else if entry.file_type.is_dir() {
                let path = dir.join(&entry.name);
                report.dirs_checked += 1;
                self.validate_fast_directory(
                    &entry.rel,
                    entry.name.to_str().unwrap_or(""),
                    dir_rules,
                    report,
                );
                if self.fail_fast && !report.violations.is_empty() {
                    break;
                }
                self.walk_lslint_fast_dir(&path, &entry.rel, report, scopes)?;
            } else if entry.file_type.is_file() {
                report.files_checked += 1;
                if let Some(rules) = dir_rules {
                    let filename = entry.name.to_str().unwrap_or("");
                    self.validate_fast_file(
                        rules,
                        &entry.rel,
                        filename,
                        file_stem(filename),
                        report,
                    );
                }
            }

            if self.fail_fast && !report.violations.is_empty() {
                break;
            }
        }
        Ok(())
    }

    fn walk_lslint_fast_dir_streaming(
        &self,
        dir: &Path,
        dir_rel: &Path,
        report: &mut StructureCheckReport,
        scopes: &[FastScope],
        dir_rules: Option<&FastRules>,
    ) -> Result<(), CheckError> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let rel = join_rel(dir_rel, &name);
            if is_excluded_rel_with(&self.exclude_patterns, &rel) {
                continue;
            }

            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            } else if file_type.is_dir() {
                let path = dir.join(&name);
                report.dirs_checked += 1;
                self.validate_fast_directory(&rel, name.to_str().unwrap_or(""), dir_rules, report);
                if self.fail_fast && !report.violations.is_empty() {
                    break;
                }
                self.walk_lslint_fast_dir(&path, &rel, report, scopes)?;
            } else if file_type.is_file() {
                report.files_checked += 1;
                if let Some(rules) = dir_rules {
                    let filename = name.to_str().unwrap_or("");
                    self.validate_fast_file(rules, &rel, filename, file_stem(filename), report);
                }
            }

            if self.fail_fast && !report.violations.is_empty() {
                break;
            }
        }
        Ok(())
    }

    fn validate_fast_directory(
        &self,
        rel: &Path,
        name: &str,
        parent_rules: Option<&FastRules>,
        report: &mut StructureCheckReport,
    ) {
        if rel.as_os_str().is_empty() || self.is_configured_dir(rel) {
            return;
        }

        let Some(rules) = parent_rules else {
            return;
        };
        let Some(directories) = rules.effective.directories.as_ref() else {
            return;
        };
        if rules.has_direct_directory_policy {
            let allowed_by_name = directories
                .allowed_names
                .as_ref()
                .map(|allowed| allowed.iter().any(|allowed| allowed == name))
                .unwrap_or(false);
            let allowed_by_pattern = matches_any_compiled_pattern(
                directories.allowed_patterns.as_deref(),
                name,
                rel,
                &self.glob_patterns,
            );
            let forbidden_by_pattern = matches_any_compiled_pattern(
                directories.forbidden_patterns.as_deref(),
                name,
                rel,
                &self.glob_patterns,
            );

            if forbidden_by_pattern {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "forbidden_directory",
                    format!("Directory '{}' is forbidden by policy", display_rel(rel)),
                    severity_for_directory_bundle(directories),
                );
                return;
            }

            if directories.allow_extra == Some(false) && !allowed_by_name && !allowed_by_pattern {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "unexpected_directory",
                    format!("Directory '{}' is not allowed here", display_rel(rel)),
                    severity_for_directory_bundle(directories),
                );
                return;
            }

            if allowed_by_name || allowed_by_pattern {
                return;
            }
        }

        let Some(naming) = rules.directory_naming.as_ref() else {
            return;
        };
        if !validate_fast_name(name, naming, &self.naming_regexes) {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "directory_naming",
                format!(
                    "Directory '{}' does not match naming convention '{}'",
                    name,
                    naming.label()
                ),
                severity_for_directory_bundle(directories),
            );
        }
    }

    fn validate_fast_file(
        &self,
        rules: &FastRules,
        rel: &Path,
        filename: &str,
        stem: &str,
        report: &mut StructureCheckReport,
    ) {
        let Some(files) = rules.effective.files.as_ref() else {
            return;
        };

        if rules.has_direct_file_policy {
            let allowed_by_name = files
                .allowed_names
                .as_ref()
                .map(|allowed| allowed.iter().any(|name| name == filename))
                .unwrap_or(false);
            let allowed_by_pattern = matches_any_compiled_pattern(
                files.allowed_patterns.as_deref(),
                filename,
                rel,
                &self.glob_patterns,
            );
            let forbidden_by_pattern = matches_any_compiled_pattern(
                files.forbidden_patterns.as_deref(),
                filename,
                rel,
                &self.glob_patterns,
            );

            if forbidden_by_pattern {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "forbidden_file",
                    format!("File '{}' is forbidden by policy", display_rel(rel)),
                    severity_for_bundle(files),
                );
            }

            if files.allow_extra == Some(false)
                && !allowed_by_name
                && !allowed_by_pattern
                && !file_matches_any_extension(filename, files.extensions.as_deref())
            {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "unexpected_file",
                    format!("File '{}' is not allowed here", display_rel(rel)),
                    severity_for_bundle(files),
                );
            }

            if allowed_by_name || allowed_by_pattern {
                return;
            }
        }

        let Some(naming) = rules
            .file_naming
            .as_ref()
            .and_then(|file_naming| file_naming.naming_for(filename, &self.glob_patterns))
        else {
            return;
        };

        if !validate_fast_file_stem(stem, naming, &self.naming_regexes) {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "file_naming",
                format!(
                    "File '{}' does not match naming convention '{}'",
                    filename,
                    naming.label()
                ),
                severity_for_bundle(files),
            );
        }
    }
}

pub(super) fn join_rel(parent: &Path, name: &OsStr) -> std::path::PathBuf {
    if parent.as_os_str().is_empty() {
        name.into()
    } else {
        parent.join(name)
    }
}

fn file_stem(filename: &str) -> &str {
    if filename == "." || filename == ".." {
        return "";
    }

    match filename.rfind('.') {
        Some(0) => filename[1..]
            .find('.')
            .map(|index| &filename[..index + 1])
            .unwrap_or(filename),
        Some(index) => &filename[..index],
        None => filename,
    }
}

#[cfg(test)]
mod tests {
    use super::file_stem;

    #[test]
    fn fast_file_stem_matches_path_semantics_for_common_names() {
        assert_eq!(file_stem("file.ts"), "file");
        assert_eq!(file_stem("foo.tar.gz"), "foo.tar");
        assert_eq!(file_stem(".env"), ".env");
        assert_eq!(file_stem(".env.local"), ".env");
        assert_eq!(file_stem("foo."), "foo");
        assert_eq!(file_stem("."), "");
        assert_eq!(file_stem(".."), "");
    }
}
