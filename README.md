# Assura

A dependency-aware file system validation engine written in Rust.

## Overview

Assura provides comprehensive file system validation with:

- **Dependency graph analysis** - Detect circular dependencies and determine optimal validation ordering
- **Rule-based validation** - Configurable severity levels (Critical, High, Medium, Low)
- **File system watching** - Continuous validation during development
- **Parallel execution** - High-performance validation for large projects
- **Extensible plugin architecture** - Custom validators for project-specific needs

## Features

- **Structure-first configuration** - Define your project hierarchy and apply validation rules
- **LS-Lint compatibility** - Migrate existing configurations seamlessly
- **12 naming conventions** - Support for snake_case, kebab-case, camelCase, PascalCase, and more
- **Markdown validation** - Frontmatter checks, link validation, heading structure
- **Rust-aware** - Built-in support for Rust project conventions

## Quick Start

```bash
# Install Assura
cargo install assura

# Initialize configuration
assura init

# Validate your project
assura check

# Watch for changes
assura watch
```

## Configuration

Create `.assura/config.yml`:

```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
    
  docs/:
    files:
      naming: kebab-case
    markdown:
      require_frontmatter: true
```

## Documentation

- [Configuration Guide](docs/config-v2.md) - Complete configuration reference
- [Contributing](CONTRIBUTING.md) - How to contribute to the project
- [Constitution](CONSTITUTION.md) - Project principles and governance

## Performance

Assura is designed for performance:
- 6.8x faster than LS-Lint on average
- Parallel directory traversal with jwalk
- Efficient dependency graph analysis with petgraph
- Optimized constraint validation

See [Performance Benchmarks](docs/PERFORMANCE_BENCHMARK_REPORT.md) for detailed comparisons.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
