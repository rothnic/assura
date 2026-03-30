---
name: constraint-development
description: "Develop custom Assura constraints. Use when you need to create new validation rules, implement custom file checks, or extend Assura's validation engine with project-specific requirements."
triggers: ["custom constraint", "new rule", "validator", "assura extension"]
---

# Constraint Development

Guide for creating custom Assura constraints.

## Overview

Assura constraints are Rust structs that implement the `Constraint` trait. They validate files and directories based on custom logic.

## Quick Start

1. Create a new constraint struct
2. Implement the `Constraint` trait
3. Register it in the constraint engine

## Basic Constraint Template

```rust
use std::path::Path;
use assura::constraints::{
    Constraint, ConstraintContext, ConstraintOutput, ConstraintResult,
    ValidationFailure, Severity,
};

/// Custom constraint example
#[derive(Debug)]
pub struct MyConstraint {
    name: String,
    severity: Severity,
}

impl MyConstraint {
    pub fn new() -> Self {
        Self {
            name: "my_constraint".to_string(),
            severity: Severity::Medium,
        }
    }
    
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}

impl Constraint for MyConstraint {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Description of what this constraint validates"
    }
    
    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let mut failures = Vec::new();
        
        // Your validation logic here
        if let Some(content) = std::fs::read_to_string(path).ok() {
            if content.contains("TODO") {
                failures.push(ValidationFailure::new(
                    &self.name,
                    path,
                    "File contains TODO comments",
                ));
            }
        }
        
        let passed = failures.is_empty();
        Ok(ConstraintOutput::new(&self.name, path, passed)
            .with_severity(self.severity)
            .with_failures(failures.into()))
    }
    
    fn applies_to(&self, path: &Path) -> bool {
        // Only apply to .rs files
        path.extension()
            .map(|ext| ext == "rs")
            .unwrap_or(false)
    }
    
    fn default_severity(&self) -> Severity {
        self.severity
    }
}
```

## Constraint Configuration

### With Custom Config

```rust
#[derive(Debug, Clone)]
pub struct MyConfig {
    max_line_length: usize,
    forbidden_words: Vec<String>,
}

impl Default for MyConfig {
    fn default() -> Self {
        Self {
            max_line_length: 120,
            forbidden_words: vec!["TODO".to_string(), "FIXME".to_string()],
        }
    }
}

pub struct MyConstraint {
    name: String,
    config: MyConfig,
}

impl MyConstraint {
    pub fn with_config(mut self, config: MyConfig) -> Self {
        self.config = config;
        self
    }
}
```

### YAML-Configurable

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyConstraintConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_max_length")]
    pub max_length: usize,
}

fn default_max_length() -> usize {
    120
}

impl Default for MyConstraintConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_length: default_max_length(),
        }
    }
}
```

## Validation Patterns

### File Content Validation

```rust
fn validate_content(&self, path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    
    for (line_num, line) in content.lines().enumerate() {
        if line.len() > self.config.max_line_length {
            return Some(format!(
                "Line {} exceeds maximum length of {} characters",
                line_num + 1,
                self.config.max_line_length
            ));
        }
    }
    
    None
}
```

### Directory Validation

```rust
fn validate_directory(&self, path: &Path) -> Vec<ValidationFailure> {
    let mut failures = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            
            // Check file count
            if self.file_count > self.config.max_files {
                failures.push(ValidationFailure::new(
                    &self.name,
                    &entry_path,
                    format!("Too many files in directory: {}", self.file_count),
                ));
            }
        }
    }
    
    failures
}
```

### Multi-File Validation

```rust
use std::collections::HashMap;

pub struct CrossFileConstraint {
    seen_files: HashMap<String, PathBuf>,
}

impl CrossFileConstraint {
    pub fn validate_unique_ids(&mut self, path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        
        // Extract IDs from file
        for id in extract_ids(&content) {
            if let Some(existing) = self.seen_files.get(&id) {
                return Some(format!(
                    "Duplicate ID '{}' found in {:?}",
                    id, existing
                ));
            }
            self.seen_files.insert(id, path.to_path_buf());
        }
        
        None
    }
}
```

## Integration with Config Engine

### Register Custom Constraint

```rust
use assura::constraints::ConstraintEngine;

fn register_constraints(engine: &mut ConstraintEngine) {
    engine.register("my_constraint", |config| {
        let my_config: MyConstraintConfig = serde_yaml::from_value(config)?;
        Ok(Box::new(MyConstraint::new().with_config(my_config)))
    });
}
```

### Config File Usage

```yaml
# .assura/config.yml
version: "1.0"

