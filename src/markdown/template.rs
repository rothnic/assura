//! Template enforcement for markdown documents
//!
//! Validates that markdown documents follow defined templates with:
//! - Required sections
//! - Section ordering rules
//! - Content patterns per section
//! - Template inheritance

use super::error::{MarkdownResult, MarkdownValidationError};
use super::parser::{HeadingLevel, MarkdownDocument};
use crate::constraints::ValidationFailure;
use serde::{Deserialize, Serialize};

mod template_section;
pub use template_section::{Section, SectionDefinition, SectionValidator};

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
        let _section_titles: Vec<String> = sections
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
            let _template_names: Vec<String> = self
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
mod template_tests;
