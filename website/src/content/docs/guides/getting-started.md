---
title: Getting Started
description: Complete guide to installing and using Assura for the first time
template: doc
sidebar:
  order: 1
---

import { Steps, Tabs, TabItem, Aside, FileTree } from '@astrojs/starlight/components';

Welcome to Assura! This guide will walk you through everything you need to know to start using Assura for dependency-aware file system validation.

## What is Assura?

Assura is a dependency-aware file system validation engine written in Rust. It provides:

- **Dependency graph analysis** for detecting circular dependencies and computing validation ordering
- **Rule-based validation** with configurable severity levels (Critical, High, Medium, Low)
- **File system watching** for continuous validation during development
- **Parallel execution** for large-scale validation performance
- **Extensible plugin architecture** for custom validators

## Installation

### Prerequisites

- **Rust**: Version 1.70.0 or later
- **Operating System**: Linux, macOS, or Windows

### Install from Crates.io

The easiest way to install Assura is using Cargo:

```bash
cargo install assura
```

### Install from Source

For the latest development version:

```bash
git clone https://github.com/anomalyco/assura
cd assura
cargo build --release
```

The binary will be available at `target/release/assura`.

### Verify Installation

Check that Assura is installed correctly:

```bash
assura --version
```

### Shell Completions

Enable tab completion for your shell:

<Tabs>
<TabItem label="Bash">
```bash
assura completions bash > /usr/share/bash-completion/completions/assura
```
</TabItem>
<TabItem label="Zsh">
```bash
assura completions zsh > /usr/share/zsh/site-functions/_assura
```
</TabItem>
<TabItem label="Fish">
```bash
assura completions fish > ~/.config/fish/completions/assura.fish
```
</TabItem>
</Tabs>

## Quick Start Tutorial

<Steps>

1. **Create a new project directory**

   ```bash
   mkdir my-project
   cd my-project
   ```

2. **Initialize Assura configuration (V2 - Recommended)**

   Create an `.assura/config.yml` file:

   ```yaml
   # .assura/config.yml - V2 Structure-first configuration
   version: "2.0"
   
   structure:
     src/:
       files:
         naming: snake_case
         max_lines: 500
   ```

   Or use the legacy V1 format:

   ```yaml
   # .assura/config.yml - V1 Legacy configuration
   name: My Project
   version: "1.0"
   
   rules:
     - name: file-naming
       severity: high
       pattern: "^[a-z][a-z0-9_]*\\.(rs|toml|md)$"
       message: "Files must use lowercase with underscores"
   ```

2. **Initialize Assura configuration**

   Create an `.assura/config.yml` file:

   ```yaml
   # .assura/config.yml
   name: My Project
   version: "1.0"
   
   rules:
     - name: file-naming
       severity: high
       pattern: "^[a-z][a-z0-9_]*\\.(rs|toml|md)$"
       message: "Files must use lowercase with underscores"
   
     - name: dependency-check
       severity: critical
       check_circular: true
   ```

3. **Create some test files**

   ```bash
   touch main.rs
   touch README.md
   ```

4. **Run your first validation**

   ```bash
   assura validate
   ```

   You should see output similar to:
   
   ```
   [INFO] Loading configuration from .assura/config.yml
   [INFO] Validating 2 files...
   [SUCCESS] All validations passed
   ```

5. **Test with an invalid file**

   Create a file that violates the naming rule:

   ```bash
   touch BadFileName.rs
   ```

   Run validation again:

   ```bash
   assura validate
   ```

   You'll see:
   
   ```
   [HIGH] BadFileName.rs: File name doesn't match pattern: ^[a-z][a-z0-9_]*\.(rs|toml|md)$
   ```

</Steps>

## Basic Configuration

### Configuration File Location

Assura looks for configuration in the following locations (in order):

1. `.assura/config.yml` (recommended)
2. `.assura/config.yaml`
3. `assura.yml`
4. `assura.yaml`
5. `assura.json`
6. `assura.toml`

<Aside type="tip" title="Best Practice">
  We recommend using `.assura/config.yml` for better organization and to keep your project root clean.
</Aside>

### File Structure

<FileTree>
- my-project/
  - .assura/
    - config.yml
  - src/
    - main.rs
  - Cargo.toml
  - README.md
</FileTree>

### Basic Configuration (V2 - Recommended)

Assura V2 uses a structure-first approach that mirrors your project directory layout:

