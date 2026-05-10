//! Heading structure validation
//!
//! Validates markdown heading hierarchy and structure:
//! - H1-H6 hierarchy validation (no skipped levels)
//! - Duplicate H1 detection
//! - Missing required headings
//! - Heading content patterns

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::error::{MarkdownError, MarkdownResult, MarkdownValidationError};
use super::parser::{Heading, HeadingLevel, MarkdownDocument};
use crate::constraints::ValidationFailure;

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

/// Pattern for matching headings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HeadingPattern {
    /// Exact text match (case-insensitive)
    Exact(String),
    /// Regex pattern match
    Regex {
        /// Regex pattern
        pattern: String,
        /// Whether case-sensitive
        #[serde(default)]
        case_sensitive: bool,
    },
    /// Minimum heading level
    MinLevel(usize),
    /// Level and pattern combination
    LevelAndPattern {
        /// Required level (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        level: Option<usize>,
        /// Text pattern to match
        pattern: String,
        /// Whether it's a regex pattern
        #[serde(default)]
        is_regex: bool,
    },
}

impl HeadingPattern {
    /// Check if this pattern matches a heading
    pub fn matches(&self, heading: &Heading) -> bool {
        match self {
            HeadingPattern::Exact(text) => heading.text.eq_ignore_ascii_case(text),
            HeadingPattern::Regex {
                pattern,
                case_sensitive,
            } => {
                let regex = if *case_sensitive {
                    Regex::new(pattern)
                } else {
                    Regex::new(&format!("(?i){}", pattern))
                };

                match regex {
                    Ok(re) => re.is_match(&heading.text),
                    Err(_) => false,
                }
            }
            HeadingPattern::MinLevel(min) => heading.level.as_usize() >= *min,
            HeadingPattern::LevelAndPattern {
                level,
                pattern,
                is_regex,
            } => {
                // Check level if specified
                if let Some(l) = level {
                    if heading.level.as_usize() != *l {
                        return false;
                    }
                }

                // Check pattern
                if *is_regex {
                    Regex::new(pattern)
                        .map(|re| re.is_match(&heading.text))
                        .unwrap_or(false)
                } else {
                    heading.text.eq_ignore_ascii_case(pattern)
                }
            }
        }
    }

    /// Check if any heading matches this pattern
    pub fn matches_any(&self, headings: &[Heading]) -> bool {
        headings.iter().any(|h| self.matches(h))
    }
}

/// Rule for validating heading text patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPatternRule {
    /// Which heading levels to apply to (optional = all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<usize>>,
    /// Pattern that text must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_match: Option<String>,
    /// Pattern that text must not match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_not_match: Option<String>,
    /// Minimum length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Maximum length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Custom error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl TextPatternRule {
    /// Create a new text pattern rule
    pub fn new() -> Self {
        Self {
            levels: None,
            must_match: None,
            must_not_match: None,
            min_length: None,
            max_length: None,
            message: None,
        }
    }

    /// Apply only to specific levels
    pub fn for_levels(mut self, levels: Vec<usize>) -> Self {
        self.levels = Some(levels);
        self
    }

    /// Text must match regex
    pub fn must_match(mut self, pattern: impl Into<String>) -> Self {
        self.must_match = Some(pattern.into());
        self
    }

    /// Text must not match regex
    pub fn must_not_match(mut self, pattern: impl Into<String>) -> Self {
        self.must_not_match = Some(pattern.into());
        self
    }

    /// Minimum text length
    pub fn min_length(mut self, len: usize) -> Self {
        self.min_length = Some(len);
        self
    }

    /// Maximum text length
    pub fn max_length(mut self, len: usize) -> Self {
        self.max_length = Some(len);
        self
    }

    /// Set custom message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Validate a heading against this rule
    pub fn validate(
        &self,
        heading: &Heading,
        path: &std::path::Path,
    ) -> MarkdownResult<Option<MarkdownValidationError>> {
        // Check if rule applies to this level
        if let Some(ref levels) = self.levels {
            if !levels.contains(&heading.level.as_usize()) {
                return Ok(None);
            }
        }

        // Check must_match pattern
        if let Some(ref pattern) = self.must_match {
            let regex = Regex::new(pattern).map_err(|e| {
                MarkdownError::validation(
                    path,
                    format!("Invalid regex pattern '{}': {}", pattern, e),
                )
            })?;
            if !regex.is_match(&heading.text) {
                return Ok(Some(
                    MarkdownValidationError::new(
                        "heading_pattern",
                        path,
                        self.message.clone().unwrap_or_else(|| {
                            format!(
                                "Heading '{}' does not match required pattern: {}",
                                heading.text, pattern
                            )
                        }),
                    )
                    .with_line(heading.line_number),
                ));
            }
        }

        // Check must_not_match pattern
        if let Some(ref pattern) = self.must_not_match {
            let regex = Regex::new(pattern).map_err(|e| {
                MarkdownError::validation(
                    path,
                    format!("Invalid regex pattern '{}': {}", pattern, e),
                )
            })?;
            if regex.is_match(&heading.text) {
                return Ok(Some(
                    MarkdownValidationError::new(
                        "heading_forbidden_pattern",
                        path,
                        self.message.clone().unwrap_or_else(|| {
                            format!(
                                "Heading '{}' matches forbidden pattern: {}",
                                heading.text, pattern
                            )
                        }),
                    )
                    .with_line(heading.line_number),
                ));
            }
        }

        // Check length constraints
        if let Some(min) = self.min_length {
            if heading.text.len() < min {
                return Ok(Some(
                    MarkdownValidationError::new(
                        "heading_too_short",
                        path,
                        self.message.clone().unwrap_or_else(|| {
                            format!(
                                "Heading '{}' is too short (minimum {} characters)",
                                heading.text, min
                            )
                        }),
                    )
                    .with_line(heading.line_number),
                ));
            }
        }

        if let Some(max) = self.max_length {
            if heading.text.len() > max {
                return Ok(Some(
                    MarkdownValidationError::new(
                        "heading_too_long",
                        path,
                        self.message.clone().unwrap_or_else(|| {
                            format!(
                                "Heading '{}' is too long (maximum {} characters)",
                                heading.text, max
                            )
                        }),
                    )
                    .with_line(heading.line_number),
                ));
            }
        }

        Ok(None)
    }
}

