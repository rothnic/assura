---
title: 'Assura v0.1.0 Release Notes'
status: active
---

# Assura v0.1.0 Release Notes

## Overview

We are excited to announce the first official release of **Assura** - a dependency-aware file system validation engine for modern software projects.

## What's New

### Core Features

- **Dependency-Aware Validation**: Analyzes import/require statements to detect circular dependencies and validate dependency ordering
- **12 Naming Conventions**: Full support for kebab-case, snake_case, camelCase, PascalCase, flatcase, FLATCASE, COBOL-CASE, Train-Case, dot.case, SCREAMING_SNAKE_CASE, and more
- **Markdown Validation**: Comprehensive markdown validation including frontmatter schemas, heading hierarchy, link checking, and template enforcement
- **Maturity Detection**: Automatically assesses project maturity based on git history, filesystem structure, CI/CD configuration, and documentation
- **File System Watching**: Continuous validation during development with intelligent change detection
- **Parallel Execution**: High-performance validation using Rayon for CPU-intensive operations and Tokio for async file operations

### Validation Rules

#### Naming Conventions
- Configurable rules for different file types and directories
- Path-specific rules (e.g., `src/**/*.rs` uses snake_case, `src/components/**` uses PascalCase)
- Multi-part extension support (.d.ts, .test.js, .min.css)
- OR syntax for multiple acceptable conventions

#### Markdown
- Frontmatter schema validation with type checking
- Heading hierarchy validation (no skipped levels, single H1)
- Dead link detection
- Template enforcement for consistent documentation
- Word count and content validation

#### Dependencies
- Circular dependency detection with detailed reporting
- Import/require graph analysis
- Custom dependency constraints

#### File Organization
- Directory structure validation
- File location constraints
- Exclusion patterns for generated files

### CLI Features

```bash
# Validate entire project
assura check

# Watch mode for continuous validation
assura watch

# Initialize configuration
assura init

# Install git hooks
assura hooks install

# Check with specific maturity level
assura check --maturity stable
```

### Git Hooks Integration

Automatic pre-commit validation ensures code quality before commits:
- Runs configured validators
- Blocks commits on critical errors
- Provides detailed error messages

### Configuration

Flexible YAML-based configuration (`.assura/config.yml`):

```yaml
version: "1.0"
maturity: stable

naming:
  conventions:
    - name: "rust_source"
      pattern: "^[a-z_][a-z0-9_]*\.rs$"
      applies_to: "src/**/*.rs"
      severity: high

markdown:
  validation:
    enabled: true
    rules:
      - id: "frontmatter-required"
        applies_to: "**/*.md"
        severity: medium
```

### IDE Integration

**OpenCode Plugin** (`@assura/opencode-plugin`):
- Real-time validation in your IDE
- Inline error reporting
- Quick fixes for common issues

## Performance

- Validated projects with 10,000+ files in under 5 seconds
- Incremental validation only checks changed files
- Parallel processing utilizes all CPU cores
- Memory-efficient with streaming file processing

## Documentation

- Comprehensive documentation website at https://assura.dev
- API documentation at https://docs.rs/assura
- Integration guides for CI/CD systems
- Best practices and examples

## Installation

### From Install Script

```bash
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
```

### With Cargo

Use Cargo only when you already have a Rust development environment:

```bash
cargo install assura
```

### From source
```bash
git clone https://github.com/rothnic/assura
cd assura
cargo build --release
```

### Git Hooks
```bash
assura hooks install
```

## Self-Validation

Assura validates itself! The project includes:
- `.assura/config.yml` with validation rules
- Git hooks for pre-commit validation
- CI/CD integration

## Community

- GitHub: https://github.com/rothnic/assura
- Documentation: https://assura.dev
- Issues: https://github.com/rothnic/assura/issues

## Acknowledgments

Thank you to all contributors and the open-source community for making this release possible.

## What's Next

- Additional language-specific validators
- Enhanced IDE integrations (VS Code, JetBrains)
- Web-based dashboard for project health
- Team collaboration features
- Custom constraint marketplace

---

**Full Changelog**: https://github.com/rothnic/assura/compare/v0.0.0...v0.1.0
