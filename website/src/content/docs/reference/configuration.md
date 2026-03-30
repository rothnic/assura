---
title: Configuration Reference
description: Complete reference for Assura configuration files
template: doc
sidebar:
  order: 1
---

import { Tabs, TabItem, Aside, Steps } from '@astrojs/starlight/components';

This reference provides complete documentation for the `.assura/config.yml` format and all available configuration options.

## Configuration File Format

Assura supports YAML, JSON, and TOML configuration files. By default, Assura looks for configuration files in the following order:

1. `.assura/config.yml` (recommended)
2. `.assura/config.yaml`
3. `assura.yml`
4. `assura.yaml`
5. `assura.json`
6. `assura.toml`

### Format Examples

<Tabs>
<TabItem label="YAML (Recommended)">
```yaml
name: My Project
version: "1.0"

settings:
  parallel: true
  max_workers: 8

rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
```
</TabItem>
<TabItem label="JSON">
```json
{
  "name": "My Project",
  "version": "1.0",
  "settings": {
    "parallel": true,
    "max_workers": 8
  },
  "rules": [
    {
      "name": "file-naming",
      "severity": "high",
      "pattern": "^[a-z][a-z0-9_]*\\.rs$"
    }
  ]
}
```
</TabItem>
<TabItem label="TOML">
```toml
name = "My Project"
version = "1.0"

[settings]
parallel = true
max_workers = 8

[[rules]]
name = "file-naming"
severity = "high"
pattern = "^[a-z][a-z0-9_]*\\.rs$"
```
</TabItem>
</Tabs>

## Top-Level Fields

### name

**Type:** `string`  
**Required:** No  
**Default:** `null`

The name of your project. Used in reports and logging.

```yaml
name: My Awesome Project
```

### description

**Type:** `string`  
**Required:** No  
**Default:** `null`

A brief description of the project.

```yaml
description: A Rust library for data processing
```

### version

**Type:** `string`  
**Required:** No  
**Default:** `"1.0"`

The configuration schema version. Currently supports `"1.0"`.

```yaml
version: "1.0"
```

## Settings Section

The `settings` section contains global configuration options that apply to all validation operations.

```yaml
settings:
  parallel: true
  max_workers: 8
  cache_enabled: true
  cache_dir: ".assura-cache"
  watch_delay: 100
  fail_fast: false
```

### parallel

**Type:** `boolean`  
**Required:** No  
**Default:** `true`

Enable parallel validation using multiple worker threads. Disabling this forces sequential validation.

```yaml
settings:
  parallel: true  # Enable multi-threaded validation
```

### max_workers

**Type:** `integer`  
**Required:** No  
**Default:** CPU count

Maximum number of worker threads for parallel validation. Only applies when `parallel: true`.

```yaml
settings:
  max_workers: 8  # Use up to 8 threads
```

### cache_enabled

**Type:** `boolean`  
**Required:** No  
**Default:** `true`

Enable result caching to improve performance on subsequent validation runs.

```yaml
settings:
  cache_enabled: true
```

### cache_dir

**Type:** `string`  
**Required:** No  
**Default:** `.assura/cache`

Directory to store cache files.

```yaml
settings:
  cache_dir: ".assura/cache"
```

### watch_delay

**Type:** `integer`  
**Required:** No  
**Default:** `100`

Debounce delay in milliseconds for file watching. Prevents excessive re-validation when multiple files change rapidly.

```yaml
settings:
  watch_delay: 250  # Wait 250ms after last change
```

### fail_fast

**Type:** `boolean`  
**Required:** No  
**Default:** `false`

Stop validation on the first error encountered.

```yaml
settings:
  fail_fast: false  # Continue validating after errors
```

## File Patterns

Control which files are included or excluded from validation.

### includes

**Type:** `array<string>`  
**Required:** No  
**Default:** `["**/*"]`

Glob patterns for files to include in validation.

```yaml
includes:
  - "src/**/*.rs"
  - "tests/**/*.rs"
  - "Cargo.toml"
```

