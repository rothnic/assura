//! Multiple rule syntax and path-specific rules for LS-Lint parity
//!
//! Provides OR syntax support (kebab-case | snake_case) and path-specific
//! naming rules with glob pattern matching.

use regex::Regex;
use std::path::Path;

use crate::constraints::error::ConstraintResult;
use crate::constraints::naming::CaseConvention;
use crate::constraints::severity::Severity;

/// Represents an alternative in a multiple rule (OR syntax)
#[derive(Debug, Clone)]
pub struct RuleAlternative {
    /// The case convention
    pub convention: CaseConvention,
    /// Optional regex pattern for additional matching
    pub pattern: Option<Regex>,
    /// Description of this alternative
    pub description: String,
}

impl RuleAlternative {
    pub fn new(convention: CaseConvention) -> Self {
        Self {
            convention,
            pattern: None,
            description: convention.description().to_string(),
        }
    }

    pub fn with_pattern(mut self, pattern: &str) -> ConstraintResult<Self> {
        let regex = Regex::new(pattern).map_err(|e| {
            crate::constraints::error::ConstraintError::pattern(
                "rule_alternative",
                format!("Invalid regex: {}", e),
            )
        })?;
        self.pattern = Some(regex);
        Ok(self)
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Validate a name against this alternative
    pub fn validate(&self, name: &str) -> bool {
        // First check case convention
        if !self.convention.validate(name) {
            return false;
        }

        // Then check pattern if present
        if let Some(ref pattern) = self.pattern {
            if !pattern.is_match(name) {
                return false;
            }
        }

        true
    }
}

/// Multiple rule syntax supporting OR operations
/// e.g., "kebab-case | snake_case | PascalCase"
#[derive(Debug, Clone)]
pub struct MultipleRuleSyntax {
    /// The alternatives to try
    pub alternatives: Vec<RuleAlternative>,
    /// Severity for violations
    pub severity: Severity,
    /// Whether all alternatives must fail for the rule to fail
    pub require_all_fail: bool,
}

impl MultipleRuleSyntax {
    pub fn new() -> Self {
        Self {
            alternatives: Vec::new(),
            severity: Severity::Medium,
            require_all_fail: false,
        }
    }

    /// Add an alternative from a case convention name
    pub fn add_convention(mut self, convention: CaseConvention) -> Self {
        self.alternatives.push(RuleAlternative::new(convention));
        self
    }

