---
id: goal-assura-maturity-execution-train
type: goal
title: Assura maturity execution train
status: active
created: 2026-09-11
owners:
  - /root
related:
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/task.json
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/backlog.json
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/recovery-plan.md
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/orchestration-plan.md
  - ../../.agents/skills/assura-goal-execution/SKILL.md
  - ../../.agents/skills/assura-goal-execution/references/execution-control-plane.md

---

# Assura maturity execution train

## Objective

Advance the canonical Trellis maturity task through one explicitly owned,
evidence-bound slice at a time. Continue through investigation, implementation,
validation, independent review, current-base integration, post-merge
reconciliation, and terminal cleanup; never end a continuation at a status
report while an authorized repair, review, integration, or cleanup action can
continue.

This is an execution-control goal, not a claim that process, evaluator,
canary, or documentation work satisfies a product card.

## Current gap

The latest reset fetched `origin/master=c9ac2a84a93dce752ec7b5216ad87639ba04dee8`
after PR #294 merged the reviewed execution-control-plane documentation. This
is the current checkpoint for this continuation; every later phase must fetch
again before using it. The revision-pinned ledger remains 32 items, zero
ready-pending rows, five unfinished rows, and three held rows: A07 is active,
W03 is verified, and R01, W02, and F01 retain their separate evidence or
authority holds. The 4560c710 candidate and packet are candidate-base/no-credit
metadata after this source advance; no current candidate, screening allocation
or product acceptance is claimed. The root unknown path, foreign dirty
worktree, stale registrations, historical branches and unfavorable evidence
remain preserved as ownership boundaries.

The reusable execution-control-plane reference defines the state machine,
checkpoint fields, layered disclosure, validation budget, measured VPS test,
review contract, post-merge push-workflow fence and terminal topology fence.
It is a phase reference, not a second goal or a substitute for the task
ledger. The coordinator's next action is to refresh source/release/tag/PR/CI/
topology and the ledger, inspect active and held work, then keep one explicitly
owned recovery or preparation action live. If A07 screening is separately
authorized, rebuild and rebind at the refreshed source before allocation;
otherwise continue an authorized recovery slice. Never stop at an empty
pending queue.

PR #292's exact-head applicable checks passed, but its separate push-triggered
Rust CI run reported a macOS `watch_stops_cleanly_without_runtime_artifacts`
SIGINT failure and cancelled the Ubuntu/Windows matrix siblings. A focused
local rerun passed once; the hosted failure remains an unresolved diagnostic,
not a green signal and not a reason to weaken or blindly retry the gate.
PR #294's reviewed process slice then passed both its exact-head and
post-merge push-triggered checks at `c9ac2a84`; this confirms the new
reconciliation fence is executable, but it does not repair the older R01
failure. A configured post-merge failure, cancellation or missing result keeps
the specific recovery route open.

## User certainty bar

Every continuation must make these answers auditable:

1. What is the fetched source SHA, ledger state, owner, checkout, phase, and
   exact next action?
2. Which observable contract and negative controls prove the current slice?
3. Which review, local, hosted, performance, authority, or cleanup gate is
   still open, and what independent work continues meanwhile?
4. Why is a card still active, verified, held, or done without relying on a
   passing process check or a skipped/zero-test job?

## Scope

- Refresh source, release/tag availability, PR/CI state, ledger, ownership and
  topology at every reset, card boundary, merge, and handoff.
- Route active, implemented, verified, and merge-ready candidates before
  pending rows; choose one named owner and one concrete next action.
- Use clean explicit worktrees, focused red/green checks, exact evidence,
  independent review, current-base integration, and verified cleanup.
- Use layered context: keep `AGENTS.md` a universal router and load the task,
  card packet, and phase reference only when that decision requires them.
- Measure validation placement and use the VPS only after fresh capacity,
  disk, toolchain, and owned-job checks; retain hosted and platform-specific
  gates as final proof.

