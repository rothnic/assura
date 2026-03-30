# Assura V2 Configuration

## Overview

Assura V2 introduces a **structure-first** approach to project validation configuration. Instead of defining rules that apply to patterns scattered throughout your project, V2 allows you to define your project structure hierarchically and apply validation bundles to each node.

### Key Benefits

- **Visual clarity**: Structure mirrors your actual directory layout
- **DRY principle**: Parent rules automatically inherited by children
- **Specific overrides**: Children can override parent settings while inheriting others
- **Performance**: Optimized resolution for large projects
- **LS-Lint compatibility**: Easy migration from existing LS-Lint configurations

## Structure-First Philosophy

In V1, you defined rules like:

```yaml
# V1 - Rule-centric approach
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
  - name: naming
    applies_to: "src/components/**/*.rs"
    convention: PascalCase
```

In V2, you define structure:

```yaml
# V2 - Structure-first approach
structure:
  src/:
    files:
      naming: snake_case
    children:
      components/:
        files:
          naming: PascalCase
```

## Complete V2 Config Format Specification

### Root Structure

```yaml
version: "2.0"  # Required - identifies V2 config

# Project metadata (optional)
project:
  name: "My Project"
  description: "Project description"
  maturity: stable  # alpha, beta, stable

# Hierarchical structure (required)
structure:
  # Structure nodes...

# LS-Lint compatibility layer (optional)
ls:
  # LS-Lint style rules...

# Global exclusions (optional)
exclude:
  - "target/**"
  - ".git/**"
```

### Structure Nodes

Each key in the `structure` map is a directory path. Each value is a `StructureNode`:

```yaml
structure:
  <path>/:
    files:           # FileValidationBundle (optional)
      # File validation options...
    
    markdown:        # MarkdownValidationBundle (optional)
      # Markdown validation options...
    
    children:        # Nested structure nodes (optional)
      <child-path>/:
        # Child structure node...
    
    inherit: true    # Whether to inherit from parent (default: true)
```

## FileValidationBundle Options

The `files` section configures file-level validations:

```yaml
files:
  # Naming convention (string, optional)
  # Options: snake_case, kebab-case, camelCase, PascalCase, 
  #          SCREAMING_SNAKE_CASE, dot.case, flatcase, FLATCASE,
  #          COBOL-CASE, Train-Case, lowercase, UPPERCASE
  #          OR prefix with "regex:" for custom patterns
  naming: "snake_case"
  
  # Maximum lines per file (integer, optional, range: 1-100000)
  max_lines: 500
  
  # Maximum file size (string, optional)
  # Format: "<number><unit>" where unit is B, KB, MB, GB, or TB
  # Examples: "500KB", "1MB", "10 MB"
  max_size: "500KB"
  
  # Require documentation (boolean, optional)
  # For Rust: requires rustdoc on public items
  # For other languages: requires module-level documentation
  require_docs: true
  
  # Allowed file extensions (array of strings, optional)
  # Only these extensions will be validated in this node
  extensions:
    - "rs"
    - "md"
  
  # Default severity for violations (string, optional)
  # Options: critical, high, medium, low
  # Can be overridden by individual rules
  severity: "high"
```

## MarkdownValidationBundle Options

The `markdown` section configures Markdown-specific validations:

```yaml
markdown:
  # Require frontmatter (boolean, optional)
  require_frontmatter: true
  
  # Required frontmatter fields (array of strings, optional)
  # Only checked if require_frontmatter is true or implied
  required_fields:
    - "title"
    - "description"
    - "date"
  
  # Maximum heading depth (integer, optional, range: 1-6)
  # Prevents excessively nested headings
  max_heading_depth: 4
  
  # Check for dead links (boolean, optional)
  # Validates internal and external links
  check_links: true
  
  # Required sections (array of strings, optional)
  # Each markdown file must contain these headings
  required_sections:
    - "Introduction"
    - "Examples"
```

## Hierarchical Inheritance

### How Inheritance Works

By default (`inherit: true`), child nodes inherit all settings from their parent:

