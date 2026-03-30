//! Severity levels and maturity-based severity mapping
//!
//! This module provides:
//! - Severity enum (Critical, High, Medium, Low)
//! - Severity mapping based on project maturity
//! - Configuration for severity adjustments
//! - Override mechanisms

use serde::{Deserialize, Serialize};

use crate::maturity::MaturityLevel;
use crate::maturity::engine::Priority;

/// Severity levels for constraint violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity - style issues, minor suggestions
    Low = 0,
    /// Medium severity - warnings, potential issues
    Medium = 1,
    /// High severity - significant issues that should be addressed
    High = 2,
    /// Critical severity - blocking issues that must be fixed
    Critical = 3,
}

impl Severity {
    /// Get the numeric value of this severity
    pub fn value(self) -> u8 {
        self as u8
    }

    /// Get a human-readable name
    pub fn name(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    /// Get a human-readable description
    pub fn description(self) -> &'static str {
        match self {
            Severity::Low => "Minor issue, can be addressed at convenience",
            Severity::Medium => "Warning, should be addressed soon",
            Severity::High => "Significant issue, should be addressed promptly",
            Severity::Critical => "Critical issue, must be fixed immediately",
        }
    }

    /// Get the color code for terminal output
    pub fn color_code(self) -> &'static str {
        match self {
            Severity::Low => "\x1b[34m",    // Blue
            Severity::Medium => "\x1b[33m", // Yellow
            Severity::High => "\x1b[35m",   // Magenta
            Severity::Critical => "\x1b[31m", // Red
        }
    }

    /// Check if this severity is at least as high as another
    pub fn is_at_least(self, other: Severity) -> bool {
        self >= other
    }

    /// Increase severity by one level (capped at Critical)
    pub fn escalate(self) -> Self {
        match self {
            Severity::Low => Severity::Medium,
            Severity::Medium => Severity::High,
            Severity::High | Severity::Critical => Severity::Critical,
        }
    }

    /// Decrease severity by one level (capped at Low)
    pub fn de_escalate(self) -> Self {
        match self {
            Severity::Low | Severity::Medium => Severity::Low,
            Severity::High => Severity::Medium,
            Severity::Critical => Severity::High,
        }
    }

    /// Convert from Priority
    pub fn from_priority(priority: Priority) -> Self {
        match priority {
            Priority::Low => Severity::Low,
            Priority::Medium => Severity::Medium,
            Priority::High => Severity::High,
            Priority::Critical => Severity::Critical,
        }
    }

    /// Convert to Priority
    pub fn to_priority(self) -> Priority {
        match self {
            Severity::Low => Priority::Low,
            Severity::Medium => Priority::Medium,
            Severity::High => Priority::High,
            Severity::Critical => Priority::Critical,
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Medium
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Configuration for severity mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityConfig {
    /// Base severity levels for constraints
    pub base_levels: std::collections::HashMap<String, Severity>,
    /// Maturity-based adjustments
    pub maturity_mappings: MaturitySeverityMapping,
    /// Whether to allow manual overrides
    pub allow_overrides: bool,
    /// Minimum severity to report
    pub min_report_severity: Severity,
    /// Whether to fail on severity
    pub fail_on_severity: Option<Severity>,
}

impl Default for SeverityConfig {
    fn default() -> Self {
        Self {
            base_levels: std::collections::HashMap::new(),
            maturity_mappings: MaturitySeverityMapping::default(),
            allow_overrides: true,
            min_report_severity: Severity::Low,
            fail_on_severity: Some(Severity::Critical),
        }
    }
}

impl SeverityConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base severity for a constraint
    pub fn with_base_severity(
        mut self,
        constraint: impl Into<String>,
        severity: Severity,
    ) -> Self {
        self.base_levels.insert(constraint.into(), severity);
        self
    }

    /// Set maturity mappings
    pub fn with_maturity_mappings(mut self, mappings: MaturitySeverityMapping) -> Self {
        self.maturity_mappings = mappings;
        self
    }

    /// Disable overrides
    pub fn without_overrides(mut self) -> Self {
        self.allow_overrides = false;
        self
    }

    /// Set minimum report severity
    pub fn with_min_severity(mut self, severity: Severity) -> Self {
        self.min_report_severity = severity;
        self
    }

    /// Set fail-on severity
    pub fn fail_on(mut self, severity: Severity) -> Self {
        self.fail_on_severity = Some(severity);
        self
    }

    /// Get the effective severity for a constraint at a given maturity level
    pub fn get_effective_severity(
        &self,
        constraint_name: &str,
        maturity_level: MaturityLevel,
        default_severity: Severity,
    ) -> Severity {
        // Get base severity
        let base = self
            .base_levels
            .get(constraint_name)
            .copied()
            .unwrap_or(default_severity);

        // Apply maturity mapping
        self.maturity_mappings
            .adjust_severity(base, maturity_level)
    }

    /// Check if a severity should be reported
    pub fn should_report(&self, severity: Severity) -> bool {
        severity >= self.min_report_severity
    }

    /// Check if a severity should cause failure
    pub fn should_fail(&self, severity: Severity) -> bool {
        self.fail_on_severity
            .map(|min| severity >= min)
            .unwrap_or(false)
    }
}

/// Severity adjustments based on project maturity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaturitySeverityMapping {
    /// Raw level: how to adjust severities
    pub raw_adjustment: SeverityAdjustment,
    /// Developing level: how to adjust severities
    pub developing_adjustment: SeverityAdjustment,
    /// Mature level: how to adjust severities
    pub mature_adjustment: SeverityAdjustment,
    /// Established level: how to adjust severities
    pub established_adjustment: SeverityAdjustment,
}

