# Assura

[![Rust CI](https://github.com/rothnic/assura/actions/workflows/ci.yml/badge.svg)](https://github.com/rothnic/assura/actions/workflows/ci.yml)
[![Documentation](https://github.com/rothnic/assura/actions/workflows/docs.yml/badge.svg)](https://github.com/rothnic/assura/actions/workflows/docs.yml)
[![Security Audit](https://github.com/rothnic/assura/actions/workflows/security.yml/badge.svg)](https://github.com/rothnic/assura/actions/workflows/security.yml)
[![Latest Release](https://img.shields.io/github/v/release/rothnic/assura?include_prereleases)](https://github.com/rothnic/assura/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

Assura is a structure-first repository validation CLI written in Rust. It
checks whether a project matches the shape declared in `.assura/config.yml`:
file names, directory names, required files, forbidden entries, direct-child
limits, and supported markdown conventions.

## Status

Assura is pre-1.0. The current public release is
[`v0.1.0`](https://github.com/rothnic/assura/releases/tag/v0.1.0), published
on 2026-05-24. The Rust package version is still `0.1.0`; there is no newer
version until the repository intentionally bumps `Cargo.toml` and pushes a new
`v*` tag.

CI builds run on pull requests and `master` pushes, including release-style
smoke builds. Those artifacts are verification evidence, not durable product
releases. Durable GitHub Releases are created only by
[`.github/workflows/release.yml`](.github/workflows/release.yml) when a `v*`
tag is pushed. Maintainers cut a new release when an intentional version bump
is ready and the release checklist passes, not merely because CI produced
successful builds.

## Features

- **Structure-first configuration** - Define the allowed project shape in one config file
- **LS-Lint migration** - Convert supported LS-Lint 2.3 configs into Assura structure config
- **Naming conventions** - Support common case styles and `regex:<pattern>` naming
- **Markdown checks** - Validate supported markdown rules when configured
- **Automation output** - Emit text, JSON, YAML, advice, status, and agent-oriented reports
- **Local quality planning** - Use `.assura/config.yml` to plan scoped project checks

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
```

Manual release archives are available for Linux, macOS, and Windows from
[GitHub Releases](https://github.com/rothnic/assura/releases/latest). Each
archive includes `assura` plus an internal `assura-full` companion; keep both
files in the same directory and use `assura` for normal commands.

Build from source when working on Assura itself:

```bash
cargo build --release
```

## Quick Start

```bash
assura init
assura check
assura migrate .ls-lint.yml --output .assura/config.yml
```

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

## Supported Surface

| Surface | Status |
| --- | --- |
| `assura check` | Supported structure validation command |
| `assura check --format json`, `yaml`, `advice`, `status`, `agent` | Supported automation output |
| `assura check --format agent --agent codex` | Supported Codex adapter on the shared agent format |
| `assura init` | Supported starter config creation |
| `assura status --format json` | Supported project/config/rule summary |
| `assura migrate` | Supported LS-Lint 2.3 config migration path |
| `assura hooks` | Supported local git-hook workflow |
| `assura quality plan` | Supported config-backed quality planning |
| `assura performance-report` | Supported performance evidence command |

`assura info`, `assura watch`, and `extensions.custom_constraints` are
experimental before 1.0. Dependency graph validation, hosted dashboards,
automatic repair, IDE plugins, remote plugin loading, plugin marketplaces, and
per-agent feedback packages are not supported release surfaces.

See [Support Policy](docs/support-policy.md) and
[Compatibility And Public Surface](docs/compatibility-and-surface.md) for the
source of truth.

## Performance

Assura performance claims are tracked through current-product benchmarks. Run:

```bash
cargo bench --bench ls_lint_comparison -- --noplot
cargo bench --bench profiling structure_check -- --noplot
```

See [Benchmark Instructions](benches/README.md) for the supported release
evidence path.

## Releases

Release publication is tag-driven:

1. Update the package version in `Cargo.toml` and release notes.
2. Pass the release candidate gates in
   [docs/release-candidate-checklist.md](docs/release-candidate-checklist.md).
3. Push an annotated `v*` tag.
4. The release workflow builds Linux, macOS, and Windows archives, verifies
   checksums, and publishes the archives plus `.sha256` files.

The live release gate verifies the public install scripts and release assets:

```bash
cargo xtask release-live
ASSURA_VERSION=v0.1.0 cargo xtask release-live
```

## Development Verification

Use the repo verification tiers instead of defaulting to the slowest Cargo
target set:

```bash
cargo xtask fast          # normal local edit gate
cargo xtask pr            # pre-push / PR gate
cargo xtask release-size  # installable archive size gate
cargo xtask release-smoke # no-Rust local archive smoke
cargo xtask release-live  # public post-release install URL gate
cargo xtask full          # includes cargo test --all-targets
```

`cargo xtask fast` runs Rust tests without benchmark harness or standalone binary
harness targets. Save `cargo xtask full` for benchmark-adjacent changes or final
release confidence. Cargo's `target/` cache can be large; the release-size gate
checks the compressed public archive instead.
See [Validation Command Tiers](docs/validation.md) for when to use each mode.

## Documentation

- [Getting Started](website/src/content/docs/guides/getting-started.md)
- [Configuration Guide](website/src/content/docs/reference/configuration.md)
- [CLI/API Reference](website/src/content/docs/reference/api.md)
- [Why Assura](website/src/content/docs/why-assura.md)
- [Support Policy](docs/support-policy.md)
- [Release Notes](docs/release-notes.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

## Security

Report security issues through GitHub private vulnerability reporting when
available, or open a minimal issue that does not include exploit details. See
[SECURITY.md](SECURITY.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
