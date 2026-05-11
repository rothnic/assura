//! Heading structure validation
//!
//! Validates markdown heading hierarchy and structure:
//! - H1-H6 hierarchy validation (no skipped levels)
//! - Duplicate H1 detection
//! - Missing required headings
//! - Heading content patterns

use serde::{Deserialize, Serialize};

use super::error::{MarkdownResult, MarkdownValidationError};
use super::parser::{Heading, HeadingLevel, MarkdownDocument};
use crate::constraints::ValidationFailure;

mod heading_patterns;
mod heading_structure;
pub use heading_patterns::{HeadingPattern, TextPatternRule};
pub use heading_structure::{
    HeadingErrorType, HeadingHierarchyNode, HeadingStructure, HeadingValidationError,
};

/// Validator for heading structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingValidator {
    /// Whether H1 is required
    #[serde(default)]
    pub require_h1: bool,
    /// Whether only one H1 is allowed
    #[serde(default)]
    pub single_h1: bool,
    /// Whether to validate hierarchy (no skipped levels)
    #[serde(default)]
    pub validate_hierarchy: bool,
    /// Maximum allowed heading depth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    /// Required headings (by text pattern)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_headings: Vec<HeadingPattern>,
    /// Forbidden headings (by text pattern)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_headings: Vec<HeadingPattern>,
    /// Heading text patterns to validate
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_patterns: Vec<TextPatternRule>,
}

impl HeadingValidator {
    /// Create a new heading validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Require an H1 heading
    pub fn require_h1(mut self) -> Self {
        self.require_h1 = true;
        self
    }

    /// Only allow a single H1 heading
    pub fn single_h1(mut self) -> Self {
        self.single_h1 = true;
        self
    }

    /// Enable hierarchy validation
    pub fn validate_hierarchy(mut self) -> Self {
        self.validate_hierarchy = true;
        self
    }

    /// Set maximum heading depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Add a required heading pattern
    pub fn with_required_heading(mut self, pattern: HeadingPattern) -> Self {
        self.required_headings.push(pattern);
        self
    }

    /// Add a forbidden heading pattern
    pub fn with_forbidden_heading(mut self, pattern: HeadingPattern) -> Self {
        self.forbidden_headings.push(pattern);
        self
    }

    /// Add a text pattern rule
    pub fn with_text_pattern(mut self, rule: TextPatternRule) -> Self {
        self.text_patterns.push(rule);
        self
    }

    /// Validate a document's heading structure
    pub fn validate(
        &self,
        document: &MarkdownDocument,
        path: &std::path::Path,
    ) -> MarkdownResult<Vec<ValidationFailure>> {
        let mut failures = Vec::new();

        // Check H1 requirements
        if self.require_h1 || self.single_h1 {
            let h1s: Vec<&Heading> = document
                .headings
                .iter()
                .filter(|h| h.level == HeadingLevel::H1)
                .collect();

            if self.require_h1 && h1s.is_empty() {
                failures.push(
                    MarkdownValidationError::new(
                        "missing_h1",
                        path,
                        "Document must have an H1 heading",
                    )
                    .with_suggestion("Add a single # Title at the beginning of the document")
                    .into_validation_failure(),
                );
            }

            if self.single_h1 && h1s.len() > 1 {
                failures.push(
                    MarkdownValidationError::new(
                        "multiple_h1",
                        path,
                        format!(
                            "Document has {} H1 headings, but only one is allowed",
                            h1s.len()
                        ),
                    )
                    .with_suggestion(
                        "Consolidate multiple H1 headings into one or use H2 for subsections",
                    )
                    .into_validation_failure(),
                );
            }
        }

        // Validate hierarchy (no skipped levels)
        if self.validate_hierarchy {
            let hierarchy_failures = self.check_hierarchy(&document.headings, path);
            failures.extend(hierarchy_failures);
        }

        // Check maximum depth
        if let Some(max_depth) = self.max_depth {
            for heading in &document.headings {
                if heading.level.as_usize() > max_depth {
                    failures.push(
                        MarkdownValidationError::new(
                            "heading_too_deep",
                            path,
                            format!(
                                "Heading '{}' is at level {}, but maximum depth is {}",
                                heading.text,
                                heading.level.as_usize(),
                                max_depth
                            ),
                        )
                        .with_line(heading.line_number)
                        .with_suggestion(format!(
                            "Reduce heading level to H{} or use a different structure",
                            max_depth
                        ))
                        .into_validation_failure(),
                    );
                }
            }
        }

        // Validate required headings
        for pattern in &self.required_headings {
            if !pattern.matches_any(&document.headings) {
                failures.push(
                    MarkdownValidationError::new(
                        "missing_required_heading",
                        path,
                        format!("Missing required heading matching pattern: {:?}", pattern),
                    )
                    .with_suggestion(format!(
                        "Add a heading that matches the pattern: {:?}",
                        pattern
                    ))
                    .into_validation_failure(),
                );
            }
        }

        // Validate forbidden headings
        for heading in &document.headings {
            for pattern in &self.forbidden_headings {
                if pattern.matches(heading) {
                    failures.push(
                        MarkdownValidationError::new(
                            "forbidden_heading",
                            path,
                            format!("Found forbidden heading: '{}'", heading.text),
                        )
                        .with_line(heading.line_number)
                        .with_suggestion("Remove or rename this heading")
                        .into_validation_failure(),
                    );
                }
            }
        }

        // Validate heading text patterns
        for heading in &document.headings {
            for rule in &self.text_patterns {
                if let Some(failure) = rule.validate(heading, path)? {
                    failures.push(failure.into_validation_failure());
                }
            }
        }

        Ok(failures)
    }