    /// Parse a rule string like "kebab-case | snake_case | camelCase"
    pub fn parse(rule_str: &str) -> ConstraintResult<Self> {
        let mut syntax = Self::new();

        for part in rule_str.split('|').map(|s| s.trim()) {
            let convention = parse_case_convention(part)?;
            syntax.alternatives.push(RuleAlternative::new(convention));
        }

        Ok(syntax)
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Validate a name against any of the alternatives
    pub fn validate(&self, name: &str) -> (bool, Vec<String>) {
        let mut failures = Vec::new();

        for alt in &self.alternatives {
            if alt.validate(name) {
                return (true, vec![]);
            } else {
                failures.push(format!(
                    "Not {} ({})",
                    alt.convention.name(),
                    alt.description
                ));
            }
        }

        (false, failures)
    }

    /// Get a description of all alternatives
    pub fn description(&self) -> String {
        let names: Vec<_> = self
            .alternatives
            .iter()
            .map(|a| a.convention.name())
            .collect();
        names.join(" | ")
    }

    /// Get the severity for this rule
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

impl Default for MultipleRuleSyntax {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a case convention from its string name
fn parse_case_convention(name: &str) -> ConstraintResult<CaseConvention> {
    match name {
        "lowercase" => Ok(CaseConvention::LowerCase),
        "UPPERCASE" => Ok(CaseConvention::UpperCase),
        "snake_case" | "snakecase" => Ok(CaseConvention::SnakeCase),
        "camelCase" | "camelcase" => Ok(CaseConvention::CamelCase),
        "PascalCase" | "pascalcase" => Ok(CaseConvention::PascalCase),
        "kebab-case" | "kebabcase" => Ok(CaseConvention::KebabCase),
        "SCREAMING_SNAKE_CASE" | "screaming_snake_case" => Ok(CaseConvention::ScreamingSnakeCase),
        "dot.case" | "dotcase" => Ok(CaseConvention::DotCase),
        "flatcase" => Ok(CaseConvention::FlatCase),
        "FLATCASE" => Ok(CaseConvention::ScreamingFlatCase),
        "COBOL-CASE" | "cobol-case" => Ok(CaseConvention::CobolCase),
        "Train-Case" | "train-case" => Ok(CaseConvention::TrainCase),
        _ => Err(crate::constraints::error::ConstraintError::pattern(
            "case_convention",
            format!("Unknown case convention: {}", name),
        )),
    }
}

/// A path-specific rule with glob pattern matching
#[derive(Debug, Clone)]
pub struct PathRule {
    /// Glob pattern for matching paths
    pub pattern: String,
    /// Compiled regex from glob pattern
    regex: Regex,
    /// The naming convention to enforce
    pub convention: CaseConvention,
    /// Severity for this path rule
    pub severity: Severity,
    /// Whether this rule overrides global rules
    pub is_override: bool,
    /// Child rules for nested paths
    pub child_rules: Vec<PathRule>,
}

impl PathRule {
    pub fn new(pattern: impl Into<String>, convention: CaseConvention) -> ConstraintResult<Self> {
        let pattern = pattern.into();
        let regex = glob_to_regex(&pattern)?;

        Ok(Self {
            pattern,
            regex,
            convention,
            severity: Severity::Medium,
            is_override: false,
            child_rules: Vec::new(),
        })
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn as_override(mut self) -> Self {
        self.is_override = true;
        self
    }

    pub fn with_child_rule(mut self, rule: PathRule) -> Self {
        self.child_rules.push(rule);
        self
    }

    /// Check if this rule matches a path
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // First try matching the full path
        if self.regex.is_match(&path_str) {
            return true;
        }

        // If that fails, try matching against path suffixes
        // This handles absolute paths like /tmp/xyz/src/main.rs
        // by matching src/main.rs against the pattern
        let path_bytes = path_str.as_bytes();
        for (i, &byte) in path_bytes.iter().enumerate() {
            if byte == b'/' {
                let suffix = &path_str[i + 1..];
                if self.regex.is_match(suffix) {
                    return true;
                }
            }
        }

        false
    }

    /// Validate a filename against this rule
    pub fn validate(&self, filename: &str) -> Option<String> {
        // Strip extension for validation
        let stem = filename.rsplit_once('.').map(|x| x.0).unwrap_or(filename);

        if !self.convention.validate(stem) {
            Some(format!(
                "Filename '{}' does not follow {} convention for path pattern '{}'",
                filename,
                self.convention.name(),
                self.pattern
            ))
        } else {
            None
        }
    }

    /// Find the most specific matching rule for a path
    pub fn find_matching_rule(&self, path: &Path) -> Option<&PathRule> {
        if !self.matches(path) {
            return None;
        }

        // Check child rules for more specific match
        for child in &self.child_rules {
            if let Some(rule) = child.find_matching_rule(path) {
                return Some(rule);
            }
        }

        Some(self)
    }
}

/// Configuration for path-specific rules
#[derive(Debug, Clone)]
pub struct PathRuleConfig {
    /// Rules organized by path pattern
    pub rules: Vec<PathRule>,
    /// Default convention for paths not matching any rule
    pub default_convention: Option<CaseConvention>,
    /// Whether child rules inherit from parent rules
    pub inherit_rules: bool,
}

impl PathRuleConfig {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_convention: None,
            inherit_rules: true,
        }
    }

    pub fn with_rule(mut self, rule: PathRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_default_convention(mut self, convention: CaseConvention) -> Self {
        self.default_convention = Some(convention);
        self
    }

    pub fn without_inheritance(mut self) -> Self {
        self.inherit_rules = false;
        self
    }

    /// Find the best matching rule for a path
    pub fn find_rule(&self, path: &Path) -> Option<&PathRule> {
        let mut best_match: Option<&PathRule> = None;
        let mut best_specificity = 0;

        for rule in &self.rules {
            if let Some(matching) = rule.find_matching_rule(path) {
                // Calculate specificity: count the number of literal (non-wildcard) path components
                // More literal components = more specific
                let specificity = matching
                    .pattern
                    .split('/')
                    .filter(|component| !component.contains('*'))
                    .count();

                if specificity > best_specificity {
                    best_specificity = specificity;
                    best_match = Some(matching);
                }
            }
        }

        best_match
    }

    /// Validate a path against the configuration
    pub fn validate(&self, path: &Path) -> Option<String> {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if filename.is_empty() {
            return Some("Path has no filename".to_string());
        }

        // Find matching rule
        if let Some(rule) = self.find_rule(path) {
            return rule.validate(filename);
        }

        // Use default convention if no rule matches
        if let Some(convention) = self.default_convention {
            let stem = filename.rsplit_once('.').map(|x| x.0).unwrap_or(filename);

            if !convention.validate(stem) {
                return Some(format!(
                    "Filename '{}' does not follow default {} convention",
                    filename,
                    convention.name()
                ));
            }
        }

        None
    }
}

impl Default for PathRuleConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a glob pattern to a regex
fn glob_to_regex(pattern: &str) -> ConstraintResult<Regex> {
    let mut regex_str = String::with_capacity(pattern.len() * 2);
    regex_str.push('^');

    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                // Check if this is a double asterisk (**)
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume the second *

                    // Check if ** is followed by /
                    if chars.peek() == Some(&'/') {
                        // **/ matches zero or more directory levels followed by /
                        // This becomes: (?:[^/]+/)*) - zero or more of (non-slash chars + /)
                        chars.next(); // consume the /
                        regex_str.push_str("(?:[^/]+/)*");
                    } else {
                        // ** at end or not followed by / - match any remaining path including files
                        // This becomes: (?:[^/]*/)*[^/]* to match directories and files
                        regex_str.push_str("(?:[^/]*/)*[^/]*");
                    }
                } else {
                    // Single * matches anything within a single directory level (no slashes)
                    regex_str.push_str("[^/]*");
                }
            }
            '?' => regex_str.push_str("[^/]"),
            '.' => regex_str.push_str("\\."),
            '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                regex_str.push('\\');
                regex_str.push(c);
            }
            '/' => regex_str.push('/'),
            _ => regex_str.push(c),
        }
    }

    regex_str.push('$');

    Regex::new(&regex_str).map_err(|e| {
        crate::constraints::error::ConstraintError::pattern(
            "glob_pattern",
            format!("Invalid glob pattern: {}", e),
        )
    })
}

#[cfg(test)]
mod rules_tests;
