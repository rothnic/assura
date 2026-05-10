//! Markdown parser wrapper
//!
//! This module provides a high-level interface for parsing markdown documents
//! and extracting structured information for validation.

use std::collections::HashMap;

use super::error::{MarkdownError, MarkdownResult};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};

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

/// Heading level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeadingLevel {
    H1 = 1,
    H2 = 2,
    H3 = 3,
    H4 = 4,
    H5 = 5,
    H6 = 6,
}

impl HeadingLevel {
    /// Create from a numeric level (1-6)
    pub fn from_usize(level: usize) -> Option<Self> {
        match level {
            1 => Some(HeadingLevel::H1),
            2 => Some(HeadingLevel::H2),
            3 => Some(HeadingLevel::H3),
            4 => Some(HeadingLevel::H4),
            5 => Some(HeadingLevel::H5),
            6 => Some(HeadingLevel::H6),
            _ => None,
        }
    }

    /// Get the numeric level
    pub fn as_usize(&self) -> usize {
        *self as usize
    }

    /// Check if this level can follow another level
    pub fn can_follow(&self, previous: Option<HeadingLevel>) -> bool {
        match previous {
            None => *self == HeadingLevel::H1,
            Some(prev) => {
                let prev_level = prev.as_usize();
                let this_level = self.as_usize();
                // Can be same level, one level deeper, or any level up
                this_level >= 1 && this_level <= prev_level + 1
            }
        }
    }
}

impl std::fmt::Display for HeadingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "H{}", self.as_usize())
    }
}

/// Represents a heading in a markdown document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// Heading level (1-6)
    pub level: HeadingLevel,
    /// Heading text content
    pub text: String,
    /// Position in the document body
    pub position: usize,
    /// Line number in the source
    pub line_number: usize,
}

/// Represents a link in a markdown document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Link URL
    pub url: String,
    /// Link text
    pub text: String,
    /// Line number in the source
    pub line_number: usize,
}

/// Represents a code block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    /// Language identifier (if any)
    pub language: Option<String>,
    /// Code content
    pub content: String,
    /// Line number in the source
    pub line_number: usize,
}

/// Hierarchical structure of headings
#[derive(Debug, Clone)]
pub struct HeadingHierarchy {
    pub nodes: Vec<HeadingNode>,
}

impl HeadingHierarchy {
    fn from_headings(headings: &[Heading]) -> Self {
        let mut nodes = Vec::new();
        let mut stack: Vec<(HeadingLevel, Vec<HeadingNode>)> = Vec::new();

        for heading in headings {
            let node = HeadingNode {
                heading: heading.clone(),
                children: Vec::new(),
            };

            // Process Section 2 (H2): H2 <= H3, so pop H3 first
            // Pop (H3, [SubsectionNode]), attach to Section 1
            // Now H2 <= H2, pop (H2, [Section1Node_with_Subsection])
            // Attach Section 1 to Title, push (H1, [TitleNode_with_Section1])
            // Now H2 > H1, push (H2, [Section2Node])
            // Stack: [(H1, [TitleNode_with_Section1]), (H2, [Section2Node])]

            // When processing Section 2 (H2), the algorithm pops until it finds a parent
            // with level < H2, which is H1. But the issue is that when it pops Section 1
            // and attaches it to Title, it replaces Title's children instead of extending them.

            // After flush:
            // Pop (H2, [Section2Node]), attach to Title, replacing Section 1
            // Result: Title has only Section 2 as child

            // The fix is to extend children instead of replacing them.

            // Pop stack until we find the parent or stack is empty
            while let Some((level, _)) = stack.last() {
                if heading.level.as_usize() > level.as_usize() {
                    break;
                }
                let (_, children) = stack.pop().unwrap();
                if let Some((_, parent_children)) = stack.last_mut() {
                    if let Some(mut parent) = parent_children.pop() {
                        parent.children.extend(children); // Extend instead of replace
                        parent_children.push(parent);
                    }
                } else {
                    // No parent, these are top-level nodes
                    nodes.extend(children);
                }
            }

            // Push current node
            stack.push((heading.level, vec![node]));
        }

        // Flush remaining stack
        while let Some((_, children)) = stack.pop() {
            if let Some((_, parent_children)) = stack.last_mut() {
                if let Some(mut parent) = parent_children.pop() {
                    parent.children.extend(children); // Extend instead of replace
                    parent_children.push(parent);
                }
            } else {
                nodes.extend(children);
            }
        }

        Self { nodes }
    }

    /// Check if the hierarchy is valid (no skipped levels)
    pub fn is_valid(&self) -> bool {
        for node in &self.nodes {
            if node.heading.level != HeadingLevel::H1 {
                return false;
            }
            if !Self::check_children_valid(&node.children, HeadingLevel::H1) {
                return false;
            }
        }
        true
    }

