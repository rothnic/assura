---
title: Assura v0.4.0 Release Notes
status: active
---

# Assura v0.4.0 Release Notes

These notes describe the pre-1.0 public command surface prepared `v0.4.0`
beta increment. `v0.3.0` remains the latest published release until
the `v0.4.0` tag completes release verification. Assura publishes installable archives from
[`.github/workflows/release.yml`](../.github/workflows/release.yml) when a
maintainer pushes an intentional `v*` tag after the release checklist in
[`docs/release-candidate-checklist.md`](./release-candidate-checklist.md)
passes.

## v0.4.0 Release Delta

- `assura watch` now performs continuous validation: one initial requested-path
  report followed by coalesced warm checks over affected paths or conservative
  full-scope fallbacks.
- `assura.watch.event.v1` exposes runtime mode, report scope, cache/reload
  state, changed paths, coalesced event count, duration, and fallback reason.
- Watch respects requested path scope and configured exclusions, reloads local
  or external policy changes even when a platform reports only an adjacent
  filesystem event, survives editor-style atomic file replacement, bounds
  sustained edit batches, and exits cleanly on interrupt without leaving
  runtime state. Backend watcher failure emits a final degraded event and
  terminates instead of remaining alive with a stale subscription.
- Warm changed-path checks now run only for path-local policy. Repository-wide
  extensions and content relationships keep the prepared plan but fall back to
  a full requested-scope report so a partial check cannot produce a false pass.
- The internal hot-check server now ignores non-invalidating access events and
  publishes status under the same generation lock used by watcher callbacks,
  so a validation result cannot overwrite a newer edit as clean and a delayed
  dirty publication cannot overwrite a later clean result. Status replacement
  is atomic on supported Unix and Windows platforms. This is separate from the
  public `assura daemon` process contract.
- Managed `assura daemon check-path` now returns validation exit `1` when its
  versioned IPC report fails, matching the local fallback schema and exit
  behavior.
- The correctness-checked cache, project-local daemon lifecycle, and continuous
  watch command are supported `v0.4.0` contracts. They remain absent from the
  already-published `v0.3.0` contract.

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
  before applying deterministic Markdown safe fixes.
- `assura fix markdown --apply --format json` for explicitly accepted
  Markdown safe-fix apply/audit output.
- `assura agent` for local coding-agent project-intelligence commands with
  JSON defaults for context, diagnostics, context packs, search/show/expand,
  missing relations, safe-fix previews, and local sessions.
- `assura agent integration` for supported `v0.4.0` install, activate,
  update, deactivate, remove, status, and doctor workflows that manage only
  Assura-owned project-local Codex, OpenCode, Claude, and Pi configuration.
- `assura editor session` for a local JSON-line editor protocol with
  LSP-shaped diagnostics, context, code-action preview requests, and
  conservative reload metadata.
- `assura content` query commands for deterministic local collection,
  relation, keyword, optional semantic-candidate, optional code-symbol, and
  bounded graph queries over modeled project facts.
- `assura content session` for a local JSON-line project-intelligence session
  that can answer repeated diagnostics, context-pack, graph, search, relation,
  and safe-fix preview requests without restarting the CLI process.
- `.assura/models/**` as the supported project-intelligence layout for model
  artifacts stored under `.assura/`.

`assura info` remains present as an experimental text diagnostic. Continuous
watch behavior is a supported `v0.4.0` contract and is not retroactively
part of the published `v0.3.0` release.

The `integrations/editors/vscode` package is a supported beta local VS Code
adapter over the shared Assura CLI, daemon, and editor-session JSON contracts.
Package test, build, doctor, and package-smoke commands gate local packaging
metadata, daemon visibility, one-shot fallback, and preview-only safe fixes. It
is not a marketplace release, does not start a hosted service, and does not
apply fixes implicitly.

Extension/API boundaries are now documented as a supported beta policy surface.
Current `extensions.*` entries are first-party config policies executed by
`assura check`; they are not a public plugin API, remote plugin loader,
shell-executed validator system, plugin marketplace, TypeScript plugin SDK, or
semver-stable Rust library API.

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

The `v0.3.0` archives were published and live-verified on 2026-07-02. Live
verification checked both the latest-release URLs and explicit `v0.3.0` URLs
for every archive and checksum file.

## Current Feature Surface

### Structure Validation

- Directory and file contracts from `.assura/config.yml`.
- Naming conventions, extension rules, line limits, size limits, child-count
  limits, markdown frontmatter checks, and markdown heading checks.
- Exclusion patterns for generated directories and local build output.
- Text, JSON, YAML, and agent output formats.

### Markdown Validation

- `markdown.lint_common` is a supported `v0.4.0` Rust-native common lint bundle for
  skipped heading levels, malformed heading marker spacing, duplicate headings,
  and multiple consecutive blank lines.
- `markdown.check_links` is supported `v0.4.0` local validation for relative
  Markdown file links, heading anchors, line anchors, and unrendered local file
  references.
- Markdown lint findings use stable `markdown_*` rule IDs and can use
  `markdown.rules.<rule_id>.severity` plus reasoned `assura-ignore`
  suppressions.
