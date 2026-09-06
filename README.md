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

Assura is pre-1.0. [`v0.3.0`](https://github.com/rothnic/assura/releases/tag/v0.3.0)
is the latest published release, live-verified on 2026-07-02. The current
source tree is the `v0.4.0` candidate: its Review, Explain, onboarding,
event-feedback, and managed-host activation contracts are assigned to that
candidate in the release-surface manifest. They become publicly installable
only after the `v0.4.0` tag passes the strict release gate and publishes.

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
- **Continuous local validation** - Keep one prepared policy warm, coalesce edit bursts, and report whether feedback covers one affected path or the complete project.

## Install

```bash
curl -fsSL https://assura.dev/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://assura.dev/install.ps1 | iex
```

Manual release archives are available for Linux, macOS, and Windows from
[GitHub Releases](https://github.com/rothnic/assura/releases/latest). Each
archive includes `assura` plus an internal `assura-full` companion; keep both
files in the same directory and use `assura` for normal commands.

Build from source when working on Assura itself:

```bash
cargo build --release
```

The public installer resolves the latest published release. To exercise the
`v0.4.0` candidate from a checkout, install that exact checkout instead:

```bash
cargo install --path . --locked
```

## Agent-Ready Source Start

```bash
assura agent onboard . --agent auto --format json
assura review
assura explain src
assura watch --format json
assura check
```

These current-source commands expose active, inactive, and unresolved setup
state rather than imply that unchecked capabilities passed. `assura review` is
advisory; `assura check` remains the authoritative configured-policy gate.

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

| Surface | Contract | Availability |
| --- | --- | --- |
| `assura check` and text/JSON/YAML/advice/status/agent formats | Supported configured-policy gate | `v0.3.0`; `v0.4.0` candidate |
| `assura init`, `status`, `migrate`, hooks, quality planning, performance report | Supported setup and automation | `v0.3.0`; `v0.4.0` candidate |
| `assura review`, `doctor`, and `explain` | Supported advisory, diagnosis, and policy-evidence workflow | `v0.4.0` candidate |
| `assura agent onboard` and `assura agent nudge` | Supported onboarding and bounded event feedback | `v0.4.0` candidate |
| `assura agent integration install|activate|update|deactivate|remove|status|doctor` | Supported Assura-owned project-local host lifecycle | `v0.4.0` candidate |
| Markdown/link/reference/guidance checks and local content/query/context packs | Supported deterministic policy and context layer | `v0.4.0` candidate |
| `assura watch`, cache, daemon, and local sessions | Supported warm local execution | `v0.4.0` candidate |

`assura info`, Markdown safe fixes, and first-party `extensions.*` policy
families remain experimental before 1.0.
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
ASSURA_VERSION=v0.4.0 cargo xtask release-live
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
