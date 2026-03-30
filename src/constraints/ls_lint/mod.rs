//! LS-Lint parity implementation
//!
//! This module provides features matching LS-Lint functionality:
//! - Directory validation
//! - Multi-part extensions (.d.ts, .test.js)
//! - Multiple rule syntax (kebab-case | snake_case)
//! - Path-specific rules

pub mod directory;
pub mod extension;
pub mod rules;

pub use directory::{DirectoryConstraint, DirectoryRule, DirectoryValidationConfig};
pub use extension::{ComplexExtension, ExtensionPattern, MultiPartExtensionRule};
pub use rules::{MultipleRuleSyntax, PathRule, PathRuleConfig, RuleAlternative};
