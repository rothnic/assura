//! Validation Module
//!
//! Provides constraint validation, rule resolution, context matching, and file pairing.

pub mod constraints;
pub mod context;
pub mod pairing;
pub mod resolver;

pub use constraints::{ConstraintValidator, ValidationResult};
pub use context::{ContextMatcher, ExecutionContext, ViolationLevel};
pub use pairing::{PairingRequirement, PairingValidator, PairingViolation};
pub use resolver::{ResolvedConstraints, RuleResolver};

/// Main validation orchestrator
pub struct ValidationEngine {
    config: crate::config::ast::LegacyNotationConfig,
    execution_context: ExecutionContext,
}

impl ValidationEngine {
    /// Create new validation engine
    pub fn new(
        config: crate::config::ast::LegacyNotationConfig,
        execution_context: ExecutionContext,
    ) -> Self {
        Self {
            config,
            execution_context,
        }
    }

    /// Validate a single file
    pub fn validate_file(
        &self,
        file_path: &std::path::Path,
        file_content: Option<&str>,
    ) -> Vec<ValidationResult> {
        // Step 1: Resolve rules for this file
        let resolved = RuleResolver::resolve(&self.config, file_path);

        // Step 2: Determine violation level from context
        let level = ContextMatcher::match_context(
            &self.config,
            &self.execution_context,
            &resolved.violation_entries,
        );

        // Step 3: Validate each constraint
        let mut results = Vec::new();

        for constraint in &resolved.constraints {
            let mut result = ConstraintValidator::validate(constraint, file_path, file_content);

            // If constraint failed, attach severity level and message
            if !result.passed {
                // Get message for this context level if available
                let level_str = level.to_string();
                let message_prefix = format!("[{}] ", level_str);

                let full_message = if let Some(ref msg) = result.message {
                    format!("{}{}", message_prefix, msg)
                } else {
                    message_prefix
                };

                result.message = Some(full_message);
            }

            results.push(result);
        }

        results
    }

    /// Check if validation should block based on results and allowed levels
    pub fn should_block(
        &self,
        results: &[ValidationResult],
        allowed_levels: &[ViolationLevel],
    ) -> bool {
        // Check if any result failed and the level is not in allowed levels
        // Results that failed would have been tagged with the appropriate level
        // based on context during validation
        results.iter().any(|r| {
            !r.passed
                && !allowed_levels.iter().any(|allowed| {
                    // Check if the result message contains the level indicator
                    // e.g., "[block]" or "[warn]"
                    if let Some(ref msg) = r.message {
                        let level_str = format!("[{}]", allowed.to_string().to_lowercase());
                        msg.to_lowercase().contains(&level_str)
                    } else {
                        false
                    }
                })
        })
    }

    /// Get the effective violation level for current context
    pub fn get_violation_level(&self, file_path: &std::path::Path) -> ViolationLevel {
        let resolved = RuleResolver::resolve(&self.config, file_path);
        ContextMatcher::match_context(
            &self.config,
            &self.execution_context,
            &resolved.violation_entries,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::parser::LegacyConfigParser;

    use std::path::Path;

    #[test]
    fn test_validation_fails_wrong_naming() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  tool:
    hook: tool

rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
"#;

        let config = LegacyConfigParser::parse(yaml).expect("Should parse");
        let exec = ExecutionContext::ci();
        let engine = ValidationEngine::new(config, exec);

        // Validate file with wrong naming (snake_case instead of PascalCase)
        let results = engine.validate_file(Path::new("src/components/my_component.tsx"), None);

        // Should have one failing result
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed, "Should fail for snake_case file");
        assert!(results[0].message.as_ref().unwrap().contains("[block]"));
    }

    #[test]
    fn test_validation_passes_correct_naming() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  tool:
    hook: tool

rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
"#;

        let config = LegacyConfigParser::parse(yaml).expect("Should parse");
        let exec = ExecutionContext::ci();
        let engine = ValidationEngine::new(config, exec);

        // Validate file with correct naming
        let results = engine.validate_file(Path::new("src/components/Button.tsx"), None);

        // Should have one passing result
        assert_eq!(results.len(), 1);
        assert!(results[0].passed, "Should pass for PascalCase file");
    }

    #[test]
    fn test_should_block_on_ci() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  tool:
    hook: tool

rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
"#;

        let config = LegacyConfigParser::parse(yaml).expect("Should parse");
        let exec = ExecutionContext::ci();
        let engine = ValidationEngine::new(config, exec);

        // Get failing results
        let results = engine.validate_file(Path::new("src/components/my_component.tsx"), None);

        // In CI context with ci:block, should block on failures
        let should_block =
            engine.should_block(&results, &[ViolationLevel::Info, ViolationLevel::Warn]);
        assert!(should_block, "Should block in CI context");
    }

    #[test]
    fn test_should_not_block_with_allowed_levels() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  tool:
    hook: tool

rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
"#;

        let config = LegacyConfigParser::parse(yaml).expect("Should parse");
        let exec = ExecutionContext::tool(); // Tool context, not CI
        let engine = ValidationEngine::new(config, exec);

        // Get failing results
        let results = engine.validate_file(Path::new("src/components/my_component.tsx"), None);

        // In tool context, might allow warnings
        let should_block =
            engine.should_block(&results, &[ViolationLevel::Warn, ViolationLevel::Block]);
        assert!(!should_block, "Should not block if warn is allowed");
    }

    #[test]
    fn test_violation_level_by_context() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  tool:
    hook: tool

rules:
  sized:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: sized
"#;

        let config = LegacyConfigParser::parse(yaml).expect("Should parse");

        // CI context should return Block level
        let ci_exec = ExecutionContext::ci();
        let ci_engine = ValidationEngine::new(config.clone(), ci_exec);
        let ci_level = ci_engine.get_violation_level(Path::new("src/components/Button.tsx"));
        assert_eq!(ci_level, ViolationLevel::Block);

        // Tool context should return Warn level
        let tool_exec = ExecutionContext::tool();
        let tool_engine = ValidationEngine::new(config, tool_exec);
        let tool_level = tool_engine.get_violation_level(Path::new("src/components/Button.tsx"));
        assert_eq!(tool_level, ViolationLevel::Warn);
    }

    #[test]
    fn test_line_constraint_validation() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  tool:
    hook: tool

rules:
  sized:
    ${name}.tsx:
      - constraints: [lines:..5]
      - violation: [block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: sized
"#;

        let config = LegacyConfigParser::parse(yaml).expect("Should parse");
        let exec = ExecutionContext::ci();
        let engine = ValidationEngine::new(config, exec);

        // File with 10 lines (exceeds 5 line limit)
        let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10";
        let results = engine.validate_file(Path::new("src/components/Button.tsx"), Some(content));

        // Should fail due to line count
        assert!(
            !results[0].passed,
            "Should fail for file exceeding line limit"
        );
    }
}