    /// Check heading hierarchy (no skipped levels)
    fn check_hierarchy(
        &self,
        headings: &[Heading],
        path: &std::path::Path,
    ) -> Vec<ValidationFailure> {
        let mut failures = Vec::new();
        let mut last_level: Option<HeadingLevel> = None;

        for heading in headings {
            let current_level = heading.level;

            // Check if this heading can follow the last one
            if let Some(prev) = last_level {
                let prev_num = prev.as_usize();
                let current_num = current_level.as_usize();

                // Can be same level, one level deeper, or go back up
                // But cannot skip levels when going deeper
                if current_num > prev_num && current_num != prev_num + 1 {
                    failures.push(
                        MarkdownValidationError::new(
                            "skipped_heading_level",
                            path,
                            format!(
                                "Skipped heading level: H{} followed by H{} without H{}",
                                prev_num,
                                current_num,
                                prev_num + 1
                            ),
                        )
                        .with_line(heading.line_number)
                        .with_suggestion(format!(
                            "Insert an H{} heading before this one, or change to H{}",
                            prev_num + 1,
                            prev_num
                        ))
                        .into_validation_failure(),
                    );
                }

                // First heading must be H1
                if last_level.is_none() && current_num != 1 {
                    failures.push(
                        MarkdownValidationError::new(
                            "first_heading_not_h1",
                            path,
                            format!("First heading should be H1, but found H{}", current_num),
                        )
                        .with_line(heading.line_number)
                        .with_suggestion("Change this to a single # Heading")
                        .into_validation_failure(),
                    );
                }
            } else {
                // First heading must be H1
                if current_level != HeadingLevel::H1 {
                    failures.push(
                        MarkdownValidationError::new(
                            "first_heading_not_h1",
                            path,
                            format!("First heading should be H1, but found {}", current_level),
                        )
                        .with_line(heading.line_number)
                        .with_suggestion("Change this to a single # Heading")
                        .into_validation_failure(),
                    );
                }
            }

            last_level = Some(current_level);
        }

        failures
    }

    /// Merge with another validator (parent)
    pub fn merge_with(mut self, parent: &Self) -> Self {
        // Child settings take precedence for boolean flags
        if !self.require_h1 {
            self.require_h1 = parent.require_h1;
        }
        if !self.single_h1 {
            self.single_h1 = parent.single_h1;
        }
        if !self.validate_hierarchy {
            self.validate_hierarchy = parent.validate_hierarchy;
        }
        if self.max_depth.is_none() {
            self.max_depth = parent.max_depth;
        }

        // Merge lists
        let mut required = parent.required_headings.clone();
        required.extend(self.required_headings);
        self.required_headings = required;

        let mut forbidden = parent.forbidden_headings.clone();
        forbidden.extend(self.forbidden_headings);
        self.forbidden_headings = forbidden;

        let mut patterns = parent.text_patterns.clone();
        patterns.extend(self.text_patterns);
        self.text_patterns = patterns;

        self
    }
}

/// A constraint for validating heading structure
#[derive(Debug)]
pub struct HeadingConstraint {
    name: String,
    validator: HeadingValidator,
}

impl HeadingConstraint {
    /// Create a new heading constraint
    pub fn new(name: impl Into<String>, validator: HeadingValidator) -> Self {
        Self {
            name: name.into(),
            validator,
        }
    }

    /// Get the validator
    pub fn validator(&self) -> &HeadingValidator {
        &self.validator
    }
}

impl crate::constraints::Constraint for HeadingConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates markdown heading structure"
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

        let failures = self.validator.validate(&document, path).map_err(|e| {
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
mod headings_tests;
