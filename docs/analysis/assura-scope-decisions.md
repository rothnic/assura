---
title: Assura scope decisions and release-surface ledger
status: active
updated: 2026-10-01
---

# Assura scope decisions and release-surface ledger

## Decision

Assura's growth lane is **executable repository conventions for
agent-assisted development**: explainable local policy, editable local
patterns, safe initialization, bounded feedback, and deterministic local/CI
gates. Existing behavior is retained under its current manifest classification
until a consumer review makes a removal or deprecation decision. This is not a
claim that every checked-source feature is publicly installable.

The checked source and latest published release are both `0.4.1`; the GitHub
release was rechecked and live-verified on 2026-10-01 from source commit
[`7ab1d82`](https://github.com/rothnic/assura/commit/7ab1d82634221817e568bf697fb56af03552d58e)
by [release workflow run 36799421108](https://github.com/rothnic/assura/actions/runs/36799421108).
The `v0.4.1` manifest rows are published claims.
`docs/data/release-surfaces.json` remains the authoritative command inventory;
this ledger adds the product decision,
published-version state, consumer references, and owning backlog card rather
than creating a competing inventory.

## Growth boundary

Invest in configured repository policy, naming and layout conventions,
required/forbidden paths, generated boundaries, local Markdown/reference
checks, project-owned patterns, agent initialization, hook lifecycle, and
bounded deterministic feedback. Integrate with native language tools and
maintainer-owned CI rather than replacing them.

Freeze new semantic-search, knowledge-platform, maturity-score, hosted
orchestration, remote pattern execution, marketplace, and arbitrary automatic
repair work. The freeze stops new growth work; it does not silently remove,
rename, or promote existing commands. Containment/removal work must be owned
by `Q05` or a separately approved consumer decision.

## Release-surface ledger

`v0.3.0` and `v0.4.1` identify published, installable release lines; `v0.4.1`
is the latest release. Evidence and consumer references are manifest-backed
paths, not adoption evidence. `P01`
owns this classification pass; the named follow-on card owns closing any
remaining contract gap.

| Manifest row / command | Scope decision | Published version | Implementation evidence | Consumer reference | Owner |
| --- | --- | --- | --- | --- | --- |
| `project-intelligence-local-surfaces` / `assura agent --help` | retained supported behavior; no new intelligence growth | v0.2.0 | `tests/agent_surface_cli.rs` | `docs/support-policy.md` | Q05 |
| `content-collections-querying` / `assura content --help` | retained supported deterministic query behavior | v0.2.0 | `tests/content_query_cli.rs` | `docs/support-policy.md` | Q05 |
| `markdown-safe-fix-preview-apply` / no standalone command | experimental; no promotion | v0.2.0 | manifest has no evidence state | `docs/support-policy.md` | Q05 |
| `markdown-common-lints` / `assura check --format json .` | supported deterministic policy | v0.4.1 | `tests/markdown_common_lint_tests.rs` | `docs/support-policy.md` | R06 |
| `markdown-local-link-checks` / `assura check --format json .` | supported deterministic policy | v0.4.1 | `tests/markdown_link_reference_tests.rs` | `docs/support-policy.md` | R06 |
| `markdown-rule-severity-suppression` / `assura check --format json .` | supported deterministic policy | v0.4.1 | `tests/markdown_suppression_severity_tests.rs` | `docs/support-policy.md` | R06 |
| `repository-reference-facts` / `assura content references --help` | retained supported deterministic references | v0.2.0 | `tests/content_runtime_references.rs` | `docs/support-policy.md` | Q05 |
| `repository-reference-checks` / `assura check --format json .` | supported deterministic policy | v0.4.1 | `tests/repository_reference_check_tests.rs` | `docs/support-policy.md` | R06 |
| `daemon-core-probe` / no standalone command | supported local process | v0.4.1 | `tests/daemon_cli_tests.rs` | `docs/support-policy.md` | R06 |
| `daemon-management-cli-preview` / no standalone command | supported local process | v0.4.1 | `tests/daemon_cli_tests.rs` | `docs/support-policy.md` | R06 |
| `structure-validation-cli` / `assura check` | retained supported core | v0.1.0 | `tests/structure_config_notation_tests.rs` | `docs/support-policy.md` | P01 |
| `structure-severity-contract` / no standalone command | retained supported core | v0.2.0 | manifest has no evidence state | `docs/support-policy.md` | Q01 |
| `ls-lint-migration` / no standalone command | retained supported compatibility behavior | v0.1.0 | manifest has no evidence state | `docs/support-policy.md` | P01 |
| `daemon-mode` / no standalone command | supported local process | v0.4.1 | `tests/daemon_cli_tests.rs` | `docs/support-policy.md` | R06 |
| `vscode-extension` / no standalone command | retained supported beta package; no marketplace expansion | v0.3.0 | manifest has no evidence state | `docs/support-policy.md` | Q05 |
| `extension-api-boundaries` / no standalone command | retained supported documentation boundary | v0.3.0 | manifest has no evidence state | `docs/extension-api-boundaries.md` | P01 |
| `agent-integration-lifecycle` / `assura agent integration activate ...` | supported managed local lifecycle; four-host proof retained | v0.4.1 | `tests/agent_integration_cli.rs` | `docs/support-policy.md` | R06 |
| `agent-ready-doctor-explain` / no standalone command | supported local diagnostics | v0.4.1 | `tests/doctor_explain_cli.rs` | `docs/support-policy.md` | R06 |
| `agent-daemon-nudges` / `assura agent nudge ...` | supported bounded local adapter | v0.4.1 | `tests/agent_surface_cli.rs` | `docs/support-policy.md` | R06 |
| `agent-guidance-contracts` / `assura check --format json .` | supported deterministic policy | v0.4.1 | `tests/agents_md.rs`, `tests/skill_contract.rs` | `docs/support-policy.md` | R06 |
| `agent-onboarding` / `assura agent onboard ...` | supported safe initialization | v0.4.1 | `tests/project_intelligence_onboarding.rs` | `docs/support-policy.md` | A01 |
| `compact-review` / `assura review` | supported advisory radar | v0.4.1 | `tests/project_review_cli.rs` | `docs/support-policy.md` | R06 |
| `path-explanation` / `assura explain apps/web/src` | supported policy explanation | v0.4.1 | `tests/explain_pattern_scope_cli.rs` | `docs/support-policy.md` | R06 |
| `configured-policy-gate` / `assura check` | retained supported core | v0.1.0 | `tests/real_project_agentic_feedback_tests.rs` | `docs/support-policy.md` | P01 |
| `codex-agent-report` / `assura check --format agent --agent codex .` | supported shared adapter | v0.4.1 | `tests/agent_surface_cli.rs` | `docs/support-policy.md` | R06 |
| `explicit-review-base` / `assura review --base main` | supported advisory radar | v0.4.1 | `tests/project_review_cli.rs` | `docs/support-policy.md` | R06 |
| `finding-history` / `assura review --format agent` | supported advisory radar | v0.4.1 | `tests/project_review_cli.rs` | `docs/support-policy.md` | R06 |
| `cache-observability` / `assura cache status --format json` | supported cache observability | v0.4.1 | `tests/project_review_cli.rs` | `docs/support-policy.md` | R06 |
| `continuous-watch` / `assura watch --format json` | supported bounded feedback; platform closure remains R01 | v0.4.1 | `tests/watch_cli.rs`, `src/cli/watch_tests.rs` | `docs/support-policy.md` | R01 |
| `cold-ls-lint-comparison` / no standalone command | retained measured evidence, not a universal performance claim | v0.3.0 | `website/public/data/performance/current.json` | `docs/performance.md` | R02 |
| `warm-session-performance` / no standalone command | retained measured evidence, not a universal performance claim | v0.3.0 | `website/public/data/performance/current.json` | `docs/performance.md` | R02 |

## Four-host support decision

No support narrowing is authorized by this card. Codex, Claude Code, OpenCode,
and Pi remain four independent proof obligations for any release that retains
the managed-integration/onboarding promise. A proposal to narrow that matrix
must name the removed host, the consumer impact, docs/manifest/test updates,
and a maintainer decision; it cannot be inferred from missing local proof.

## Historical supersession

The claim-complete goal records completed historical implementation evidence,
but its active branch and release instructions are superseded for new work by
the Maturity Execution Train backlog. Historical claims remain auditable;
release publication is still owned by `R06` and requires separate authority.