- Broad third-party markdownlint-compatible coverage remains future work until
  Assura has an MSRV-compatible dependency or external-binary contract.

### Agent Nudges

- `assura agent nudge` is a supported `v0.4.0` shared event-aware payload for local
  Codex, OpenCode, Claude, and Pi wrappers. It reports compact daemon health,
  changed-path findings, affected-reference context, and performance-gate
  reminders without adding per-agent validation commands.

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
- There are no package feedback CLIs, per-agent host-specific CLI entrypoints,
  or per-agent `--format` values in this release.

### Project Intelligence Queries

- `assura agent context`, `diagnostics`, `context-pack`, `show`, `search`,
  `missing-relations`, `expand`, `safe-fixes`, and `session` provide the
  supported local coding-agent entrypoint. They reuse the same content-query
  contracts and do not require MCP, remote access, or a daemon.
- `assura content agent-context`, `collections`, `instances`, `show`,
  `agent-query`, `context-pack`, `search`, `missing-relations`, `references`,
  and `expand` provide the supported modeled collection, keyword, relation,
  repository-reference, bounded context-pack, and bounded graph query surface
  over the local project-intelligence fact model.
- `assura content agent-context` emits the shared generic
  `assura.project-intelligence.agent-context.v1` schema for wrappers that need
  to discover diagnostics, safe-fix, graph/search, semantic, and code-symbol
  capabilities without creating per-agent command families.
- `assura content agent-query <capability> --format json` emits
  `assura.project-intelligence.agent-query.v1`, a shared request/response
  envelope for diagnostics, graph expansion, keyword search, semantic
  candidates, code-symbol relationships, and safe-fix fact summaries.
- `assura content references --target <path> --format json` reports inbound
  repository references before moving or deleting a path, and
  `--source <path>` reports outbound references from a changed source path.
- Object-mode `assura content context-pack --collection <name> --id <id>
  --format json` includes bounded `repository_references.inbound` and
  `repository_references.outbound` arrays for the modeled object's path.
- Keyword search is deterministic local text matching over indexed chunks and
  returns lexical scores for ranking. These scores do not decide validation
  correctness.
- `assura content semantic-search` is experimental, opt-in through
  `--enable-local`, and uses local candidate retrieval. Scores do not decide
  validation correctness.
- `assura content symbols` and `assura content symbol-refs` are experimental
  candidate-enrichment commands over configured modeled fields and optional
  provider evidence. The built-in Rust token baseline can resolve rough local
  declarations; missing providers preserve unresolved refs instead of failing
  validation.
- `assura editor session` exposes the current editor integration surface with
  `textDocument/diagnostics`, `textDocument/context`, and
  `textDocument/codeAction` JSON-line methods. It is local, does not require
  MCP or remote access, and does not claim full LSP server framing or editor
  marketplace packaging.
- `assura daemon status`, `start`, `stop`, `restart`, `doctor`, `logs`,
  `health`, `check-path`, and `references` are supported local daemon
  commands. In the `v0.4.0` beta increment, lifecycle commands manage a local
  process with versioned health, check-path, and repository-reference IPC,
  project-local `.assura/daemon/` status and log files, bounded log output, and
  one-shot fallback guidance. Broader daemon-backed workflows remain beta-track
  work and must keep stale-state safety before support promotion.
- Full LSP server packaging and MCP are not part of this
  content-query surface.
- Runtime schema or source model artifacts stored under `.assura/` must live
  under `.assura/models/**`; artifacts outside `.assura/`, such as `schemas/**`,
  remain valid project-relative paths.

### Safe Fixes

- `assura fix markdown --dry-run --format json` now defaults to `--rule all`
  and previews every supported deterministic Markdown safe-fix class for
  configured Markdown scopes.
- `assura fix markdown --apply --format json` applies the same supported
  safe-fix subset only after explicit opt-in and reports changed paths,
  applied fix IDs, skipped fixes, and rollback guidance.
- `assura fix markdown --rule trailing-spaces --dry-run --format json` emits
  `assura.safe-fix.markdown.v1` and reports proposed files and line fixes
  without writing.
- `assura fix markdown --rule trailing-spaces --apply --format json` applies
  the bounded Markdown trailing-space fix and reports changed paths, applied
  fix IDs, skipped fixes, and rollback guidance.
- `assura fix markdown --rule required-sections --dry-run --format json`
  previews deterministic missing-heading insertions for configured
  `markdown.required_sections`; `--apply` appends those headings and reports
  the same safe-fix audit fields.
- Omitting both `--dry-run` and `--apply` previews fixes without writing;
  every write path requires `--apply`.

### Custom Constraints

- `extensions.custom_constraints` is experimental and first-party.
- The initial supported custom constraint is `paired_file_exists`.
- Custom constraints execute through `assura check` and report normal
  `StructureViolation` records with `custom:<id>` rule names.
- Remote plugin loading, marketplaces, shell-executed plugins, and third-party
  Rust/TypeScript plugin APIs are not part of v0.4.0.

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
ASSURA_VERSION=v0.4.0 cargo xtask release-live
```

## Next

The parent post-beta capabilities program records `v0.4.0` release candidate
evidence for this beta increment. Assura still remains pre-1.0 beta software.
