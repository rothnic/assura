//! Template enforcement for markdown documents
//!
//! Validates that markdown documents follow defined templates with:
//! - Required sections
//! - Section ordering rules
//! - Content patterns per section
//! - Template inheritance

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{MarkdownError, MarkdownResult, MarkdownValidationError};
use super::parser::{Heading, HeadingLevel, MarkdownDocument};
use crate::constraints::ValidationFailure;

/// Definition of a template for markdown documents
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateDefinition {
    /// Template name
    pub name: String,
    /// Template description
    pub description: Option<String>,
    /// Required sections in order
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SectionDefinition>,
    /// Whether all sections are required
    #[serde(default)]
    pub all_sections_required: bool,
    /// Whether section order is enforced
    #[serde(default)]
    pub enforce_order: bool,
    /// Minimum number of sections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_sections: Option<usize>,
    /// Maximum number of sections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sections: Option<usize>,
    /// Allow additional sections not in the template
    #[serde(default)]
    pub allow_additional_sections: bool,
    /// Parent template name (for inheritance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
}

impl TemplateDefinition {
    /// Create a new template definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            sections: Vec::new(),
            all_sections_required: false,
            enforce_order: false,
            min_sections: None,
            max_sections: None,
            allow_additional_sections: true,
            extends: None,
        }
    }

    /// Set template description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a section definition
    pub fn with_section(mut self, section: SectionDefinition) -> Self {
        self.sections.push(section);
        self
    }

    /// Require all sections
    pub fn all_sections_required(mut self) -> Self {
        self.all_sections_required = true;
        self
    }

    /// Enforce section order
    pub fn enforce_order(mut self) -> Self {
        self.enforce_order = true;
        self
    }

    /// Set minimum number of sections
    pub fn with_min_sections(mut self, min: usize) -> Self {
        self.min_sections = Some(min);
        self
    }

    /// Set maximum number of sections
    pub fn with_max_sections(mut self, max: usize) -> Self {
        self.max_sections = Some(max);
        self
    }

    /// Disallow additional sections
    pub fn no_additional_sections(mut self) -> Self {
        self.allow_additional_sections = false;
        self
    }

    /// Set parent template
    pub fn extends(mut self, parent: impl Into<String>) -> Self {
        self.extends = Some(parent.into());
        self
    }

    /// Validate a document against this template
    pub fn validate(
        &self,
        document: &MarkdownDocument,
        path: &std::path::Path,
    ) -> MarkdownResult<Vec<ValidationFailure>> {
        let mut failures = Vec::new();

        // Get all H2-level headings as sections (or use H1 if no H2s)
        let sections = self.extract_sections(document);

        // Check min/max sections
        if let Some(min) = self.min_sections {
            if sections.len() < min {
                failures.push(
                    MarkdownValidationError::new(
                        "too_few_sections",
                        path,
                        format!(
                            "Document has {} sections, but minimum required is {}",
                            sections.len(),
                            min
                        ),
                    )
                    .with_suggestion(format!(
                        "Add at least {} more section(s)",
                        min - sections.len()
                    ))
                    .into_validation_failure(),
                );
            }
        }

        if let Some(max) = self.max_sections {
            if sections.len() > max {
                failures.push(
                    MarkdownValidationError::new(
                        "too_many_sections",
                        path,
                        format!(
                            "Document has {} sections, but maximum allowed is {}",
                            sections.len(),
                            max
                        ),
                    )
                    .with_suggestion(format!(
                        "Remove at least {} section(s)",
                        sections.len() - max
                    ))
                    .into_validation_failure(),
                );
            }
        }

        // Check required sections
        let section_titles: Vec<String> = sections
            .iter()
            .map(|s| s.heading.text.to_lowercase())
            .collect();

        for template_section in &self.sections {
            let found = template_section.matches_any(&sections);

            if template_section.required && !found {
                failures.push(
                    MarkdownValidationError::new(
                        "missing_required_section",
                        path,
                        format!("Missing required section: {}", template_section.name),
                    )
                    .with_suggestion(format!(
                        "Add a section titled '{}' to the document",
                        template_section.name
                    ))
                    .into_validation_failure(),
                );
            }

            if found {
                // Validate the section content
                let matched_sections: Vec<_> = sections
                    .iter()
                    .filter(|s| template_section.matches(s))
                    .collect();

                for matched_section in matched_sections {
                    let content_validation =
                        template_section.validate_content(matched_section, document, path)?;
                    failures.extend(content_validation);
                }
            }
        }

        // Check for additional sections if not allowed
        if !self.allow_additional_sections {
            let template_names: Vec<String> = self
                .sections
                .iter()
                .map(|s| s.name.to_lowercase())
                .collect();

            for section in &sections {
                let is_template_section = self.sections.iter().any(|ts| ts.matches(section));

                if !is_template_section {
                    failures.push(
                        MarkdownValidationError::new(
                            "additional_section",
                            path,
                            format!("Additional section not allowed: '{}'", section.heading.text),
                        )
                        .with_line(section.heading.line_number)
                        .with_suggestion("Remove this section or add it to the template")
                        .into_validation_failure(),
                    );
                }
            }
        }

        // Validate section order
        if self.enforce_order {
            let order_failures = self.validate_section_order(&sections, path);
            failures.extend(order_failures);
        }

        Ok(failures)
    }

    /// Extract sections from a document
    /// Sections are defined as H2 headings (or H1 if document has no H2s)
    fn extract_sections(&self, document: &MarkdownDocument) -> Vec<Section> {
        let mut sections = Vec::new();
        let h2_headings: Vec<_> = document
            .headings
            .iter()
            .filter(|h| h.level == HeadingLevel::H2)
            .collect();

        let headings_to_use = if h2_headings.is_empty() {
            // Use H1s as sections if no H2s
            document
                .headings
                .iter()
                .filter(|h| h.level == HeadingLevel::H1)
                .collect()
        } else {
            h2_headings
        };

        for (i, heading) in headings_to_use.iter().enumerate() {
            let start = heading.position;
            let end = if i + 1 < headings_to_use.len() {
                headings_to_use[i + 1].position
            } else {
                document.body.len()
            };

            let content = if start < document.body.len() && end <= document.body.len() {
                &document.body[start..end]
            } else {
                ""
            };

            sections.push(Section {
                heading: (*heading).clone(),
                content: content.to_string(),
            });
        }

        sections
    }

    /// Validate section ordering
    fn validate_section_order(
        &self,
        sections: &[Section],
        path: &std::path::Path,
    ) -> Vec<ValidationFailure> {
        let mut failures = Vec::new();
        let mut last_matched_index: Option<usize> = None;

        for section in sections {
            if let Some(template_index) = self.find_section_index(&section.heading.text) {
                if let Some(last_index) = last_matched_index {
                    if template_index < last_index {
                        failures.push(
                            MarkdownValidationError::new(
                                "section_order",
                                path,
                                format!("Section '{}' appears out of order", section.heading.text),
                            )
                            .with_line(section.heading.line_number)
                            .with_suggestion(
                                "Move this section to the correct position in the document",
                            )
                            .into_validation_failure(),
                        );
                    }
                }
                last_matched_index = Some(template_index);
            }
        }

        failures
    }

    /// Find the index of a section in the template by name
    fn find_section_index(&self, section_name: &str) -> Option<usize> {
        let name_lower = section_name.to_lowercase();
        self.sections
            .iter()
            .position(|s| s.matches_by_name(&name_lower))
    }

    /// Merge with another template (parent)
    pub fn merge_with(mut self, parent: &Self) -> Self {
        // Merge sections - child sections take precedence
        let mut merged_sections = parent.sections.clone();
        for section in &self.sections {
            // Remove any parent section with same name
            merged_sections.retain(|s| !s.matches_by_name(&section.name.to_lowercase()));
            merged_sections.push(section.clone());
        }
        self.sections = merged_sections;

        // Child settings take precedence for booleans
        if !self.all_sections_required {
            self.all_sections_required = parent.all_sections_required;
        }
        if !self.enforce_order {
            self.enforce_order = parent.enforce_order;
        }
        if self.min_sections.is_none() {
            self.min_sections = parent.min_sections;
        }
        if self.max_sections.is_none() {
            self.max_sections = parent.max_sections;
        }
        if self.allow_additional_sections {
            self.allow_additional_sections = parent.allow_additional_sections;
        }

        self
    }
}

