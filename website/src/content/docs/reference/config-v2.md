---
title: V2 Configuration Reference
description: Complete reference for Assura V2 structure-first configuration
template: doc
sidebar:
  order: 2
---

import { Tabs, TabItem, Aside, Steps } from '@astrojs/starlight/components';

This reference provides complete documentation for the V2 configuration format with structure-first hierarchical configuration.

## Overview

V2 configuration uses a **structure-first** approach where you define your project hierarchy and apply validation bundles to each node. This approach is more intuitive, maintainable, and powerful than V1's flat rule-based configuration.

## Configuration File Format

### Supported Formats

V2 configuration supports YAML, JSON, and TOML:

<Tabs>
<TabItem label="YAML (Recommended)">
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
```
</TabItem>
<TabItem label="JSON">
```json
{
  "version": "2.0",
  "structure": {
    "src/": {
      "files": {
        "naming": "snake_case",
        "max_lines": 500
      }
    }
  }
}
```
</TabItem>
<TabItem label="TOML">
```toml
version = "2.0"

[structure]

[structure."src/"]

[structure."src/".files]
naming = "snake_case"
max_lines = 500
```
</TabItem>
</Tabs>

## Top-Level Fields

### version

**Type:** `string`  
**Required:** Yes  
**Pattern:** `"2.x"`

Identifies this as a V2 configuration. Must be exactly `"2.0"`.

```yaml
version: "2.0"
```

### project

**Type:** `object`  
**Required:** No

Project metadata for reports and documentation.

```yaml
project:
  name: "My Project"
  description: "A brief description"
  maturity: stable  # Options: alpha, beta, stable
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `description` | string | Project description |
| `maturity` | string | Project phase: `alpha`, `beta`, or `stable` |

### structure

**Type:** `object<string, StructureNode>`  
**Required:** Yes

The hierarchical structure definition. Each key is a directory path, each value is a `StructureNode`.

```yaml
structure:
  src/:
    files:
      naming: snake_case
    children:
      components/:
        files:
          naming: PascalCase
```

### ls

**Type:** `LsLintCompatibility`  
**Required:** No

LS-Lint compatibility layer for migrating from LS-Lint configurations.

```yaml
ls:
  .rs: snake_case
  .ts: camelCase
  src/:
    .rs: PascalCase
```

### exclude

**Type:** `array<string>`  
**Required:** No  
**Default:** `[]`

Global patterns to exclude from validation.

```yaml
exclude:
  - "target/**"
  - ".git/**"
  - "node_modules/**"
```

## StructureNode

Each node in the structure hierarchy.

```yaml
structure:
  src/:
    # File validation bundle
    files:
      naming: snake_case
      max_lines: 500
    
    # Markdown validation bundle
    markdown:
      require_frontmatter: true
    
    # Child directories
    children:
      components/:
        files:
          naming: PascalCase
    
    # Inheritance control
    inherit: true
```

### files

**Type:** `FileValidationBundle`  
**Required:** No

File-level validation rules for this node.

```yaml
files:
  naming: snake_case
  max_lines: 500
  max_size: "1MB"
  require_docs: true
  extensions: ["rs", "md"]
  severity: high
```

### markdown

**Type:** `MarkdownValidationBundle`  
**Required:** No

Markdown-specific validation rules for this node.

```yaml
markdown:
  require_frontmatter: true
  required_fields: ["title", "description"]
  max_heading_depth: 4
  check_links: true
  required_sections: ["Introduction"]
```

### children

**Type:** `object<string, StructureNode>`  
**Required:** No

Nested structure nodes for subdirectories.

```yaml
children:
  components/:
    files:
      naming: PascalCase
  utils/:
    files:
      naming: snake_case
```

### inherit

**Type:** `boolean`  
**Required:** No  
**Default:** `true`

Whether to inherit settings from parent nodes.

```yaml
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
    
    children:
      # Inherits naming and max_lines from src/
      lib/:
        inherit: true
      
      # Does not inherit - clean slate
      vendor/:
        inherit: false
        files:
          naming: regex:.*
```

## FileValidationBundle

Bundle of file-level validation rules.

### naming

**Type:** `string`  
**Required:** No

Naming convention for files in this node.

**Options:**

| Convention | Example | Description |
|------------|---------|-------------|
| `snake_case` | `my_file_name` | Lowercase with underscores |
| `kebab-case` | `my-file-name` | Lowercase with hyphens |
| `camelCase` | `myFileName` | Lower camel case |
| `PascalCase` | `MyFileName` | Upper camel case |
| `SCREAMING_SNAKE_CASE` | `MY_FILE_NAME` | Uppercase with underscores |
| `dot.case` | `my.file.name` | Lowercase with dots |
| `flatcase` | `myfilename` | All lowercase, no separators |
| `FLATCASE` | `MYFILENAME` | All uppercase, no separators |
| `COBOL-CASE` | `MY-FILE-NAME` | Uppercase with hyphens |
| `Train-Case` | `My-File-Name` | Capitalized with hyphens |
| `lowercase` | `myfilename` | All lowercase (alias) |
| `UPPERCASE` | `MYFILENAME` | All uppercase (alias) |
| `regex:PATTERN` | - | Custom regex pattern |

