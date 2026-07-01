//! Reasoned Markdown finding suppressions.

use super::is_fence_start;
/// Parsed Markdown suppression comments for one document.
pub(super) struct MarkdownSuppressions {
    suppressions: Vec<MarkdownSuppression>,
    invalid: Vec<InvalidMarkdownSuppression>,
}

struct MarkdownSuppression {
    rule: String,
    line_number: usize,
    used: bool,
}

/// Invalid suppression comment found in Markdown content.
pub(super) struct InvalidMarkdownSuppression {
    pub(super) line_number: usize,
    pub(super) reason: String,
}

impl MarkdownSuppressions {
    /// Parse `assura-ignore` comments in Markdown content.
    pub(super) fn parse(content: &str) -> Self {
        let mut suppressions = Vec::new();
        let mut invalid = Vec::new();

        let mut in_fence = false;
        for (line_index, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if is_fence_start(trimmed) {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            let mut rest = line;
            while let Some(start) = rest.find("<!--") {
                let after_open = &rest[start + 4..];
                if is_inside_inline_code_span(line, start) {
                    rest = after_open;
                    continue;
                }
                let Some(end) = after_open.find("-->") else {
                    if after_open.trim_start().starts_with("assura-ignore") {
                        invalid.push(InvalidMarkdownSuppression {
                            line_number: line_index + 1,
                            reason: "suppression comment must end with `-->`".to_string(),
                        });
                    }
                    break;
                };
                let comment = after_open[..end].trim();
                if let Some(body) = comment.strip_prefix("assura-ignore") {
                    match parse_suppression_body(body.trim()) {
                        Ok(rule) => {
                            suppressions.push(MarkdownSuppression {
                                rule: rule.to_string(),
                                line_number: line_index + 1,
                                used: false,
                            });
                        }
                        Err(reason) => invalid.push(InvalidMarkdownSuppression {
                            line_number: line_index + 1,
                            reason: reason.to_string(),
                        }),
                    }
                }
                rest = &after_open[end + 3..];
            }
        }

        Self {
            suppressions,
            invalid,
        }
    }

    /// Whether one Markdown finding for `rule` and `line_number` should be suppressed.
    pub(super) fn suppresses(&mut self, rule: &str, line_number: usize) -> bool {
        let Some(index) = self
            .suppressions
            .iter()
            .enumerate()
            .filter(|(_, suppression)| {
                !suppression.used
                    && suppression.rule == rule
                    && suppression.line_number < line_number
            })
            .max_by_key(|(_, suppression)| suppression.line_number)
            .map(|(index, _)| index)
        else {
            return false;
        };
        self.suppressions[index].used = true;
        true
    }

    /// Invalid suppression comments that should be reported.
    pub(super) fn invalid(&self) -> &[InvalidMarkdownSuppression] {
        &self.invalid
    }
}

fn parse_suppression_body(body: &str) -> Result<&str, &'static str> {
    let Some((rule, reason)) = body.split_once(':') else {
        return Err("suppression must use `assura-ignore <rule>: <reason>`");
    };
    let rule = rule.trim();
    let reason = reason.trim();
    if rule.is_empty() {
        return Err("suppression rule id must not be empty");
    }
    if !is_supported_markdown_rule(rule) {
        return Err("suppression rule id must be a supported markdown_* rule");
    }
    if reason.is_empty() {
        return Err("suppression reason must not be empty");
    }
    Ok(rule)
}

fn is_inside_inline_code_span(line: &str, byte_index: usize) -> bool {
    let bytes = line.as_bytes();
    let mut in_code = false;
    let mut index = 0;
    while index < byte_index {
        if bytes[index] == b'`' {
            while index < byte_index && bytes[index] == b'`' {
                index += 1;
            }
            in_code = !in_code;
        } else {
            index += 1;
        }
    }
    in_code
}

fn is_supported_markdown_rule(rule: &str) -> bool {
    matches!(
        rule,
        "markdown_frontmatter"
            | "markdown_heading_depth"
            | "markdown_required_section"
            | "markdown_outline"
            | "markdown_trailing_spaces"
            | "markdown_heading_increment"
            | "markdown_heading_marker_spacing"
            | "markdown_duplicate_heading"
            | "markdown_multiple_blank_lines"
            | "markdown_link_format"
            | "markdown_link_target"
            | "markdown_link_heading_anchor"
            | "markdown_link_line_anchor"
    )
}
