---
title: Custom Constraints
description: How to create and use custom validation constraints
template: doc
sidebar:
  order: 2
---

import { Steps, Tabs, TabItem, Aside, Code } from '@astrojs/starlight/components';

This example shows how to create custom validation constraints to enforce project-specific rules.

## Overview

Assura's constraint system allows you to create custom validation logic that integrates seamlessly with the existing rule engine. You can implement constraints in Rust or use the TypeScript plugin API.

## Creating a Custom Constraint in Rust

<Steps>

1. **Add Assura as a dependency**

   In your `Cargo.toml`:

   ```toml
   [dependencies]
   assura = "0.1"
   async-trait = "0.1"
   serde_json = "1.0"
   ```

2. **Implement the Constraint trait**

   ```rust
   use assura::constraint::{Constraint, ConstraintResult, Context, Violation};
   use async_trait::async_trait;
   use serde_json::Value;

   /// Constraint that detects TODO comments
   pub struct TodoConstraint {
       severity: Severity,
       allowed_patterns: Vec<String>,
       require_issue_reference: bool,
   }

   impl TodoConstraint {
       pub fn new() -> Self {
           Self {
               severity: Severity::Low,
               allowed_patterns: vec![],
               require_issue_reference: false,
           }
       }
   }

   #[async_trait]
   impl Constraint for TodoConstraint {
       fn name(&self) -> &str {
           "todo-detection"
       }

       fn description(&self) -> &str {
           "Detects TODO comments in code"
       }

       fn severity(&self) -> Severity {
           self.severity
       }

       fn configure(&mut self, config: &Value) -> Result<(), ConfigError> {
           if let Some(sev) = config.get("severity") {
               let sev_str = sev.as_str()
                   .ok_or(ConfigError::Validation("severity must be a string".into()))?;
               self.severity = Severity::from_str(sev_str)
                   .ok_or(ConfigError::InvalidSeverity)?;
           }

           if let Some(patterns) = config.get("allowed_patterns") {
               self.allowed_patterns = patterns.as_array()
                   .map(|arr| {
                       arr.iter()
                           .filter_map(|v| v.as_str().map(String::from))
                           .collect()
                   })
                   .unwrap_or_default();
           }

           if let Some(require) = config.get("require_issue_reference") {
               self.require_issue_reference = require.as_bool()
                   .ok_or(ConfigError::Validation("require_issue_reference must be boolean".into()))?;
           }

           Ok(())
       }

       async fn check(&self, context: &Context) -> Result<ConstraintResult, ConstraintError> {
           let mut violations = Vec::new();

           for (line_num, line) in context.file_content.lines().enumerate() {
               // Skip comments and strings
               let code_line = remove_comments(line);
               
               if code_line.to_uppercase().contains("TODO") {
                   // Check if it matches an allowed pattern
                   let is_allowed = self.allowed_patterns.iter()
                       .any(|pattern| code_line.contains(pattern));

                   if !is_allowed {
                       let has_issue_ref = code_line.contains('#');
                       
                       if self.require_issue_reference && !has_issue_ref {
                           violations.push(Violation {
                               file: context.file_path.clone(),
                               line: Some(line_num + 1),
                               column: find_todo_column(&code_line),
                               message: "TODO without issue reference".to_string(),
                               suggestion: Some(
                                   "Add an issue reference: TODO(#123)".to_string()
                               ),
                           });
                       } else {
                           violations.push(Violation {
                               file: context.file_path.clone(),
                               line: Some(line_num + 1),
                               column: find_todo_column(&code_line),
                               message: "TODO comment found".to_string(),
                               suggestion: Some(
                                   "Create an issue or complete the task".to_string()
                               ),
                           });
                       }
                   }
               }
           }

           if violations.is_empty() {
               Ok(ConstraintResult::success())
           } else {
               Ok(ConstraintResult::violations(violations))
           }
       }
   }

   fn remove_comments(line: &str) -> String {
       // Simple comment removal for demonstration
       line.split("//").next().unwrap_or(line).to_string()
   }

   fn find_todo_column(line: &str) -> Option<usize> {
       line.to_uppercase().find("TODO").map(|i| i + 1)
   }
   ```

