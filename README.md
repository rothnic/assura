# Assura

[![Rust CI](https://github.com/rothnic/assura/actions/workflows/ci.yml/badge.svg)](https://github.com/rothnic/assura/actions/workflows/ci.yml)
[![Documentation](https://github.com/rothnic/assura/actions/workflows/docs.yml/badge.svg)](https://github.com/rothnic/assura/actions/workflows/docs.yml)
[![Security Audit](https://github.com/rothnic/assura/actions/workflows/security.yml/badge.svg)](https://github.com/rothnic/assura/actions/workflows/security.yml)
[![Latest Release](https://img.shields.io/github/v/release/rothnic/assura?include_prereleases)](https://github.com/rothnic/assura/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

Assura catches project drift while the fix is still small. It gives AI coding
agents one fast local signal layer for repository structure, naming, file and
section limits, Markdown, references, branch/worktree change pressure, and
project-specific policy.

**Onboard once. Review while working. Explain when needed. Check before
merge.** Review is the radar; Check is the gate.

## Status

Assura is pre-1.0. The current public release is
[`v0.3.0`](https://github.com/rothnic/assura/releases/tag/v0.3.0), published
and live-verified on 2026-07-02. Assura remains a pre-1.0 beta: supported
surfaces are release-gated, while experimental surfaces may change as the
agent workflow is refined.

CI builds run on pull requests and `master` pushes, including release-style
smoke builds. Those artifacts are verification evidence, not durable product
releases. Durable GitHub Releases are created only by
[`.github/workflows/release.yml`](.github/workflows/release.yml) when a `v*`
tag is pushed. Maintainers cut a new release when an intentional version bump
is ready and the release checklist passes, not merely because CI produced
successful builds.

## What Assura Does

- **Project-level validation** - Define naming, placement, required/forbidden paths, child limits, file limits, Markdown, references, and generated-output boundaries in one local policy.
- **Compact project review** - See configured checks, inactive capabilities, branch/worktree changes, hot directories, and ranked next actions without turning every signal into a gate.
- **Repairable agent feedback** - Emit bounded text, JSON, YAML, advice, status, and agent reports with rule-specific context.
- **Agent-ready onboarding** - Establish a broad baseline, install project-local guidance, verify what is active, and leave human decisions visible.
- **Reusable policy layers** - Start language-agnostic, reuse shared rules, then attach language or domain checks where deeper validation belongs.
- **LS-Lint migration** - Convert supported LS-Lint 2.3 configuration and extend beyond filesystem naming.
- **Cold and warm performance evidence** - Protect the one-shot CLI path and measure persistent sessions separately for agent/editor loops.

## Install

```bash
curl -fsSL https://assura.dev/install.sh | sh
```

Manual release archives are available for Linux, macOS, and Windows from
[GitHub Releases](https://github.com/rothnic/assura/releases/latest). Each
archive includes `assura` plus an internal `assura-full` companion; keep both
files in the same directory and use `assura` for normal commands.

Build from source when working on Assura itself:

```bash
cargo build --release
```

## Agent-Ready Start

```bash
assura agent onboard . --agent auto --format json
assura review
assura explain src
assura check
```

`agent onboard`, `review`, and `explain` are experimental local beta surfaces.
They are designed to expose active, inactive, and unresolved setup state rather
than imply that unchecked capabilities passed. `assura check` remains the
authoritative configured-policy gate.

For a minimal supported structure-only start:

```bash
assura init
assura check
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
| `assura review` | Experimental compact project-health radar |
| `assura doctor` | Experimental configured/inactive capability diagnosis |
| `assura explain` | Experimental path and inherited-rule explanation |
| `assura agent onboard` | Experimental local agent-ready onboarding |
| `assura agent nudge` | Experimental bounded lifecycle feedback |
| `assura agent integration` | Experimental Codex, OpenCode, Claude Code, and Pi integration bundles |
| `assura migrate` | Supported LS-Lint 2.3 config migration path |
| `assura hooks` | Supported local git-hook workflow |
| `assura quality plan` | Supported config-backed quality planning |
| `assura performance-report` | Supported performance evidence command |

`assura info`, `assura watch`, Markdown safe fixes, first-party `extensions.*`
policy families, and local daemon lifecycle are experimental before 1.0.
Hosted dashboards, automatic broad repair, remote plugin loading, plugin
marketplaces, and per-agent validation engines are not supported release
surfaces.

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
ASSURA_VERSION=v0.3.0 cargo xtask release-live
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
