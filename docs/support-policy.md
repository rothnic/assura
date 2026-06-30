---
title: Support Policy
status: active
---

# Support Policy

This policy applies to Assura pre-1.0 releases.

## Support Levels

| Surface | Level | Policy |
| --- | --- | --- |
| `assura check` structure validation | Supported | Bugs that produce incorrect pass/fail results are release blockers. |
| `assura check --format json` and `--format yaml` | Supported | Schema changes must be called out in release notes before 1.0. |
| `assura check --format advice` and `--format status` | Supported | Guided and compact output must stay deterministic enough for local hooks and agent tools. |
| `assura check --format agent` | Supported | Agent JSON shape must remain deterministic and documented. |
| `--agent codex` delivery | Supported adapter | Delivery may depend on user-approved Codex hooks, but it must not require a separate CLI. |
| `assura init` | Supported | Starter config output must be valid and self-checkable. |
| `assura status --format json` | Supported | JSON summaries must remain usable for automation. |
| `assura migrate` for complete LS-Lint 2.3 config semantics | Supported | Invalid LS-Lint config shapes and unsupported rule syntax must fail clearly. CLI drop-in parity is out of scope. |
| `assura hooks` for local git hooks | Supported local workflow | Hooks must be opt-in and local to a checkout. |
| `assura quality plan` | Supported local workflow | Quality-scope planning must stay config-backed and deterministic. |
| `assura performance-report` | Supported evidence command | Claims must cite checked benchmark or CI artifacts. |
| `assura fix markdown --dry-run --format json` | Experimental safe-fix preview contract | Dry-run output must report proposed bounded writes without modifying files. |
| `assura fix markdown --apply --format json` | Experimental safe-fix apply/audit contract | Apply output must report changed paths, applied fix IDs, skipped fixes, and VCS-first rollback guidance. |
| `assura agent` | Supported local agent project-intelligence surface | JSON-default commands for context, diagnostics, context packs, search/show/expand, missing relations, safe-fix previews, and local sessions must delegate to the shared content-query contracts. MCP or remote access is not required. |
| `assura editor session` | Supported local editor project-intelligence surface | JSON-line request/response loop with LSP-shaped diagnostics, context, and code-action preview methods. It must reuse shared content-query contracts, reload conservatively, avoid implicit writes, and must not require MCP, remote access, or a hosted language server. |
| `assura content` query commands | Supported first project-intelligence query surface | JSON output for generic agent context, context packs, agent-query envelopes, collection, relation, keyword, semantic-candidate, code-symbol, bounded graph, and JSON-line session queries must remain deterministic enough for agent use. Semantic scores and baseline code-symbol evidence are candidate context only and do not decide validation correctness. |
| `.assura/models/**` model artifact layout | Supported project-intelligence layout policy | Model artifacts stored under `.assura/` must live under `.assura/models/**`; projects may still keep artifacts outside `.assura/` when that better fits their repository. |
| `assura content session` | Supported local project-intelligence session | JSON-line request/response loop for repeated local agent/editor queries. It reloads conservatively when project files change and does not apply fixes or require a hosted daemon. |
| `assura info` | Experimental diagnostic | Text output can change before a documented automation contract exists. |
| `extensions.custom_constraints` | Experimental first-party | Specialized constraint execution only. Common repository relationships should use `structure` captures, `exists:1`, `needs`, and `provides`. Breaking changes are allowed before 1.0 with release-note disclosure. |
| `extensions.support_matrices` | Experimental first-party | Public command/API classification checks for repository policy. Rows must use `supported`, `experimental`, `internal`, `roadmap`, or `unsupported`. Breaking changes are allowed before 1.0 with release-note disclosure. |
| `extensions.manifest_semantics` | Experimental first-party | Configured Cargo manifest metadata checks for package policy, publish status, descriptions, keywords, and declared binaries. It does not replace Cargo, dependency hygiene, license/source policy, or semver tooling. |
| `extensions.test_relationships` | Experimental first-party | Configured source/test evidence, ignored/manual test, and fixture-family ownership checks. It does not claim coverage percentage or semantic test adequacy. |
| `extensions.module_topologies` | Experimental first-party | Configured Rust module-family ownership, root existence, and bounded public export classification checks. It does not provide a full Rust parser, public API semver guarantee, or refactoring mandate. |
| `extensions.docs_lifecycles` | Experimental first-party | Configured documentation lifecycle, historical-reference exception, and deterministic stale-claim evidence checks. It does not provide a broad natural-language classifier or automatic archival. |
| `assura watch` | Experimental | Do not advertise as release-grade until watch-mode tests and docs exist. |
| Internal Rust APIs | Unstable | Public Rust module visibility in `src/lib.rs` is for binaries, tests, and benchmark harnesses unless a row explicitly promotes the API. No compatibility guarantee before 1.0. |

## Unsupported Surfaces

Do not document these as supported:

- package feedback CLIs such as `assura-codex-feedback`;
- per-agent host-specific command entrypoints;
- per-agent `--format` values such as `--format codex-hook`;
- remote plugin loading;
- plugin marketplaces;
- shell-executed validation plugins;
- hosted telemetry or dashboards;
- required MCP or remote agent transports for local project-intelligence usage;
- full LSP server framing or editor marketplace packages as current supported
  editor behavior;
- automatic repair;
- dependency graph validation as a current release feature.
- required code-intelligence providers for normal validation.

## Issue Triage

Use these labels or equivalent GitHub issue language:

- `release-blocker`: install, crash, data-loss, or incorrect validation
  behavior on a supported surface.
- `compatibility`: LS-Lint migration, output shape, install archive, or
  documented platform mismatch.
- `docs`: stale command, unclear limitation, broken example, or missing
  release-note disclosure.
- `experimental`: custom constraint or watch-mode behavior that is not yet a
  supported release contract.
- `roadmap`: accepted idea that needs a future goal before implementation.

## Response Targets

- Release blockers: triage within two business days.
- Compatibility and docs issues: triage within one week.
- Experimental and roadmap issues: batch into roadmap planning unless they
  expose a security or install blocker.

These are maintainer targets, not a paid service-level agreement.

## Breaking Changes Before 1.0

Assura can make breaking changes before 1.0 when they are intentional and
documented. Every breaking release note must identify:

- affected command, config field, output field, or archive name;
- replacement path, if one exists;
- migration command or manual edit;
- validation command that proves the new behavior; and
- whether old behavior is removed, deprecated, or experimental-only.

## Security

Report security issues through GitHub private vulnerability reporting when
available, or by opening a minimal issue that does not include exploit details.
See [`SECURITY.md`](../SECURITY.md) for the reporting path. Security fixes can
bypass normal roadmap sequencing when necessary.

## Maintainer Completion Rules

A release PR cannot close if:

- it advertises unsupported surfaces;
- issue/support policy language conflicts with release notes;
- install docs name assets that the release workflow does not publish;
- agent feedback docs drift from `assura check --format agent`; or
- Codex delivery is described as anything other than `--agent codex` on the
  shared agent format.
