//! Heading text pattern matching and validation rules.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::markdown::error::{MarkdownError, MarkdownResult, MarkdownValidationError};
use crate::markdown::parser::Heading;

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