## Non-goals and authority boundaries

- Do not weaken thresholds, hide failures, retry an unchanged failing method,
  or treat skipped, unavailable, zero-test, or scope-uncertain checks as pass.
- Do not mutate, stash, reset, commit, delete, or absorb unknown/user-owned
  dirt. Do not create duplicate goals for this active task.
- A07 preparation is separate from A07 implementation and acceptance. Its
  identity freeze, source-only no-credit canary, private two-condition packet,
  six-handle binding, 30-cell matrix, isolated protocol review, screening,
  holdout, follow-up feature, and final acceptance are separate gates; none
  grants screening or acceptance authority by itself.
- R01, W02, W03 publication, F01 outreach, release, deployment, invitation,
  protection changes, and CI-infrastructure changes retain their named holds
  or require their own explicit authority.

## Operating contract

1. **Reset and route.** Run the workflow gate, fetch `origin/master`, record
   release/tag and PR/CI availability, run the revision-pinned ledger and
   topology report, and inspect actual owner/session liveness. Preserve dirty,
   unreadable, prunable, and historical state rather than normalizing it away.
2. **Own and prove.** Work from a clean checkout based on the recorded source;
   keep one owner per candidate and commit coherent work before switching,
   yielding, reviewing, or handing off. Start with cheap structure/scope and
   focused red tests, then run only the smallest applicable validation tier.
3. **Review and correct.** Freeze the diff and public evidence, dispatch the
   Assura review brief, and continue independent authorized preparation while
   the reviewer or CI runs. Turn vague feedback into a contract, location,
   failure scenario, and smallest verification; fix accepted findings as a
   delta, rerun affected gates, and obtain scoped rereview.
4. **Integrate and reconcile.** Merge only reviewed current-base work with all
   applicable local/hosted/performance gates passing and no unresolved
   authority or zero-test issue. Fetch again, prove reachability and tree
   equality, rerun the ledger, run strict topology, and remove only the exact
   clean owned branch/worktree or record a verified archive.
5. **Continue.** If no pending row is executable, advance the active recovery
   or preparation slice. If an action is externally held, record its contract,
   evidence, owner, smallest resolution, and independent work that continues;
   do not convert that hold into a whole-goal stop.

## Definition of done

The goal is complete only when every authorized card outcome is proven by its
observable packet, review findings are resolved, required local and hosted
proof covers the final source, merged reachability is verified, and every
goal-owned branch/worktree is merged or verified-archived and clean. A process
merge, passing canary/evaluator, or protocol `PASS` alone never marks a card
done.

## Validation matrix

Run from the current clean owned checkout, substituting the task path as
needed:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex --task \
  .trellis/tasks/09-04-maturity-portfolio-strategy
python3 ./.agents/skills/assura-goal-execution/scripts/audit-context-routing.py \
  "$repo_root" .trellis/tasks/09-04-maturity-portfolio-strategy
/Users/nroth/.codex/skills/assura-orchestration/scripts/audit-topology.sh \
  "$repo_root" --report
git fetch origin master --prune
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

For Rust, Cargo, CI, release, behavior, or performance changes, add the
focused red test and the applicable `cargo fmt`, `cargo test`, `cargo clippy`,
benchmark, `cargo xtask pr`, release, and hosted checks from the selected card.
The strict topology audit is a required handoff gate; preserved foreign or
unknown state is reported, not silently repaired.

## Review tasks and blocking criteria

- R1: Verify current source/ledger pointers and that every older pointer is
  labeled as-of or historical.
- R2: Verify one owner, clean worktree discipline, exact candidate identity,
  and terminal closure evidence.
- R3: Verify focused tests, negative controls, expected test counts, and honest
  treatment of skipped/unavailable/performance checks.
- R4: Verify A07 privacy, candidate/fixture provenance, condition receipts,
  protocol scope, and separate screening authority.