impl Default for MaturitySeverityMapping {
    fn default() -> Self {
        Self {
            // Raw projects: escalate everything (strict mode)
            raw_adjustment: SeverityAdjustment::Escalate(1),
            // Developing: slight escalation
            developing_adjustment: SeverityAdjustment::Escalate(1),
            // Mature: use as-is
            mature_adjustment: SeverityAdjustment::AsIs,
            // Established: strict enforcement
            established_adjustment: SeverityAdjustment::Escalate(1),
        }
    }
}

impl MaturitySeverityMapping {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adjust severity based on maturity level
    pub fn adjust_severity(&self, severity: Severity, maturity: MaturityLevel) -> Severity {
        let adjustment = match maturity {
            MaturityLevel::Raw => &self.raw_adjustment,
            MaturityLevel::Developing => &self.developing_adjustment,
            MaturityLevel::Mature => &self.mature_adjustment,
            MaturityLevel::Established => &self.established_adjustment,
        };

        adjustment.apply(severity)
    }

    /// Set adjustment for a specific maturity level
    pub fn with_adjustment(
        mut self,
        level: MaturityLevel,
        adjustment: SeverityAdjustment,
    ) -> Self {
        match level {
            MaturityLevel::Raw => self.raw_adjustment = adjustment,
            MaturityLevel::Developing => self.developing_adjustment = adjustment,
            MaturityLevel::Mature => self.mature_adjustment = adjustment,
            MaturityLevel::Established => self.established_adjustment = adjustment,
        }
        self
    }
}

/// How to adjust severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SeverityAdjustment {
    /// Keep severity as-is
    AsIs,
    /// Escalate by N levels
    Escalate(u8),
    /// De-escalate by N levels
    DeEscalate(u8),
    /// Force a specific severity
    Force(Severity),
    /// Use minimum of current and N
    CapAt(Severity),
}

impl SeverityAdjustment {
    /// Apply this adjustment to a severity
    pub fn apply(self, severity: Severity) -> Severity {
        match self {
            SeverityAdjustment::AsIs => severity,
            SeverityAdjustment::Escalate(n) => {
                let mut result = severity;
                for _ in 0..n {
                    result = result.escalate();
                }
                result
            }
            SeverityAdjustment::DeEscalate(n) => {
                let mut result = severity;
                for _ in 0..n {
                    result = result.de_escalate();
                }
                result
            }
            SeverityAdjustment::Force(s) => s,
            SeverityAdjustment::CapAt(max) => severity.min(max),
        }
    }
}

/// Override for a specific constraint's severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityOverride {
    /// Constraint name
    pub constraint: String,
    /// Override severity
    pub severity: Severity,
    /// Reason for override
    pub reason: Option<String>,
    /// Expiration time (Unix timestamp)
    pub expires_at: Option<u64>,
}

impl SeverityOverride {
    pub fn new(constraint: impl Into<String>, severity: Severity) -> Self {
        Self {
            constraint: constraint.into(),
            severity,
            reason: None,
            expires_at: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn expires_at(mut self, timestamp: u64) -> Self {
        self.expires_at = Some(timestamp);
        self
    }

    /// Check if this override is still valid
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            None => true,
            Some(expires) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now < expires
            }
        }
    }
}

/// Manager for severity overrides
#[derive(Debug, Clone, Default)]
pub struct SeverityMapping {
    overrides: Vec<SeverityOverride>,
}