```yaml
# V2 Configuration - Structure-first
version: "2.0"

project:
  name: My Awesome Project
  description: A brief description of your project

structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
      require_docs: true
      severity: high
    
    children:
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      naming: snake_case
      max_lines: 1000
      severity: medium

exclude:
  - "target/**/*"
  - "**/node_modules/**/*"
```

### Legacy V1 Configuration

For backwards compatibility, V1 flat rule-based configuration is also supported:

```yaml
# V1 Configuration - Legacy format
name: My Awesome Project
description: A brief description of your project
version: "1.0"

settings:
  parallel: true
  max_workers: 8
  cache_enabled: true

includes:
  - "src/**/*.rs"
  - "tests/**/*.rs"

excludes:
  - "target/**/*"
  - "**/*.gen.rs"

rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
  
  - name: dependency-check
    severity: critical
    check_circular: true
```

<Aside type="tip" title="Which format should I use?">
  **For new projects**: Use V2 - it's more intuitive and maintainable.
  **For existing projects**: You can keep using V1, or migrate to V2 using `assura migrate --from v1 --to v2`.
</Aside>

## First Validation Run

### Basic Validation

Validate all files in your project:

```bash
assura validate
```

### Validate Specific Files

```bash
assura validate src/main.rs src/lib.rs
```

### Validate with Custom Config

```bash
assura validate --config /path/to/config.yml
```

### Output Formats

Assura supports multiple output formats:

<Tabs>
<TabItem label="Pretty (default)">
```bash
assura validate
```
Human-readable output with colors and formatting.
</TabItem>
<TabItem label="JSON">
```bash
assura validate --format json
```
Machine-readable JSON output for integrations.
</TabItem>
<TabItem label="Check">
```bash
assura validate --format check
```
Minimal output with exit codes for CI/CD.
</TabItem>
<TabItem label="Markdown">
```bash
assura validate --format markdown
```
Generate a markdown report.
</TabItem>
</Tabs>

### Watch Mode

Enable continuous validation during development:

```bash
assura watch
```

Assura will monitor your files and automatically re-validate when changes are detected.

## Common Use Cases

### Use Case 1: Enforcing Code Standards

Ensure all Rust files follow naming conventions:

```yaml
rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
    message: "Rust files must use snake_case naming"
  
  - name: line-length
    severity: medium
    max_length: 100
    ignore_urls: true
```

### Use Case 2: Detecting Circular Dependencies

Prevent circular dependencies in your codebase:

```yaml
rules:
  - name: dependency-check
    severity: critical
    check_circular: true
    max_depth: 10
```

### Use Case 3: File Size Limits

Prevent files from growing too large:

```yaml
rules:
  - name: file-size
    severity: medium
    max_size: "500KB"
    include:
      - "src/**/*.rs"
```

### Use Case 4: Documentation Requirements

Ensure public APIs are documented:

```yaml
rules:
  - name: documentation
    severity: medium
    require_public: true
    require_module: true
```

### Use Case 5: CI/CD Integration

Use in your CI/CD pipeline with check format:

```yaml
# .github/workflows/validate.yml
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

## Severity Levels Explained

Assura uses four severity levels to categorize issues:

| Level | Description | Behavior |
|-------|-------------|----------|
| **Critical** | Must fix immediately | Validation fails, blocks CI/CD |
| **High** | Serious issues | Reported prominently, should fix soon |
| **Medium** | Potential problems | Reported, should review |
| **Low** | Minor suggestions | Reported, optional to fix |

Configure severity per rule:

```yaml
rules:
  - name: dependency-check
    severity: critical  # Circular dependencies are critical
  
  - name: line-length
    severity: low       # Line length is just a suggestion
```

## Next Steps

Now that you have Assura installed and running, explore these topics:

- **[Configuration Overview](/docs/configuration/)** - Learn about V2 and V1 configuration formats
- **[V2 Configuration Reference](/reference/config-v2/)** - Complete V2 configuration documentation
- **[Migration Guide](/guides/migration/)** - Migrate from V1 to V2
- **[API Documentation](/reference/api/)** - Programmatic API usage
- **[Examples](/examples/basic-setup/)** - Practical usage examples
- **[Rules](/docs/rules/)** - Available validation rules

<Aside type="note" title="Need Help?">
  Join our [GitHub Discussions](https://github.com/anomalyco/assura/discussions) for community support.
</Aside>
