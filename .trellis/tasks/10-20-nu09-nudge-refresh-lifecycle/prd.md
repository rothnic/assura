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

## Implementation and focused validation record

Candidate `152c94ebdee4d07842df4f760d38a0fcd2b504f5` extracts the reviewed 13-function refresh lifecycle into the 485-line private child module. All 13 function bodies and 21 parent test bodies match the baseline; no public surface, test body, deadline, or runtime policy changed.

Focused commands (Rust/Cargo 1.94.1; `CARGO_BUILD_JOBS=2`; private APFS TMPDIR; worktree-local target):

- `cargo test --lib cli::agent_nudge_delivery::tests`: 21 passed, 0 failed, 0 ignored.
- `cargo test -p assura --test agent_surface_cli agent_nudge`: 11 passed, 0 failed, 0 ignored.
- `cargo test -p assura --test agent_trajectory_cli`: 9 passed, 0 failed, 0 ignored.
- `cargo fmt --all` and `git diff --check`: exit 0.

Raw command receipts and resource samples are in the external implementation evidence directory `nu09/nudge-refresh-implementation-20261020`. Earlier compile failures and the successful-but-zero-test `--bin` selector are retained there without pass credit; see `implementation-report.md`. Full PR/current-base hosted gates remain pending.

## Full PR correction

The first full PR attempt at 7e97bf55 exited 1 after 1,491 passing tests,
zero failures and one existing platform ignore. Evidence and target-state
checks passed; strict Clippy rejected spawn_refresh_pair because extraction
left its existing documented function-local allowance on mark_refresh_queued.
Docs were not reached. Three independent Luna triages accepted moving the
original comment/attribute pair to its original function in the new module;
no new allowance, signature, policy threshold or runtime behavior changes.
Preserve attached attributes and documentation as well as all function bodies.
Raw failure and root decision remain in external NU09 full-validation evidence.
Affected strict Clippy and final full PR checks remain required after correction.
