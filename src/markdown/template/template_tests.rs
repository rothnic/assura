//! Unit tests for the parent module.
use super::*;
use crate::markdown::parser::Heading;

#[test]
fn test_template_definition_builder() {
    let template = TemplateDefinition::new("api_doc")
        .with_description("API documentation template")
        .with_section(SectionDefinition::new("Overview").required())
        .with_section(SectionDefinition::new("API Reference"))
        .all_sections_required()
        .enforce_order()
        .with_min_sections(2)
        .with_max_sections(5);

    assert_eq!(template.name, "api_doc");
    assert_eq!(template.sections.len(), 2);
    assert!(template.all_sections_required);
    assert!(template.enforce_order);
    assert_eq!(template.min_sections, Some(2));
    assert_eq!(template.max_sections, Some(5));
}

#[test]
fn test_section_definition_builder() {
    let section = SectionDefinition::new("Introduction")
        .required()
        .with_alias("Intro")
        .with_title_pattern(r"^Introduction$")
        .with_word_count(50, 500)
        .with_required_pattern(r"## Prerequisites")
        .with_forbidden_pattern(r"TODO|FIXME")
        .with_required_heading("Examples");

    assert_eq!(section.name, "Introduction");
    assert!(section.required);
    assert_eq!(section.aliases, vec!["Intro"]);
    assert!(section.title_pattern.is_some());
    assert_eq!(section.min_words, Some(50));
    assert_eq!(section.max_words, Some(500));
    assert_eq!(section.required_patterns.len(), 1);
    assert_eq!(section.forbidden_patterns.len(), 1);
    assert_eq!(section.required_headings.len(), 1);
}

#[test]
fn test_section_matches() {
    let section_def = SectionDefinition::new("Getting Started").with_alias("Quick Start");

    let matching = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "Getting Started".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "Content".to_string(),
    };

    let alias_match = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "Quick Start".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "Content".to_string(),
    };

    let non_matching = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "Other Section".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "Content".to_string(),
    };

    assert!(section_def.matches(&matching));
    assert!(section_def.matches(&alias_match));
    assert!(!section_def.matches(&non_matching));
}

#[test]
fn test_section_matches_pattern() {
    let section_def =
        SectionDefinition::new("API Endpoint").with_title_pattern(r"^GET|POST|PUT|DELETE");

    let matching = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "GET /users".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "Content".to_string(),
    };

    let non_matching = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "Overview".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "Content".to_string(),
    };

    assert!(section_def.matches(&matching));
    assert!(!section_def.matches(&non_matching));
}

#[test]
fn test_template_validation() {
    let template = TemplateDefinition::new("simple_doc")
        .with_section(SectionDefinition::new("Overview").required())
        .with_section(SectionDefinition::new("Details"));

    let doc = MarkdownDocument {
        content: "# Title\n\n## Overview\n\nOverview content.\n\n## Details\n\nDetails content."
            .to_string(),
        frontmatter: None,
        body: "# Title\n\n## Overview\n\nOverview content.\n\n## Details\n\nDetails content."
            .to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H1,
                text: "Title".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Overview".to_string(),
                position: 10,
                line_number: 3,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Details".to_string(),
                position: 40,
                line_number: 7,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Title Overview content. Details content.".to_string(),
        line_count: 8,
        word_count: 6,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = template.validate(&doc, &path).unwrap();
    assert!(failures.is_empty());

    // Test missing required section
    let doc_no_overview = MarkdownDocument {
        content: "# Title\n\n## Details\n\nDetails content.".to_string(),
        frontmatter: None,
        body: "# Title\n\n## Details\n\nDetails content.".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H1,
                text: "Title".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Details".to_string(),
                position: 10,
                line_number: 3,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Title Details content.".to_string(),
        line_count: 4,
        word_count: 3,
    };

    let failures = template.validate(&doc_no_overview, &path).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].message.contains("Overview"));
}

#[test]
fn test_section_order_validation() {
    let template = TemplateDefinition::new("ordered_doc")
        .with_section(SectionDefinition::new("First"))
        .with_section(SectionDefinition::new("Second"))
        .enforce_order();

    // Valid order
    let doc_valid = MarkdownDocument {
        content: "## First\n\n## Second".to_string(),
        frontmatter: None,
        body: "## First\n\n## Second".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H2,
                text: "First".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Second".to_string(),
                position: 10,
                line_number: 3,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "First Second".to_string(),
        line_count: 3,
        word_count: 2,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = template.validate(&doc_valid, &path).unwrap();
    assert!(failures.is_empty());

    // Invalid order
    let doc_invalid = MarkdownDocument {
        content: "## Second\n\n## First".to_string(),
        frontmatter: None,
        body: "## Second\n\n## First".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H2,
                text: "Second".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "First".to_string(),
                position: 10,
                line_number: 3,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Second First".to_string(),
        line_count: 3,
        word_count: 2,
    };

    let failures = template.validate(&doc_invalid, &path).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].message.contains("out of order"));
}

#[test]
fn test_section_content_validation() {
    let section = SectionDefinition::new("Introduction")
        .with_word_count(3, 10)
        .with_required_pattern(r"welcome|Welcome");

    let matching_section = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "Introduction".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "Welcome to the project.".to_string(),
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = section
        .validate_content(&matching_section, &MarkdownDocument::default(), &path)
        .unwrap();
    assert!(failures.is_empty());

    let non_matching_section = Section {
        heading: Heading {
            level: HeadingLevel::H2,
            text: "Introduction".to_string(),
            position: 0,
            line_number: 1,
        },
        content: "This is the introduction.".to_string(),
    };

    let failures = section
        .validate_content(&non_matching_section, &MarkdownDocument::default(), &path)
        .unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].message.contains("missing required pattern"));
}

#[test]
fn test_template_merge() {
    let parent = TemplateDefinition::new("parent")
        .with_section(SectionDefinition::new("Overview"))
        .with_section(SectionDefinition::new("Details"));

    let child = TemplateDefinition::new("child")
        .with_section(SectionDefinition::new("Custom"))
        .extends("parent");

    let merged = child.merge_with(&parent);

    // Should have sections from both
    assert_eq!(merged.sections.len(), 3);
    assert!(merged.sections.iter().any(|s| s.name == "Overview"));
    assert!(merged.sections.iter().any(|s| s.name == "Details"));
    assert!(merged.sections.iter().any(|s| s.name == "Custom"));
}

#[test]
fn test_section_validator() {
    let validator = SectionValidator::new()
        .register_template(
            TemplateDefinition::new("blog_post")
                .with_section(SectionDefinition::new("Introduction").required())
                .with_section(SectionDefinition::new("Body").required()),
        )
        .with_default_template("blog_post");

    let doc = MarkdownDocument {
        content: "## Introduction\n\n## Body".to_string(),
        frontmatter: None,
        body: "## Introduction\n\n## Body".to_string(),
        headings: vec![
            Heading {
                level: HeadingLevel::H2,
                text: "Introduction".to_string(),
                position: 0,
                line_number: 1,
            },
            Heading {
                level: HeadingLevel::H2,
                text: "Body".to_string(),
                position: 20,
                line_number: 3,
            },
        ],
        links: vec![],
        code_blocks: vec![],
        text_content: "Introduction Body".to_string(),
        line_count: 3,
        word_count: 2,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = validator.validate(&doc, None, &path).unwrap();
    assert!(failures.is_empty());
}
