//! File, directory, and markdown validators for structure-first checks.

use super::case::{validate_file_stem_with_path, validate_name_with_path};
use super::patterns::{
    best_lslint_suffix_match, is_lslint_extension_pattern, lslint_file_stem,
    matches_any_compiled_pattern, matches_single_compiled_pattern,
};
use super::repository_references::{is_source_reference_file, SOURCE_REFERENCE_FILE_SIZE_LIMIT};
use super::rules::{
    count_satisfies, display_rel, file_matches_any_extension, parse_size, severity_for_bundle,
    severity_for_directory_bundle,
};
use super::{direct_contents::DirectFilePolicy, StructureCheckReport, StructureChecker};
use crate::config::config::FileBundle;
use std::fs;
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_directory(&mut self, path: &Path, report: &mut StructureCheckReport) {
        let rel = self.relative_path(path);
        self.validate_directory_contents(path, report);

        if rel.as_os_str().is_empty() {
            return;
        }

        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let self_rules = self.resolve_rules(&rel);
        if let Some(directory) = self_rules.self_directory.as_ref() {
            if let Some(exists) = directory.exists.as_ref() {
                for expected in exists.values() {
                    if count_satisfies(1, expected) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        rel.clone(),
                        "exists_count",
                        format!(
                            "Directory '{}' exists 1 times, expected {}",
                            display_rel(&rel),
                            expected
                        ),
                        severity_for_directory_bundle(directory),
                    );
                }
            }

            if let Some(naming) = directory.naming.as_deref() {
                if !validate_name_with_path(
                    name,
                    &super::rules::rel_to_string(&rel),
                    naming,
                    &self.naming_regexes,
                ) {
                    self.push_violation(
                        report,
                        rel.clone(),
                        "directory_naming",
                        format!(
                            "Directory '{}' does not match naming convention '{}'",
                            name, naming
                        ),
                        severity_for_directory_bundle(directory),
                    );
                }
            }
        }

        let rules = self.resolve_rules(parent_rel);
        if let Some(directories) = rules.directories.as_ref() {
            let configured_child = self.is_configured_dir(&rel);
            let allowed_by_name = directories
                .allowed_names
                .as_ref()
                .map(|allowed| allowed.iter().any(|allowed| allowed == name))
                .unwrap_or(false);
            let allowed_by_pattern = matches_any_compiled_pattern(
                directories.allowed_patterns.as_deref(),
                name,
                &rel,
                &self.glob_patterns,
            );
            let forbidden_by_pattern = matches_any_compiled_pattern(
                directories.forbidden_patterns.as_deref(),
                name,
                &rel,
                &self.glob_patterns,
            );

            if forbidden_by_pattern {
                self.push_violation(
                    report,
                    rel.clone(),
                    "forbidden_directory",
                    format!("Directory '{}' is forbidden by policy", display_rel(&rel)),
                    severity_for_directory_bundle(directories),
                );
                return;
            }

            if directories.allow_extra == Some(false)
                && !configured_child
                && !allowed_by_name
                && !allowed_by_pattern
            {
                self.push_violation(
                    report,
                    rel.clone(),
                    "unexpected_directory",
                    format!("Directory '{}' is not allowed here", display_rel(&rel)),
                    severity_for_directory_bundle(directories),
                );
                return;
            }

            if !configured_child && !allowed_by_name && !allowed_by_pattern {
                if let Some(naming) = directories.naming.as_deref() {
                    if !validate_name_with_path(
                        name,
                        &super::rules::rel_to_string(&rel),
                        naming,
                        &self.naming_regexes,
                    ) {
                        self.push_violation(
                            report,
                            rel,
                            "directory_naming",
                            format!(
                                "Directory '{}' does not match naming convention '{}'",
                                name, naming
                            ),
                            severity_for_directory_bundle(directories),
                        );
                    }
                    return;
                }
            }
        }

        if self.is_configured_dir(&rel) {
            return;
        }

        let Some(files) = rules.files else {
            return;
        };
        let Some(naming) = files.naming.as_deref() else {
            return;
        };

        if !validate_name_with_path(
            name,
            &super::rules::rel_to_string(&rel),
            naming,
            &self.naming_regexes,
        ) {
            self.push_violation(
                report,
                rel,
                "directory_naming",
                format!(
                    "Directory '{}' does not match naming convention '{}'",
                    name, naming
                ),
                severity_for_bundle(&files),
            );
        }
    }

    pub(super) fn validate_file(&mut self, path: &Path, report: &mut StructureCheckReport) {
        let rel = self.relative_path(path);
        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let rules = self.resolve_rules(parent_rel);

        #[cfg(feature = "yaml-config")]
        let needs_markdown =
            path.extension().and_then(|ext| ext.to_str()) == Some("md") && rules.markdown.is_some();
        #[cfg(not(feature = "yaml-config"))]
        let needs_markdown = false;
        let needs_file_content = rules.files.as_ref().is_some_and(|files| {
            files.max_lines.is_some()
                || (files.require_docs == Some(true)
                    && path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        });
        let repository_reference_severity = self
            .repository_reference_severity_for_path(&rel)
            .filter(|_| is_source_reference_file(path))
            .filter(|_| {
                fs::metadata(path)
                    .map(|metadata| metadata.len() <= SOURCE_REFERENCE_FILE_SIZE_LIMIT)
                    .unwrap_or(false)
            });
        let content =
            if needs_file_content || needs_markdown || repository_reference_severity.is_some() {
                match fs::read_to_string(path) {
                    Ok(content) => Some(content),
                    Err(error) => {
                        let severity = rules
                            .files
                            .as_ref()
                            .map(|files| severity_for_bundle(files))
                            .unwrap_or_else(|| "medium".to_string());
                        self.push_violation(
                            report,
                            rel.clone(),
                            "read_file",
                            format!("Could not read '{}': {}", display_rel(&rel), error),
                            severity,
                        );
                        None
                    }
                }
            } else {
                None
            };

        if let Some(files) = rules.files {
            self.validate_file_bundle(path, &rel, &files, content.as_deref(), report);
        }

        #[cfg(feature = "yaml-config")]
        if needs_markdown {
            if let (Some(markdown), Some(content)) = (rules.markdown, content.as_deref()) {
                self.validate_markdown(&rel, &markdown, content, report);
            }
        }

        if let (Some(severity), Some(content)) = (repository_reference_severity, content.as_deref())
        {
            self.validate_repository_references(&rel, content, &severity, report);
        }
    }

    fn validate_file_bundle(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
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

        self.validate_direct_file_policy(
            rel,
            files,
            DirectFilePolicy {
                filename,
                allowed_by_name,
                allowed_by_pattern,
                forbidden_by_pattern,
            },
            report,
        );
        self.validate_file_extension(path, rel, files, filename, report);
        self.validate_file_name(
            path,
            rel,
            files,
            filename,
            allowed_by_name || allowed_by_pattern,
            report,
        );
        self.validate_file_size(path, rel, files, report);
        self.validate_file_line_count(rel, files, content, report);
        self.validate_rust_docs(path, rel, files, content, report);
    }

    fn validate_file_extension(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        report: &mut StructureCheckReport,
    ) {
        if let Some(extensions) = &files.extensions {
            if !file_matches_any_extension(filename, Some(extensions)) {
                let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "extension",
                    format!("File '{}' has disallowed extension '{}'", filename, ext),
                    severity_for_bundle(files),
                );
            }
        }
    }

    fn validate_file_name(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        allowed_by_name: bool,
        report: &mut StructureCheckReport,
    ) {
        if allowed_by_name {
            return;
        }
        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));

        if let Some(naming_patterns) = &files.naming_patterns {
            let best_match = best_lslint_suffix_match(naming_patterns, filename).or_else(|| {
                naming_patterns
                    .iter()
                    .filter(|(pattern, _)| {
                        matches_single_compiled_pattern(pattern, filename, &self.glob_patterns)
                    })
                    .map(|(pattern, naming)| (pattern.as_str(), naming.as_str()))
                    .max_by(|(left, _), (right, _)| {
                        left.len().cmp(&right.len()).then_with(|| right.cmp(left))
                    })
            });

            if let Some((pattern, naming)) = best_match {
                let stem = if is_lslint_extension_pattern(pattern) {
                    lslint_file_stem(filename)
                } else {
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("")
                };
                if !validate_file_stem_with_path(
                    stem,
                    &super::rules::rel_to_string(parent_rel),
                    naming,
                    &self.naming_regexes,
                ) {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "file_naming",
                        format!(
                            "File '{}' does not match naming convention '{}'",
                            filename, naming
                        ),
                        severity_for_bundle(files),
                    );
                }
                return;
            }
        }

        if let Some(naming) = &files.naming {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");
            if !validate_file_stem_with_path(
                stem,
                &super::rules::rel_to_string(parent_rel),
                naming,
                &self.naming_regexes,
            ) {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "file_naming",
                    format!(
                        "File '{}' does not match naming convention '{}'",
                        filename, naming
                    ),
                    severity_for_bundle(files),
                );
            }
        }
    }

    fn validate_file_size(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        report: &mut StructureCheckReport,
    ) {
        if let Some(max_size) = &files.max_size {
            if let Some(max_bytes) = parse_size(max_size) {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() > max_bytes {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "max_size",
                            format!(
                                "File '{}' is {} bytes, exceeding limit {}",
                                display_rel(rel),
                                metadata.len(),
                                max_size
                            ),
                            severity_for_bundle(files),
                        );
                    }
                }
            }
        }
    }

    fn validate_file_line_count(
        &self,
        rel: &Path,
        files: &FileBundle,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        if let (Some(max_lines), Some(content)) = (files.max_lines, content) {
            let line_count = content.lines().count();
            if line_count > max_lines {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "max_lines",
                    format!(
                        "File '{}' has {} lines, exceeding limit {}",
                        display_rel(rel),
                        line_count,
                        max_lines
                    ),
                    severity_for_bundle(files),
                );
            }
        }
    }

    fn validate_rust_docs(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        if files.require_docs == Some(true)
            && path.extension().and_then(|ext| ext.to_str()) == Some("rs")
        {
            if let Some(content) = content {
                if !content.contains("//!") && !content.contains("///") {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "require_docs",
                        format!("Rust file '{}' is missing rustdoc", display_rel(rel)),
                        severity_for_bundle(files),
                    );
                }
            }
        }
    }
}