### excludes

**Type:** `array<string>`  
**Required:** No  
**Default:** `[]`

Glob patterns for files to exclude from validation.

```yaml
excludes:
  - "target/**/*"
  - "**/node_modules/**/*"
  - "**/*.gen.rs"
  - "**/*.min.js"
```

<Aside type="tip" title="Glob Pattern Syntax">
Assura supports standard glob patterns:
- `*` - Match any characters except `/`
- `**` - Match any characters including `/`
- `?` - Match a single character
- `[abc]` - Match any character in the set
- `{a,b,c}` - Match any of the alternatives
</Aside>

## Constraint Configuration

Rules are configured in the `rules` array. Each rule is an object with required and optional fields.

### Rule Structure

```yaml
rules:
  - name: rule-name
    severity: high
    enabled: true
    # Rule-specific options...
```

### Common Rule Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | - | Rule identifier |
| `severity` | string | Yes | - | Severity level: `critical`, `high`, `medium`, `low` |
| `enabled` | boolean | No | `true` | Whether the rule is active |
| `include` | array<string> | No | `[]` | Files to check (overrides global includes) |
| `exclude` | array<string> | No | `[]` | Files to skip |

### Severity Levels

```yaml
rules:
  - name: dependency-check
    severity: critical  # Must fix, blocks CI/CD
  
  - name: file-size
    severity: high      # Serious issue
  
  - name: documentation
    severity: medium    # Should review
  
  - name: line-length
    severity: low       # Minor suggestion
```

## Rule Syntax Reference

### file-naming

Validates file names match a regular expression pattern.

```yaml
rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
    message: "Files must use snake_case"
    case_sensitive: true
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `pattern` | string | (required) | Regular expression pattern |
| `message` | string | `"File name doesn't match pattern"` | Error message |
| `case_sensitive` | boolean | `true` | Case-sensitive matching |

### dependency-check

Analyzes import dependencies and detects circular references.

```yaml
rules:
  - name: dependency-check
    severity: critical
    check_circular: true
    max_depth: 10
    forbidden:
      - "deprecated_crate"
      - "old_module"
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `check_circular` | boolean | `true` | Detect circular dependencies |
| `max_depth` | integer | `10` | Maximum dependency depth |
| `forbidden` | array<string> | `[]` | Forbidden module/crate names |
| `allowed_cycles` | array<string> | `[]` | Allowed circular dependencies |
| `workspace_mode` | boolean | `false` | Check cross-crate dependencies |

### file-size

Enforces file size limits.

```yaml
rules:
  - name: file-size
    severity: medium
    max_size: "500KB"
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_size` | string | (required) | Maximum file size (e.g., "500KB", "1MB", "100B") |

Size suffixes: `B` (bytes), `KB` (kilobytes), `MB` (megabytes), `GB` (gigabytes)

### line-length

Validates maximum line length.

```yaml
rules:
  - name: line-length
    severity: low
    max_length: 100
    ignore_urls: true
    ignore_comments: false
    ignore_strings: false
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_length` | integer | (required) | Maximum characters per line |
| `ignore_urls` | boolean | `false` | Don't count long URLs |
| `ignore_comments` | boolean | `false` | Don't count comment lines |
| `ignore_strings` | boolean | `false` | Don't count long strings |

### documentation

Checks for required documentation.

```yaml
rules:
  - name: documentation
    severity: medium
    require_public: true
    require_module: true
    require_traits: true
    min_description_length: 10
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `require_public` | boolean | `false` | Public items must be documented |
| `require_module` | boolean | `false` | Modules must have documentation |
| `require_traits` | boolean | `false` | Traits must be documented |
| `min_description_length` | integer | `0` | Minimum characters in description |

### import-order

Validates import statement organization.

```yaml
rules:
  - name: import-order
    severity: low
    groups:
      - "std"
      - "external"
      - "crate"
      - "super"
      - "self"
    alphabetical: true
    newline_between_groups: true
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `groups` | array<string> | `["std", "external", "crate", "super", "self"]` | Import group order |
| `alphabetical` | boolean | `true` | Sort imports alphabetically within groups |
| `newline_between_groups` | boolean | `true` | Require blank line between groups |

