//! File, directory, and markdown validators for structure-first checks.

use super::rules::{
    display_rel, parse_frontmatter, parse_size, severity_for_bundle, validate_file_stem,
    validate_name,
};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::{FileBundle, MarkdownBundle};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_directory(&mut self, path: &Path, report: &mut StructureCheckReport) {
        let rel = self.relative_path(path);
        if rel.as_os_str().is_empty() || self.configured_dirs.contains(&rel) {
            return;
        }

        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let rules = self.resolve_rules(parent_rel);
        let Some(files) = rules.files else {
            return;
        };

        let Some(naming) = files.naming.as_deref() else {
            return;
        };

        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !validate_name(name, naming, &self.naming_regexes) {
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

        let needs_markdown =
            path.extension().and_then(|ext| ext.to_str()) == Some("md") && rules.markdown.is_some();
        let needs_file_content = rules.files.as_ref().is_some_and(|files| {
            files.max_lines.is_some()
                || (files.require_docs == Some(true)
                    && path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        });
        let content = if needs_file_content || needs_markdown {
            match fs::read_to_string(path) {
                Ok(content) => Some(content),
                Err(error) => {
                    let severity = rules
                        .files
                        .as_ref()
                        .map(severity_for_bundle)
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

        if needs_markdown {
            if let (Some(markdown), Some(content)) = (rules.markdown, content.as_deref()) {
                self.validate_markdown(&rel, &markdown, content, report);
            }
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

        self.validate_file_extension(path, rel, files, filename, report);
        self.validate_file_name(path, rel, files, filename, allowed_by_name, report);
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
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !extensions.iter().any(|allowed| allowed == ext) {
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

        if let Some(naming) = &files.naming {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");
            if !validate_file_stem(stem, naming, &self.naming_regexes) {
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

    fn validate_markdown(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let frontmatter = parse_frontmatter(content);

        self.validate_markdown_frontmatter(rel, markdown, frontmatter, report);
        self.validate_markdown_heading_depth(rel, markdown, content, report);
        self.validate_markdown_required_sections(rel, markdown, content, report);
    }

    fn validate_markdown_frontmatter(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        frontmatter: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        if markdown.require_frontmatter == Some(true) && frontmatter.is_none() {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "markdown_frontmatter",
                format!(
                    "Markdown file '{}' is missing YAML frontmatter",
                    display_rel(rel)
                ),
                "medium",
            );
        }

        if let Some(required_fields) = &markdown.required_fields {
            match frontmatter {
                Some(frontmatter) => match serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                    Ok(value) => {
                        for field in required_fields {
                            if value.get(field).is_none() {
                                push_missing_frontmatter_field(self, report, rel, field);
                            }
                        }
                    }
                    Err(error) => {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "markdown_frontmatter_parse",
                            format!(
                                "Markdown file '{}' has invalid frontmatter: {}",
                                display_rel(rel),
                                error
                            ),
                            "medium",
                        );
                    }
                },
                None => {
                    for field in required_fields {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "markdown_frontmatter_field",
                            format!(
                                "Markdown file '{}' cannot satisfy required field '{}' without frontmatter",
                                display_rel(rel),
                                field
                            ),
                            "medium",
                        );
                    }
                }
            }
        }
    }

    fn validate_markdown_heading_depth(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        if let Some(max_depth) = markdown.max_heading_depth {
            for line in content.lines() {
                let depth = line.chars().take_while(|ch| *ch == '#').count();
                if depth > usize::from(max_depth) && line.chars().nth(depth) == Some(' ') {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "markdown_heading_depth",
                        format!(
                            "Markdown file '{}' has heading depth {}, exceeding limit {}",
                            display_rel(rel),
                            depth,
                            max_depth
                        ),
                        "medium",
                    );
                    break;
                }
            }
        }
    }

    fn validate_markdown_required_sections(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        if let Some(required_sections) = &markdown.required_sections {
            let mut headings = HashSet::new();
            for line in content.lines() {
                if let Some(section) = line.strip_prefix("# ").or_else(|| line.strip_prefix("## "))
                {
                    headings.insert(section);
                }
            }

            for section in required_sections {
                if !headings.contains(section.as_str()) {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "markdown_required_section",
                        format!(
                            "Markdown file '{}' is missing required section '{}'",
                            display_rel(rel),
                            section
                        ),
                        "medium",
                    );
                }
            }
        }
    }
}

fn push_missing_frontmatter_field(
    checker: &StructureChecker,
    report: &mut StructureCheckReport,
    rel: &Path,
    field: &str,
) {
    checker.push_violation(
        report,
        rel.to_path_buf(),
        "markdown_frontmatter_field",
        format!(
            "Markdown file '{}' is missing frontmatter field '{}'",
            display_rel(rel),
            field
        ),
        "medium",
    );
}
