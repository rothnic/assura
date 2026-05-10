//! Markdown parser wrapper
//!
//! This module provides a high-level interface for parsing markdown documents
//! and extracting structured information for validation.

use std::collections::HashMap;

use super::error::{MarkdownError, MarkdownResult};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};

mod parser_types;
pub use parser_types::{CodeBlock, Heading, HeadingHierarchy, HeadingLevel, HeadingNode, Link};

/// Represents a parsed markdown document
#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    /// Raw content
    pub content: String,
    /// Frontmatter content (if any)
    pub frontmatter: Option<String>,
    /// Document body (without frontmatter)
    pub body: String,
    /// All headings in the document
    pub headings: Vec<Heading>,
    /// Links found in the document
    pub links: Vec<Link>,
    /// Code blocks
    pub code_blocks: Vec<CodeBlock>,
    /// All text content
    pub text_content: String,
    /// Line count
    pub line_count: usize,
    /// Word count
    pub word_count: usize,
}

impl MarkdownDocument {
    /// Get the document title (first H1 heading)
    pub fn title(&self) -> Option<&str> {
        self.headings
            .iter()
            .find(|h| h.level == HeadingLevel::H1)
            .map(|h| h.text.as_str())
    }

    /// Get all headings of a specific level
    pub fn headings_by_level(&self, level: HeadingLevel) -> Vec<&Heading> {
        self.headings.iter().filter(|h| h.level == level).collect()
    }

    /// Check if the document has a heading with the given text
    pub fn has_heading(&self, text: &str) -> bool {
        self.headings.iter().any(|h| h.text == text)
    }