```yaml
files:
  naming: snake_case
  # Or use custom regex
  naming: "regex:^[a-z]+_[a-z_]+_v\\d+$"
```

### max_lines

**Type:** `integer`  
**Required:** No  
**Range:** 1-100000

Maximum number of lines allowed per file.

```yaml
files:
  max_lines: 500
```

### max_size

**Type:** `string`  
**Required:** No  
**Format:** `"<number><unit>"`

Maximum file size allowed.

**Units:**
- `B` - Bytes
- `KB` - Kilobytes
- `MB` - Megabytes
- `GB` - Gigabytes
- `TB` - Terabytes

```yaml
files:
  max_size: "500KB"
  max_size: "10 MB"  # Space allowed
```

### require_docs

**Type:** `boolean`  
**Required:** No  
**Default:** `false`

Whether files must have documentation.

- **Rust**: Requires rustdoc on public items
- **Other languages**: Requires module-level documentation

```yaml
files:
  require_docs: true
```

### extensions

**Type:** `array<string>`  
**Required:** No

Only validate files with these extensions. If omitted, all files are validated.

```yaml
files:
  extensions:
    - "rs"
    - "md"
    - "toml"
```

### severity

**Type:** `string`  
**Required:** No  
**Default:** `high`

Default severity level for violations in this node.

**Options:**
- `critical` - Must fix immediately, blocks CI/CD
- `high` - Serious issue, should fix soon
- `medium` - Should review
- `low` - Optional suggestion

```yaml
files:
  severity: critical
```

## MarkdownValidationBundle

Bundle of Markdown-specific validation rules.

### require_frontmatter

**Type:** `boolean`  
**Required:** No  
**Default:** `false`

Whether Markdown files must have YAML frontmatter.

```yaml
markdown:
  require_frontmatter: true
```

### required_fields

**Type:** `array<string>`  
**Required:** No

Required frontmatter fields. Only checked if `require_frontmatter` is true or implied.

```yaml
markdown:
  require_frontmatter: true
  required_fields:
    - "title"
    - "description"
    - "date"
```

### max_heading_depth

**Type:** `integer`  
**Required:** No  
**Range:** 1-6

Maximum allowed heading depth (prevents excessive nesting).

```yaml
markdown:
  max_heading_depth: 4
```

### check_links

**Type:** `boolean`  
**Required:** No  
**Default:** `false`

Whether to check for dead links in Markdown files.

```yaml
markdown:
  check_links: true
```

### required_sections

**Type:** `array<string>`  
**Required:** No

Required section headings that must appear in each file.

```yaml
markdown:
  required_sections:
    - "Introduction"
    - "Examples"
    - "See Also"
```

## LS-Lint Compatibility

Convert LS-Lint configurations to V2:

```yaml
version: "2.0"

ls:
  .rs: snake_case
  .ts: camelCase
  .tsx: PascalCase
  src/:
    .rs: PascalCase
  tests/:
    .rs: snake_case
```

### Supported LS-Lint Rules

All LS-Lint naming conventions are supported:

- `kebab-case` → `kebab-case`
- `snake_case` → `snake_case`
- `camelCase` → `camelCase`
- `PascalCase` → `PascalCase`
- `SCREAMING_SNAKE_CASE` → `SCREAMING_SNAKE_CASE`
- `dot.case` → `dot.case`
- `flatcase` → `flatcase`
- `FLATCASE` → `FLATCASE`
- `COBOL-CASE` → `COBOL-CASE`
- `Train-Case` → `Train-Case`
- `lowercase` → `lowercase`
- `UPPERCASE` → `UPPERCASE`
- `regex:PATTERN` → `regex:PATTERN`

## Hierarchical Inheritance

### How Inheritance Works

By default, child nodes inherit all non-specified settings from their parent:

```yaml
structure:
  src/:
    files:
      naming: snake_case      # Parent value
      max_lines: 500          # Parent value
      severity: high          # Parent value
    
    children:
      components/:
        files:
          naming: PascalCase  # Override parent
        # max_lines: 500 inherited
        # severity: high inherited
```

### Inheritance Rules

1. **Explicit values override inherited values**
2. **Missing values are inherited from parent**
3. **`inherit: false` creates a clean slate**
4. **Specificity increases with depth**

### Specificity Scoring

When multiple rules could apply, specificity determines precedence:

- Base score: `depth × 10`
- Exact paths: `+5`
- Path length bonus

**Example order (most specific first):**

1. `src/components/Button.tsx` - Exact match
2. `src/components/**/*.tsx` - Glob pattern
3. `src/components/` - Directory prefix
4. `src/` - Parent directory

```yaml
structure:
  src/:
    files:
      naming: snake_case        # Specificity: ~10
    
    children:
      components/:
        files:
          naming: PascalCase    # Specificity: ~25
```

## Complete Examples

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

exclude:
  - "target/**"
  - ".git/**"
