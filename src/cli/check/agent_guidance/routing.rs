//! Skill routing helpers for agent guidance contract checks.

use super::markdown::{inline_code_spans, markdown_links};
use crate::cli::check::CheckError;
use glob::Pattern;
use std::collections::HashSet;
use std::path::Path;

pub(super) struct SkillRoutingTable<'a> {
    skill_column: usize,
    rows: Vec<Vec<&'a str>>,
}

pub(super) struct ReferenceRoutingTable<'a> {
    reference_column: usize,
    rows: Vec<Vec<&'a str>>,
}

impl<'a> SkillRoutingTable<'a> {
    pub(super) fn parse(content: &'a str) -> Option<Self> {
        let table_lines = content
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with('|') && line.ends_with('|'))
            .collect::<Vec<_>>();
        if table_lines.len() < 2 {
            return None;
        }
        let headers = split_table_row(table_lines[0]);
        let skill_column = headers
            .iter()
            .position(|header| normalized_header_matches(header, &["skill", "skills", "load"]))?;
        if !headers
            .iter()
            .any(|header| normalized_header_matches(header, &["when", "use case", "usecase"]))
        {
            return None;
        }
        let rows = table_lines
            .iter()
            .skip(1)
            .filter(|line| !is_markdown_table_separator(line))
            .map(|line| split_table_row(line))
            .filter(|row| row.len() > skill_column)
            .collect::<Vec<_>>();
        (!rows.is_empty()).then_some(Self { skill_column, rows })
    }

    pub(super) fn skill_references(&self) -> Vec<String> {
        self.rows
            .iter()
            .flat_map(|row| skill_reference_tokens(row[self.skill_column]))
            .collect()
    }
}

impl<'a> ReferenceRoutingTable<'a> {
    pub(super) fn parse(content: &'a str) -> Option<Self> {
        let table_lines = markdown_table_lines(content);
        if table_lines.len() < 2 {
            return None;
        }
        let headers = split_table_row(table_lines[0]);
        let reference_column = headers.iter().position(|header| {
            normalized_header_matches(
                header,
                &["read", "reference", "references", "doc", "docs", "load"],
            )
        })?;
        if !headers
            .iter()
            .any(|header| normalized_header_matches(header, &["when", "use case", "usecase"]))
        {
            return None;
        }
        let rows = markdown_table_rows(&table_lines, reference_column);
        (!rows.is_empty()).then_some(Self {
            reference_column,
            rows,
        })
    }

    pub(super) fn reference_targets(&self) -> Vec<String> {
        self.rows
            .iter()
            .flat_map(|row| local_reference_targets(row[self.reference_column]))
            .collect()
    }
}

pub(super) fn skill_name_from_path(path: &Path) -> Option<String> {
    let mut components = path.components().rev();
    if components.next()?.as_os_str() != "SKILL.md" {
        return None;
    }
    Some(components.next()?.as_os_str().to_string_lossy().to_string())
}

pub(super) fn compile_skill_name_patterns(patterns: &[String]) -> Result<Vec<Pattern>, CheckError> {
    patterns
        .iter()
        .map(|pattern| {
            Pattern::new(pattern).map_err(|error| {
                CheckError::Config(crate::cli::config::ConfigError::Invalid(format!(
                    "agent guidance skill-name pattern `{pattern}` is invalid: {error}"
                )))
            })
        })
        .collect()
}

pub(super) fn skill_reference_allowed(
    skill_ref: &str,
    skill_names: &HashSet<String>,
    allowed_patterns: &[Pattern],
) -> bool {
    skill_names.contains(skill_ref)
        || allowed_patterns
            .iter()
            .any(|pattern| pattern.matches(skill_ref))
}

pub(super) fn local_reference_targets(content: &str) -> Vec<String> {
    let mut targets = markdown_links(content);
    targets.extend(inline_code_spans(content));
    targets
}

fn split_table_row(row: &str) -> Vec<&str> {
    row.trim_matches('|').split('|').map(str::trim).collect()
}

fn markdown_table_lines(content: &str) -> Vec<&str> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('|') && line.ends_with('|'))
        .collect()
}

fn markdown_table_rows<'a>(table_lines: &[&'a str], required_column: usize) -> Vec<Vec<&'a str>> {
    table_lines
        .iter()
        .skip(1)
        .filter(|line| !is_markdown_table_separator(line))
        .map(|line| split_table_row(line))
        .filter(|row| row.len() > required_column)
        .collect()
}

fn is_markdown_table_separator(row: &str) -> bool {
    row.trim_matches('|')
        .split('|')
        .all(|cell| cell.trim().chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

fn normalized_header_matches(header: &str, accepted: &[&str]) -> bool {
    let normalized = header
        .to_ascii_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    accepted
        .iter()
        .any(|candidate| normalized.contains(candidate))
}

fn skill_reference_tokens(cell: &str) -> Vec<String> {
    let code_spans = inline_code_spans(cell);
    if !code_spans.is_empty() {
        return code_spans;
    }
    cell.split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '*' | '?')))
        .map(str::trim)
        .filter(|token| token.contains('-') || token.contains('_') || token.contains('*'))
        .map(ToOwned::to_owned)
        .collect()
}