## Maturity Override Options

Override rule behavior based on project maturity or other conditions.

### maturity_overrides

Temporarily relax rules during early development phases.

```yaml
maturity_overrides:
  # Allow TODOs during development
  - rule: todo-detection
    phases:
      - "alpha"
      - "beta"
    severity: low
  
  # Relax documentation requirements early on
  - rule: documentation
    phases:
      - "alpha"
    enabled: false
```

### per_file_overrides

Configure rules differently for specific files or directories.

```yaml
per_file_overrides:
  # Generated files have different rules
  - path: "src/generated/**/*"
    rules:
      - name: documentation
        enabled: false
      - name: line-length
        enabled: false
  
  # Tests can have longer lines
  - path: "tests/**/*.rs"
    rules:
      - name: line-length
        max_length: 120
```

## Environment Variables

Reference environment variables in your configuration:

```yaml
settings:
  project_root: ${PROJECT_ROOT}
  api_key: ${API_KEY}
  max_workers: ${MAX_WORKERS:-4}  # Default to 4 if not set
```

Environment variable syntax:
- `${VAR}` - Replace with variable value
- `${VAR:-default}` - Use default if variable not set
- `${VAR:=default}` - Use default and set variable if not set

## Complete Configuration Example

```yaml
# Project metadata
name: My Rust Project
description: A comprehensive Rust application
version: "1.0"

# Global settings
settings:
  parallel: true
  max_workers: 8
  cache_enabled: true
  cache_dir: ".assura/cache"
  watch_delay: 100
  fail_fast: false

# File patterns
includes:
  - "src/**/*.rs"
  - "tests/**/*.rs"
  - "benches/**/*.rs"
  - "Cargo.toml"
  - "Cargo.lock"

excludes:
  - "target/**/*"
  - "**/node_modules/**/*"
  - "**/*.gen.rs"
  - "src/generated/**/*"

# Validation rules
rules:
  # File naming convention
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
    message: "Rust files must use snake_case"
  
  # Detect circular dependencies
  - name: dependency-check
    severity: critical
    check_circular: true
    max_depth: 10
  
  # File size limits
  - name: file-size
    severity: medium
    max_size: "500KB"
  
  # Line length limit
  - name: line-length
    severity: low
    max_length: 100
    ignore_urls: true
    ignore_comments: false
  
  # Documentation requirements
  - name: documentation
    severity: medium
    require_public: true
    require_module: true
    min_description_length: 10
  
  # Import organization
  - name: import-order
    severity: low
    groups:
      - "std"
      - "external"
      - "crate"
      - "super"
      - "self"
    alphabetical: true

# Per-file overrides
per_file_overrides:
  - path: "tests/**/*.rs"
    rules:
      - name: line-length
        max_length: 120
      - name: documentation
        enabled: false
  
  - path: "src/generated/**/*"
    rules:
      - name: file-naming
        enabled: false
      - name: documentation
        enabled: false
```

## Configuration Validation

Validate your configuration file:

```bash
assura config validate
```

Check for errors and warnings in your configuration:

```bash
assura config validate --strict
```

## Best Practices

<Steps>

1. **Use `.assura/config.yml`**

   Keep configuration in a dedicated directory for better organization.

2. **Start with high severity for critical rules**

   Set circular dependency detection and security rules to `critical`.

3. **Use appropriate exclusions**

   Exclude generated files, vendor directories, and build artifacts.

4. **Configure per-file overrides**

   Different rules for tests, generated code, and examples.

5. **Enable caching**

   Leave `cache_enabled: true` for better performance.

6. **Use environment variables**

   Reference environment variables for CI/CD-specific values.

</Steps>

<Aside type="caution" title="Breaking Changes">
  Configuration schema may change in future versions. Check the changelog when upgrading.
</Aside>