- R5: Verify VPS/CI efficiency proposals are measurement-backed and do not
  replace hosted, platform, or protection gates.
- R6: Verify compaction/continuation always records a live handle or concrete
  next action and does not create a replacement goal.

Block integration if source is stale, ownership is ambiguous, review findings
are unresolved, required evidence is missing, performance thresholds regress,
checks are skipped or zero-test, authority is inferred, or the owned topology
cannot be closed or archived with an explicit recovery record.

## Progress log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-09-11 | PR #294 merged the reviewed execution-control-plane correction as `c9ac2a8`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as did the configured post-merge Rust CI, Documentation and Security workflows. The correction makes merge-SHA push workflows a separate reconciliation gate. A retained read-only diagnosis bound R01's macOS failure to job `103316578631`, `tests/watch_cli.rs:196`, where `watch_stops_cleanly_without_runtime_artifacts` did not stop after SIGINT; the focused local Darwin rerun passed once and remains non-substitutive. No product, threshold, allocation, screening or authority state changed. | PR #294; push run `34625737610`; retained failure `34615572565`; Iteration 152 in `research/progress-current.md`. |
| 2026-09-11 | PR #292 merged the reviewed agent-content-preservation recovery slice as `4560c710`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates, Rust/platform tests, performance, release/adoption smoke and GitGuardian checks passed (Security Audit was scope-skipped). The subsequent push-triggered Rust CI run failed in the macOS watch SIGINT test and cancelled Ubuntu/Windows siblings; this remains retained unfavorable hosted evidence. | PR #292; hosted check rollup and push run `34615572565`; focused local `watch_stops_cleanly_without_runtime_artifacts` rerun passed once. |
| 2026-09-11 | Rebuilt the A07 candidate and no-credit protocol packet at current `origin/master=4560c710`: exact login-shell identity with wrong-target/wrong-root rejects, two fresh sibling-free source-only canaries with all seven evaluator dimensions passing, six immutable holdouts, second-readonly pass, current fixture/invariant/evaluation bindings, exactly two conditions and 30 reserved cells. The packet was initially pending protocol review; the subsequent scoped rereview PASS is recorded below. Screening and credit remain false. | Private packet under `/private/tmp/assura-a07-private-4560c71`; candidate freeze, identity, canary receipts/evaluations, holdout binding, fixture freshness, invariants, evaluation binding, mapping and manifest. |
| 2026-09-11 | Recorded the scoped independent protocol rereview as `PASS` after repairing the explicit receipt-condition aliases and current construction digest. The private packet is now protocol-pass/no-credit; screening, allocation, credit and product acceptance remain false. | `/private/tmp/assura-a07-private-4560c71/screening/protocol-review-current-4560c71.json`; reviewed hashes are preserved separately from post-verdict hashes. |
| 2026-09-11 | Independent process review returned `PASS` for the committed goal/task reconciliation at `831dab5`: current-source routing, historical supersession, hosted-failure honesty, layered context, one-owner continuation and the A07 line limit were verified; no product or authority expansion was found. | `/root/context_routing_review_fast`; review covered `da432ad` plus the committed reconciliation. |
| 2026-09-11 | PR #289 merged the reviewed durable goal, layered-routing skill metadata and current-route corrections as `40f1155c`; the 851a6 candidate packet is now candidate-base/no-credit and must be rebuilt after the next source refresh. | PR #289; `research/progress-current.md` Iteration 147; post-merge ledger/context/structure/evidence/docs gates. |
| 2026-09-11 | Created the durable active execution goal from the corrected Trellis recovery contract. The current source is `851a6b8`; A07 is the only active lane while the ledger has no ready-pending card. | `.trellis/tasks/09-04-maturity-portfolio-strategy/task.json`; `research/orchestration-plan.md`; `research/process-corrections-2026-09-11.md`; `research/recovery-plan.md`; workflow gate, ledger and topology reset evidence. |
