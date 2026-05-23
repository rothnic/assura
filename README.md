# Assura

A structure-first repository validation CLI written in Rust.

## Overview

Assura v0.1 provides:

- **Structure validation** - Check file names, directory names, required entries, forbidden entries, and direct-child counts
- **LS-Lint migration** - Convert supported `.ls-lint.yml` files into `.assura/config.yml`
- **Watch command** - A truthful one-shot wrapper over `assura check`
- **CI reports** - Text, JSON, and YAML output for automation

## Features

- **Structure-first configuration** - Define the allowed project shape in one config file
- **LS-Lint compatibility tests** - Keep migration behavior covered against LS-Lint 2.3 fixtures
- **Naming conventions** - Support common case styles and `regex:<pattern>` naming
- **Markdown checks** - Validate supported markdown rules when configured

## Quick Start

```bash
# Install Assura
curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
sudo install -m 755 assura assura-full /usr/local/bin/

# Initialize configuration
assura init

# Validate your project
assura check

# Run the current watch wrapper
assura watch

# Migrate a supported LS-Lint config
assura migrate .ls-lint.yml --output .assura/config.yml
```

Prebuilt release archives are available for Linux, macOS, and Windows. Cargo
install remains available for Rust development environments. Release archives
include `assura` plus an internal `assura-full` companion; put both files in
the same directory and use `assura` for normal commands.

## Configuration

Create `.assura/config.yml`:

```yaml
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
exclude:
  - "target/**"
```

## Documentation

- [Website Getting Started](website/src/content/docs/guides/getting-started.md) - Current onboarding flow
- [Configuration Guide](website/src/content/docs/docs/configuration.md) - Supported structure-first configuration
- [Contributing](CONTRIBUTING.md) - How to contribute to the project
- [Constitution](CONSTITUTION.md) - Project principles and governance

## Performance

Assura performance claims are tracked through current-product benchmarks. Run:

```bash
cargo bench --bench ls_lint_comparison -- --noplot
cargo bench --bench profiling structure_check -- --noplot
```

See [Benchmark Instructions](benches/README.md) for the supported release
evidence path.

## Roadmap

Agent nudges, quality measurement, long-running watch mode, dependency graph
validation, and plugin APIs are future work. They are not current v0.1
onboarding features.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
