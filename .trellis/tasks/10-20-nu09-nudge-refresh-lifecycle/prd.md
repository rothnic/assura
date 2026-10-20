# NU09: Nudge refresh lifecycle cohesion

## Goal

Bead ae-whr.6 owns the live queue. Reduce the navigation/context cost of refresh
process lifecycle changes by extracting one cohesive private nested module from
agent_nudge_delivery.rs. Selection, persisted state and shared lease/store
primitives remain in their current owner. This is partial cohesion work; the
parent max_lines advisory remains visible with owner/review2026-11-03.

## Authority

Nick authorized ready Beads execution, independent GPT-6 Luna review/corrections
and root-owned gated merge. Root owns native/provider/Beads/push/PR/merge/closure.
nu09-refresh-implement owns only the two production files and task context.
No public API break, dependency, policy/threshold, deadline or controller change.

## Contract

Root accepts the reviewed nudge-lifecycle-design-f5-20261020.md and independent
nudge-lifecycle-design-review-f5-20261020.md in external nu09 evidence. Extract
exactly the complete request/launch/supervisor/completion/watchdog/deadline group,
including refresh_deadline_millis, into flat agent_nudge_delivery_refresh.rs.
Keep state/selection, queue/sidecar and lease/store primitives parent-owned.
Use explicit private imports, no wildcard imports or duplicate machinery.
Formatted new module must remain at most500lines without compression, arbitrary
subsplits, removed meaningful comments or policy changes; otherwise stop for
root judgment. Existing21tests remain in parent with same names and bodies.

Root visibility disposition: preserve original accessible scopes. External
internal entrypoints may use pub(in crate::cli) in the child and the existing
parent pub(super) reexports; child pub(super) helper access plus private parent
aliases restores the former parent-private scope. Test-only aliases belong
under cfg(test). No public visibility, new consumer API, compatibility wrappers,
new state model or new tests merely mirroring a move. Resolve every changed
super:: reference to its original semantic target and document path-only deltas.

## Acceptance

Record all21existing test identities and bodies plus every moved production
function/helper before edits; preserve each exactly once and inventory any
necessary namespace/visibility change. No helpers may disappear. Preserve
cancellation, idempotency, queues/recovery, token fencing, bounded child cleanup,
errors/diagnostics, JSON/text/exit behavior, stdio and all timeout/environment
values. Run existing five lifecycle tests (contention, cooldown, supervisor child
cleanup, not-ready recovery), complete delivery unit group, and meaningful
agent_surface_cli/nudge/trajectory consumers. Record exact commands/counts/exits,
including existing negative and valid-exception cases. Run affected Rust tier
and final fullPR/current-base hosted OS/performance/install/docs gates; no gate
lowering or ignored-zero-test credit. Independent Luna final source review and
scoped rereview after accepted corrections, root gated current-base merge,
actual postmerge/native close and clean owned worktree/ref disposition required.
If no honest complexity-reducing extraction fits, retain visible scoped debt
with reviewed evidence; do not force a split for counts.

## Owned files

src/cli/agent_nudge_delivery.rs; src/cli/agent_nudge_delivery_refresh.rs; this task.
No configuration, other production modules, dependencies, workflows, broad test
refactor or public contract edits. Request root judgment on a concrete exception.