```

### Example 2: Full-Stack TypeScript Project

```yaml
version: "2.0"

structure:
  server/:
    files:
      naming: camelCase
      max_lines: 400
    
    children:
      src/:
        files:
          naming: camelCase
        
        children:
          models/:
            files:
              naming: PascalCase
          
          routes/:
            files:
              naming: camelCase
  
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
              naming: PascalCase
              extensions: ["tsx", "css"]
          
          pages/:
            files:
              naming: kebab-case
              extensions: ["tsx"]
  
  docs/:
    markdown:
      require_frontmatter: true
      required_fields: ["title", "description"]
      check_links: true

exclude:
  - "node_modules/**"
  - "dist/**"
  - ".next/**"
```

### Example 3: Documentation-Heavy Project

```yaml
version: "2.0"

project:
  name: "documentation-site"
  maturity: stable

structure:
  api-docs/:
    markdown:
      require_frontmatter: true
      required_fields: ["title", "version", "api_level"]
      required_sections: ["Endpoints", "Parameters", "Response"]
      max_heading_depth: 4
      check_links: true
  
  guides/:
    markdown:
      require_frontmatter: true
      required_fields: ["title", "description"]
      required_sections: ["Introduction", "Prerequisites", "Steps"]
      max_heading_depth: 3
  
  blog/:
    markdown:
      require_frontmatter: true
      required_fields: ["title", "author", "date"]
      max_heading_depth: 2
  
  src/:
    files:
      naming: snake_case
      max_lines: 300
      require_docs: true

exclude:
  - "_site/**"
  - "node_modules/**"
```

## Environment Variables

Use environment variables for dynamic configuration:

```yaml
version: "2.0"

structure:
  src/:
    files:
      max_lines: ${MAX_LINES:-500}
      severity: ${CI:+critical}
```

**Syntax:**

- `${VAR}` - Variable value
- `${VAR:-default}` - Default if not set
- `${VAR:=default}` - Default and set if not set

## Configuration Validation

Validate your V2 configuration:

```bash
# Validate current config
assura config validate

# Validate specific file
assura config validate --config .assura/config.yml

# Strict mode (fail on warnings)
assura config validate --strict
```

## Migration from V1

Use the migration tool to convert V1 configs:

```bash
# Dry run
assura migrate --from v1 --to v2 --dry-run

# Perform migration
assura migrate --from v1 --to v2 --output .assura/config-v2.yml
```

See the [Migration Guide](/guides/migration/) for detailed instructions.

## Troubleshooting

### Config Not Detected as V2

**Problem:** Assura treats config as V1

**Solution:** Ensure `version: "2.0"` is at the top:

```yaml
version: "2.0"  # Must be first and exact
structure:
  # ...
```

### Inheritance Not Working

**Problem:** Child not inheriting from parent

**Solution:**
- Check parent ends with `/`
- Ensure child is in `children:`
- Verify `inherit: true`

```yaml
structure:
  src/:  # Trailing slash required
    files:
      naming: snake_case
    
    children:  # Must use children
      lib/:
        inherit: true  # Or omit (default)
```

### Patterns Not Matching

**Problem:** Files not being validated

**Solution:** Use trailing slashes for directories:

```yaml
# Correct
structure:
  src/:
    files:
      naming: snake_case

# Incorrect
structure:
  src:  # Missing /
    files:
      naming: snake_case
```

## Best Practices

<Steps>

1. **Start with parent rules**

   Define common settings at the highest level:

   ```yaml
   structure:
     src/:
       files:
         naming: snake_case
         max_lines: 500
         severity: high
   ```

2. **Override only what's different**

   Children should only specify changed values:

   ```yaml
   children:
     components/:
       files:
         naming: PascalCase  # Only override naming
   ```

3. **Document exceptions**

   Add comments for unusual configurations:

   ```yaml
   children:
     # Third-party code - minimal validation
     vendor/:
       inherit: false
       files:
         naming: regex:.*
   ```

4. **Use appropriate severity**

   Match severity to importance:
   - `critical` - Blocks CI/CD
   - `high` - Must fix soon
   - `medium` - Should review
   - `low` - Optional

5. **Validate frequently**

   Run `assura config validate` after changes:

   ```bash
   assura config validate
   ```

6. **Keep exclusions minimal**

   Only exclude truly necessary paths:

   ```yaml
   exclude:
     - "target/**"
     - ".git/**"
     - "node_modules/**"
   ```

</Steps>

## API Reference

### Programmatic Usage

```rust
use assura::config::v2::{StructureConfig, StructureConfigLoader};

// Load config
let config = StructureConfigLoader::load(".assura/config.yml")?;

// Resolve for specific path
let bundle = config.resolve_for_path(Path::new("src/main.rs"));

// Access properties
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

// Resolve specific path
let bundle = resolver.resolve_for_path(Path::new("src/main.rs"));
```

## Related Documentation

- [Configuration Overview](/docs/configuration/)
- [Migration Guide](/guides/migration/)
- [Examples](/examples/basic-setup/)
- [V1 Reference (Legacy)](/reference/configuration-v1/)
