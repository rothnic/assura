//! Required-section Markdown safe-fix planning.

use super::markdown::markdown_headings;
use super::markdown_fix_report::{MarkdownFixRecord, MarkdownFixStatus};
use crate::stable_hash::stable_hash;
use std::path::Path;

pub(super) const REQUIRED_SECTION_OPERATION: &str = "insert_required_section_heading";

pub(super) struct RequiredSectionFix {
    title: String,
    line_number: usize,
    column_number: usize,
    inserted_text: String,
}

pub(super) fn fix_missing_required_sections(
    content: &str,
    required_sections: &[String],
) -> (String, Vec<RequiredSectionFix>) {
    let headings = markdown_headings(content);
    let heading_depth = if headings.iter().any(|heading| heading.depth == 1) {
        2
    } else {
        1
    };
    let existing = headings
        .iter()
        .map(|heading| heading.text)
        .collect::<std::collections::HashSet<_>>();
    let missing = required_sections
        .iter()
        .filter(|section| !existing.contains(section.as_str()))
        .collect::<Vec<_>>();

    let mut output = content.to_string();
    let mut fixes = Vec::new();
    let newline = newline_for(content);
    for section in missing {
        ensure_markdown_append_boundary(&mut output, newline);
        let line_number = output.lines().count().saturating_add(1);
        let inserted_text = format!("{} {section}", "#".repeat(heading_depth));
        output.push_str(&inserted_text);
        output.push_str(newline);
        fixes.push(RequiredSectionFix {
            title: section.clone(),
            line_number,
            column_number: 1,
            inserted_text,
        });
    }

    (output, fixes)
}

pub(super) fn required_section_fix_record(
    rel: &Path,
    fix: &RequiredSectionFix,
    status: MarkdownFixStatus,
) -> MarkdownFixRecord {
    MarkdownFixRecord {
        id: required_section_fix_id(rel, fix),
        path: rel.to_path_buf(),
        operation: REQUIRED_SECTION_OPERATION,
        status,
        line: fix.line_number,
        column: fix.column_number,
        before_trailing_whitespace: 0,
        after_trailing_whitespace: 0,
        inserted_text: Some(fix.inserted_text.clone()),
    }
}

pub(super) fn required_section_fix_id(rel: &Path, fix: &RequiredSectionFix) -> String {
    let key = format!(
        "{}:{}:{}",
        REQUIRED_SECTION_OPERATION,
        rel.to_string_lossy().replace('\\', "/"),
        fix.title
    );
    format!("markdown.safe_fix.{:016x}", stable_hash(key.as_bytes()))
}

fn ensure_markdown_append_boundary(output: &mut String, newline: &str) {
    if output.is_empty() {
        return;
    }
    if !output.ends_with('\n') {
        output.push_str(newline);
    }
    let blank_boundary = format!("{newline}{newline}");
    if !output.ends_with(&blank_boundary) {
        output.push_str(newline);
    }
}

fn newline_for(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}
