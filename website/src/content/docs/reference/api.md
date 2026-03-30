---
title: API Reference
description: Complete API documentation for the Assura library and TypeScript plugin
template: doc
sidebar:
  order: 2
---

import { Tabs, TabItem, Aside, Steps, Card, CardGrid } from '@astrojs/starlight/components';

This reference documents the complete Assura API for both the Rust library and TypeScript plugin development.

## Rust API Overview

The Assura library provides a programmatic API for building custom validation tools and integrations.

### Installation

Add Assura to your `Cargo.toml`:

```toml
[dependencies]
assura = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use assura::{Config, Validator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(".assura/config.yml")?;
    let validator = Validator::new(config);
    let results = validator.validate_all().await?;
    
    for result in results {
        println!("[{}] {}", result.severity, result.message);
    }
    
    Ok(())
}
```

## Core Types

### Config

Configuration structure loaded from configuration files.

```rust
pub struct Config {
    pub name: Option<String>,
    pub version: Option<String>,
    pub settings: Settings,
    pub rules: Vec<Rule>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub maturity_overrides: Vec<MaturityOverride>,
    pub per_file_overrides: Vec<PerFileOverride>,
}

impl Config {
    /// Load configuration from a file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;
    
    /// Load configuration from default locations
    pub fn load_default() -> Result<Self, ConfigError>;
    
    /// Load configuration from a string
    pub fn from_str(s: &str, format: ConfigFormat) -> Result<Self, ConfigError>;
    
    /// Merge with another configuration
    pub fn merge(&mut self, other: Config);
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Vec<ConfigError>>;
}
```

### Settings

Global validation settings.

```rust
pub struct Settings {
    pub parallel: bool,
    pub max_workers: usize,
    pub cache_enabled: bool,
    pub cache_dir: PathBuf,
    pub watch_delay: u64,
    pub fail_fast: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            parallel: true,
            max_workers: num_cpus::get(),
            cache_enabled: true,
            cache_dir: PathBuf::from(".assura/cache"),
            watch_delay: 100,
            fail_fast: false,
        }
    }
}
```

### Rule

Validation rule definition.

```rust
pub struct Rule {
    pub name: String,
    pub severity: Severity,
    pub enabled: bool,
    pub config: Value,  // Rule-specific configuration
}

impl Rule {
    /// Create a new rule with default settings
    pub fn new(name: impl Into<String>, severity: Severity) -> Self;
    
    /// Disable the rule
    pub fn disable(&mut self);
    
    /// Enable the rule
    pub fn enable(&mut self);
    
    /// Set rule configuration
    pub fn with_config(&mut self, config: Value) -> &mut Self;
}
```

### Severity

Severity level for validation issues.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    /// Check if this severity is at least as severe as another
    pub fn is_at_least(&self, other: Severity) -> bool;
    
    /// Get the severity as a numeric rank
    pub fn rank(&self) -> u8;
    
    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self>;
}
```

## Intelligence Graph API

The Intelligence Graph API provides access to the dependency graph and project intelligence.

### DependencyGraph

Represents the project's dependency structure.

```rust
pub struct DependencyGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl DependencyGraph {
    /// Build a dependency graph from source files
    pub fn from_files(files: &[PathBuf]) -> Result<Self, GraphError>;
    
    /// Build a dependency graph from file contents
    pub fn from_contents(contents: &[(PathBuf, String)]) -> Self;
    
    /// Detect circular dependencies
    pub fn detect_cycles(&self) -> Vec<Vec<NodeIndex>>;
    
    /// Get topological sort order
    pub fn topological_sort(&self) -> Result<Vec<NodeIndex>, GraphError>;
    
    /// Get nodes that depend on a given node
    pub fn dependents(&self, node: NodeIndex) -> Vec<NodeIndex>;
    
    /// Get nodes that a given node depends on
    pub fn dependencies(&self, node: NodeIndex) -> Vec<NodeIndex>;
    
    /// Calculate depth of each node
    pub fn calculate_depths(&self) -> HashMap<NodeIndex, usize>;
    
