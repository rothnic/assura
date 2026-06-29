---
title: Assura v0.1.0 Release Notes
status: active
---

# Assura v0.1.0 Release Notes

These notes describe the current pre-1.0 public command surface on this branch.
Assura publishes installable archives from
[`.github/workflows/release.yml`](../.github/workflows/release.yml) when a
maintainer pushes an intentional `v*` tag after the release checklist in
[`docs/release-candidate-checklist.md`](./release-candidate-checklist.md)
passes.

## Supported Commands

The current pre-1.0 command surface supports these public commands:

- `assura check` for structure-first repository validation.
- `assura check --format json`, `--format yaml`, `--format advice`,
  `--format status`, and `--format agent` for automation and agent feedback.
- `assura check --format agent --agent codex` for optional Codex
  `UserPromptSubmit` delivery when Codex hooks are enabled and approved.
- `assura init` for starter `.assura/config.yml` creation.
- `assura status --format json` for project/config/rule summaries.
- `assura migrate` for supported LS-Lint configuration migration.
- `assura hooks` for local git hook installation and inspection.
- `assura quality plan` for deterministic local quality-gate planning from
  `.assura/config.yml`.
- `assura performance-report` for checked Assura versus LS-Lint performance
  evidence.
- `assura fix markdown --dry-run --format json` for safe-fix preview output
  before applying deterministic Markdown trailing-space fixes.
- `assura content` query commands for deterministic local collection,
  relation, keyword, optional semantic-candidate, optional code-symbol, and
  bounded graph queries over modeled project facts.
- `assura content session` for a local JSON-line project-intelligence session
  that can answer repeated diagnostics, context-pack, graph, search, relation,
  and safe-fix preview requests without restarting the CLI process.

`assura info` and `assura watch` remain present in the CLI, but the release
support policy treats `assura info` as an experimental diagnostic and
long-running watch behavior as experimental until dedicated goals add
release-grade tests and docs.

## Installable Archives

Release automation builds these archives:

| Platform | Archive | Smoke Evidence |
| --- | --- | --- |
| Linux x86_64 GNU | `assura-linux-amd64.tar.gz` | CI installable adoption smoke and release bundle smoke |
| Linux x86_64 musl | `assura-linux-musl-amd64.tar.gz` | Release workflow build and size gate |
| macOS Apple Silicon | `assura-macos-arm64.tar.gz` | CI installable adoption smoke |
| macOS Intel | `assura-macos-amd64.tar.gz` | CI installable adoption smoke |
| Windows x86_64 | `assura-windows-amd64.zip` | CI installable adoption smoke and Windows installer smoke |

Each archive contains `assura` and the internal `assura-full` companion. Keep
both files together. The public command is still `assura`.

Release automation publishes a `.sha256` checksum file next to every archive.
The release workflow verifies those checksums before upload, and
`cargo xtask release-live` checks that public checksum URLs are reachable
after a tag is published.

The original `v0.1.0` archives were published on 2026-05-24. On 2026-06-10,
maintainers uploaded the missing `.sha256` sidecar files generated from those
published archives. No release binary was rebuilt and no new version was cut
for that asset repair.

## Current Feature Surface

### Structure Validation

- Directory and file contracts from `.assura/config.yml`.
- Naming conventions, extension rules, line limits, size limits, child-count
  limits, markdown frontmatter checks, and markdown heading checks.
- Exclusion patterns for generated directories and local build output.
- Text, JSON, YAML, and agent output formats.

### LS-Lint Migration

- Supported LS-Lint 2.3 naming and ignore patterns migrate through
  `assura migrate`.
- Unsupported LS-Lint behavior is reported explicitly instead of silently
  translated.
- Compatibility claims are backed by checked fixtures and parity tests.

### Agent Feedback

- The stable agent feedback surface is `assura check --format agent`.
- Codex delivery is an adapter on that shared surface:
  `assura check --format agent --agent codex`.
- There are no package feedback CLIs, per-agent CLI entrypoints, or per-agent
  `--format` values in this release.

### Project Intelligence Queries

- `assura content agent-context`, `collections`, `instances`, `show`,
  `agent-query`, `search`, `semantic-search`, `symbols`, `symbol-refs`,
  `missing-relations`, and `expand` query modeled content facts through the
  local project-intelligence fact model.
- `assura content agent-context` emits the shared generic
  `assura.project-intelligence.agent-context.v1` schema for wrappers that need
  to discover diagnostics, safe-fix, graph/search, semantic, and code-symbol
  capabilities without creating per-agent command families.
- `assura content agent-query <capability> --format json` emits
  `assura.project-intelligence.agent-query.v1`, a shared request/response
  envelope for diagnostics, graph expansion, keyword search, semantic
  candidates, code-symbol relationships, and safe-fix fact summaries.
- Keyword search is deterministic local text matching over indexed chunks.
- `assura content semantic-search` is opt-in through `--enable-local` and uses
  local candidate retrieval. Scores do not decide validation correctness.
- `assura content symbols` and `assura content symbol-refs` use configured
  modeled fields and optional provider evidence. The built-in Rust token
  baseline can resolve rough local declarations; missing providers preserve
  unresolved refs instead of failing validation.
- Daemon APIs, LSP, and MCP are not part of this content-query surface.

### Safe Fixes

- `assura fix markdown --rule trailing-spaces --dry-run --format json` emits
  `assura.safe-fix.markdown.v1` and reports proposed files and line fixes
  without writing.
- `assura fix markdown --rule trailing-spaces --apply --format json` applies
  the bounded Markdown trailing-space fix and reports changed paths, applied
  fix IDs, skipped fixes, and rollback guidance.
- Omitting both `--dry-run` and `--apply` previews fixes without writing;
  every write path requires `--apply`.

### Custom Constraints

- `extensions.custom_constraints` is experimental and first-party.
- The initial supported custom constraint is `paired_file_exists`.
- Custom constraints execute through `assura check` and report normal
  `StructureViolation` records with `custom:<id>` rule names.
- Remote plugin loading, marketplaces, shell-executed plugins, and third-party
  Rust/TypeScript plugin APIs are not part of v0.1.0.

## Removed Or Superseded Surfaces

The following older ideas are not supported release surfaces:

- `assura-codex-feedback` or other package feedback CLIs.
- Unsupported historical: `assura check --format codex-hook` or one format
  value per agent.
- Per-agent command entrypoints.
- Hosted telemetry, dashboards, or automatic remote reporting.
- Dependency graph validation as a release claim.
- Marketplace or remote plugin loading.

## Support And Compatibility

The release support policy is in
[`docs/support-policy.md`](./support-policy.md). The compatibility matrix is in
[`docs/compatibility-and-surface.md`](./compatibility-and-surface.md).

Assura remains pre-1.0. Configuration formats, experimental extension fields,
and internal APIs can change before 1.0, but public release notes must identify
breaking changes and removed experimental surfaces.

## Verification

Release candidate validation uses:

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo run --quiet -- check --format json .
cargo xtask fast
cargo xtask docs
cargo xtask release-smoke
cargo xtask evidence
git diff --check
```

After publishing a tag, maintainers must also run:

```bash
cargo xtask release-live
ASSURA_VERSION=v0.1.0 cargo xtask release-live
```

## Next

Iteration 01 closes with release readiness. The next planned roadmap iteration
is
[`Assura Roadmap Iteration 02: Policy Depth And Ecosystem`](./goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md).
