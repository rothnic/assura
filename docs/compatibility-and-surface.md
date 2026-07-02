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
| `assura fix markdown --dry-run --format json` | Experimental safe-fix preview contract | Markdown safe-fix CLI tests prove default all-rule preview output covers every supported deterministic fix class without writing files. |
| `assura fix markdown --apply --format json` | Experimental safe-fix apply/audit contract | Markdown safe-fix CLI tests prove default all-rule apply output reports changed paths, applied fix IDs, skipped fixes, and idempotent reruns. |
| `assura agent` | Supported local agent project-intelligence surface | Agent-surface CLI tests prove JSON-default commands delegate to existing content-query contracts. |
| `assura agent context` | Supported local agent context | Agent-surface CLI tests compare output with `assura content agent-context`. |
| `assura agent diagnostics` | Supported local agent diagnostics | Agent-surface CLI tests compare output with `assura content agent-query diagnostics`. |
| `assura agent context-pack` | Supported local agent handoff packet | Agent-surface CLI tests compare output with `assura content context-pack`. |
| `assura agent show` | Supported local agent content inspection | Agent-surface CLI tests compare output with `assura content show`. |
| `assura agent search` | Supported local agent keyword search | Agent-surface CLI tests compare output with `assura content search`. |
| `assura agent missing-relations` | Supported local agent relation query | Agent-surface CLI tests compare output with `assura content missing-relations`. |
| `assura agent expand` | Supported local agent graph expansion | Agent-surface CLI tests compare output with `assura content expand`. |
| `assura agent safe-fixes` | Supported local agent safe-fix preview | Agent-surface CLI tests compare output with `assura content agent-query safe-fixes`. |
| `assura agent onboard` | Experimental local agent-ready onboarding surface | Project-intelligence onboarding tests prove the command generates a broad baseline, onboarding packet, checked/unchecked report, reusable dynamic skill-directory contract, and non-destructive existing-file behavior. |
| `assura agent nudge` | Experimental local agent nudge payload | Agent-surface CLI tests prove bounded event-aware JSON for session start, before/after tool events, daemon fallback, and performance-gate path hints without per-agent validation commands. |
| `assura agent integration` | Experimental local agent integration lifecycle | Agent-surface CLI tests prove install, update, remove, status, and doctor workflows generate reviewable `.assura/integrations/<agent>/` bundles for Codex, OpenCode, Claude, and Pi without embedding validation logic. |
| `assura agent integration install` | Experimental local agent integration lifecycle | Generates manifest, wrapper, and README files under `.assura/integrations/<agent>/`; `--dry-run` previews writes. |
| `assura agent integration update` | Experimental local agent integration lifecycle | Regenerates an existing Assura-managed bundle over the same shared nudge/check/daemon commands. |
| `assura agent integration remove` | Experimental local agent integration lifecycle | Removes only Assura-managed bundle files and leaves host-agent configuration to the user. |
| `assura agent integration status` | Experimental local agent integration lifecycle | Reports expected files, managed status, and host wiring guidance. |
| `assura agent integration doctor` | Experimental local agent integration lifecycle | Checks config presence, managed bundle state, and shared nudge/check/daemon command delegation. |
| `assura agent session` | Supported local agent session alias | Agent-surface CLI tests prove the alias emits the same JSON-line session envelope as `assura content session`. |
| `assura editor` | Supported local editor project-intelligence surface | Editor-surface CLI tests prove help output and local session availability. |
| `assura editor session` | Supported local editor session | Editor-surface CLI tests prove LSP-shaped diagnostics, context, safe-fix code-action previews, invalid-method errors, and conservative reload metadata. |
| `assura content` | Supported first project-intelligence query surface | Content query CLI fixture tests and product docs. |
| `assura content agent-context` | Supported generic agent context | Agent-context CLI fixture tests; wrappers must reuse this contract instead of creating per-agent query commands. |
| `assura content agent-query` | Supported generic agent query envelope | Agent-query CLI fixture tests prove diagnostics, graph, search, semantic, and code-symbol queries reuse one wrapper schema. |
| `assura content context-pack` | Supported bounded project-intelligence context bundle | Context-pack tests prove diagnostics, graph/search context, repository-reference context, relation status, and safe-fix preview metadata compose without writes. |
| `assura content session` | Supported local project-intelligence session | Session tests prove repeated JSON-line requests reuse context and reload conservatively after modeled content changes. |
| `assura content collections` | Supported | Content query CLI fixture tests. |
| `assura content instances` | Supported | Content query CLI fixture tests. |
| `assura content show` | Supported | Content query CLI fixture tests. |
| `assura content search` | Supported scored keyword search | Content query CLI fixture tests prove deterministic lexical scores; semantic candidate retrieval uses the separate `semantic-search` command. |
| `assura content semantic-search` | Experimental optional local candidate search | Semantic search fixture tests; candidates do not decide validation correctness. |
| `assura content symbols` | Experimental optional code-symbol query | Code-symbol fixture tests; baseline evidence is candidate context and does not decide validation correctness. |
| `assura content symbol-refs` | Experimental optional code-symbol query | Code-symbol fixture tests; unresolved provider refs remain queryable. |
| `assura content missing-relations` | Supported relation query | Content query CLI fixture tests. |
| `assura content expand` | Supported bounded graph expansion | Content query CLI fixture tests. |
| `assura content references` | Supported repository-reference graph query | Content query CLI tests prove bounded inbound references by target path and outbound references by source path. |
| `assura daemon` | Experimental local daemon process | Daemon CLI tests prove JSON status, start, stop, restart, doctor, logs, health, changed-path, and reference-context contracts over local daemon-ready state. Lifecycle commands manage a real local process with versioned health, check-path, and repository-reference IPC. |
| `assura daemon status` | Experimental local daemon status | Daemon CLI tests prove JSON health, protocol, process metadata, crashed-process detection, and management command hints. |
| `assura daemon start` | Experimental local daemon lifecycle | Daemon CLI tests prove idempotent process-backed JSON start behavior with PID and IPC address metadata. |
| `assura daemon stop` | Experimental local daemon lifecycle | Daemon CLI tests prove idempotent process-backed JSON stop behavior. |
| `assura daemon restart` | Experimental local daemon lifecycle | Daemon CLI tests prove process replacement, JSON restart behavior, and runtime log updates. |
| `assura daemon doctor` | Experimental local daemon doctor | Daemon CLI tests prove JSON diagnostics and remediation commands for loaded, unavailable, stopped, running, and crashed project state. |
| `assura daemon logs` | Experimental local daemon logs preview | Daemon CLI tests prove bounded JSON log output from `.assura/daemon/daemon.log`. |
| `assura info` | Experimental diagnostic | CLI exists, but text output is not an automation contract. |
| `assura watch` | Experimental | CLI exists, but release-grade watch behavior is not claimed. |

