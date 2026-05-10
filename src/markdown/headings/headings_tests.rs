//! Unit tests for the parent module.
use super::*;

#[test]
fn test_heading_validator_builder() {
    let validator = HeadingValidator::new()
        .require_h1()
        .single_h1()
        .validate_hierarchy()
        .with_max_depth(4)
        .with_required_heading(HeadingPattern::Exact("Introduction".to_string()));

    assert!(validator.require_h1);
    assert!(validator.single_h1);
    assert!(validator.validate_hierarchy);
    assert_eq!(validator.max_depth, Some(4));
    assert_eq!(validator.required_headings.len(), 1);
}

#[test]
fn test_heading_pattern_exact() {
    let pattern = HeadingPattern::Exact("Introduction".to_string());

    let matching = Heading {
        level: HeadingLevel::H2,
        text: "Introduction".to_string(),
        position: 0,
        line_number: 1,
    };

    let non_matching = Heading {
        level: HeadingLevel::H2,
        text: "Conclusion".to_string(),
        position: 0,
        line_number: 1,
    };

    assert!(pattern.matches(&matching));
    assert!(!pattern.matches(&non_matching));
    assert!(pattern.matches_any(&[matching.clone(), non_matching.clone()]));
    assert!(!pattern.matches_any(&[non_matching]));
}

#[test]
fn test_heading_pattern_regex() {
    let pattern = HeadingPattern::Regex {
        pattern: r"^\d+\.\s".to_string(),
        case_sensitive: false,
    };

    let matching = Heading {
        level: HeadingLevel::H2,
        text: "1. Introduction".to_string(),
        position: 0,
        line_number: 1,
    };

    let non_matching = Heading {
        level: HeadingLevel::H2,
        text: "Introduction".to_string(),
        position: 0,
        line_number: 1,
    };

    assert!(pattern.matches(&matching));
    assert!(!pattern.matches(&non_matching));
}

#[test]
fn test_text_pattern_rule() {
    let rule = TextPatternRule::new()
        .for_levels(vec![1, 2])
        .min_length(5)
        .max_length(100)
        .must_not_match(r"^[0-9]");

    let heading = Heading {
        level: HeadingLevel::H1,
        text: "Valid Title".to_string(),
        position: 0,
        line_number: 1,
    };

    let path = std::path::PathBuf::from("/test.md");
    let result = rule.validate(&heading, &path).unwrap();
    assert!(result.is_none());

    let short_heading = Heading {
        level: HeadingLevel::H1,
        text: "Hi".to_string(),
        position: 0,
        line_number: 1,
    };

    let result = rule.validate(&short_heading, &path).unwrap();
    assert!(result.is_some());
}

#[test]
fn test_hierarchy_validation() {
    let validator = HeadingValidator::new().validate_hierarchy();

    // Valid hierarchy
    let valid_doc = MarkdownDocument {
        content: "# Title\n\n## Section\n\n### Subsection".to_string(),
        frontmatter: None,
        body: "# Title\n\n## Section\n\n### Subsection".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H1,
                text: "Title".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Section".to_string(),
                position: 10,
                line_number: 3,
            },
            Heading {
                level: HeadingLevel::H3,
                text: "Subsection".to_string(),
                position: 20,
                line_number: 5,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Title Section Subsection".to_string(),
        line_count: 5,
        word_count: 3,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = validator.validate(&valid_doc, &path).unwrap();
    assert!(failures.is_empty());

    // Invalid hierarchy (H1 to H3)
    let invalid_doc = MarkdownDocument {
        content: "# Title\n\n### Section".to_string(),
        frontmatter: None,
        body: "# Title\n\n### Section".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H1,
                text: "Title".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H3,
                text: "Section".to_string(),
                position: 10,
                line_number: 3,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Title Section".to_string(),
        line_count: 3,
        word_count: 2,
    };

    let failures = validator.validate(&invalid_doc, &path).unwrap();
    assert_eq!(failures.len(), 1);
}

#[test]
fn test_missing_h1() {
    let validator = HeadingValidator::new().require_h1();

    let doc = MarkdownDocument {
        content: "## Section".to_string(),
        frontmatter: None,
        body: "## Section".to_string(),
        headings: vec![Heading {
            level: HeadingLevel::H2,
            text: "Section".to_string(),
            position: 0,
            line_number: 1,
        }],
        links: vec![],
        code_blocks: vec![],
        text_content: "Section".to_string(),
        line_count: 1,
        word_count: 1,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = validator.validate(&doc, &path).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].message.contains("H1"));
}

#[test]
fn test_multiple_h1() {
    let validator = HeadingValidator::new().single_h1();

    let doc = MarkdownDocument {
        content: "# Title 1\n# Title 2".to_string(),
        frontmatter: None,
        body: "# Title 1\n# Title 2".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H1,
                text: "Title 1".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H1,
                text: "Title 2".to_string(),
                position: 10,
                line_number: 2,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Title 1 Title 2".to_string(),
        line_count: 2,
        word_count: 4,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = validator.validate(&doc, &path).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].message.contains("only one"));
}

#[test]
fn test_heading_structure_analyze() {
    let doc = MarkdownDocument {
        content: "# Title\n## Section\n### Subsection".to_string(),
        frontmatter: None,
        body: "# Title\n## Section\n### Subsection".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H1,
                text: "Title".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Section".to_string(),
                position: 8,
                line_number: 2,
            },
            Heading {
                level: HeadingLevel::H3,
                text: "Subsection".to_string(),
                position: 18,
                line_number: 3,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Title Section Subsection".to_string(),
        line_count: 3,
        word_count: 3,
    };

    let structure = HeadingStructure::analyze(&doc);
    assert!(structure.is_valid());
    assert_eq!(structure.headings.len(), 3);
    assert_eq!(structure.hierarchy.len(), 1);
    assert_eq!(structure.hierarchy[0].children.len(), 1);
}
