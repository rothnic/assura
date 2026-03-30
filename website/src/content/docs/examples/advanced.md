---
title: Advanced Patterns
description: Advanced usage patterns and techniques
---

Explore advanced features and integration patterns for Assura.

## Custom Validators

Create custom validation logic:

```rust
use assura::{ValidationRule, ValidationResult, Severity};
use std::path::Path;

pub struct MyCustomRule;

impl ValidationRule for MyCustomRule {
    fn name(&self) -> &str {
        "my-custom-rule"
    }
    
    fn validate(&self, path: &Path, content: &str) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // Custom validation logic
        if content.contains("TODO") {
            results.push(ValidationResult {
                rule: self.name().to_string(),
                severity: Severity::Low,
                file: path.to_path_buf(),
                line: Some(42),
                column: None,
                message: "TODO found in code".to_string(),
                suggestion: Some("Consider creating an issue instead".to_string()),
            });
        }
        
        results
    }
}
```

## Programmatic API

Build custom validation workflows:

```rust
use assura::{Config, Validator, Severity};
use std::process;

#[tokio::main]
async fn main() {
    let config = Config::load("assura.yaml")
        .expect("Failed to load config");
    
    let validator = Validator::new(config);
    let results = validator.validate_all().await
        .expect("Validation failed");
    
    // Custom result processing
    let critical_count = results.iter()
        .filter(|r| matches!(r.severity, Severity::Critical))
        .count();
    
    if critical_count > 0 {
        eprintln!("{} critical issues found!", critical_count);
        process::exit(1);
    }
    
    // Filter and display
    let warnings: Vec<_> = results.iter()
        .filter(|r| matches!(r.severity, Severity::Medium | Severity::Low))
        .collect();
    
    if !warnings.is_empty() {
        println!("Warnings ({}):", warnings.len());
        for warning in warnings {
            println!("  - {}: {}", warning.file.display(), warning.message);
        }
    }
}
```

## Multi-Project Workspace

Configure Assura for a workspace with multiple crates:

```yaml
# assura.yaml
name: My Workspace

includes:
  - "crates/*/src/**/*.rs"
  - "crates/*/Cargo.toml"

excludes:
  - "**/target/**/*"
  - "crates/*/examples/**/*"

rules:
  - name: dependency-check
    severity: critical
    check_circular: true
    workspace_mode: true  # Check cross-crate dependencies
```

## Custom Reports

Generate custom reports from validation results:

```rust
use assura::{Validator, ValidationResult};
use std::collections::HashMap;

fn generate_markdown_report(results: &[ValidationResult]) -> String {
    let mut by_severity: HashMap<String, Vec<&ValidationResult>> = HashMap::new();
    
    for result in results {
        by_severity
            .entry(format!("{:?}", result.severity))
            .or_default()
            .push(result);
    }
    
    let mut report = String::from("# Validation Report\n\n");
    
    for (severity, items) in by_severity {
        report.push_str(&format!("## {} ({} issues)\n\n", severity, items.len()));
        for item in items {
            report.push_str(&format!(
                "- **{}**: {}\n",
                item.file.display(),
                item.message
            ));
        }
        report.push('\n');
    }
    
    report
}
```

## Performance Tuning

Optimize validation performance for large codebases:

```yaml
settings:
  parallel: true
  max_workers: 16
  cache_enabled: true
  cache_dir: ".assura-cache"

# Exclude large generated files
excludes:
  - "**/generated/**/*"
  - "**/vendor/**/*"
  - "**/*.pb.go"  # Protocol buffer generated files
  - "**/*.gen.ts" # Generated TypeScript
```

## Watch Mode with Custom Events

React to specific file changes:

```rust
use assura::{Config, Validator};
use notify::{Event, EventKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("assura.yaml")?;
    let validator = Validator::new(config);
    
    // Watch with custom event handling
    validator.watch_with_callback(|event: Event| {
        match event.kind {
            EventKind::Modify(_) => {
                println!("File modified: {:?}", event.paths);
            }
            EventKind::Create(_) => {
                println!("File created: {:?}", event.paths);
            }
            _ => {}
        }
    }).await?;
    
    Ok(())
}
```

## Integration with Testing Frameworks

Use Assura in your test suite:

```rust
#[cfg(test)]
mod validation_tests {
    use assura::{Config, Validator};
    
    #[tokio::test]
    async fn test_no_critical_issues() {
        let config = Config::load("assura.yaml").unwrap();
        let validator = Validator::new(config);
        
        let results = validator.validate_all().await.unwrap();
        
        let critical_count = results.iter()
            .filter(|r| matches!(r.severity, Severity::Critical))
            .count();
        
        assert_eq!(critical_count, 0, "Critical validation issues found");
    }
}
```
