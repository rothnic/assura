---
id: goal-assura-claim-complete-v0-4-and-v1
type: goal
title: Assura claim-complete v0.4 and evidence-backed v1.0
status: archived
created: 2026-08-11
owners:
  - assura-maintainers
---

# Assura Claim-Complete v0.4 And Evidence-Backed v1.0

> Historical goal. Its implementation and review record remains useful, but it
> is not an active execution instruction. New work is owned by the
> [Maturity Execution Train](../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/execution-backlog.md).
> The current scope ledger is
> [Assura scope decisions](../analysis/assura-scope-decisions.md).

## Supersession

This goal's active-branch, PR, and release sequencing language was accurate for
its original claim-complete program. It is superseded for new work by the
Maturity Execution Train: establish current-source trust, reconcile the
release/support contract, evaluate safe initialization, and then prepare a
same-candidate release decision. It does not authorize publication or change
the retained four-host proof obligation. Historical entries below are evidence,
not instructions to revive already merged implementation work.

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
- Product examples on the landing site and in docs must be generated from the
  supported CLI renderer and schemas. Styling may adapt those semantics to the
  page, but text hierarchy, states, labels, and command behavior must not invent
  a parallel experience.
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
| Supported policy depth | Marketed deterministic project signals are supported and future intelligence is routed to roadmap | Every core page claim has a supported surface and behavioral evidence; generated CLI examples match the supported renderer at six breakpoints in both themes |
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
- Landing and docs terminal examples are renderer-derived, distinguish
  advisory review from blocking check output, fail the build when stale, and
  pass light/dark mobile and desktop wrap/overflow checks.
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
the website teaches non-executable syntax or a visually different output
hierarchy, Windows differs from Unix contract
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
| 2026-08-11 | 2 | Closed the claim/release-contract child after hosted Windows run `31539326739` passed all native LS-Lint golden cases, including the four prior path-semantics failures. The run exposed an unrelated dirty-status race in `assura-checkd`; that defect is carried into the support-grade watch and warm-runtime child rather than weakening the completed cross-platform claim proof. Context level: not exposed. | Windows job `93937933614`; `ls_lint_rule_coverage_tests` all passed; `native_lslint_golden_explicit_directory_target_*` and `native_lslint_golden_explicit_file_target_*` passed |
| 2026-08-11 | 3 | Implemented the continuous watch and warm-runtime child through its local proof boundary. Watch now stays resident, preserves requested file or directory scope, survives atomic file replacement, bounds edit bursts, reloads external config, exposes fallback and actionable findings, and terminates visibly on watcher failure. Changed-path checks conservatively fall back for cross-path policy. Public managed-daemon IPC now preserves validation exit codes and one schema. Internal status publication is ordered in both race directions and uses cross-platform atomic replacement. Context level: not exposed. Hosted Windows execution remains open. | 12 watch integration tests; 16 public daemon tests; 15 shared-state tests; 3 internal status tests; five warm budgets; successful `x86_64-pc-windows-gnu` full-CLI and companion-daemon cross-checks |
| 2026-08-12 | 4 | Closed the watch and warm-runtime child after hosted Windows exposed and verified fixes for inherited daemon handles and platform-specific atomic-replacement events. The full matrix now executes the managed daemon and cancellation paths on Windows and passes Linux, macOS, performance, installer, coverage, and adoption jobs. Added renderer-derived visual parity to the release contract so marketing output cannot diverge from the supported CLI experience. Context level: not exposed. | Rust CI run `31616404517`; Windows test job `94180233780`; performance job `94180233746`; Windows installer job `94180233801`; Linux job `94180233710`; macOS job `94180233865` |
| 2026-08-14 | 5 | Implemented explicit managed activation for Codex, Claude Code, OpenCode, and Pi. Independent review found and prompted fixes for non-Git Codex projects, partial lifecycle writes, and unsafe removal of marker-preserving user edits. Activation mutations are now transactional, reject symlink path escapes, preserve unmanaged host configuration, and expose generated, activated, verified, and conflicted states. Context level: not exposed. | 13 integration lifecycle and event tests; 18 agent-surface tests; 16 onboarding/config-merge tests; `cargo fmt --all -- --check` |
| 2026-08-14 | 6 | Aligned the product presentation with executable behavior. Homepage and focused examples now select exact Review, Check, and onboarding renderer lines from reproducible fixtures; Review is advisory, Check is blocking, and onboarding exposes generated, activated, verified, and conflicted state. Promoted the marketed deterministic Markdown, local-link, reference, severity/suppression, agent-guidance, event-nudge, doctor, and managed-integration contracts only after focused behavioral proof. Updated current docs, release metadata, command inventory, and target-state guards to the same support model. Context level: not exposed. | 75 focused policy/integration tests; 8 doctor/explain tests; 20 command-surface tests; `cargo xtask website-demo-data --check` validated 20 marketed surfaces and 54 YAML examples; `cargo xtask target-state` |
| 2026-08-14 | 7 | Resolved independent review findings at the product boundary. Project Intelligence now uses a real deterministic context-pack renderer, the policy tree is explicitly a fixture explanation backed by passing and failing checks, onboarding distinguishes inactive signals from deferred specialization, nested agent commands are release-inventoried, and production docs builds reject unreleased marketing surfaces. The browser may adapt spacing and container treatment, but it consumes the same renderer labels, hierarchy, states, thresholds, and actions as the CLI. Context level: not exposed. | `cargo test --workspace --all-features --quiet`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; 88 responsive light/dark browser checks; `cargo xtask website-demo-data --check`; `cargo xtask docs`; independent review agent `01a0028e-b00b-7a11-95b6-b0b2adc8efc7` |
| 2026-08-14 | 8 | Closed the preview delivery mismatch. The primary agent prompt and manual setup now share one pinned install of the exact implementation revision that generated the displayed output. The setup dialog exposes that revision without an experimental badge, and the docs build requires source delivery whenever a marketed capability is newer than the package, then requires the public installer once every claim ships. The homepage tree labels nonviolating files as observed fixture paths rather than invented per-path check passes. Follow-up independent review found no remaining blockers. Context level: not exposed. | Remote and local pinned Git install plus `assura review --help` proof for `b8f8375835095ce4f83b872c33b9d4e163ab283a`; byte-for-byte pinned Review, Check, onboarding, and intelligence artifacts; 95 responsive light/dark browser checks including 320/360/390 setup screenshots, target size, overflow, accessibility, and clipboard failure; 34 xtask tests; expected strict-release rejection for unreleased v0.3.0 marketing; independent review agent `01a002e9-650d-7180-9865-4c992bb30d2d` |

