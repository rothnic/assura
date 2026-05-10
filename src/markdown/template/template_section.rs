//! Template section definitions and section validators.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::TemplateDefinition;
use crate::constraints::ValidationFailure;
use crate::markdown::error::{MarkdownError, MarkdownResult, MarkdownValidationError};
use crate::markdown::parser::{Heading, MarkdownDocument};

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