## Editor Adapter Compatibility

| Surface | Status | Evidence |
| --- | --- | --- |
| `integrations/editors/vscode` | Supported beta local VS Code package | Package tests and smoke scripts prove daemon command construction, one-shot check fallback, safe-fix preview-only command construction, status summaries, diagnostic mapping, support metadata, marketplace deferral, and lifecycle doctor/package checks over shared Assura CLI JSON contracts. |

## Config Compatibility

| Surface | Status | Evidence |
| --- | --- | --- |
| `config:markdown.lint_common` | Experimental | Common-lint CLI tests prove stable findings for heading increments, heading marker spacing, duplicate headings, multiple blank lines, suppressions, and severity overrides. |
| `config:extensions.repository_references` | Experimental | Repository-reference check tests prove opt-in source/comment/docstring diagnostics for missing targets, missing Markdown anchors, and invalid line anchors. |
| `project-intelligence:repository-reference-facts` | Supported | Repository-reference graph tests prove Markdown, source-comment, docstring, and string-literal path candidates become bounded `RepositoryReference` edges with confidence labels. Object-mode context-pack tests prove those edges are available as bounded inbound/outbound document-graph context. |

## Project Intelligence Layout Compatibility

`.assura/models/**` is the supported project-intelligence model artifact layout
for model files stored under `.assura/`. Content runtime validation tests and
Assura self-check prove that root-level `.assura/` model artifacts are rejected
while project-relative `schemas/**` artifacts remain valid.

