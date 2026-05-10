//! Unit tests for the parent module.
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