```yaml
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
      severity: high
    
    children:
      components/:
        files:
          naming: PascalCase  # Overrides parent's naming
        # max_lines: 500 is inherited
        # severity: high is inherited
```

### Disabling Inheritance

To completely isolate a child from its parent's rules:

```yaml
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
    
    children:
      third_party/:
        inherit: false  # Don't inherit anything from src/
        files:
          # Only these settings apply
          naming: regex:.*
```

### Inheritance Rules

1. **Explicit values always override inherited values**
2. **Missing values are filled from parent**
3. **`inherit: false` creates a clean slate**
4. **Specificity increases with depth** - deeper rules win conflicts

### Specificity Scoring

When multiple rules could apply to a file, Assura uses specificity:

- Depth × 10 (deeper paths are more specific)
- +5 for exact paths (no wildcards)
- +length of path (longer paths are more specific)

Example precedence order:
1. `src/components/Button.tsx` (exact match)
2. `src/components/**/*.tsx` (glob pattern)
3. `src/components/` (directory prefix)
4. `src/` (parent directory)

## LS-Lint Compatibility Layer

Assura V2 includes built-in support for LS-Lint configurations:

### Automatic Conversion

If you have an existing `.ls-lint.yml`:

```yaml
ls:
  .rs: snake_case
  .ts: camelCase
  src/:
    .rs: PascalCase
```

Simply wrap it in the V2 structure:

```yaml
version: "2.0"
ls:
  .rs: snake_case
  .ts: camelCase
  src/:
    .rs: PascalCase

# Or convert to native V2 structure:
structure:
  "":
    files:
      naming: snake_case
  src/:
    files:
      naming: PascalCase
```

### Supported LS-Lint Conventions

All LS-Lint naming conventions are supported:

| LS-Lint | Assura V2 |
|---------|-----------|
| `kebab-case` | `kebab-case` |
| `snake_case` | `snake_case` |
| `camelCase` | `camelCase` |
| `PascalCase` | `PascalCase` |
| `SCREAMING_SNAKE_CASE` | `SCREAMING_SNAKE_CASE` |
| `dot.case` | `dot.case` |
| `flatcase` | `flatcase` |
| `FLATCASE` | `FLATCASE` |
| `COBOL-CASE` | `COBOL-CASE` |
| `Train-Case` | `Train-Case` |
| `lowercase` | `lowercase` |
| `UPPERCASE` | `UPPERCASE` |
| `regex:pattern` | `regex:pattern` |

## Multiple Examples

### Example 1: Simple Rust Project

```yaml
version: "2.0"

project:
  name: "my-rust-app"
  maturity: stable

structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
      require_docs: true
      severity: high
    
    children:
      bin/:
        files:
          naming: snake_case
      
      lib/:
        files:
          naming: snake_case
          require_docs: true
  
  tests/:
    files:
      naming: snake_case
      max_lines: 1000
      require_docs: false
      severity: medium
  
  benches/:
    files:
      naming: snake_case
      max_lines: 1000
      require_docs: false
  
  docs/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
        - "description"
      check_links: true

exclude:
  - "target/**"
  - ".git/**"
  - "Cargo.lock"
```

### Example 2: Full-Stack TypeScript/React Project

```yaml
version: "2.0"

project:
  name: "fullstack-app"
  maturity: beta

structure:
  # Backend
  server/:
    files:
      naming: camelCase
      max_lines: 400
    
    children:
      src/:
        files:
          naming: camelCase
          max_lines: 400
        
        children:
          models/:
            files:
              naming: PascalCase  # Model classes
          
          routes/:
            files:
              naming: camelCase
          
          utils/:
            files:
              naming: camelCase
  
  # Frontend
  client/:
    files:
      naming: camelCase
    
    children:
      src/:
        files:
          naming: camelCase
        
        children:
          components/:
            files:
              naming: PascalCase  # React components
              extensions:
                - "tsx"
                - "css"
          
          hooks/:
            files:
              naming: camelCase
              extensions:
                - "ts"
          
          pages/:
            files:
              naming: kebab-case  # Next.js pages
              extensions:
                - "tsx"
  
  # Documentation
  docs/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
        - "description"
        - "last_updated"
      max_heading_depth: 3
      required_sections:
        - "Overview"
  
  # Configuration files
  "":
    files:
      naming: kebab-case
      extensions:
        - "json"
        - "yml"
        - "yaml"

exclude:
  - "node_modules/**"
  - "dist/**"
  - ".next/**"
  - "coverage/**"
```

