//! Heading hierarchy analysis helpers.

use crate::markdown::parser::{Heading, HeadingLevel, MarkdownDocument};

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
                    parent.children.extend(children);
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
                parent.children.extend(children);
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
