---
title: API Reference
description: Complete API documentation for the Assura library
---

import { Aside } from '@astrojs/starlight/components';

The Assura library provides a programmatic API for building custom validation tools and integrations.

<Aside type="caution">
  This API is still in development and may change in future versions.
</Aside>

## Core Types

### Config

Configuration structure loaded from YAML/JSON/TOML files.

```rust
pub struct Config {
    pub name: Option<String>,
    pub version: Option<String>,
    pub settings: Settings,
    pub rules: Vec<Rule>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
}
```

### Validator

Main validation engine that orchestrates rule execution.

```rust
impl Validator {
    /// Create a new validator with the given configuration
    pub fn new(config: Config) -> Self;
    
    /// Validate all configured rules
    pub async fn validate_all(&self) -> Result<Vec<ValidationResult>, Error>;
    
    /// Validate specific paths
    pub async fn validate_paths(&self, paths: &[PathBuf]) -> Result<Vec<ValidationResult>, Error>;
    
    /// Watch for file changes and re-validate
    pub async fn watch(&self) -> Result<(), Error>;
}
```

### ValidationResult

Result of a validation operation.

```rust
pub struct ValidationResult {
    pub rule: String,
    pub severity: Severity,
    pub file: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}
```

## Usage Examples

### Basic Validation

```rust
use assura::{Config, Validator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load("assura.yaml")?;
    
    // Create validator
    let validator = Validator::new(config);
    
    // Run validation
    let results = validator.validate_all().await?;
    
    // Process results
    for result in results {
        println!("[{}] {}: {}", 
            result.severity, 
            result.file.display(), 
            result.message
        );
    }
    
    Ok(())
}
```

### Watch Mode

```rust
use assura::{Config, Validator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("assura.yaml")?;
    let validator = Validator::new(config);
    
    println!("Starting file watcher...");
    validator.watch().await?;
    
    Ok(())
}
```

### Custom Reporter

```rust
use assura::{Validator, ValidationResult, Severity};

struct JsonReporter;

impl JsonReporter {
    fn report(&self, results: &[ValidationResult]) {
        let json = serde_json::to_string_pretty(results).unwrap();
        println!("{}", json);
    }
}
```

## Error Types

### ConfigError

Errors that occur during configuration loading and parsing.

```rust
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}
```

### ValidationError

Errors that occur during the validation process.

```rust
pub enum ValidationError {
    RuleExecution(String),
    FileAccess(PathBuf, std::io::Error),
    GraphConstruction(String),
}
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `default` | Standard validation features | Yes |
| `watch` | File system watching | Yes |
| `parallel` | Parallel validation using Rayon | Yes |
| `json` | JSON configuration support | Yes |
| `toml` | TOML configuration support | Yes |
