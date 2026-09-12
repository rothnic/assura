# Compact Feedback Implementation Plan

> **For agentic workers:** Use `superpowers:executing-plans` after explicit
> start. Implement card by card; use the Assura independent review brief.
> Checkbox steps track acceptance and do not create a second queue.

**Goal:** Tiny configurable statistics that help agents self-correct, with
repeated full-context work removed from the feedback path.

**Architecture:** Git/policy facts feed bounded cached snapshots. A shared
deterministic scheduler selects one short line; thin adapters deliver it.
Reuse nudge, policy engine, daemon and review heatmap.

**Tech stack:** Existing Rust CLI/config/daemon, Git, Python Codex adapter,
generated host bundles, integration tests and performance tooling.

**Spec:** [feedback-contract](feedback-contract.md).
**Execution:** [start and closure](execution.md).

## Global constraints

- Planning only until explicit start; no product implementation has begun.
- Routine injection: one line, <=256 UTF-8 bytes including wrapper, default
  spacing 600 seconds, <=4 messages and <=1024 bytes per rolling hour.
- Configurable entry/clear thresholds, material steps, bounded reminders,
  periodic mode and off switch. Cooldown expiry alone never repeats a signal.
- No foreground full scan/history walk/model/network fetch. Cache/async
  correctness, missing-data semantics and actual host delivery are acceptance.
- Preserve existing critical policy delivery, hard checks and required
  performance gates. Keep LS-Lint comparison priority and truthful labels.
- No framework replacement, generic orchestration, semantic productivity
  scoring, automatic repair, proposed-change feature or turn attribution.
- Newly proposed paths/flags/types below are implementation destinations;
  they do not claim those interfaces already exist.

## Backlog index

This is the only new card-status table. Resume an existing candidate before
opening another. Independent review/validation can overlap.

| Card | Outcome | Depends on | State | Candidate / result |
| --- | --- | --- | --- | --- |
| CF01 | Cheap, bounded existing feedback and convergent instructions | None | pending | Not started |
| CF02 | Cached Git trajectory facts with honest coverage | CF01 | pending | Not started |
| CF03 | Configurable one-line scheduling and async host delivery | CF02 | pending | Not started |
| CF04 | Performance evidence, project trial and instruction handoff | CF03 | pending | Not started |

Inspected at `c957f89`: maturity A06 owns the existing bounded-feedback baseline;
R01 owns watch correctness; R02/R03 own comparison evidence/many-scope repair.
The July compact-review task already implemented `assura review`. Reuse that
work; refresh only relevant current state and link follow-up proof when it lands.
Do not reopen all cards, copy private packets or change old statuses just to
register this plan. Unresolved R01 coverage cannot be assumed fixed by caching;
Git-only facts and batch fallback deduplication can progress independently.

## CF01 — Make existing feedback cheap and bounded

**Outcome:** Remove repeated work and oversized output before adding signals.
Ship the small instruction corrections with this product fix.

**Modify:** `src/cli/agent_nudge.rs`, `agent_nudge_helpers.rs`,
`agent_nudge_cooldown.rs`; `src/cli/content_query/agent_query.rs`,
`context.rs`; `src/cli/check/prepared.rs`; `src/daemon/mod.rs`;
`.codex/hooks/assura-agent-nudge.py`; relevant generated carriers in
`src/cli/agent_integration_templates.rs` and `agent_integration_bundle.rs`.
Confirm the generated carrier; editing the repository hook alone is insufficient.

**Test:** `tests/content_query_cli.rs`, `tests/agent_surface_cli.rs`;
new focused `tests/agent_feedback_delivery.rs` for added delivery cases.

**Interfaces:** Preserve public nudge/check surfaces. Add a private batch-check
operation whose changed paths share one full fallback and coverage metadata.
Query loading requests only needed data. Shared rendering supplies compact text
to adapters; explicit reports remain available.

- [ ] Add a failing `keyword_search_respects_limit` regression: one match for
  `--limit 1`, zero for `--limit 0`, a bounded large match, omitted counts and
  valid UTF-8/JSON. Check existing zero-limit semantics first; preserve or
  explicitly version a demonstrated conflicting consumer contract.

  ```sh
  cargo test --test content_query_cli keyword_search_respects_limit -- --exact
  ```

  Expected assertions on a deliberately multi-match fixture:

  ```rust
  assert_eq!(output["matches"].as_array().unwrap().len(), 1);
  assert!(output["omitted"].as_u64().unwrap() > 0);
  ```

