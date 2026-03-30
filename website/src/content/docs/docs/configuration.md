---
title: Configuration
description: Learn how to configure Assura for your project
template: doc
sidebar:
  order: 1
---

import { Tabs, TabItem, Aside, Steps } from '@astrojs/starlight/components';

Assura supports two configuration formats:

- **Version 2 (V2)** ✨ **Recommended**: Structure-first hierarchical configuration
- **Version 1 (V1)** Legacy: Rule-based flat configuration

<Aside type="tip" title="New to Assura?">
  We recommend starting with **V2 configuration**. It's more intuitive, maintainable, and powerful.
</Aside>

## Configuration File Locations

Assura looks for configuration files in the following order:

1. `.assura/config.yml` (recommended)
2. `.assura/config.yaml`
3. `assura.yml`
4. `assura.yaml`
5. `assura.json`
6. `assura.toml`

## Version 2 Configuration (Recommended)

V2 uses a **structure-first** approach where your configuration mirrors your project directory layout.

### Basic V2 Example

```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
    
    children:
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      naming: snake_case
      max_lines: 1000

exclude:
  - "target/**"
  - ".git/**"
```

### Key V2 Concepts

<Steps>

1. **Structure Mirrors Directories**
   
   Each key under `structure:` represents a directory path ending with `/`.

2. **Hierarchical Inheritance**
   
   Child directories automatically inherit parent settings. Use `inherit: false` to disable.

3. **Validation Bundles**
   
   Use `files:` for file-level validations and `markdown:` for Markdown-specific rules.

4. **Specificity Wins**
   
   More specific paths take precedence over general ones.

</Steps>

### V2 Configuration Sections

#### version

**Type:** `string`  
**Required:** Yes for V2

Must be exactly `"2.0"` to enable V2 features.

```yaml
version: "2.0"
```

#### structure

**Type:** `object`  
**Required:** Yes

Hierarchical structure definition where each key is a directory path.

```yaml
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
```

#### FileValidationBundle (files)

Configure file-level validations:

```yaml
files:
  naming: "snake_case"           # Naming convention
  max_lines: 500                 # Maximum lines per file
  max_size: "500KB"              # Maximum file size
  require_docs: true             # Require documentation
  extensions: ["rs", "md"]       # Allowed extensions
  severity: "high"               # Default severity
```

**Naming Conventions:**

- `snake_case` - lowercase_with_underscores
- `kebab-case` - lowercase-with-hyphens
- `camelCase` - camelCase
- `PascalCase` - PascalCase
- `SCREAMING_SNAKE_CASE` - UPPERCASE_WITH_UNDERSCORES
- `dot.case` - lowercase.with.dots
- `flatcase` - lowercase
- `FLATCASE` - UPPERCASE
- `COBOL-CASE` - UPPERCASE-WITH-HYPHENS
- `Train-Case` - Capitalized-With-Hyphens
- `lowercase` - alllowercase
- `UPPERCASE` - ALLUPPERCASE
- `regex:pattern` - Custom regex pattern

#### MarkdownValidationBundle (markdown)

Configure Markdown-specific validations:

```yaml
markdown:
  require_frontmatter: true
  required_fields: ["title", "description"]
  max_heading_depth: 4
  check_links: true
  required_sections: ["Introduction", "Examples"]
```

#### children

Nested structure nodes for subdirectories:

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

#### inherit

Control inheritance from parent nodes:

```yaml
structure:
  src/:
    files:
      naming: snake_case
    
    children:
      third_party/:
        inherit: false  # Don't inherit from src/
        files:
          naming: regex:.*
```

#### exclude

Global exclusion patterns:

```yaml
exclude:
  - "target/**"
  - ".git/**"
  - "node_modules/**"
```

### Complete V2 Example

```yaml
version: "2.0"

project:
  name: "My Rust Project"
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
      
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      naming: snake_case
      max_lines: 1000
      severity: medium
  
  docs/:
    markdown:
      require_frontmatter: true
      required_fields: ["title", "description"]
      check_links: true

exclude:
  - "target/**"
  - ".git/**"
  - "Cargo.lock"
```

## Version 1 Configuration (Legacy)

V1 uses a flat array of rules with `applies_to` patterns. While still supported, we recommend migrating to V2.

<Aside type="caution" title="Legacy Format">
  V1 configuration is maintained for backwards compatibility. New projects should use V2.
</Aside>

### Basic V1 Example

```yaml
version: "1.0"

rules:
  - name: file-naming
    applies_to: "src/**/*.rs"
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
```

### V1 Configuration Sections

#### name

**Type:** `string`  
**Required:** No