The beta-supported content contract is the modeled collection path: content
runtime validation, deterministic collection queries, keyword search, relation
queries, repository-reference queries, bounded graph expansion, context packs,
and local JSON-line sessions. Semantic search and code-symbol queries are
candidate-enrichment surfaces. They can help an agent choose where to inspect
next, but they are not validation truth and are not required for collection
modeling or querying to work.

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

Event-aware wrappers can use one shared nudge payload:

```bash
assura agent nudge --event after-tool --changed docs/guide.md --agent codex .
```

`--agent codex|opencode|claude|pi` labels the host integration path only; the
payload still reuses shared Assura check and daemon contracts.

No release compatibility claim may depend on package feedback CLIs, per-agent
command names, or one `--format` value per agent.

## Rust Surface Compatibility

| Surface | Status | Evidence |
| --- | --- | --- |
| `rust:content_repository` | Experimental | `tests/content_runtime_validation.rs` exercises the first repo-native content runtime validation slice. |

## Extension/API Boundary Compatibility

[`docs/extension-api-boundaries.md`](extension-api-boundaries.md) is the
canonical pre-1.0 boundary for extension and plugin language. Current
`extensions.*` entries are first-party config policy families executed by
`assura check`; they are not a public third-party plugin API, remote plugin
loader, shell-executed validator system, plugin marketplace, TypeScript plugin
SDK, or semver-stable Rust library API.

| Surface | Status | Evidence |
| --- | --- | --- |
| `config:extensions.custom_constraints` | Experimental first-party | Custom constraint tests prove specialized built-in relationship checks and command-surface docs checks without shell execution, remote loading, or marketplace behavior. |
| `config:extensions.release_contracts` | Experimental first-party | Release contract checks prove artifact, checksum, workflow, docs, installer, and branch-reference synchronization for configured release artifacts. |
| `config:extensions.support_matrices` | Experimental first-party | Support matrix checks prove configured commands, Rust export families, docs claim tables, manifest packages, and binaries are classified with the supported status vocabulary. |
| `config:extensions.manifest_semantics` | Experimental first-party | Manifest semantics checks prove configured Cargo package metadata, publish status, descriptions, keywords, and declared binaries. |
| `config:extensions.test_relationships` | Experimental first-party | Test relationship checks prove configured source/test evidence, fixture-family ownership, and accepted manual-test exceptions. |
| `config:extensions.module_topologies` | Experimental first-party | Module topology checks prove configured Rust module-family ownership, roots, public export classification, and internal visibility boundaries. |
| `config:extensions.docs_lifecycles` | Experimental first-party | Docs lifecycle checks prove configured active/historical docs, frontmatter status, stale-claim evidence, and historical exceptions. |
| `config:extensions.repository_references` | Experimental first-party | Repository-reference check tests prove opt-in source/comment/docstring diagnostics for missing targets, missing Markdown anchors, and invalid line anchors. |
| `config:extensions.relationships` | Internal generated first-party | Structure notation tests prove capture relationships are normalized from `structure` captures, `exists:1`, `needs`, and `provides`. |
| Public plugin API or SDK | Roadmap only | No current command, package, Rust module, or docs surface provides remote plugin loading, shell-executed validators, a plugin marketplace, TypeScript plugin APIs, or semver-stable Rust APIs. |

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
        - surface: "config:markdown.lint_common"
          status: experimental
        - surface: "config:extensions.repository_references"
          status: experimental
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
