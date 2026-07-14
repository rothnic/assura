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
        let max_size = files
            .max_size_patterns
            .as_ref()
            .and_then(|patterns| {
                best_file_pattern_match(patterns, filename, rel, &self.glob_patterns)
                    .map(|(_, max_size)| max_size)
            })
            .or(files.max_size.as_ref());
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

    pub(super) fn validate_file_line_count(
        &self,
        rel: &Path,
        files: &FileBundle,
        filename: &str,
        content: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        let max_lines = files
            .max_lines_patterns
            .as_ref()
            .and_then(|patterns| {
                best_file_pattern_match(patterns, filename, rel, &self.glob_patterns)
                    .map(|(_, max_lines)| *max_lines)
            })
            .or(files.max_lines);
        if let (Some(max_lines), Some(content)) = (max_lines, content) {
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
}