3. **Register the constraint**

   ```rust
   use assura::constraint::ConstraintRegistry;

   fn register_custom_constraints(registry: &mut ConstraintRegistry) {
       registry.register("todo-detection", || {
           Box::new(TodoConstraint::new())
       });
   }
   ```

4. **Use in configuration**

   ```yaml
   rules:
     - name: todo-detection
       severity: medium
       allowed_patterns:
         - "TODO(#"
       require_issue_reference: true
   ```

</Steps>

## Creating a Constraint in TypeScript

For projects that prefer TypeScript, you can use the plugin API:

```typescript
import { AssuraPlugin, ValidationResult, FileChangeEvent } from '@assura/plugin-sdk';

interface TodoConfig {
  severity: 'critical' | 'high' | 'medium' | 'low';
  allowedPatterns: string[];
  requireIssueReference: boolean;
}

class TodoConstraintPlugin implements AssuraPlugin {
  name = 'todo-constraint';
  version = '1.0.0';
  
  private config: TodoConfig = {
    severity: 'low',
    allowedPatterns: [],
    requireIssueReference: false
  };

  async initialize(pluginConfig: Record<string, unknown>): Promise<void> {
    this.config = {
      severity: (pluginConfig.severity as TodoConfig['severity']) || 'low',
      allowedPatterns: (pluginConfig.allowedPatterns as string[]) || [],
      requireIssueReference: (pluginConfig.requireIssueReference as boolean) || false
    };
  }

  async onValidationComplete(results: ValidationResult[]): Promise<void> {
    // This is called after all built-in validations
    // We could modify results here if needed
  }

  async onFileChange(event: FileChangeEvent): Promise<void> {
    // React to file changes
    if (event.type === 'modify') {
      await this.validateFile(event.path);
    }
  }

  private async validateFile(filePath: string): Promise<ValidationResult[]> {
    const violations: ValidationResult[] = [];
    
    // Read file and check for TODOs
    // Implementation would depend on file system access
    
    return violations;
  }

  async dispose(): Promise<void> {
    // Cleanup
  }
}

export default TodoConstraintPlugin;
```

## Advanced Constraint: Import Graph Validator

Here's a more complex example that validates import patterns:

