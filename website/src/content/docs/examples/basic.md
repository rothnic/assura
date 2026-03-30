---
title: Basic Usage
description: Basic usage examples for Assura
---

Learn the fundamentals of using Assura through practical examples.

## Validating a Single File

```bash
assura validate src/main.rs
```

## Validating Multiple Files

```bash
assura validate src/*.rs tests/*.rs
```

## Using a Custom Config

```bash
assura validate --config /path/to/config.yaml
```

## Filtering by Severity

Only report issues of a certain severity or higher:

```bash
assura validate --severity high
```

## Output Formats

Assura supports multiple output formats:

### JSON Output

```bash
assura validate --format json
```

Example output:

```json
{
  "results": [
    {
      "rule": "file-naming",
      "severity": "high",
      "file": "src/BadName.rs",
      "line": 1,
      "message": "File name doesn't match pattern: ^[a-z][a-z0-9_]*\\.rs$",
      "suggestion": "Rename to 'bad_name.rs'"
    }
  ],
  "summary": {
    "total": 1,
    "critical": 0,
    "high": 1,
    "medium": 0,
    "low": 0
  }
}
```

### Check Mode (Exit Codes)

Useful for CI/CD pipelines:

```bash
assura validate --format check
# Exit code: 0 = no issues, 1 = issues found
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Assura
        run: cargo install assura
        
      - name: Run validation
        run: assura validate --format check
```

### GitLab CI

```yaml
validate:
  image: rust:latest
  before_script:
    - cargo install assura
  script:
    - assura validate --format check
```

## Pre-commit Hook

Add to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: assura
        name: Assura Validation
        entry: assura validate --format check
        language: system
        pass_filenames: false
```

## Batch Validation

Validate multiple projects at once:

```bash
#!/bin/bash
for dir in */; do
    echo "Validating $dir..."
    (cd "$dir" && assura validate)
done
```