- [ ] Add negative controls: five changed paths cause only one actual full
  fallback; unchanged reads do not build the query graph; policy changes
  invalidate reuse. Use an actual-validation observer/counter in tests;
  latency alone cannot prove that work was skipped safely.
- [ ] Implement count/byte limits, shared batch reports and demand-driven
  context. Missing coverage remains pending/unknown; explicit full checks stay
  authoritative. Retain deletion/sibling/reference dependencies.
- [ ] Render routine hook output <=256 bytes, one selected item. Keep newly
  critical notifications visible under existing semantics and test precedence.
  Remove repeated metadata and log paths from automatic text.
- [ ] Apply CF01 instruction edits from the table below; include this planning
  package in the same candidate instead of a separate reconciliation PR.
- [ ] Run focused red/green controls, then affected suites. Confirm the named
  tests ran (zero selected tests is not proof). Run current applicable Rust
  validation once the candidate settles, avoiding duplicate nested suites:

  ```sh
  cargo test --test content_query_cli --test agent_surface_cli --quiet
  cargo test --test agent_feedback_delivery --quiet
  cargo xtask pr
  ```

- [ ] Measure before/after actual hook idle, single edit, five-file burst and
  config-edit paths on the same idle host and policy-rich repository. Record
  startup, bytes, binary identity, counts and raw samples. September 11 reused
  source-archive probes remain diagnostics, not fresh release evidence.
- [ ] Commit, independently review, resolve concrete findings, satisfy applicable
  final-candidate CI and pursue authorized integration. End with integration or
  the precise held action; no new checkpoint-reconciliation campaign.

## CF02 — Collect trajectory facts once and reuse them

**Outcome:** Bounded Git facts with a reusable cached snapshot and explicit query.

**Files:** reuse/extract `src/cli/project_review/heatmap/git.rs` and `heatmap.rs`;
create private `src/cli/agent_trajectory.rs` with `agent_trajectory/git.rs` and
`agent_trajectory/snapshot.rs` only when separate responsibilities warrant it;
wire `src/cli/mod.rs`, `src/cli/agent_nudge.rs`, `src/cli/agent_args.rs`,
`src/cli/agent.rs` and existing daemon state.
**Tests:** new `tests/agent_trajectory_cli.rs`; reuse relevant temp-Git patterns
from `tests/project_review_cli.rs`, not another general test framework.

**Interfaces:** `TrajectorySnapshot` carries schema/generation, repo/worktree
identity, local ref/SHA, capture time, coverage, optional window and pending
metrics. Unknown values are null/absent. Add optional `trajectory` to nudge JSON
and a new `--delivery inspect` option on the existing nudge command.

- [ ] Test minute/commit windows, normal merge/squash, rename/deletion,
  generated/binary/untracked paths, category overlap, shallow history, absent
  refs, detached HEAD, force-push and multiple worktrees. Pin the clock.

  ```text
  Main: source commit +5/-2, then coordination-only commit +20/-1
  Branch: source commit +8/-0; worktree adds another +2/-0
  Expect: main commits=2, source +5/-2; pending commits=1, net lines +10/-0
  Squash branch into main and retain original candidate ref
  Expect: no false pending-work alert from ancestry difference alone
  ```

- [ ] Implement batched first-parent collection with configured caps and honest
  coverage. No per-commit subprocess loop or untracked-content LOC scan. Keep
  gross history separate from final pending diff; unknown squash equivalence
  suppresses the automatic signal.
- [ ] Implement atomic bounded snapshots and incremental invalidation. A cache
  hit must not repeat history; changed ref/config/window bucket refreshes
  affected facts. Test corrupt state, clock rollback and stale generations.
- [ ] Add explicit inspect. New automatic stats remain disabled until CF03.
  Cold inspect may wait within its bounded timeout; hooks cannot use that path.

  ```sh
  cargo test --test agent_trajectory_cli --test project_review_cli --quiet
  ```

  Exercise the newly implemented inspect option through that integration test;
  it is not a runnable public command before CF02 lands.

- [ ] Measure cold/delta/cache-hit latency, peak memory and CPU under 1/4/8
  simultaneous worktrees. Prove one in-flight refresh per key, bounded state,
  no remote fetch and no private content upload.
