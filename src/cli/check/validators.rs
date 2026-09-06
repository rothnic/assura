//! File, directory, and markdown validators for structure-first checks.

use super::direct_contents::exists_patterns_allow_name;
use super::patterns::{
    best_file_pattern_match, file_pattern_uses_lslint_stem, file_stem_for_pattern,
    matches_any_compiled_pattern,
};
use super::repository_references::{is_source_reference_file, SOURCE_REFERENCE_FILE_SIZE_LIMIT};
use super::rules::{
    count_satisfies, display_rel, file_matches_any_extension, severity_for_bundle,
    severity_for_directory_bundle,
};
use super::{direct_contents::DirectFilePolicy, StructureCheckReport, StructureChecker};
use crate::config::config::FileBundle;
use crate::policy::naming::{validate_file_stem_with_path, validate_name_with_path};
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
        let naming_target = name.strip_prefix('.').unwrap_or(name);
        let parent_rules = self.resolve_rules(parent_rel);
        let self_rules = self.resolve_rules(&rel);
        if let Some(directory) = self_rules.self_directory.as_ref() {
            if let Some(exists) = directory.exists.as_ref() {
                for expected in exists.values() {
                    if expected == "0"
                        && parent_rules
                            .directories
                            .as_ref()
                            .is_some_and(|directories| {
                                exists_patterns_allow_name(
                                    directories.exists.as_ref(),
                                    name,
                                    &self.glob_patterns,
                                )
                            })
                    {
                        continue;
                    }
                    if count_satisfies(1, expected) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        rel.clone(),
                        "exists_count",
                        append_directory_repair(
                            format!(
                                "Directory '{}' exists 1 times, expected {}",
                                display_rel(&rel),
                                expected
                            ),
                            directory.message.as_deref(),
                        ),
                        severity_for_directory_bundle(directory),
                    );
                }
            }

            if let Some(naming) = directory.naming.as_deref() {
                if !validate_name_with_path(
                    naming_target,
                    &super::rules::rel_to_string(&rel),
                    naming,
                    &self.naming_regexes,
                ) {
                    self.push_violation(
                        report,
                        rel.clone(),
                        "directory_naming",
                        append_directory_repair(
                            format!(
                                "Directory '{}' does not match naming convention '{}'",
                                name, naming
                            ),
                            directory.message.as_deref(),
                        ),
                        severity_for_directory_bundle(directory),
                    );
                }
            }
        }

        let rules = parent_rules;
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
            let allowed_by_exists =
                exists_patterns_allow_name(directories.exists.as_ref(), name, &self.glob_patterns);
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
                && !allowed_by_exists
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

            if !configured_child && !allowed_by_name && !allowed_by_pattern && !allowed_by_exists {
                if let Some(naming) = directories.naming.as_deref() {
                    if !validate_name_with_path(
                        naming_target,
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
            naming_target,
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

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        #[cfg(feature = "yaml-config")]
        let markdown = rules
            .files
            .as_ref()
            .and_then(|files| files.markdown_patterns.as_ref())
            .and_then(|patterns| {
                best_file_pattern_match(patterns, filename, &rel, &self.glob_patterns)
                    .map(|(_, markdown)| markdown)
            })
            .or(rules.markdown.as_deref());
        #[cfg(feature = "yaml-config")]
        let needs_markdown =
            path.extension().and_then(|ext| ext.to_str()) == Some("md") && markdown.is_some();
        #[cfg(not(feature = "yaml-config"))]
        let needs_markdown = false;
        let needs_file_content = rules.files.as_ref().is_some_and(|files| {
            files.max_lines.is_some()
                || files.max_lines_patterns.as_ref().is_some_and(|patterns| {
                    best_file_pattern_match(patterns, filename, &rel, &self.glob_patterns).is_some()
                })
                || (files.require_docs == Some(true)
                    && path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        });
        let repository_reference_policy = self
            .repository_reference_policy_for_path(&rel)
            .filter(|_| {
                is_source_reference_file(path)
                    || (self.has_repository_reference_frontmatter_fields_for_path(&rel)
                        && path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            })
            .filter(|_| {
                fs::metadata(path)
                    .map(|metadata| metadata.len() <= SOURCE_REFERENCE_FILE_SIZE_LIMIT)
                    .unwrap_or(false)
            });
        let content =
            if needs_file_content || needs_markdown || repository_reference_policy.is_some() {
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

        if let Some(files) = rules.files.as_ref() {
            self.validate_file_bundle(path, &rel, files, content.as_deref(), report);
        }

        #[cfg(feature = "yaml-config")]
        if needs_markdown {
            if let (Some(markdown), Some(content)) = (markdown, content.as_deref()) {
                self.validate_markdown(&rel, markdown, content, report);
            }
        }

        if let (Some(policy), Some(content)) = (repository_reference_policy, content.as_deref()) {
            let severity = policy.severity.as_deref().unwrap_or("medium");
            self.validate_repository_references(&rel, content, severity, report);
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
        let allowed_by_exists =
            exists_patterns_allow_name(files.exists.as_ref(), filename, &self.glob_patterns);
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
                allowed_by_pattern: allowed_by_pattern || allowed_by_exists,
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
            allowed_by_name || allowed_by_pattern || allowed_by_exists,
            report,
        );
        self.validate_file_size(path, rel, files, filename, report);
        self.validate_file_line_count(rel, files, filename, content, report);
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
            let best_match =
                best_file_pattern_match(naming_patterns, filename, rel, &self.glob_patterns);

            if let Some((pattern, naming)) = best_match {
                let stem = if file_pattern_uses_lslint_stem(pattern) {
                    file_stem_for_pattern(pattern, filename)
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

fn append_directory_repair(mut message: String, repair: Option<&str>) -> String {
    if let Some(repair) = repair {
        message.push_str(". ");
        message.push_str(repair.trim_end_matches('.'));
        message.push('.');
    }
    message
}