    fn check_children_valid(children: &[HeadingNode], parent_level: HeadingLevel) -> bool {
        for child in children {
            let level_diff = child.heading.level.as_usize() - parent_level.as_usize();
            // Children must be exactly one level deeper
            if level_diff != 1 {
                return false;
            }
            if !Self::check_children_valid(&child.children, child.heading.level) {
                return false;
            }
        }
        true
    }

    /// Get all heading texts at a specific level
    pub fn get_headings_at_level(&self, level: HeadingLevel) -> Vec<String> {
        let mut result = Vec::new();
        for node in &self.nodes {
            Self::collect_headings_at_level(node, level, &mut result);
        }
        result
    }

    fn collect_headings_at_level(
        node: &HeadingNode,
        level: HeadingLevel,
        result: &mut Vec<String>,
    ) {
        if node.heading.level == level {
            result.push(node.heading.text.clone());
        }
        for child in &node.children {
            Self::collect_headings_at_level(child, level, result);
        }
    }
}

/// A node in the heading hierarchy
#[derive(Debug, Clone)]
pub struct HeadingNode {
    pub heading: Heading,
    pub children: Vec<HeadingNode>,
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
                Event::End(Tag::CodeBlock(_kind)) => {
                    if in_code_block {
                        code_blocks.push(CodeBlock {
                            language: code_language.take(),
                            content: code_content.clone(),
                            line_number: code_start_line,
                        });
                        in_code_block = false;
                        code_content.clear();
                    }
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
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let parser = MarkdownParser::new();
        let content = "# Title\n\nSome body text.";
        let doc = parser.parse(content).unwrap();

        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].level, HeadingLevel::H1);
        assert_eq!(doc.headings[0].text, "Title");
        assert!(doc.frontmatter.is_none());
    }

    #[test]
    fn test_parse_with_frontmatter() {
        let parser = MarkdownParser::new();
        let content = "---\ntitle: Test\n---\n\n# Title\n\nBody.";
        let doc = parser.parse(content).unwrap();

        assert!(doc.frontmatter.is_some());
        assert!(doc.frontmatter.as_ref().unwrap().contains("title: Test"));
        assert_eq!(doc.title(), Some("Title"));
    }

    #[test]
    fn test_heading_levels() {
        let parser = MarkdownParser::new();
        let content = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
        let doc = parser.parse(content).unwrap();

        assert_eq!(doc.headings.len(), 6);
        assert_eq!(doc.headings[0].level, HeadingLevel::H1);
        assert_eq!(doc.headings[5].level, HeadingLevel::H6);
    }

    #[test]
    fn test_heading_hierarchy() {
        let parser = MarkdownParser::new();
        let content = "# Title\n## Section 1\n### Subsection\n## Section 2";
        let doc = parser.parse(content).unwrap();

        let hierarchy = doc.heading_hierarchy();
        assert_eq!(hierarchy.nodes.len(), 1);
        assert_eq!(hierarchy.nodes[0].children.len(), 2);
        assert_eq!(hierarchy.nodes[0].children[0].children.len(), 1);
    }

    #[test]
    fn test_code_blocks() {
        let parser = MarkdownParser::new();
        let content = r#"# Title

```rust
fn main() {}
```
"#;
        let doc = parser.parse(content).unwrap();

        assert_eq!(doc.code_blocks.len(), 1);
        assert_eq!(doc.code_blocks[0].language, Some("rust".to_string()));
        assert!(doc.code_blocks[0].content.contains("fn main"));
    }

    #[test]
    fn test_links() {
        let parser = MarkdownParser::new();
        let content = "# Title\n\n[Link text](https://example.com)";
        let doc = parser.parse(content).unwrap();

        assert_eq!(doc.links.len(), 1);
        assert_eq!(doc.links[0].url, "https://example.com");
    }

    #[test]
    fn test_heading_level_can_follow() {
        assert!(HeadingLevel::H1.can_follow(None));
        assert!(HeadingLevel::H2.can_follow(Some(HeadingLevel::H1)));
        assert!(HeadingLevel::H1.can_follow(Some(HeadingLevel::H1)));
        assert!(HeadingLevel::H3.can_follow(Some(HeadingLevel::H2)));
        assert!(!HeadingLevel::H3.can_follow(Some(HeadingLevel::H1)));
    }

    #[test]
    fn test_word_count() {
        let parser = MarkdownParser::new();
        let content = "# Title\n\nThis is a test with five words.";
        let doc = parser.parse(content).unwrap();

        assert_eq!(doc.count_words(), 8); // "Title" + "This is a test with five words" = 8 words
    }

    #[test]
    fn test_frontmatter_yaml() {
        let parser = MarkdownParser::new();
        let content = "---\ntitle: Test\ndate: 2024-01-01\n---\n\n# Title";
        let doc = parser.parse(content).unwrap();

        let yaml = doc.frontmatter_yaml().unwrap();
        assert!(yaml.is_some());

        let map = doc.frontmatter_map().unwrap();
        assert!(map.is_some());
        assert!(map.as_ref().unwrap().contains_key("title"));
    }
}
