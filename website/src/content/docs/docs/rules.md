---
title: Rules
description: Available validation rules in Assura
---

import { Aside, Badge } from '@astrojs/starlight/components';

Assura provides a comprehensive set of validation rules that can be configured to enforce project standards.

## Severity Levels

Each rule has a severity level that determines how violations are treated:

- <Badge text="Critical" variant="danger" /> - Validation fails, must be fixed
- <Badge text="High" variant="caution" /> - Serious issues, strongly recommended to fix
- <Badge text="Medium" variant="note" /> - Potential problems, should be reviewed
- <Badge text="Low" variant="success" /> - Minor suggestions

## Built-in Rules

### file-naming

Validates that file names follow a specified pattern.

```yaml
rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"  # Only lowercase with underscores
    message: "File names must be lowercase with underscores"
```

### dependency-check

Analyzes and validates project dependencies.

```yaml
rules:
  - name: dependency-check
    severity: critical
    check_circular: true      # Detect circular dependencies
    max_depth: 10             # Maximum dependency depth
    forbidden:
      - "deprecated_crate"
```

### file-size

Enforces file size limits.

```yaml
rules:
  - name: file-size
    severity: medium
    max_size: "500KB"
    include:
      - "src/**/*.rs"
```

### line-length

Validates maximum line length.

```yaml
rules:
  - name: line-length
    severity: low
    max_length: 100
    ignore_urls: true
    ignore_comments: false
```

### documentation

Checks for required documentation.

```yaml
rules:
  - name: documentation
    severity: medium
    require_public: true      # Public items must be documented
    require_module: true      # Modules must have documentation
```

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
```

## Custom Rules

<Aside type="tip" title="Extending Assura">
  You can create custom rules by implementing the `ValidationRule` trait. This will be documented in detail in Phase 8B.
</Aside>

## Rule Configuration Examples

### Web Development Project

```yaml
rules:
  - name: file-naming
    severity: medium
    pattern: "^[a-z0-9-]+\\.(js|ts|tsx|css|scss)$"
    
  - name: file-size
    severity: high
    max_size: "1MB"
    exclude:
      - "**/assets/**/*"
      
  - name: dependency-check
    severity: critical
    check_circular: true
    max_depth: 15
```

### Rust Library Project

```yaml
rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
    
  - name: documentation
    severity: medium
    require_public: true
    require_module: true
    
  - name: dependency-check
    severity: high
    check_circular: true
```
