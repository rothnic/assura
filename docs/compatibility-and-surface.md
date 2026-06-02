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
| `assura check --format agent` | Supported | Agent feedback tests and real-project feedback fixtures. |
| `assura check --format agent --agent codex` | Supported adapter | Codex delivery fixture under the shared agent format. |
| `assura init` | Supported | Installable adoption smoke. |
| `assura status --format json` | Supported | Installable adoption smoke. |
| `assura migrate` | Supported for documented LS-Lint rules | LS-Lint migration tests and adoption smoke. |
| `assura hooks` | Supported for local git hooks | CLI help and local hook behavior. |
| `assura performance-report` | Supported evidence command | Performance report CI job and checked report data. |
| `assura watch` | Experimental | CLI exists, but release-grade watch behavior is not claimed. |

## LS-Lint Compatibility

Assura supports migration for the LS-Lint 2.3 naming and ignore patterns
documented in the LS-Lint migration guide. Compatibility is not a promise to
run LS-Lint itself or to implement every LS-Lint edge case.

Supported claims require one of:

- a parser or config test under `src/ls_compat/` or `src/config/`;
- a migration fixture in the Rust test suite;
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

## Extension Compatibility

Custom constraints are experimental and first-party in v0.1.0:

- config lives under `extensions.custom_constraints`;
- supported type: `paired_file_exists`;
- execution surface: `assura check`;
- diagnostics: normal report entries with `custom:<id>` rule names;
- safety: no absolute paths, parent escapes, Windows prefixes, remote loading,
  shell execution, or marketplace behavior.

Breaking changes to this experimental surface are allowed before 1.0, but must
be named in release notes.

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