impl Default for TextPatternRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the structure of headings in a document
#[derive(Debug, Clone)]
pub struct HeadingStructure {
    /// All headings
    pub headings: Vec<Heading>,
    /// Heading hierarchy
    pub hierarchy: Vec<HeadingHierarchyNode>,
    /// Validation errors
    pub errors: Vec<HeadingValidationError>,
}

impl HeadingStructure {
    /// Analyze a document's heading structure
    pub fn analyze(document: &MarkdownDocument) -> Self {
        let headings = document.headings.clone();
        let hierarchy = build_hierarchy(&headings);
        let errors = validate_structure(&headings);

        Self {
            headings,
            hierarchy,
            errors,
        }
    }

    /// Check if the structure is valid
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// A node in the heading hierarchy
#[derive(Debug, Clone)]
pub struct HeadingHierarchyNode {
    pub heading: Heading,
    pub children: Vec<HeadingHierarchyNode>,
}

/// Heading validation error
#[derive(Debug, Clone)]
pub struct HeadingValidationError {
    pub error_type: HeadingErrorType,
    pub heading: Option<Heading>,
    pub message: String,
}

/// Types of heading errors
#[derive(Debug, Clone)]
pub enum HeadingErrorType {
    MissingH1,
    MultipleH1,
    SkippedLevel,
    TooDeep,
    EmptyHeading,
}

/// Build heading hierarchy from a list of headings
fn build_hierarchy(headings: &[Heading]) -> Vec<HeadingHierarchyNode> {
    let mut result = Vec::new();
    let mut stack: Vec<(usize, Vec<HeadingHierarchyNode>)> = Vec::new();

    for heading in headings {
        let level = heading.level.as_usize();
        let node = HeadingHierarchyNode {
            heading: heading.clone(),
            children: Vec::new(),
        };

        // Find parent level
        while let Some((parent_level, _)) = stack.last() {
            if level > *parent_level {
                break;
            }
            let (_, children) = stack.pop().unwrap();
            if let Some((_, parent_children)) = stack.last_mut() {
                if let Some(mut parent) = parent_children.pop() {
                    parent.children = children;
                    parent_children.push(parent);
                }
            } else {
                result.extend(children);
            }
        }

        stack.push((level, vec![node]));
    }

    // Flush remaining stack
    while let Some((_, children)) = stack.pop() {
        if let Some((_, parent_children)) = stack.last_mut() {
            if let Some(mut parent) = parent_children.pop() {
                parent.children = children;
                parent_children.push(parent);
            }
        } else {
            result.extend(children);
        }
    }

    result
}

/// Validate heading structure
fn validate_structure(headings: &[Heading]) -> Vec<HeadingValidationError> {
    let mut errors = Vec::new();

    // Check for H1
    let h1_count = headings
        .iter()
        .filter(|h| h.level == HeadingLevel::H1)
        .count();
    if h1_count == 0 {
        errors.push(HeadingValidationError {
            error_type: HeadingErrorType::MissingH1,
            heading: None,
            message: "Document has no H1 heading".to_string(),
        });
    } else if h1_count > 1 {
        errors.push(HeadingValidationError {
            error_type: HeadingErrorType::MultipleH1,
            heading: None,
            message: format!("Document has {} H1 headings", h1_count),
        });
    }

    // Check for skipped levels
    let mut last_level: Option<usize> = None;
    for heading in headings {
        let current_level = heading.level.as_usize();

        if let Some(last) = last_level {
            if current_level > last + 1 {
                errors.push(HeadingValidationError {
                    error_type: HeadingErrorType::SkippedLevel,
                    heading: Some(heading.clone()),
                    message: format!("Skipped from H{} to H{}", last, current_level),
                });
            }
        } else if current_level != 1 {
            errors.push(HeadingValidationError {
                error_type: HeadingErrorType::SkippedLevel,
                heading: Some(heading.clone()),
                message: format!("First heading should be H1, but found H{}", current_level),
            });
        }

        last_level = Some(current_level);
    }

    errors
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
mod tests {
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
}
