---
id: goal-assura-claim-complete-v0-4-and-v1
type: goal
title: Assura claim-complete v0.4 and evidence-backed v1.0
status: active
created: 2026-08-11
owners:
  - assura-maintainers
---

# Assura Claim-Complete v0.4 And Evidence-Backed v1.0

## Objective

Make every capability promoted on the core Assura website a supported,
release-available CLI contract, publish that coherent product as v0.4, and
promote the CLI to v1.0 only after a measured multi-repository soak proves the
contract is stable.

The release sequence is deliberately two-stage:

1. **v0.4 claim-complete:** implementation, docs, website, evidence, and release
   artifacts agree.
2. **v1.0 contract-stable:** v0.4 survives 30 days, at least 50 agent sessions,
   three repositories, all four supported agent hosts, and a final 14-day
   public-contract freeze.

## Live Revalidation

The goal is valid as of 2026-08-11.

- The landing and documentation evidence gates pass locally.
- The release-readiness gate is blocked because the package and latest GitHub
  release remain v0.3.0 while newer supported surfaces are unreleased.
- `assura watch` is currently a one-shot check despite the website's continuous
  feedback story.
- Agent integration bundles exist, but host activation remains manual and the
  four-host support promise is not yet one managed lifecycle.
- Supported deterministic policy surfaces and future intelligence concepts are
  mixed together on public pages.
- Pull request 140 is blocked on Windows LS-Lint golden tests.

## User Certainty Bar

A user should be able to choose a supported capability from a core marketing
page, install the current release, run the exact command shown, and receive the
documented behavior without an experimental qualifier or a manual integration
step that the page omitted.

## Locked Decisions

- Ship v0.4 before v1.0.
- v1.0 is a CLI contract; the Rust library remains internal and is not a stable
  public API.
- Core marketing claims may reference only `supported` release surfaces with
  verified or measured evidence available in the promoted release.
- Agent activation is explicit and managed, not silently inferred.
- Agent onboarding gains an explicit managed-activation flag; it remains out of
  public command examples until the flag is implemented and tested.
- Supported activation covers Codex, Claude Code, OpenCode, and Pi.
- Semantic search, symbol intelligence, deeper dependency intelligence, a
  plugin SDK, MCP, full LSP, marketplace distribution, and automatic repair
  remain roadmap work until separately implemented and proven.

## Ordered Delivery

| Phase | Outcome | Measurable exit |
| --- | --- | --- |
| Claim and release contract | Public claims, support metadata, release availability, and Windows evidence agree | Zero core claims mapped to non-supported or unreleased surfaces; Windows golden tests pass |
| Watch and warm runtime | Continuous watch, daemon, cache, and bounded warm feedback are support-grade | Real change-triggered watch test; all five warm p95 budgets pass; fallback is observable |
| Managed host activation | Four harnesses install, activate, diagnose, update, and remove safely | Golden config and event tests for Codex, Claude Code, OpenCode, and Pi |
| Supported policy depth | Marketed deterministic project signals are supported and future intelligence is routed to roadmap | Every core page claim has a supported surface and behavioral evidence |
| v0.4 publication | The current binary, docs, website, checksums, and release notes are one release | Release smoke passes on Linux, macOS, and Windows; live artifact commands pass |
| v1.0 proof | The v0.4 CLI contract survives real agent work without incompatible change | 30 days, 50 sessions, 3 repos, 4 hosts, final 14-day freeze, zero unresolved release blockers |

## Scope

- Claim-to-evidence and release-availability enforcement.
- Real watch behavior and support-grade warm runtime diagnostics.
- Explicit managed activation for Codex, Claude Code, OpenCode, and Pi.
- Deterministic structure, Markdown, reference, severity, suppression, and
  agent-guidance surfaces required by core pages.
- Core page and docs alignment, release metadata, package versioning, and
  release proof.
- CLI public-contract isolation and the v1.0 soak record.

## Non-Goals