### Example 3: Multi-Language Monorepo

```yaml
version: "2.0"

project:
  name: "monorepo"
  maturity: stable

structure:
  # Rust packages
  packages/rust/:
    files:
      naming: snake_case
      max_lines: 500
      require_docs: true
    
    children:
      core/:
        files:
          naming: snake_case
          severity: critical
      
      utils/:
        files:
          naming: snake_case
          severity: medium
  
  # TypeScript packages
  packages/ts/:
    files:
      naming: camelCase
      max_lines: 400
    
    children:
      ui-kit/:
        files:
          naming: PascalCase
        
        children:
          components/:
            files:
              naming: PascalCase
          
          styles/:
            files:
              naming: kebab-case
      
      shared/:
        files:
          naming: camelCase
  
  # Python packages
  packages/python/:
    files:
      naming: snake_case
      max_lines: 600
    
    children:
      api/:
        files:
          naming: snake_case
          require_docs: true
  
  # Documentation
  docs/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
      check_links: true
      max_heading_depth: 4
  
  # CI/CD configs
  .github/:
    files:
      naming: kebab-case
      extensions:
        - "yml"
  
  # Root configuration files
  "":
    files:
      naming: kebab-case

exclude:
  - "**/target/**"
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/build/**"
  - "**/__pycache__/**"
  - "**/.venv/**"
```

### Example 4: Library with Mixed Conventions

```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      require_docs: true
    
    children:
      # Core library code
      lib/:
        files:
          naming: snake_case
          severity: critical
      
      # Public API - stricter rules
      api/:
        files:
          naming: snake_case
          max_lines: 300
          require_docs: true
          severity: critical
      
      # Internal utilities - more relaxed
      internal/:
        files:
          naming: snake_case
          require_docs: false
          severity: medium
      
      # Generated code - minimal validation
      generated/:
        inherit: false
        files:
          naming: regex:.*
          require_docs: false
          severity: low
  
  tests/:
    files:
      naming: snake_case
      max_lines: 2000
      require_docs: false
    
    children:
      integration/:
        files:
          naming: snake_case
          max_lines: 2000
      
      unit/:
        files:
          naming: snake_case
          max_lines: 500
  
  examples/:
    files:
      naming: snake_case
      require_docs: false
      severity: low

exclude:
  - "target/**"
```

### Example 5: Documentation-Heavy Project

```yaml
version: "2.0"

project:
  name: "docs-project"
  maturity: stable

structure:
  # API documentation
  api-docs/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
        - "version"
        - "api_level"
      required_sections:
        - "Endpoints"
        - "Parameters"
        - "Response"
      max_heading_depth: 4
      check_links: true
  
  # User guides
  guides/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
        - "description"
      required_sections:
        - "Introduction"
        - "Prerequisites"
        - "Steps"
      max_heading_depth: 3
      check_links: true
  
  # Tutorials
  tutorials/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
        - "difficulty"
        - "time_estimate"
      required_sections:
        - "Goal"
        - "Instructions"
      max_heading_depth: 3
  
  # Blog posts
  blog/:
    markdown:
      require_frontmatter: true
      required_fields:
        - "title"
        - "author"
        - "date"
      max_heading_depth: 2
  
  # Source code
  src/:
    files:
      naming: snake_case
      max_lines: 300
      require_docs: true

exclude:
  - "_site/**"
  - "node_modules/**"
```

## Migration Guide from V1 to V2

### Quick Reference

