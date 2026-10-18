//! Pattern-specific and directory-wide file limit checks.

use super::patterns::best_file_pattern_match;
use super::rules::{display_rel, parse_size, severity_for_bundle};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::FileBundle;
use std::fs;
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_file_size(
        &self,
        path: &Path,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        report: &mut StructureCheckReport,
    ) {
        let matched = files.max_size_patterns.as_ref().and_then(|patterns| {
            best_file_pattern_match(patterns, filename, rel, &self.glob_patterns)
        });
        let matched_pattern = matched.map(|(pattern, _)| pattern);
        let max_size = matched.map(|(_, value)| value).or(files.max_size.as_ref());
        let (Some(max_size), Ok(metadata)) = (max_size, fs::metadata(path)) else {
            return;
        };
        let Some(max_bytes) = parse_size(max_size) else {
            return;
        };
        if metadata.len() > max_bytes {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "max_size",
                append_repair_message(
                    format!(
                        "File '{}' is {} bytes, exceeding limit {}",
                        display_rel(rel),
                        metadata.len(),
                        max_size
                    ),
                    repair_message(files, matched_pattern, filename, rel, &self.glob_patterns),
                ),
                pattern_severity(files, matched_pattern, filename, rel, &self.glob_patterns),
            );
        }
    }

    pub(super) fn validate_file_line_count(
        &self,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        let matched = files.max_lines_patterns.as_ref().and_then(|patterns| {
            best_file_pattern_match(patterns, filename, rel, &self.glob_patterns)
        });
        let matched_pattern = matched.map(|(pattern, _)| pattern);
        let max_lines = matched.map(|(_, value)| *value).or(files.max_lines);
        if let (Some(max_lines), Some(content)) = (max_lines, content) {
            let line_count = content.lines().count();
            if line_count > max_lines {
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    "max_lines",
                    append_repair_message(
                        format!(
                            "File '{}' has {} lines, exceeding limit {}",
                            display_rel(rel),
                            line_count,
                            max_lines
                        ),
                        repair_message(files, matched_pattern, filename, rel, &self.glob_patterns),
                    ),
                    pattern_severity(files, matched_pattern, filename, rel, &self.glob_patterns),
                );
            }
        }
    }
}

fn pattern_severity(
    files: &FileBundle,
    matched_pattern: Option<&str>,
    filename: &str,
    rel: &Path,
    compiled: &std::collections::HashMap<String, glob::Pattern>,
) -> String {
    matched_pattern
        .and_then(|pattern| files.severity_patterns.as_ref()?.get(pattern))
        .or_else(|| {
            files.severity_patterns.as_ref().and_then(|patterns| {
                best_file_pattern_match(patterns, filename, rel, compiled).map(|(_, value)| value)
            })
        })
        .cloned()
        .unwrap_or_else(|| severity_for_bundle(files))
}

fn repair_message<'a>(
    files: &'a FileBundle,
    matched_pattern: Option<&str>,
    filename: &str,
    rel: &Path,
    compiled: &std::collections::HashMap<String, glob::Pattern>,
) -> Option<&'a str> {
    matched_pattern
        .and_then(|pattern| files.message_patterns.as_ref()?.get(pattern))
        .or_else(|| {
            files.message_patterns.as_ref().and_then(|patterns| {
                best_file_pattern_match(patterns, filename, rel, compiled).map(|(_, value)| value)
            })
        })
        .map(String::as_str)
}

fn append_repair_message(mut message: String, repair: Option<&str>) -> String {
    if let Some(repair) = repair {
        message.push_str(". ");
        message.push_str(repair.trim_end_matches('.'));
        message.push('.');
    }
    message
}