- A stable Rust library API in v1.0.
- Hosted services or remote policy execution.
- Shipping future intelligence features merely to remove a roadmap label.
- Claiming v1.0 before the time-based soak can be measured.

## Definition Of Done

### v0.4

- Every core product-page capability maps to a supported release surface.
- Website generation rejects experimental, planned, missing, or unreleased
  claim mappings.
- Watch is continuous and covered by deterministic filesystem-event tests.
- Warm feedback, daemon state, cache state, and fallback behavior are bounded
  and observable.
- Managed activation passes lifecycle and payload tests for all four hosts.
- Core pages contain no planned/experimental fallback language.
- Future capabilities appear only on roadmap-oriented surfaces.
- Current release binaries, checksums, notes, version metadata, and website
  examples agree on v0.4.
- Linux, macOS, and Windows release smoke tests pass.

### v1.0

- The v0.4 CLI contract records 30 consecutive days of use.
- Evidence includes at least 50 agent sessions across three repositories.
- Codex, Claude Code, OpenCode, and Pi each complete installation, activation,
  feedback, diagnosis, update, and removal proof.
- The final 14 days contain no incompatible CLI, config, schema, or managed
  integration change.
- No unresolved severity-one or release-blocking findings remain.
- Public Rust internals are isolated from the supported CLI contract.

## Validation

Run focused checks during each child task and the complete release boundary at
v0.4:

```bash
cargo fmt --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
cargo xtask website-demo-data --check
cargo xtask warm-loop-no-regression benches/history/warm-loop-current.json
cargo xtask release-readiness --format json
cargo xtask release-smoke
cargo run --quiet -- check --format json .
pnpm --dir website test:marketing
git diff --check
```

## Reviewer Blocking Criteria

Block release if a core claim is not available in the promoted binary, a
command is still a one-shot placeholder, host activation changes unmanaged
configuration, repeated feedback is unbounded, fallback behavior is hidden,
the website teaches non-executable syntax, Windows differs from Unix contract
behavior, or v1.0 evidence does not meet every stated count and duration.

## Trellis Tasks

- Parent: `.trellis/tasks/08-11-assura-claim-complete-release-program`
- Claim contract: `.trellis/tasks/08-11-claim-release-contract`
- Watch runtime: `.trellis/tasks/08-11-supported-watch-warm-runtime`
- Host activation: `.trellis/tasks/08-11-managed-agent-activation`
- Policy depth: `.trellis/tasks/08-11-supported-policy-depth`
- v0.4 release: `.trellis/tasks/08-11-publish-claim-complete-v0-4`
- v1.0 proof: `.trellis/tasks/08-11-prove-publish-v1`

## Progress Log

| Date | Iteration | Update | Evidence |
| --- | ---: | --- | --- |
| 2026-08-11 | 0 | Revalidated the plan against the live branch, release metadata, site claims, CLI implementation, warm budgets, agent bundles, and pull-request CI. Split execution into six measurable child tasks. Context level: not exposed. | PR 140; `cargo xtask release-readiness --format json`; `docs/data/release-surfaces.json`; `benches/history/warm-loop-current.json` |
| 2026-08-11 | 1 | Implemented and independently reviewed the local claim/release contract. Native behavioral tests use LS-Lint 2.3.1 with its Windows path fix; core marketing claims require supported status; strict candidate validation is read-only and rejects unreleased/future claims; tag publication is blocked on the strict claim and readiness gates; stable and release-candidate versions share SemVer parsing; active goals require structured Trellis ownership; and roadmap/task state is current. Hosted Windows proof remains open before this child can close. | `cargo test --test ls_lint_rule_coverage_tests --quiet`; `cargo test -p xtask --quiet`; `cargo xtask website-demo-data --check`; expected failures from `cargo xtask website-demo-data --released` and `cargo xtask release-readiness --format json` on unreleased v0.3.0 claims; `cargo xtask evidence`; `cargo xtask target-state`; `cargo xtask docs`; `cargo run --quiet -- check --format json .` |