| V1 Concept | V2 Equivalent |
|------------|---------------|
| `rules: []` | `structure: {}` |
| `applies_to: "pattern"` | Structure node path |
| Multiple `applies_to` for same rule | Single structure node with children |
| `includes: []` | Structure paths |
| `excludes: []` | `exclude: []` (unchanged) |
| Severity per rule | `severity` in bundle |

### Step-by-Step Migration

1. **Add version field**: Set `version: "2.0"` at the top

2. **Convert rules to structure**:
   - Replace `rules:` array with `structure:` map
   - Each `applies_to` becomes a structure node path
   - Rule options become bundle properties

3. **Handle inheritance**:
   - Remove duplicate settings that can be inherited
   - Use `children:` for nested paths
   - Set `inherit: false` where needed

4. **Validate**: Run `assura config validate` to check your config

### Before/After Examples

#### Example: Naming Conventions

**V1:**
```yaml
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
    severity: high
  
  - name: naming
    applies_to: "src/components/**/*.rs"
    convention: PascalCase
    severity: high
  
  - name: naming
    applies_to: "tests/**/*.rs"
    convention: snake_case
    severity: medium
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      severity: high
    
    children:
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      naming: snake_case
      severity: medium
```

#### Example: File Size Limits

**V1:**
```yaml
rules:
  - name: file-size
    applies_to: "src/**/*.rs"
    max_size: "500KB"
    severity: medium
  
  - name: file-size
    applies_to: "tests/**/*.rs"
    max_size: "1MB"
    severity: low
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      max_size: "500KB"
      severity: medium
  
  tests/:
    files:
      max_size: "1MB"
      severity: low
```

#### Example: Documentation Requirements

**V1:**
```yaml
rules:
  - name: documentation
    applies_to: "src/**/*.rs"
    require_public: true
    severity: medium
  
  - name: documentation
    applies_to: "src/internal/**/*.rs"
    require_public: false
    severity: low
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      require_docs: true
      severity: medium
    
    children:
      internal/:
        files:
          require_docs: false
          severity: low
```

#### Example: Markdown Validation

**V1:**
```yaml
markdown:
  validation:
    enabled: true
    rules:
      - id: frontmatter-required
        applies_to: "docs/**/*.md"
        severity: medium
      
      - id: link-check
        applies_to: "docs/**/*.md"
        severity: high
```

**V2:**
```yaml
version: "2.0"

structure:
  docs/:
    markdown:
      require_frontmatter: true
      check_links: true
      severity: high
```

## Troubleshooting

### Common Issues

#### Config Not Recognized as V2

**Problem**: Assura treats your config as V1

**Solution**: Ensure `version: "2.0"` is at the top level:

```yaml
version: "2.0"  # Must be exactly "2.0"

structure:
  # ...
```

#### Inheritance Not Working

**Problem**: Child settings aren't inheriting from parent

