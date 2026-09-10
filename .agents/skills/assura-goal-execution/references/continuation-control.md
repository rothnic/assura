# Continuation control

Use this reference after a reset, compaction, review result, failed gate, or
empty ready-pending query. It turns the current ledger into one owned next
action without changing card acceptance or authority.

## Route one real action

1. Refresh `origin/master`, the pull-request/CI view, topology and the
   revision-pinned ledger. Read active, implemented, verified and merge-ready
   records before pending records.
2. Choose exactly one route with an owner and phase:
   - resume a live test, CI or review handle;
   - run a bounded diagnosis for a failed gate;
   - implement or validate an authorized slice;
   - obtain the named external/authority decision while preparing independent
     work; or
   - integrate a reviewed current-base candidate and reconcile its closure.
3. Record the route in the authoritative card evidence: owner/session,
   source SHA, worktree/branch, phase, proof and exact triggering observation.
   A status observation without a route is not a checkpoint.

## Command identity fence

The terminal/tool `workdir` is not proof when wrappers nest commands. Before
every check, test or build, explicitly `cd` to the owned worktree and verify
`pwd`, `git rev-parse --show-toplevel`, branch, expected source SHA and target
directory. Use `git -C` or absolute paths for read-only queries. If the
effective cwd, repository or source SHA differs, discard the result as invalid
evidence and rerun from the owned checkout.

An empty `READY_PENDING` set is a routing result, not completion or a whole-goal
block. An active or verified card still owns the next observation. If its
contract cannot run yet, record a held action with the exact missing evidence,
owner, smallest resolution and independent work that remains authorized. Do
not invent product inputs, convert old evidence into current proof, or launch
an allocation before its prerequisite gate.

## Continue across interruptions

- Keep the same live handle after an observation timeout; inspect or resume it
  before starting another run.
- Commit one coherent owned change before changing cards, yielding, review or
  handoff. Review fixes as a delta and rerun every affected gate.
- While a review, external decision or CI handle is pending, perform only
  independent authorized preparation; do not mutate its frozen scope.
- Use the smallest context layer that answers the next decision. Keep private
  conditions, mappings, fixtures and raw results outside public evidence.

## Handoff and stop predicate

Before a handoff, ask whether any live handle, repair, review, integration or
cleanup action remains. If yes, continue that action or record its exact wait
handle and an independent route. Do not send a status-only handoff.

The supported goal may finish only when every authorized slice is accepted by
its full contract, final current-base gates are evidenced, and all owned
branches/worktrees are merged or deliberately archived with restore proof. An
external hold may remain only after an independent impasse review challenges
all plausible candidates and records the held action plus exhausted
independent work. A card-level hold never licenses weakening thresholds,
counting skipped checks, or marking the whole goal blocked.