/// Definition of a section in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionDefinition {
    /// Section name
    pub name: String,
    /// Whether this section is required
    #[serde(default)]
    pub required: bool,
    /// Alternative names/titles that match this section
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Pattern to match section title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_pattern: Option<String>,
    /// Minimum content length (characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_content_length: Option<usize>,
    /// Maximum content length (characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_content_length: Option<usize>,
    /// Minimum word count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_words: Option<usize>,
    /// Maximum word count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_words: Option<usize>,
    /// Required content patterns (regex)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_patterns: Vec<String>,
    /// Forbidden content patterns (regex)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_patterns: Vec<String>,
    /// Required headings within this section
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_headings: Vec<String>,
    /// Maximum heading depth within this section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_heading_depth: Option<usize>,
}

impl SectionDefinition {
    /// Create a new section definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
            aliases: Vec::new(),
            title_pattern: None,
            min_content_length: None,
            max_content_length: None,
            min_words: None,
            max_words: None,
            required_patterns: Vec::new(),
            forbidden_patterns: Vec::new(),
            required_headings: Vec::new(),
            max_heading_depth: None,
        }
    }

    /// Make this section required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Add an alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Set title pattern
    pub fn with_title_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.title_pattern = Some(pattern.into());
        self
    }

    /// Set content length constraints
    pub fn with_content_length(mut self, min: usize, max: usize) -> Self {
        self.min_content_length = Some(min);
        self.max_content_length = Some(max);
        self
    }

    /// Set word count constraints
    pub fn with_word_count(mut self, min: usize, max: usize) -> Self {
        self.min_words = Some(min);
        self.max_words = Some(max);
        self
    }

    /// Add required content pattern
    pub fn with_required_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.required_patterns.push(pattern.into());
        self
    }

    /// Add forbidden content pattern
    pub fn with_forbidden_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.forbidden_patterns.push(pattern.into());
        self
    }

    /// Add required subheading
    pub fn with_required_heading(mut self, heading: impl Into<String>) -> Self {
        self.required_headings.push(heading.into());
        self
    }

    /// Set maximum heading depth
    pub fn with_max_heading_depth(mut self, depth: usize) -> Self {
        self.max_heading_depth = Some(depth);
        self
    }

    /// Check if this definition matches a section
    pub fn matches(&self, section: &Section) -> bool {
        let title_lower = section.heading.text.to_lowercase();
        let name_lower = self.name.to_lowercase();

        // Check exact match or alias
        if title_lower == name_lower || self.aliases.iter().any(|a| a.to_lowercase() == title_lower)
        {
            return true;
        }

        // Check pattern match
        if let Some(ref pattern) = self.title_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&section.heading.text) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if this definition matches a section name
    pub fn matches_by_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        let self_name_lower = self.name.to_lowercase();

        if name_lower == self_name_lower {
            return true;
        }

        self.aliases.iter().any(|a| a.to_lowercase() == name_lower)
    }

    /// Check if any section in the list matches this definition
    pub fn matches_any(&self, sections: &[Section]) -> bool {
        sections.iter().any(|s| self.matches(s))
    }

    /// Validate section content
    pub fn validate_content(
        &self,
        section: &Section,
        document: &MarkdownDocument,
        path: &std::path::Path,
    ) -> MarkdownResult<Vec<ValidationFailure>> {
        let mut failures = Vec::new();

        // Check content length
        if let Some(min) = self.min_content_length {
            let content_len = section.content.len();
            if content_len < min {
                failures.push(
                    MarkdownValidationError::new(
                        "section_too_short",
                        path,
                        format!(
                            "Section '{}' is too short ({} chars, minimum {})",
                            section.heading.text, content_len, min
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .with_suggestion("Add more content to this section")
                    .into_validation_failure(),
                );
            }
        }

        if let Some(max) = self.max_content_length {
            let content_len = section.content.len();
            if content_len > max {
                failures.push(
                    MarkdownValidationError::new(
                        "section_too_long",
                        path,
                        format!(
                            "Section '{}' is too long ({} chars, maximum {})",
                            section.heading.text, content_len, max
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .with_suggestion("Reduce the content in this section")
                    .into_validation_failure(),
                );
            }
        }

        // Check word count
        let word_count = section.content.split_whitespace().count();

        if let Some(min) = self.min_words {
            if word_count < min {
                failures.push(
                    MarkdownValidationError::new(
                        "section_too_few_words",
                        path,
                        format!(
                            "Section '{}' has too few words ({} words, minimum {})",
                            section.heading.text, word_count, min
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .into_validation_failure(),
                );
            }
        }

        if let Some(max) = self.max_words {
            if word_count > max {
                failures.push(
                    MarkdownValidationError::new(
                        "section_too_many_words",
                        path,
                        format!(
                            "Section '{}' has too many words ({} words, maximum {})",
                            section.heading.text, word_count, max
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .into_validation_failure(),
                );
            }
        }

        // Check required patterns
        for pattern in &self.required_patterns {
            let regex = Regex::new(pattern).map_err(|e| {
                MarkdownError::configuration(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;

            if !regex.is_match(&section.content) {
                failures.push(
                    MarkdownValidationError::new(
                        "section_missing_pattern",
                        path,
                        format!(
                            "Section '{}' missing required pattern: {}",
                            section.heading.text, pattern
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .into_validation_failure(),
                );
            }
        }

        // Check forbidden patterns
        for pattern in &self.forbidden_patterns {
            let regex = Regex::new(pattern).map_err(|e| {
                MarkdownError::configuration(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;

            if regex.is_match(&section.content) {
                failures.push(
                    MarkdownValidationError::new(
                        "section_forbidden_pattern",
                        path,
                        format!(
                            "Section '{}' contains forbidden pattern: {}",
                            section.heading.text, pattern
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .into_validation_failure(),
                );
            }
        }

        // Check required subheadings
        for required_heading in &self.required_headings {
            let found = document.headings.iter().any(|h| {
                h.text.eq_ignore_ascii_case(required_heading)
                    && h.position > section.heading.position
                    && (h.position - section.heading.position) < section.content.len()
            });

            if !found {
                failures.push(
                    MarkdownValidationError::new(
                        "section_missing_heading",
                        path,
                        format!(
                            "Section '{}' missing required subheading: '{}'",
                            section.heading.text, required_heading
                        ),
                    )
                    .with_line(section.heading.line_number)
                    .into_validation_failure(),
                );
            }
        }

        Ok(failures)
    }
}

/// A section extracted from a document
#[derive(Debug, Clone)]
pub struct Section {
    /// Section heading
    pub heading: Heading,
    /// Section content (text after heading until next section)
    pub content: String,
}

/// Validator for templates
#[derive(Debug)]
pub struct SectionValidator {
    templates: HashMap<String, TemplateDefinition>,
    default_template: Option<String>,
}

impl SectionValidator {
    /// Create a new section validator
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            default_template: None,
        }
    }

    /// Register a template
    pub fn register_template(mut self, template: TemplateDefinition) -> Self {
        self.templates.insert(template.name.clone(), template);
        self
    }

    /// Set the default template
    pub fn with_default_template(mut self, name: impl Into<String>) -> Self {
        self.default_template = Some(name.into());
        self
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&TemplateDefinition> {
        self.templates.get(name)
    }

    /// Validate a document against a template
    pub fn validate(
        &self,
        document: &MarkdownDocument,
        template_name: Option<&str>,
        path: &std::path::Path,
    ) -> MarkdownResult<Vec<ValidationFailure>> {
        let name = template_name
            .or(self.default_template.as_deref())
            .ok_or_else(|| {
                MarkdownError::configuration(
                    "No template specified and no default template set".to_string(),
                )
            })?;

        let template = self.templates.get(name).ok_or_else(|| {
            MarkdownError::configuration(format!("Template '{}' not found", name))
        })?;

        template.validate(document, path)
    }
}

impl Default for SectionValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// A constraint for template enforcement
#[derive(Debug)]
pub struct TemplateConstraint {
    name: String,
    template: TemplateDefinition,
}

impl TemplateConstraint {
    /// Create a new template constraint
    pub fn new(name: impl Into<String>, template: TemplateDefinition) -> Self {
        Self {
            name: name.into(),
            template,
        }
    }

    /// Get the template
    pub fn template(&self) -> &TemplateDefinition {
        &self.template
    }
}

impl crate::constraints::Constraint for TemplateConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates markdown documents against a template"
    }

    fn validate(
        &self,
        path: &std::path::Path,
        _context: &crate::constraints::ConstraintContext,
    ) -> crate::constraints::ConstraintResult<crate::constraints::ConstraintOutput> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::constraints::ConstraintError::io(path, format!("Failed to read file: {}", e))
        })?;

        let parser = super::parser::MarkdownParser::new();
        let document = parser.parse(&content).map_err(|e| {
            crate::constraints::ConstraintError::validation(
                &self.name,
                path,
                format!("Failed to parse markdown: {}", e),
            )
        })?;

        let failures = self.template.validate(&document, path).map_err(|e| {
            crate::constraints::ConstraintError::validation(
                &self.name,
                path,
                format!("Validation failed: {}", e),
            )
        })?;

        let passed = failures.is_empty();
        let failures_collection = crate::constraints::ValidationFailures::from(failures);

        Ok(
            crate::constraints::ConstraintOutput::new(&self.name, path, passed)
                .with_failures(failures_collection),
        )
    }

    fn applies_to(&self, path: &std::path::Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            content:
                "# Title\n\n## Overview\n\nOverview content.\n\n## Details\n\nDetails content."
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
}