**Solution**: Check that:
1. Parent path ends with `/` (e.g., `src/` not `src`)
2. `inherit: true` is set (or omitted, as it's the default)
3. Child path is under parent's `children:`

```yaml
structure:
  src/:  # Note the trailing slash
    files:
      naming: snake_case
    
    children:  # Must use children key
      components/:
        inherit: true  # Default, can be omitted
        files:
          naming: PascalCase  # Will inherit max_lines, etc.
```

#### Patterns Not Matching

**Problem**: Files aren't being validated

**Solution**: 
1. Check path patterns use `/` separators (even on Windows)
2. Ensure paths end with `/` for directories
3. Use `assura check --verbose` to see which files are being checked

```yaml
# Good
structure:
  src/:
    files:
      naming: snake_case

# Bad - missing trailing slash
structure:
  src:
    files:
      naming: snake_case
```

#### Severity Not Applied

**Problem**: Violations show wrong severity

**Solution**: Ensure `severity` is in the `files` or `markdown` bundle, not at the node level:

```yaml
# Good
structure:
  src/:
    files:
      severity: high  # In the bundle
      naming: snake_case

# Bad - wrong location
structure:
  src/:
    severity: high  # Won't work here
    files:
      naming: snake_case
```

### Performance Characteristics

#### Configuration Loading

- **V1**: O(n) where n = number of rules
- **V2**: O(n) where n = number of structure nodes
- **V2 with inheritance**: O(n × d) where d = tree depth (typically < 10)

For most projects, loading time is under 1ms.

#### Resolution Performance

- **Rule lookup**: O(log n) using specificity-sorted rules
- **Path matching**: O(m) where m = pattern complexity
- **Inheritance resolution**: Done once at config load, not during validation

#### Memory Usage

- **V1**: ~100 bytes per rule
- **V2**: ~150 bytes per node (includes inheritance metadata)
- **Typical project**: < 10KB for entire config

#### Validation Performance

Both V1 and V2 have the same runtime performance once loaded:
- **Parallel validation**: Uses all CPU cores
- **Incremental validation**: Only changed files re-checked
- **Large projects**: 10,000+ files in < 5 seconds

### CLI Migration Tool

Assura includes a migration helper:

```bash
# Dry run - see what would change
assura migrate --from v1 --to v2 --dry-run

# Perform migration
assura migrate --from v1 --to v2 --output .assura/config-v2.yml

# Validate migrated config
assura config validate --config .assura/config-v2.yml
```

### Validation Commands

```bash
# Validate entire project with V2 config
assura validate

# Validate specific files
assura validate src/main.rs src/lib.rs

# Watch mode for development
assura watch

# Check config validity
assura config validate

# Verbose output
assura validate --verbose
```

## Best Practices

1. **Use descriptive paths**: Clear path names make config self-documenting

2. **Leverage inheritance**: Define common settings at parent level

3. **Group related directories**: Use children instead of separate root nodes

4. **Document exceptions**: Add comments for `inherit: false` usage

5. **Keep exclusions minimal**: Only exclude truly generated/automated files

6. **Version your config**: Track changes to `.assura/config.yml` in git

7. **Test your config**: Run `assura config validate` after changes

8. **Use appropriate severity**: Critical for blockers, low for suggestions

## Advanced Topics

### Custom Regex Patterns

For special naming needs, use the `regex:` prefix:

```yaml
structure:
  src/:
    files:
      naming: "regex:^[a-z]+_[a-z_]+_v\d+$"  # e.g., api_client_v2
```

### Multiple File Types

Apply different rules to different extensions:

```yaml
structure:
  src/:
    files:
      naming: snake_case
      extensions:
        - "rs"
        - "toml"
  
  assets/:
    files:
      naming: kebab-case
      extensions:
        - "png"
        - "jpg"
        - "svg"
```

### Conditional Configs

Use environment variables in your config:

```yaml
version: "2.0"

structure:
  src/:
    files:
      max_lines: ${MAX_LINES:-500}
      severity: ${CI:+critical}  # Critical in CI, default otherwise
```

## API Reference

### Configuration Schema

For programmatic access to configuration:

```rust
use assura::config::v2::{StructureConfig, StructureConfigLoader};

// Load from file
let config = StructureConfigLoader::load(".assura/config.yml")?;

// Resolve rules for a specific path
let bundle = config.resolve_for_path(Path::new("src/main.rs"));

// Access bundle properties
if let Some(bundle) = bundle {
    println!("Naming: {:?}", bundle.naming);
    println!("Max lines: {:?}", bundle.max_lines);
}
```

### RuleResolver

```rust
use assura::config::v2::{RuleResolver, ResolvedRule};

let resolver = RuleResolver::new(&config);

// Get all resolved rules
let rules: Vec<ResolvedRule> = resolver.resolve();

// Find rule for specific file
let bundle = resolver.resolve_for_path(Path::new("src/main.rs"));
```

## Version History

- **v2.0** (Current): Initial V2 release with structure-first configuration
  - Hierarchical inheritance
  - FileValidationBundle
  - MarkdownValidationBundle
  - LS-Lint compatibility layer

## Support

For help with V2 configuration:

- Documentation: https://assura.dev/docs/config-v2
- Issues: https://github.com/assura/assura/issues
- Discussions: https://github.com/assura/assura/discussions