- [ ] Review, satisfy applicable gates and integrate. Host-turn attribution
  and semantic task evaluation stay outside this card.

## CF03 — Configure delivery and keep the hook path fast

**Outcome:** All automatic selection/cadence/async behavior in feedback-contract.

**Modify/create:** `src/config/config.rs`, new
`src/config/config/agent_feedback.rs` and its validation/notation registration;
`src/cli/agent_args.rs`, `src/cli/agent.rs`, `src/cli/agent_nudge.rs`,
`src/cli/agent_nudge_cooldown.rs`, new private
`src/cli/agent_nudge_delivery.rs`; daemon worker lifecycle; repository hook and
`src/cli/agent_integration_templates.rs`, `agent_integration_bundle.rs`,
`agent_integration_host.rs`.
**Tests:** `tests/agent_feedback_delivery.rs`, `tests/agent_surface_cli.rs`,
`tests/git_hook_lifecycle.rs` and affected generated-bundle tests.

**Interfaces:** typed `AgentFeedbackConfig` implements the exact contract keys.
`DeliveryDecision` is emit(context) or quiet(reason), determined by config,
snapshot, event, clock and persisted episode/budget state. Adapters call new
`--delivery automatic`; Rust selects/renders, adapters map/wrap events.

- [ ] Test all config switches and rejected combinations. Unknown keys cannot
  silently fall back. Expose effective settings on inspect; retain unrelated flags.
- [ ] Implement fake-clock, table-driven delivery tests:

  ```text
  pending 29m -> quiet; 30m -> one line; 31m -> quiet
  new main SHA, same episode -> quiet
  cooldown expiry alone -> quiet
  one eligible reminder on later event -> one line; no second reminder
  exhausted budget -> quiet; expired bucket still needs eligible event
  clean <60s then dirty -> same episode; sustained clean/reentry -> new episode
  stale/incomplete/missing/obsolete snapshot -> quiet + refresh request
  periodic boundary during idle -> no wake; next event -> at most one line
  off -> no trajectory collection/injection; explicit inspect available
  new critical finding plus stats -> compact critical precedence, semantics preserved
  ```

- [ ] Persist atomic episode and delivery state independent of raw counts.
  Session restart must not cause repeat spam. Concurrent calls cannot duplicate
  sends or overspend caps. Enforce the whole injected-text byte ceiling.
- [ ] Wire single-flight async refresh, event coalescing, worker-tree timeout,
  lease recovery, obsolete-result discard and background CPU/I/O limits. Read
  the current snapshot, not an ever-growing JSONL. Do not spawn per tool call.
- [ ] Verify worker survival through actual host lifecycle before enabling.
  Start with Codex; maintain shared contracts and supported adapters. Unsupported
  async hosts stay quiet with explicit query fallback.
- [ ] Test adapter entry and installed/generated artifacts for idle reads,
  edits, failures, critical findings, restart and concurrency. A forced
  two-second collector cannot make the foreground hook await completion.

  ```sh
  cargo test --test agent_feedback_delivery --test agent_surface_cli --quiet
  cargo test --test git_hook_lifecycle --quiet
  cargo xtask pr
  ```

- [ ] Measure actual adapter entry end to end: unchanged p95 <=25 ms, <=50 ms
  under slow-collector injection; warm structural edit target <=100 ms with
  coverage explicit. Use >=100 timed repetitions after separate warmup, report
  idle and four-worker cases separately. A process-floor miss requires carrier
  work or an honest held gate; do not rename a microbenchmark to claim success.
- [ ] Review failure/coverage/budget cases, pass required gates and integrate.
  Demonstrate real host receipt before claiming support; mocked/generated output
  alone does not establish an operational integration.

## CF04 — Prove the benefit and hand off truthful defaults

**Outcome:** Current performance evidence, explicit Assura project trial and
instructions that keep feedback small and useful.

**Files:** `xtask/src/main.rs` existing warm-loop/performance routes;
`tests/performance_report_contract_tests.rs`; current history/export paths via
the performance-reporting skill; `website/src/components/marketing/performance-proof.astro`,
`website/src/pages/performance/index.astro`; `.assura/config.yml` for opt-in;
instruction/spec routes below. Website deployment is not implied.