    /// Export to DOT format for visualization
    pub fn to_dot(&self) -> String;
    
    /// Export to JSON format
    pub fn to_json(&self) -> Result<String, serde_json::Error>;
}
```

### Node

Represents a file or module in the dependency graph.

```rust
pub struct Node {
    pub id: NodeIndex,
    pub path: PathBuf,
    pub module_name: String,
    pub exports: Vec<Export>,
    pub imports: Vec<Import>,
}

pub struct Export {
    pub name: String,
    pub kind: ExportKind,
    pub is_public: bool,
}

pub struct Import {
    pub source: String,
    pub items: Vec<ImportItem>,
}

pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

pub enum ExportKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Type,
}
```

### Graph Usage Example

```rust
use assura::graph::{DependencyGraph, NodeIndex};

async fn analyze_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    // Build graph from project files
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/utils.rs"),
    ];
    
    let graph = DependencyGraph::from_files(&files)?;
    
    // Check for circular dependencies
    let cycles = graph.detect_cycles();
    if !cycles.is_empty() {
        println!("Circular dependencies detected:");
        for cycle in cycles {
            let names: Vec<_> = cycle.iter()
                .map(|&idx| graph.get_node(idx).module_name.clone())
                .collect();
            println!("  - {}", names.join(" → "));
        }
    }
    
    // Get validation order
    let order = graph.topological_sort()?;
    println!("Validation order:");
    for idx in order {
        println!("  {}", graph.get_node(idx).path.display());
    }
    
    // Export for visualization
    let dot = graph.to_dot();
    std::fs::write("dependencies.dot", dot)?;
    
    Ok(())
}
```

## Constraint System API

The Constraint System API allows you to create custom validation rules and constraints.

### Constraint

Trait for implementing custom constraints.

```rust
#[async_trait]
pub trait Constraint: Send + Sync {
    /// Unique constraint identifier
    fn name(&self) -> &str;
    
    /// Human-readable description
    fn description(&self) -> &str;
    
    /// Check if the constraint is satisfied
    async fn check(&self, context: &Context) -> Result<ConstraintResult, ConstraintError>;
    
    /// Get the severity level for violations
    fn severity(&self) -> Severity;
    
    /// Configure the constraint from a value
    fn configure(&mut self, config: &Value) -> Result<(), ConfigError>;
}
```

### ConstraintResult

Result of a constraint check.

```rust
pub struct ConstraintResult {
    pub satisfied: bool,
    pub violations: Vec<Violation>,
    pub metadata: HashMap<String, Value>,
}

pub struct Violation {
    pub file: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

impl ConstraintResult {
    /// Create a successful result
    pub fn success() -> Self;
    
    /// Create a result with violations
    pub fn violations(violations: Vec<Violation>) -> Self;
    
    /// Add a violation
    pub fn add_violation(&mut self, violation: Violation);
    
    /// Check if result has no violations
    pub fn is_ok(&self) -> bool;
}
```

### Context

Validation context passed to constraints.

```rust
pub struct Context {
    pub config: Arc<Config>,
    pub file_path: PathBuf,
    pub file_content: String,
    pub graph: Arc<DependencyGraph>,
    pub cache: Arc<dyn Cache>,
}

impl Context {
    /// Get file content as AST (if parseable)
    pub fn ast(&self) -> Option<&SyntaxTree>;
    
    /// Get parsed imports
    pub fn imports(&self) -> Vec<Import>;
    
    /// Get file metadata
    pub fn metadata(&self) -> &FileMetadata;
    
    /// Access the dependency graph
    pub fn graph(&self) -> &DependencyGraph;
}
```

### Custom Constraint Example

```rust
use assura::constraint::{Constraint, ConstraintResult, Context, Violation};
use async_trait::async_trait;

pub struct TodoConstraint {
    severity: Severity,
    allowed_patterns: Vec<String>,
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
        if let Some(severity) = config.get("severity") {
            self.severity = Severity::from_str(severity.as_str().unwrap_or("low"))
                .ok_or(ConfigError::InvalidSeverity)?;
        }
        if let Some(patterns) = config.get("allowed_patterns") {
            self.allowed_patterns = patterns.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
        }
        Ok(())
    }
    