```rust
use assura::constraint::{Constraint, ConstraintResult, Context, Violation};
use assura::graph::DependencyGraph;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;

/// Validates that imports follow architectural boundaries
pub struct ArchitectureConstraint {
    severity: Severity,
    layers: Vec<Layer>,
    forbidden_imports: Vec<ForbiddenImport>,
}

struct Layer {
    name: String,
    pattern: String,
    allowed_dependencies: Vec<String>,
}

struct ForbiddenImport {
    from: String,
    to: String,
    reason: String,
}

#[async_trait]
impl Constraint for ArchitectureConstraint {
    fn name(&self) -> &str {
       "architecture-check"
    }

    fn description(&self) -> &str {
        "Validates architectural import boundaries"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn configure(&mut self, config: &Value) -> Result<(), ConfigError> {
        if let Some(severity) = config.get("severity") {
            self.severity = parse_severity(severity)?;
        }

        if let Some(layers) = config.get("layers") {
            self.layers = layers.as_array()
                .ok_or_else(|| ConfigError::Validation("layers must be an array".into()))?
                .iter()
                .map(|layer| Layer {
                    name: layer.get("name")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .ok_or_else(|| ConfigError::Validation("layer must have a name".into()))?,
                    pattern: layer.get("pattern")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .unwrap_or_default(),
                    allowed_dependencies: layer.get("allowed_dependencies")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                })
                .collect::<Result<Vec<_>, _>>()?;
        }

        Ok(())
    }

    async fn check(&self, context: &Context) -> Result<ConstraintResult, ConstraintError> {
        let mut violations = Vec::new();

        // Determine which layer this file belongs to
        let file_layer = self.determine_layer(&context.file_path);
        
        if let Some(layer) = file_layer {
            // Check imports against layer rules
            let imports = context.imports();
            
            for import in imports {
                let import_layer = self.determine_layer_from_import(&import.source);
                
                if let Some(import_layer_name) = import_layer {
                    if !layer.allowed_dependencies.contains(&import_layer_name) {
                        violations.push(Violation {
                            file: context.file_path.clone(),
                            line: import.line,
                            column: None,
                            message: format!(
                                "Layer '{}' cannot import from layer '{}'",
                                layer.name, import_layer_name
                            ),
                            suggestion: Some(format!(
                                "Move code to an allowed layer or add '{}' to allowed dependencies",
                                import_layer_name
                            )),
                        });
                    }
                }
            }
        }

        // Check forbidden imports
        for forbidden in &self.forbidden_imports {
            if context.file_content.contains(&forbidden.to) {
                violations.push(Violation {
                    file: context.file_path.clone(),
                    line: None,
                    column: None,
                    message: format!(
                        "Forbidden import: {}. Reason: {}",
                        forbidden.to, forbidden.reason
                    ),
                    suggestion: Some("Remove or replace this import".to_string()),
                });
            }
        }

        Ok(ConstraintResult::violations(violations))
    }
}
```

## Configuration Examples

### Simple TODO Detection

```yaml
rules:
  - name: todo-detection
    severity: medium
```

### Strict TODO Management

```yaml
rules:
  - name: todo-detection
    severity: high
    allowed_patterns:
      - "TODO(#"  # Only allow TODOs with issue references
    require_issue_reference: true
```

### Architecture Validation

```yaml
rules:
  - name: architecture-check
    severity: critical
    layers:
      - name: "domain"
        pattern: "src/domain/**"
        allowed_dependencies: []
      
      - name: "application"
        pattern: "src/application/**"
        allowed_dependencies:
          - "domain"
      
      - name: "infrastructure"
        pattern: "src/infrastructure/**"
        allowed_dependencies:
          - "domain"
          - "application"
```

## Testing Custom Constraints

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todo_detection() {
        let constraint = TodoConstraint::new();
        let context = create_test_context(
            "test.rs",
            "// TODO: Fix this bug"
        );

        let result = constraint.check(&context).await.unwrap();
        
        assert!(!result.is_ok());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].message, "TODO comment found");
    }

    #[tokio::test]
    async fn test_allowed_todo_pattern() {
        let mut constraint = TodoConstraint::new();
        constraint.configure(&serde_json::json!({
            "allowed_patterns": ["TODO(#"]
        })).unwrap();

        let context = create_test_context(
            "test.rs",
            "// TODO(#123): Fix this bug"
        );

        let result = constraint.check(&context).await.unwrap();
        assert!(result.is_ok());
    }

    fn create_test_context(path: &str, content: &str) -> Context {
        // Helper to create test context
        todo!()
    }
}
```

<Aside type="tip" title="Performance Tips">
  - Cache expensive computations in your constraint
  - Use the context's graph to avoid re-parsing
  - Mark constraints as async only if they need I/O
  - Return early when possible
</Aside>

## Publishing Your Constraint

To share your constraint with others:

1. Create a Rust crate with your constraint
2. Implement the `Constraint` trait
3. Publish to crates.io
4. Document usage in your README

Example crate structure:

```
assura-todo-constraint/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── todo_constraint.rs
├── tests/
│   └── integration_tests.rs
└── README.md
```

<Aside type="note" title="Community Constraints">
  Check the [Assura GitHub Discussions](https://github.com/anomalyco/assura/discussions) for community-contributed constraints.
</Aside>
