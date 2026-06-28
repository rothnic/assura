---
title: Compatibility And Public Surface
status: active
---

# Compatibility And Public Surface

This matrix is the release-time source of truth for compatibility claims.

## Platform Compatibility

| Platform | Archive | Release Workflow | PR Smoke |
| --- | --- | --- | --- |
| Linux x86_64 GNU | `assura-linux-amd64.tar.gz` | `.github/workflows/release.yml` | Installable Adoption Smoke (ubuntu-x86_64), Release Bundle Smoke |
| Linux x86_64 musl | `assura-linux-musl-amd64.tar.gz` | `.github/workflows/release.yml` | Release workflow build and size gate |
| macOS Apple Silicon | `assura-macos-arm64.tar.gz` | `.github/workflows/release.yml` | Installable Adoption Smoke (macos-arm64) |
| macOS Intel | `assura-macos-amd64.tar.gz` | `.github/workflows/release.yml` | Installable Adoption Smoke (macos-x86_64) |
| Windows x86_64 | `assura-windows-amd64.zip` | `.github/workflows/release.yml` | Installable Adoption Smoke (windows-x86_64), Windows Installer Smoke |

Install scripts must continue to consume these archive names unless a release
note and migration notice change the public contract. Release automation
publishes and verifies a `.sha256` file next to every archive.

## Command Compatibility

| Command | Status | Evidence |
| --- | --- | --- |
| `assura check` | Supported | Rust test suite, Assura self-check, installable adoption smoke. |
| `assura check --format json` | Supported | Installable adoption smoke and CLI report tests. |
| `assura check --format yaml` | Supported | CLI report formatter tests. |
| `assura check --format advice` | Supported | Agent feedback rendering tests and real-project feedback fixtures. |
| `assura check --format status` | Supported | Agent feedback rendering tests and real-project feedback fixtures. |
| `assura check --format agent` | Supported | Agent feedback tests and real-project feedback fixtures. |
| `assura check --format agent --agent codex` | Supported adapter | Codex delivery fixture under the shared agent format. |
| `assura init` | Supported | Installable adoption smoke. |
| `assura status --format json` | Supported | Installable adoption smoke. |
| `assura migrate` | Supported for complete LS-Lint 2.3 config semantics | LS-Lint feature matrix, native golden parity tests, migration tests, and adoption smoke. |
| `assura hooks` | Supported for local git hooks | CLI help and local hook behavior. |
| `assura quality plan` | Supported for local quality planning | `.assura/config.yml`, `docs/validation.md`, and changed-check scripts. |
| `assura performance-report` | Supported evidence command | Performance report CI job and checked report data. |
| `assura info` | Experimental diagnostic | CLI exists, but text output is not an automation contract. |
| `assura watch` | Experimental | CLI exists, but release-grade watch behavior is not claimed. |

## LS-Lint Compatibility

Assura supports migration for complete LS-Lint 2.3 config semantics documented
in `docs/ls-lint-2.3-feature-matrix.md`: `ls`, `ignore`, extension and
subextension rules, wildcard extension rules, `.dir`, nested/glob/brace
directory scopes, multiple rules, LS-Lint naming rules and aliases, `regex:`,
regex negation, regex directory substitutions, `exists`, file and directory
existence checks, LS-Lint scalar naming no-op keys, multi-config merge
behavior, and explicit target-path semantics as an Assura validation mode.

Compatibility is not a promise to run LS-Lint itself, match LS-Lint's CLI flags,
or provide exact LS-Lint JSON output. CLI drop-in parity is out of scope.

Supported claims require one of:

- a feature row in `docs/ls-lint-2.3-feature-matrix.md`;
- a converter or config test under `src/config/`;
- a migration fixture in the Rust test suite;
- a native LS-Lint golden parity test;
- an adoption smoke that runs `assura migrate`; or
- a checked analysis report linked from a PR.

Unsupported LS-Lint behavior must fail clearly during migration or be called out
in docs before release.

## Agent Compatibility

The agent contract is vendor-neutral by default:

```bash
assura check --format agent .
```

Codex delivery is opt-in:

```bash
assura check --format agent --agent codex .
```

No release compatibility claim may depend on package feedback CLIs, per-agent
command names, or one `--format` value per agent.

## Rust Surface Compatibility

| Surface | Status | Evidence |
| --- | --- | --- |
| `rust:content_repository` | Experimental | `tests/content_runtime_validation.rs` exercises the first repo-native content runtime validation slice. |

## Extension Compatibility

Common repository relationships are authored in `structure` with single-brace
captures, `exists:1`, `needs`, and `provides`. Custom constraints are
experimental and first-party in v0.1.0:

- config lives under `extensions.custom_constraints`;
- supported specialized types: `paired_file_exists` and `command_surface_docs`;
- execution surface: `assura check`;
- diagnostics: normal report entries with `custom:<id>` rule names;
- safety: no absolute paths, parent escapes, Windows prefixes, remote loading,
  shell execution, or marketplace behavior.

Breaking changes to this experimental surface are allowed before 1.0, but must
be named in release notes.

Support matrices are experimental and first-party in v0.1.0. They classify
public commands and Rust export families so `assura check` can report newly
exposed surfaces that do not have an explicit support status:

```yaml
extensions:
  support_matrices:
    - id: public_surface
      severity: high
      command_contracts:
        - .assura/command-surface.yml
      rust_exports:
        - src/lib.rs
      docs_claim_sources:
        - path: docs/compatibility-and-surface.md
      manifest_policies:
        - cargo_workspace
      entries:
        - surface: "command:assura check"
          status: supported
        - surface: "rust:intelligence"
          status: internal
        - surface: "package:assura"
          status: supported
        - surface: "binary:assura"
          status: supported
```

The supported status vocabulary is `supported`, `experimental`, `internal`,
`roadmap`, and `unsupported`. Command surfaces are read from configured
command-surface contracts. Rust surfaces are bounded to top-level `pub mod`
and `pub use` families in configured source files. Docs claim sources are
bounded Markdown tables, and manifest policies reuse configured
`extensions.manifest_semantics` package and binary declarations. Diagnostics
use `support_matrix:<id>` rule names.

## Rust Library Surface

The Rust crate exposes modules used by the binaries, tests, and benchmark
harnesses. These exports are unstable internal APIs before 1.0 unless a support
matrix row explicitly says otherwise.

Public module visibility in `src/lib.rs` does not imply release support for
dependency graph validation, maturity detection, or broad validation-engine
APIs.

## Unsupported Claims

Do not claim release support for:

- dependency graph validation;
- maturity detection;
- hosted dashboards;
- automatic repair;
- IDE plugins;
- remote custom plugins;
- plugin marketplaces;
- hidden agent setup;
- per-agent feedback packages.