    async fn check(&self, context: &Context) -> Result<ConstraintResult, ConstraintError> {
        let mut violations = Vec::new();
        
        for (line_num, line) in context.file_content.lines().enumerate() {
            if line.to_uppercase().contains("TODO") {
                // Check if it matches an allowed pattern
                let allowed = self.allowed_patterns.iter()
                    .any(|pattern| line.contains(pattern));
                
                if !allowed {
                    violations.push(Violation {
                        file: context.file_path.clone(),
                        line: Some(line_num + 1),
                        column: None,
                        message: "TODO comment found".to_string(),
                        suggestion: Some("Create an issue or remove the TODO".to_string()),
                    });
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
```

## Maturity Detection API

The Maturity Detection API analyzes code to determine project maturity levels.

### MaturityAnalyzer

Analyzes project maturity based on various metrics.

```rust
pub struct MaturityAnalyzer {
    metrics: Vec<Box<dyn MaturityMetric>>,
}

impl MaturityAnalyzer {
    /// Create a new analyzer with default metrics
    pub fn new() -> Self;
    
    /// Add a custom metric
    pub fn add_metric(&mut self, metric: Box<dyn MaturityMetric>);
    
    /// Analyze a project
    pub async fn analyze(&self, project_path: &Path) -> Result<MaturityReport, AnalysisError>;
    
    /// Analyze specific files
    pub async fn analyze_files(&self, files: &[PathBuf]) -> Result<MaturityReport, AnalysisError>;
}
```

### MaturityReport

Report containing maturity analysis results.

```rust
pub struct MaturityReport {
    pub overall_score: f64,  // 0.0 to 1.0
    pub level: MaturityLevel,
    pub metrics: Vec<MetricResult>,
    pub recommendations: Vec<Recommendation>,
}

pub enum MaturityLevel {
    Alpha,      // Early development
    Beta,       // Feature complete, testing
    Release,    // Production ready
    Mature,     // Stable, well-maintained
}

pub struct MetricResult {
    pub name: String,
    pub score: f64,
    pub weight: f64,
    pub details: Value,
}

pub struct Recommendation {
    pub priority: Priority,
    pub message: String,
    pub action: Option<String>,
}
```

### Built-in Maturity Metrics

```rust
// Documentation coverage
pub struct DocumentationMetric;

// Test coverage
pub struct TestCoverageMetric;

// Dependency freshness
pub struct DependencyFreshnessMetric;

// Code complexity
pub struct ComplexityMetric;

// Security posture
pub struct SecurityMetric;
```

### Maturity Detection Example

```rust
use assura::maturity::{MaturityAnalyzer, MaturityLevel};

async fn check_maturity() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = MaturityAnalyzer::new();
    let report = analyzer.analyze(Path::new(".")).await?;
    
    println!("Project Maturity: {:?}", report.level);
    println!("Overall Score: {:.1}%", report.overall_score * 100.0);
    
    println!("\nMetrics:");
    for metric in &report.metrics {
        println!("  {}: {:.1}%", metric.name, metric.score * 100.0);
    }
    
    if !report.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &report.recommendations {
            println!("  [{}] {}", rec.priority, rec.message);
        }
    }
    
    // Use maturity level to adjust validation rules
    match report.level {
        MaturityLevel::Alpha => {
            // Relaxed rules for early development
        }
        MaturityLevel::Release | MaturityLevel::Mature => {
            // Strict rules for production
        }
        _ => {}
    }
    
    Ok(())
}
```

## TypeScript Plugin API

Assura provides a TypeScript plugin API for creating custom integrations and extensions.

### Plugin Interface

```typescript
interface AssuraPlugin {
  name: string;
  version: string;
  
  // Initialize the plugin
  initialize(config: PluginConfig): Promise<void>;
  
  // Process validation results
  onValidationComplete(results: ValidationResult[]): Promise<void>;
  
  // Handle file changes
  onFileChange(event: FileChangeEvent): Promise<void>;
  
  // Clean up resources
  dispose(): Promise<void>;
}

interface PluginConfig {
  [key: string]: any;
}

interface ValidationResult {
  rule: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  file: string;
  line?: number;
  column?: number;
  message: string;
  suggestion?: string;
}

interface FileChangeEvent {
  type: 'create' | 'modify' | 'delete';
  path: string;
  timestamp: number;
}
```

### Creating a Plugin

```typescript
import { AssuraPlugin, ValidationResult } from '@assura/plugin-sdk';

class SlackNotificationPlugin implements AssuraPlugin {
  name = 'slack-notifications';
  version = '1.0.0';
  
  private webhookUrl: string;
  private minSeverity: string;
  
  async initialize(config: PluginConfig): Promise<void> {
    this.webhookUrl = config.webhookUrl;
    this.minSeverity = config.minSeverity || 'high';
    
    if (!this.webhookUrl) {
      throw new Error('webhookUrl is required');
    }
  }
  
  async onValidationComplete(results: ValidationResult[]): Promise<void> {
    const important = results.filter(r => 
      this.severityRank(r.severity) >= this.severityRank(this.minSeverity)
    );
    
    if (important.length === 0) return;
    
    const message = {
      text: `Assura found ${important.length} issues`,
      attachments: important.map(r => ({
        color: this.severityColor(r.severity),
        text: `[${r.severity.toUpperCase()}] ${r.file}:${r.line} - ${r.message}`
      }))
    };
    
    await fetch(this.webhookUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(message)
    });
  }
  
  async onFileChange(event: FileChangeEvent): Promise<void> {
    // React to file changes if needed
    console.log(`File ${event.type}d: ${event.path}`);
  }
  
  async dispose(): Promise<void> {
    // Clean up resources
  }
  
  private severityRank(severity: string): number {
    const ranks: Record<string, number> = {
      critical: 4,
      high: 3,
      medium: 2,
      low: 1
    };
    return ranks[severity] || 0;
  }
  
  private severityColor(severity: string): string {
    const colors: Record<string, string> = {
      critical: 'danger',
      high: 'warning',
      medium: '#439FE0',
      low: 'good'
    };
    return colors[severity] || 'good';
  }
}

export default SlackNotificationPlugin;
```

### Plugin Registration

```typescript
// Register your plugin
import { registerPlugin } from '@assura/plugin-sdk';
import SlackNotificationPlugin from './slack-plugin';

registerPlugin('slack-notifications', SlackNotificationPlugin);
```

### Configuration

Enable plugins in your `.assura/config.yml`:

```yaml
plugins:
  - name: slack-notifications
    enabled: true
    config:
      webhookUrl: "${SLACK_WEBHOOK_URL}"
      minSeverity: "high"
```

## Error Types

### ConfigError

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Invalid severity level: {0}")]
    InvalidSeverity,
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Unknown rule: {0}")]
    UnknownRule(String),
}
```

### ValidationError

```rust
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Rule execution failed: {0}")]
    RuleExecution(String),
    
    #[error("File access error for {0}: {1}")]
    FileAccess(PathBuf, #[source] std::io::Error),
    
    #[error("Graph construction error: {0}")]
    GraphConstruction(String),
    
    #[error("Constraint error: {0}")]
    Constraint(#[from] ConstraintError),
    
    #[error("Cache error: {0}")]
    Cache(String),
}
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `default` | Standard validation features | Yes |
| `watch` | File system watching with `notify` | Yes |
| `parallel` | Parallel validation using Rayon | Yes |
| `json` | JSON configuration support | Yes |
| `toml` | TOML configuration support | Yes |
| `graphql` | GraphQL schema validation | No |
| `wasm` | WebAssembly bindings | No |

Enable features in `Cargo.toml`:

```toml
[dependencies]
assura = { version = "0.1", features = ["graphql", "wasm"] }
```

<Aside type="note" title="API Stability">
  The Rust API is still evolving. Check the changelog for breaking changes between versions.
</Aside>