    /// Get frontmatter as a YAML value
    pub fn frontmatter_yaml(&self) -> MarkdownResult<Option<serde_yaml::Value>> {
        match &self.frontmatter {
            Some(fm) => {
                let value: serde_yaml::Value = serde_yaml::from_str(fm).map_err(|e| {
                    MarkdownError::yaml("<frontmatter>", format!("Invalid YAML: {}", e))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get frontmatter as a HashMap
    pub fn frontmatter_map(&self) -> MarkdownResult<Option<HashMap<String, serde_yaml::Value>>> {
        match self.frontmatter_yaml()? {
            Some(serde_yaml::Value::Mapping(map)) => {
                let mut result = HashMap::new();
                for (key, value) in map {
                    if let serde_yaml::Value::String(k) = key {
                        result.insert(k, value);
                    }
                }
                Ok(Some(result))
            }
            Some(_) => Ok(None),
            None => Ok(None),
        }
    }

    /// Check if the document has frontmatter
    pub fn has_frontmatter(&self) -> bool {
        self.frontmatter.is_some()
    }

    /// Get the heading hierarchy as a tree structure
    pub fn heading_hierarchy(&self) -> HeadingHierarchy {
        HeadingHierarchy::from_headings(&self.headings)
    }

    /// Count words in the document body
    pub fn count_words(&self) -> usize {
        self.text_content
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .count()
    }

    /// Get content between two headings
    pub fn content_between(&self, start: &Heading, end: Option<&Heading>) -> Option<&str> {
        let start_pos = start.position;
        let end_pos = match end {
            Some(e) => e.position,
            None => self.body.len(),
        };

        if start_pos < self.body.len() && end_pos <= self.body.len() {
            Some(&self.body[start_pos..end_pos])
        } else {
            None
        }
    }
}

/// Markdown parser
#[derive(Debug, Clone)]
pub struct MarkdownParser;

impl MarkdownParser {
    /// Create a new markdown parser
    pub fn new() -> Self {
        Self
    }

    /// Parse markdown content into a document
    pub fn parse(&self, content: &str) -> MarkdownResult<MarkdownDocument> {
        let (frontmatter, body) = self.extract_frontmatter(content);

        let mut headings = Vec::new();
        let mut links = Vec::new();
        let mut code_blocks = Vec::new();
        let mut text_content = String::new();

        let parser = Parser::new(&body);
        let mut current_heading: Option<(HeadingLevel, String, usize)> = None;
        let mut in_code_block = false;
        let mut code_language: Option<String> = None;
        let mut code_content = String::new();
        let mut code_start_line = 0usize;

        for (event, range) in parser.into_offset_iter() {
            match event {
                Event::Start(Tag::Heading(level, _id, _classes)) => {
                    let heading_level = match level {
                        pulldown_cmark::HeadingLevel::H1 => HeadingLevel::H1,
                        pulldown_cmark::HeadingLevel::H2 => HeadingLevel::H2,
                        pulldown_cmark::HeadingLevel::H3 => HeadingLevel::H3,
                        pulldown_cmark::HeadingLevel::H4 => HeadingLevel::H4,
                        pulldown_cmark::HeadingLevel::H5 => HeadingLevel::H5,
                        pulldown_cmark::HeadingLevel::H6 => HeadingLevel::H6,
                    };
                    current_heading = Some((heading_level, String::new(), range.start));
                }
                Event::End(Tag::Heading(_level, _id, _classes)) => {
                    if let Some((level, text, position)) = current_heading.take() {
                        let line_number = self.position_to_line(&body, position);
                        headings.push(Heading {
                            level,
                            text: text.trim().to_string(),
                            position,
                            line_number,
                        });
                    }
                }
                Event::Text(text) => {
                    let text_str = &text;
                    if in_code_block {
                        code_content.push_str(text_str);
                    } else {
                        if let Some((_, ref mut heading_text, _)) = current_heading {
                            heading_text.push_str(text_str);
                        }
                        text_content.push_str(text_str);
                        text_content.push(' ');
                    }
                }
                Event::Code(code) => {
                    if let Some((_, ref mut heading_text, _)) = current_heading {
                        heading_text.push_str(&code);
                    }
                    if in_code_block {
                        code_content.push_str(&code);
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_language = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() {
                                None
                            } else {
                                Some(lang.to_string())
                            }
                        }
                        CodeBlockKind::Indented => None,
                    };
                    code_content.clear();
                    code_start_line = self.position_to_line(&body, range.start);
                }
                Event::End(Tag::CodeBlock(_kind)) if in_code_block => {
                    code_blocks.push(CodeBlock {
                        language: code_language.take(),
                        content: code_content.clone(),
                        line_number: code_start_line,
                    });
                    in_code_block = false;
                    code_content.clear();
                }
                Event::Start(Tag::Link(_type, dest_url, _title)) => {
                    let line_number = self.position_to_line(&body, range.start);
                    links.push(Link {
                        url: dest_url.to_string(),
                        text: String::new(),
                        line_number,
                    });
                }
                _ => {}
            }
        }

        let line_count = content.lines().count();
        let word_count = text_content.split_whitespace().count();

        Ok(MarkdownDocument {
            content: content.to_string(),
            frontmatter,
            body,
            headings,
            links,
            code_blocks,
            text_content,
            line_count,
            word_count,
        })
    }

    /// Extract frontmatter from markdown content
    fn extract_frontmatter(&self, content: &str) -> (Option<String>, String) {
        // Check for YAML frontmatter (--- delimiters)
        if let Some(stripped) = content.strip_prefix("---") {
            if let Some(end_idx) = stripped.find("---") {
                let frontmatter = stripped[..end_idx].trim().to_string();
                let body = stripped[end_idx + 3..].trim_start().to_string();
                return (Some(frontmatter), body);
            }
        }

        // Check for TOML frontmatter (+++ delimiters)
        if let Some(stripped) = content.strip_prefix("+++") {
            if let Some(end_idx) = stripped.find("+++") {
                let frontmatter = stripped[..end_idx].trim().to_string();
                let body = stripped[end_idx + 3..].trim_start().to_string();
                return (Some(frontmatter), body);
            }
        }

        // Check for JSON frontmatter ({ delimiters)
        if content.starts_with("{") {
            let mut brace_count = 0;
            let mut in_string = false;
            let mut escape_next = false;

            for (idx, ch) in content.char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                match ch {
                    '\\' => escape_next = true,
                    '"' if !escape_next => in_string = !in_string,
                    '{' if !in_string => brace_count += 1,
                    '}' if !in_string => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            let body_start = idx + ch.len_utf8();
                            let frontmatter = content[..body_start].trim().to_string();
                            let body = content[body_start..].trim_start().to_string();
                            return (Some(frontmatter), body);
                        }
                    }
                    _ => {}
                }
            }
        }

        (None, content.to_string())
    }

    /// Convert a byte position to a line number
    fn position_to_line(&self, content: &str, position: usize) -> usize {
        content[..position.min(content.len())].lines().count()
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod parser_tests;
