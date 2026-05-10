//! Custom violation message configuration.

use serde::{Deserialize, Serialize};

/// Custom message configuration for violations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    /// Violation description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violation: Option<String>,

    /// Explanation of why this rule exists
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,

    /// How to fix the violation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,

    /// How to override this rule
    #[serde(rename = "override", skip_serializing_if = "Option::is_none")]
    pub override_: Option<String>,

    /// Link to documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

impl Message {
    /// Create a new empty message
    pub fn new() -> Self {
        Self::default()
    }

    /// Set violation message
    pub fn with_violation(mut self, msg: impl Into<String>) -> Self {
        self.violation = Some(msg.into());
        self
    }

    /// Set why message
    pub fn with_why(mut self, msg: impl Into<String>) -> Self {
        self.why = Some(msg.into());
        self
    }

    /// Set fix message
    pub fn with_fix(mut self, msg: impl Into<String>) -> Self {
        self.fix = Some(msg.into());
        self
    }

    /// Set override message
    pub fn with_override(mut self, msg: impl Into<String>) -> Self {
        self.override_ = Some(msg.into());
        self
    }

    /// Set docs link
    pub fn with_docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
    }
}