constraints:
  - id: "my_constraint"
    enabled: true
    config:
      max_length: 100
      forbidden_words:
        - "TODO"
        - "FIXME"
```

## Testing Constraints

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_valid_file() {
        let constraint = MyConstraint::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();
        
        let context = ConstraintContext::new();
        let result = constraint.validate(&file_path, &context).unwrap();
        
        assert!(result.passed);
    }
    
    #[test]
    fn test_invalid_file() {
        let constraint = MyConstraint::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "// TODO: fix this").unwrap();
        
        let context = ConstraintContext::new();
        let result = constraint.validate(&file_path, &context).unwrap();
        
        assert!(!result.passed);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_with_config() {
    let config = MyConstraintConfig {
        enabled: true,
        max_length: 80,
    };
    
    let constraint = MyConstraint::new().with_config(config);
    // Test with the configured constraint
}
```

## Best Practices

1. **Fail Fast**: Return as soon as you find a violation
2. **Clear Error Messages**: Include file paths, line numbers, and suggestions
3. **Performance**: Cache expensive operations, use parallel validation when possible
4. **Documentation**: Document all configuration options
5. **Testing**: Test edge cases and error conditions

## Common Pitfalls

### Don't Block on I/O

```rust
// Bad - blocks thread
let content = std::fs::read_to_string(path)?;

// Better - handle errors gracefully
let content = match std::fs::read_to_string(path) {
    Ok(c) => c,
    Err(e) => {
        failures.push(ValidationFailure::new(
            &self.name,
            path,
            format!("Failed to read file: {}", e),
        ));
        return Ok(output);
    }
};
```

### Check Applies To

```rust
// Always check if the constraint should apply
fn applies_to(&self, path: &Path) -> bool {
    // Don't try to validate binary files
    if is_binary_file(path) {
        return false;
    }
    
    path.extension()
        .map(|ext| ext == "rs")
        .unwrap_or(false)
}
```

### Use Appropriate Severity

```rust
// Critical: Blocks commit/release
// High: Should fix before merge
// Medium: Code review comment
// Low: Style preference
fn default_severity(&self) -> Severity {
    Severity::Medium
}
```

## Example: Complete Custom Constraint

```rust
//! Line length constraint

use std::path::Path;
use assura::constraints::*;

#[derive(Debug, Clone)]
pub struct LineLengthConfig {
    max_length: usize,
    exclude_patterns: Vec<String>,
}

impl Default for LineLengthConfig {
    fn default() -> Self {
        Self {
            max_length: 120,
            exclude_patterns: vec![
                "http".to_string(),
                "https".to_string(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct LineLengthConstraint {
    name: String,
    config: LineLengthConfig,
}

impl LineLengthConstraint {
    pub fn new() -> Self {
        Self {
            name: "line_length".to_string(),
            config: LineLengthConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: LineLengthConfig) -> Self {
        self.config = config;
        self
    }
}

impl Constraint for LineLengthConstraint {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Validates that lines don't exceed maximum length"
    }
    
    fn validate(
        &self,
        path: &Path,
        _context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let mut failures = Vec::new();
        
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(ConstraintOutput::new(&self.name, path, false)
                    .with_failures(ValidationFailures::from(vec![
                        ValidationFailure::new(
                            &self.name,
                            path,
                            format!("Failed to read file: {}", e),
                        )
                    ])));
            }
        };
        
        for (line_num, line) in content.lines().enumerate() {
            // Skip lines matching exclude patterns
            if self.config.exclude_patterns.iter().any(|p| line.contains(p)) {
                continue;
            }
            
            if line.len() > self.config.max_length {
                failures.push(ValidationFailure::new(
                    &self.name,
                    path,
                    format!(
                        "Line {} exceeds {} characters (found {})",
                        line_num + 1,
                        self.config.max_length,
                        line.len()
                    ),
                ));
            }
        }
        
        let passed = failures.is_empty();
        Ok(ConstraintOutput::new(&self.name, path, passed)
            .with_failures(failures.into()))
    }
    
    fn applies_to(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| matches!(ext.to_str(), Some("rs") | Some("md") | Some("js")))
            .unwrap_or(false)
    }
    
    fn default_severity(&self) -> Severity {
        Severity::Medium
    }
}
```

## Resources

- Constraint trait documentation: https://docs.rs/assura/latest/assura/constraints/trait.Constraint.html
- Example constraints: https://github.com/assura/assura/tree/main/src/constraints
- Test examples: https://github.com/assura/assura/tree/main/tests
