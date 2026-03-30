---
title: "Introducing Assura: Dependency-Aware File System Validation"
description: "Announcing the first release of Assura - a powerful validation engine for enforcing naming conventions, validating markdown, detecting circular dependencies, and maintaining project structure."
published: 2026-03-19
author: "Assura Team"
---

# Introducing Assura: Dependency-Aware File System Validation

Today we're excited to announce the first official release of **Assura** - a comprehensive file system validation engine designed for modern software projects.

## Why Assura?

As projects grow, maintaining consistency becomes increasingly challenging. Different team members have different preferences, and without automated enforcement, codebases quickly become inconsistent. Assura solves this by providing:

- **Automated validation** that runs continuously
- **Flexible configuration** that adapts to your project's needs  
- **Fast performance** that doesn't slow down development
- **Clear feedback** that helps developers fix issues quickly

## Key Features

### 1. Naming Convention Validation

Assura supports 12 different naming conventions out of the box:

- `kebab-case` for CSS files and configuration
- `snake_case` for Rust, Python, and Ruby
- `camelCase` for JavaScript and Java
- `PascalCase` for React components and C# classes
- `flatcase` and `FLATCASE` for specific use cases
- `COBOL-CASE` and `Train-Case` for legacy systems
- And more...

You can define different conventions for different parts of your project:

```yaml
naming:
  conventions:
    - name: "rust_source"
      pattern: "^[a-z_][a-z0-9_]*\.rs$"
      applies_to: "src/**/*.rs"
      severity: high
      
    - name: "react_components"
      pattern: "^[A-Z][a-zA-Z0-9]*\.tsx$"
      applies_to: "src/components/**/*.tsx"
      severity: medium
```

### 2. Dependency Analysis

Assura builds a graph of your project's dependencies and can:

- Detect circular dependencies before they cause issues
- Validate that imports follow your architectural rules
- Ensure dependencies are declared in the right order

### 3. Markdown Validation

Keep your documentation consistent with:

- **Frontmatter validation**: Ensure all markdown files have required metadata
- **Heading hierarchy**: Prevent skipped heading levels and multiple H1s
- **Link checking**: Catch broken internal links
- **Template enforcement**: Standardize documentation structure

### 4. Maturity Detection

Assura can automatically assess your project's maturity level based on:

- Git history (commit frequency, branch count)
- Filesystem structure (documentation, tests, CI/CD)
- Configuration files (package managers, linters)

This helps teams understand where to focus improvement efforts.

## Getting Started

Install Assura with cargo:

```bash
cargo install assura
```

Initialize configuration in your project:

```bash
assura init
```

Run validation:

```bash
assura check
```

## Real-World Example

Let's say you have a TypeScript project with inconsistent file naming. Some files use `kebab-case`, others use `snake_case`, and some use `camelCase`. This makes it hard to:

1. Find files quickly
2. Know what convention to use for new files
3. Maintain consistency across the team

With Assura, you can define rules like:

```yaml
naming:
  conventions:
    - name: "typescript_source"
      pattern: "^[a-z][a-z0-9]*(-[a-z0-9]+)*\.ts$"
      applies_to: "src/**/*.ts"
      severity: high
      message: "TypeScript files must use kebab-case"
```

Now when someone creates `myComponent.ts` or `my_component.ts`, they'll get an immediate error:

```
Error: File 'src/utils/myComponent.ts' does not follow kebab-case convention
Suggestion: Rename to 'src/utils/my-component.ts'
```

## Performance

We know validation needs to be fast. Assura is designed for performance:

- **Parallel processing**: Utilizes all CPU cores
- **Incremental validation**: Only checks changed files in watch mode
- **Efficient file system operations**: Uses async I/O with Tokio
- **Smart caching**: Avoids re-validating unchanged files

In benchmarks, Assura validates projects with 10,000+ files in under 5 seconds.

## IDE Integration

Assura includes an OpenCode plugin for real-time validation:

```bash
npm install @assura/opencode-plugin
```

Get instant feedback as you code, with inline error messages and quick fixes.

## CI/CD Integration

Add Assura to your GitHub Actions workflow:

```yaml
name: Validate
on: [push, pull_request]
jobs:
  assura:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: assura/assura-action@v1
      - run: assura check --strict
```

## Self-Validation

We dogfood Assura - it validates itself! The `.assura/config.yml` in our repository ensures:

- All Rust files follow naming conventions
- Documentation has proper frontmatter
- No circular dependencies in the codebase
- Project structure follows best practices

## What's Next?

This is just the beginning. We're working on:

- **Language-specific validators**: Go, Rust, Python, and more
- **Enhanced IDE support**: VS Code and JetBrains plugins
- **Web dashboard**: Visualize project health over time
- **Team features**: Share configurations and validation rules
- **Marketplace**: Community-contributed constraints

## Try It Today

Assura is open source and available now:

- **Install**: `cargo install assura`
- **Documentation**: https://assura.dev
- **GitHub**: https://github.com/assura/assura
- **Issues**: https://github.com/assura/assura/issues

We'd love to hear your feedback and see how you're using Assura in your projects!

---

*Assura is built with Rust and released under the MIT/Apache-2.0 dual license.*