### Iteration 3 Context Review

- The active user request is implementation of the claim-complete release plan,
  with no reduction in marketed behavior to make support metadata pass.
- Independent review found false-success daemon exits, divergent schemas, both
  status-publication race directions, replaceable file subscriptions, and
  missing Windows execution evidence; the implementation now addresses each
  local defect.
- The core watch implementation and shared watcher-state module were split to
  satisfy their own 500-line project-health policy.
- Existing goal-execution, local-build, and performance-reporting skills cover
  the repeatable workflow; no new project skill is warranted for this slice.
- The next decision boundary is hosted Windows execution, followed by managed
  four-host activation rather than further watch-surface expansion.

### Iteration 6 Context Review

- The active request remains implementation of the claim-complete plan, with
  visual presentation treated as an executable product contract rather than a
  separate marketing mockup.
- The landing site may adapt spacing, color, and responsive composition, but
  renderer labels, hierarchy, states, values, and command behavior must come
  from supported CLI fixtures and stale generated artifacts must fail the docs
  build.
- Review remains the advisory branch/worktree radar, Check remains the blocking
  configured policy gate, Explain remains scoped policy evidence, and Doctor
  remains setup and inactive-capability diagnosis.
- Automatic repair, hosted orchestration, semantic inference as validation
  truth, a public plugin SDK, and full marketplace/LSP promises remain outside
  the core marketed contract.
- The next boundary is full local proof and independent review of this policy
  depth slice before publishing the refreshed PR preview.