impl SeverityMapping {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a severity override
    pub fn add_override(&mut self, override_spec: SeverityOverride) {
        self.overrides.push(override_spec);
    }

    /// Remove overrides for a constraint
    pub fn remove_override(&mut self, constraint: &str) {
        self.overrides.retain(|o| o.constraint != constraint);
    }

    /// Get the effective severity for a constraint
    pub fn get_severity(
        &self,
        constraint: &str,
        base_severity: Severity,
        maturity: MaturityLevel,
    ) -> Severity {
        // Check for overrides first
        for override_spec in &self.overrides {
            if override_spec.constraint == constraint && override_spec.is_valid() {
                return override_spec.severity;
            }
        }

        // Apply maturity-based adjustment
        let mapping = MaturitySeverityMapping::default();
        mapping.adjust_severity(base_severity, maturity)
    }

    /// Clear all overrides
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }

    /// List all active overrides
    pub fn active_overrides(&self) -> Vec<&SeverityOverride> {
        self.overrides.iter().filter(|o| o.is_valid()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_severity_escalation() {
        assert_eq!(Severity::Low.escalate(), Severity::Medium);
        assert_eq!(Severity::Medium.escalate(), Severity::High);
        assert_eq!(Severity::High.escalate(), Severity::Critical);
        assert_eq!(Severity::Critical.escalate(), Severity::Critical);
    }

    #[test]
    fn test_severity_de_escalation() {
        assert_eq!(Severity::Critical.de_escalate(), Severity::High);
        assert_eq!(Severity::High.de_escalate(), Severity::Medium);
        assert_eq!(Severity::Medium.de_escalate(), Severity::Low);
        assert_eq!(Severity::Low.de_escalate(), Severity::Low);
    }

    #[test]
    fn test_severity_adjustment() {
        let adj = SeverityAdjustment::Escalate(2);
        assert_eq!(adj.apply(Severity::Low), Severity::High);

        let adj = SeverityAdjustment::DeEscalate(1);
        assert_eq!(adj.apply(Severity::High), Severity::Medium);

        let adj = SeverityAdjustment::Force(Severity::Critical);
        assert_eq!(adj.apply(Severity::Low), Severity::Critical);

        let adj = SeverityAdjustment::CapAt(Severity::Medium);
        assert_eq!(adj.apply(Severity::Critical), Severity::Medium);
        assert_eq!(adj.apply(Severity::Low), Severity::Low);
    }

    #[test]
    fn test_maturity_severity_mapping() {
        let mapping = MaturitySeverityMapping::default();

        // Raw should escalate
        assert_eq!(mapping.adjust_severity(Severity::Low, MaturityLevel::Raw), Severity::Medium);

        // Mature should be as-is
        assert_eq!(mapping.adjust_severity(Severity::High, MaturityLevel::Mature), Severity::High);
    }

    #[test]
    fn test_severity_config() {
        let config = SeverityConfig::new()
            .with_base_severity("file_size", Severity::High)
            .with_min_severity(Severity::Medium)
            .fail_on(Severity::Critical);

        assert_eq!(
            config.get_effective_severity("file_size", MaturityLevel::Mature, Severity::Medium),
            Severity::High
        );

        assert!(config.should_report(Severity::Medium));
        assert!(!config.should_report(Severity::Low));

        assert!(config.should_fail(Severity::Critical));
        assert!(!config.should_fail(Severity::High));
    }

    #[test]
    fn test_severity_override() {
        let mut mapping = SeverityMapping::new();

        let override_spec = SeverityOverride::new("test_constraint", Severity::Low)
            .with_reason("Temporary exception");

        mapping.add_override(override_spec);

        let effective = mapping.get_severity("test_constraint", Severity::Critical, MaturityLevel::Mature);
        assert_eq!(effective, Severity::Low);

        // Non-overridden constraint
        let effective = mapping.get_severity("other_constraint", Severity::High, MaturityLevel::Mature);
        assert_eq!(effective, Severity::High);
    }

    #[test]
    fn test_severity_override_expiration() {
        let override_spec = SeverityOverride::new("test", Severity::Low)
            .expires_at(0); // Already expired

        assert!(!override_spec.is_valid());
    }

    #[test]
    fn test_severity_from_priority() {
        assert_eq!(Severity::from_priority(Priority::Critical), Severity::Critical);
        assert_eq!(Severity::from_priority(Priority::High), Severity::High);
        assert_eq!(Severity::from_priority(Priority::Medium), Severity::Medium);
        assert_eq!(Severity::from_priority(Priority::Low), Severity::Low);
    }
}
