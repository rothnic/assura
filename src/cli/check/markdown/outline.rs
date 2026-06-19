//! Markdown outline validation for the structure-first CLI check path.

use super::super::rules::display_rel;
use super::super::{StructureCheckReport, StructureChecker};
use super::{markdown_headings, MarkdownHeading};
use crate::config::config::{MarkdownBundle, MarkdownOutlineEntry, MarkdownOutlineView};
use regex_lite::Regex;
use std::path::Path;

pub(super) fn validate_markdown_outline(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    report: &mut StructureCheckReport,
) {
    let Some(outline) = markdown.outline.as_deref() else {
        return;
    };

    let headings = markdown_headings(content);
    if let Err(message) = validate_heading_levels(&headings) {
        push_outline_violation(checker, report, rel, &message);
        return;
    }

    if let Err(message) = validate_outline(outline, &headings) {
        push_outline_violation(checker, report, rel, &message);
    }
}

fn validate_heading_levels(headings: &[MarkdownHeading<'_>]) -> Result<(), String> {
    for pair in headings.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        if current.depth > previous.depth + 1 {
            return Err(format!(
                "markdown.outline observed skipped heading level H{} '{}' on line {} after H{} '{}'",
                current.depth, current.text, current.line_number, previous.depth, previous.text
            ));
        }
    }
    Ok(())
}

fn validate_outline(
    outline: &[MarkdownOutlineEntry],
    headings: &[MarkdownHeading<'_>],
) -> Result<(), String> {
    let Some(first_required) = first_required_node(outline)? else {
        return Ok(());
    };

    if headings.is_empty() {
        return Err(
            "Configured markdown.outline cannot match because the document has no headings"
                .to_string(),
        );
    }

    let root = choose_root_match(first_required, headings)?;
    match_outline_entries(
        outline,
        headings,
        root.start_index,
        headings.len(),
        root.level,
    )
}

fn first_required_node(
    outline: &[MarkdownOutlineEntry],
) -> Result<Option<MarkdownOutlineView<'_>>, String> {
    for (index, entry) in outline.iter().enumerate() {
        let view = entry.view(&format!("markdown.outline[{index}]"))?;
        if !view.optional {
            return Ok(Some(view));
        }
    }
    Ok(None)
}

struct RootMatch {
    start_index: usize,
    level: usize,
}

fn choose_root_match(
    first_required: MarkdownOutlineView<'_>,
    headings: &[MarkdownHeading<'_>],
) -> Result<RootMatch, String> {
    if document_title_boundary(headings).is_some() {
        let root_level = 2;
        let start_index = 1;
        let end_index = root_scope_end(headings, start_index, root_level);
        let mut matches = 0usize;
        for heading in &headings[start_index..end_index] {
            if heading.depth == root_level && matches_heading(first_required, heading)? {
                matches += 1;
            }
        }
        match matches {
            0 => {}
            1 => {
                return Ok(RootMatch {
                    start_index,
                    level: root_level,
                });
            }
            _ => {
                return Err(format!(
                    "markdown.outline root entry '{}' is ambiguous; multiple headings can start the outline",
                    first_required.title
                ));
            }
        }
    }

    let mut matches = Vec::new();
    for (index, heading) in headings.iter().enumerate() {
        if matches_heading(first_required, heading)? {
            matches.push((index, heading.depth));
        }
    }

    match matches.as_slice() {
        [] => Err(format!(
            "markdown.outline root entry '{}' was not found in the document headings",
            first_required.title
        )),
        [(index, level)] => Ok(RootMatch {
            start_index: *index,
            level: *level,
        }),
        _ => Err(format!(
            "markdown.outline root entry '{}' is ambiguous; multiple headings can start the outline",
            first_required.title
        )),
    }
}

fn document_title_boundary(headings: &[MarkdownHeading<'_>]) -> Option<usize> {
    if headings.len() < 2 || headings.first()?.depth != 1 {
        return None;
    }
    if headings[1].depth > 1 {
        Some(1)
    } else {
        None
    }
}

fn root_scope_end(headings: &[MarkdownHeading<'_>], start_index: usize, level: usize) -> usize {
    headings
        .iter()
        .enumerate()
        .skip(start_index)
        .find_map(|(index, heading)| (heading.depth < level).then_some(index))
        .unwrap_or(headings.len())
}

fn match_outline_entries(
    entries: &[MarkdownOutlineEntry],
    headings: &[MarkdownHeading<'_>],
    start_index: usize,
    end_index: usize,
    level: usize,
) -> Result<(), String> {
    let mut cursor = start_index;
    for (entry_index, entry) in entries.iter().enumerate() {
        let context = format!("markdown.outline[{entry_index}]");
        let view = entry.view(&context)?;
        let Some(match_index) = find_heading(view, headings, cursor, end_index, level)? else {
            if view.optional {
                continue;
            }
            return Err(format!(
                "{} expected heading '{}' at H{} after line {}",
                context,
                view.title,
                level,
                previous_line(headings, cursor)
            ));
        };

        let child_start = match_index + 1;
        let child_end = section_end(headings, child_start, end_index, level);
        if !view.children.is_empty() {
            match_outline_entries(view.children, headings, child_start, child_end, level + 1)?;
        }
        cursor = child_end;
    }
    Ok(())
}

fn find_heading(
    view: MarkdownOutlineView<'_>,
    headings: &[MarkdownHeading<'_>],
    start_index: usize,
    end_index: usize,
    level: usize,
) -> Result<Option<usize>, String> {
    for (index, heading) in headings
        .iter()
        .enumerate()
        .take(end_index)
        .skip(start_index)
    {
        if heading.depth < level {
            return Ok(None);
        }
        if heading.depth > level {
            return Err(format!(
                "markdown.outline expected '{}' at H{} but observed skipped heading level H{} '{}' on line {}",
                view.title, level, heading.depth, heading.text, heading.line_number
            ));
        }
        if heading.depth == level && matches_heading(view, heading)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn section_end(
    headings: &[MarkdownHeading<'_>],
    start_index: usize,
    end_index: usize,
    parent_level: usize,
) -> usize {
    headings
        .iter()
        .enumerate()
        .take(end_index)
        .skip(start_index)
        .find_map(|(index, heading)| (heading.depth <= parent_level).then_some(index))
        .unwrap_or(end_index)
}

fn matches_heading(
    view: MarkdownOutlineView<'_>,
    heading: &MarkdownHeading<'_>,
) -> Result<bool, String> {
    match view.match_mode {
        "exact" => Ok(heading.text == view.title),
        "regex" => Regex::new(view.title)
            .map(|regex| regex.is_match(heading.text))
            .map_err(|error| {
                format!(
                    "markdown.outline regex '{}' could not be compiled: {}",
                    view.title, error
                )
            }),
        mode => Err(format!(
            "markdown.outline match mode '{}' is not supported",
            mode
        )),
    }
}

fn previous_line(headings: &[MarkdownHeading<'_>], cursor: usize) -> usize {
    cursor
        .checked_sub(1)
        .and_then(|index| headings.get(index))
        .map(|heading| heading.line_number)
        .unwrap_or(0)
}

fn push_outline_violation(
    checker: &StructureChecker,
    report: &mut StructureCheckReport,
    rel: &Path,
    detail: &str,
) {
    checker.push_violation(
        report,
        rel.to_path_buf(),
        "markdown_outline",
        format!(
            "Markdown file '{}' failed outline validation: {detail}",
            display_rel(rel)
        ),
        "medium",
    );
}