- [ ] Extend existing benchmarks with actual installed-hook idle/edit/config
  cases plus background resource measurements. Separate naming-equivalent
  LS-Lint work from rich-policy feedback; include representative real repos.
- [ ] Compare native released LS-Lint for equivalent cold and explicit-target
  operations. Preserve pinned history and add current upstream after verifying
  its release. Balanced alternating samples, distributions, ties and misses
  remain visible. Run existing no-slower/native/warm gates.
- [ ] Bind every published number to operation/build/cohort/host/method. Use
  “Persistent-session benchmark” for that measured operation; distinguish the
  released build from development source. Never promote local probes into
  release proof or leave installed-hook cost implied by a different benchmark.
- [ ] Explicitly opt Assura into compact feedback. Run a fixed six matched task
  pairs: small unintegrated edits, coordination-only activity, healthy delivery;
  feedback on/off with balanced order and unchanged acceptance. Include docs-only
  work as a negative control. Record accepted/integrated outcomes, time/cost,
  bytes and nuisance reports. At trial end decide keep/tune/off; no claim of
  statistical generality and no open-ended evaluation program.
- [ ] Finish instruction/spec/guidance edits below. No default context dump,
  required acknowledgment, productivity score or framework replacement.
- [ ] Review evidence/copy, validate affected website/docs and product gates,
  pursue authorized integration. Name any publication hold; do not claim live
  website changes until they are actually deployed under appropriate authority.

## Exact instruction and backlog updates

| Timing | Surface | Required change |
| --- | --- | --- |
| Planning now | `AGENTS.md` | One route to this skill, no embedded protocol. |
| Planning now | This skill, contract, execution and backlog | Durable start command and pending sequence; no runtime start. |
| Planning now | `.trellis/tasks/09-12-compact-feedback-plan` | Thin planning record/context pointers; no duplicated card statuses. |
| CF01 | `.agents/skills/assura-goal-execution/references/source-pointer-lifecycle.md` | Replace unconditional pointer reconciliation with immutable as-of evidence and refresh-before-use; preserve actual integration checks. A commit never needs to contain its own final SHA. |
| CF01 | Same skill's `references/continuation-control.md` and `execution-control-plane.md` | One candidate/next action; remove checkpoint-only commits and repeated whole-ledger rewrites, retain ownership/review/evidence/authority. |
| CF01 | `.agents/skills/assura-goal-execution/SKILL.md` | Route compact-feedback scope here; replace repeated instruction-improvement cycles with bounded diagnosis and one durable correction when warranted. |
| CF01 | `/Users/nroth/.codex/skills/assura-orchestration/SKILL.md` | Prepare matching minimal edits to reconciliation/iteration guidance. Apply only on an authorized host; otherwise retain the patch and state the limit. No fleet-wide settings rollout. |
| CF01–CF03 | `.trellis/spec/assura/codex-agent-feedback.md`, `agent-harness-hooks.md` | Update implemented payload, scheduling, coverage, worker lifecycle and error contracts with each change; retain explicit check semantics. |
| CF03 | `.agents/skills/assura-agent-harness-hooks/SKILL.md` and `references/harness-hook-matrix.md` | Require shared scheduling/rendering, bounded async collection, actual delivery proof and honest unsupported modes. Keep host detail in the matrix. |
| CF04 | `.agents/skills/assura-performance-reporting/SKILL.md` | Require actual adapter latency, bytes/task/hour, background CPU/I/O and equivalent LS-Lint rows; label resident microbenchmarks precisely. |
| CF04 | `src/cli/agent_onboarding_templates.rs` and related content/handoff templates | One short label legend, settings location and details query; no repeated instruction to acknowledge or obey heuristics. |
| Real transitions | This backlog; links from A06/R02/R03 evidence where relevant | Record implementation/review/integration proof once; no reset of accepted cards or competing queue snapshot. |

## Completion

Before integration: accepted contract, meaningful negatives, source/binary
identity, scoped independent review, applicable current-candidate checks and
existing authority. Use validation routing without duplicate unchanged suites.
Keep failed gates visible and fix their causes.

At the end: CF01–CF04 terminal, actual hook measurements, proved byte/cadence/
concurrency controls, correctly scoped claims, and owned candidates integrated
or explicitly held/preserved. Honor user pause/scope changes. Do not invent
additional cards to keep execution running.