The name of your project.

#### version

**Type:** `string`  
**Required:** No  
**Default:** `"1.0"`

Configuration schema version. Use `"1.0"` for V1 configs.

#### settings

Global validation settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `parallel` | boolean | `true` | Enable parallel validation |
| `max_workers` | integer | CPU count | Maximum worker threads |
| `cache_enabled` | boolean | `true` | Enable result caching |
| `watch_delay` | integer | 100 | Debounce delay in milliseconds |

#### rules

Array of validation rules:

```yaml
rules:
  - name: rule-name
    severity: high
    enabled: true
    applies_to: "src/**/*.rs"
    # Rule-specific options...
```

**Common fields:**

- **name**: Rule identifier (required)
- **severity**: One of `critical`, `high`, `medium`, `low` (required)
- **enabled**: Whether rule is active (default: true)
- **applies_to**: File pattern (optional)

#### includes/excludes

File patterns for inclusion/exclusion:

```yaml
includes:
  - "src/**/*.rs"
  - "tests/**/*.rs"

excludes:
  - "target/**/*"
  - "**/node_modules/**/*"
```

## Migrating from V1 to V2

Use the built-in migration tool:

```bash
# Dry run to preview changes
assura migrate --from v1 --to v2 --dry-run

# Perform migration
assura migrate --from v1 --to v2 --output .assura/config-v2.yml

# Validate migrated config
assura config validate --config .assura/config-v2.yml
```

See the [Migration Guide](/guides/migration/) for detailed instructions.

## Environment Variables

Reference environment variables in your configuration:

```yaml
# V2 format
version: "2.0"

structure:
  src/:
    files:
      max_lines: ${MAX_LINES:-500}
      severity: ${CI:+critical}

# V1 format
settings:
  project_root: ${PROJECT_ROOT}
  max_workers: ${MAX_WORKERS:-4}
```

**Syntax:**
- `${VAR}` - Replace with variable value
- `${VAR:-default}` - Use default if variable not set
- `${VAR:=default}` - Use default and set variable if not set

## Configuration Validation

Validate your configuration file:

```bash
# Auto-detect version and validate
assura config validate

# Validate specific file
assura config validate --config .assura/config-v2.yml

# Strict validation (fail on warnings)
assura config validate --strict
```

## Best Practices

<Steps>

1. **Use V2 for new projects**

   Start with `version: "2.0"` and structure-first configuration.

2. **Leverage inheritance**

   Define common settings at parent levels to reduce duplication.

3. **Keep paths specific**

   Use clear, descriptive path names that mirror your project structure.

4. **Use appropriate severity levels**

   - `critical`: Blocks CI/CD, must fix immediately
   - `high`: Serious issues, should fix soon
   - `medium`: Should review
   - `low`: Optional suggestions

5. **Document exceptions**

   Add comments explaining why certain directories have different rules:

   ```yaml
   structure:
     src/:
       children:
         # Third-party code - minimal validation
         vendor/:
           inherit: false
           files:
             naming: regex:.*
   ```

6. **Exclude wisely**

   Only exclude truly generated or external files:

   ```yaml
   exclude:
     - "target/**"
     - ".git/**"
     - "node_modules/**"
   ```

7. **Validate after changes**

   Always run `assura config validate` after editing your configuration.

</Steps>

## Format Comparison

| Feature | V1 | V2 |
|---------|-----|-----|
| **Structure** | Flat rules array | Hierarchical tree |
| **Inheritance** | Manual duplication | Automatic |
| **Visual clarity** | Pattern-based | Directory-based |
| **Learning curve** | Moderate | Easy |
| **Maintainability** | Harder | Easier |
| **Performance** | Good | Better |
| **LS-Lint compat** | Partial | Full |

## Troubleshooting

### Config Not Recognized

**Problem:** V2 config treated as V1

**Solution:** Ensure `version: "2.0"` is at the top:

```yaml
version: "2.0"  # Must be first
structure:
  # ...
```

### Inheritance Not Working

**Problem:** Child settings not inheriting

**Solution:** Check that:
- Parent path ends with `/`
- Child is under `children:`
- `inherit: false` is not set

```yaml
structure:
  src/:  # Note trailing slash
    files:
      naming: snake_case
    
    children:  # Must use children key
      components/:
        inherit: true  # Or omit (default)
```

### Rules Not Matching

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
  src:  # Missing trailing slash
    files:
      naming: snake_case
```

## Related Documentation

- [V2 Configuration Reference](/reference/config-v2/)
- [Migration Guide](/guides/migration/)
- [Examples](/examples/basic-setup/)
- [API Reference](/reference/api/)
