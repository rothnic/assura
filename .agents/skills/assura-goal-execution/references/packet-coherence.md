# Candidate-bound packet coherence

Use this special lane for A07 (or any blinded, candidate-bound evaluation)
when a packet contains several JSON records that reference one another. It is
loaded only for packet construction, correction, protocol review, or verdict
reconciliation; it is not a second ledger or an authorization source.

## Two-phase hash fence

Treat a protocol review as two immutable phases:

1. **Review snapshot.** Build an explicit role-to-relative-path index for the
   candidate freeze, identity control, construction, binding, second-readonly
   record, receipts, public evaluator summaries, canary summary, invariants,
   manifest, mapping, rebind summary, and packet validator. Hash every file.
   Keep the packet in `PENDING`, with `protocol_reviewed_at=null`, and keep all
   screening, allocation, credit, and acceptance flags false. The reviewer
   verifies this exact map and records the verdict without mutating packet
   state.
2. **Coordinator disposition.** Only after a scoped independent verdict may
   the coordinator apply a no-credit disposition. Update all dependent status,
   timestamp, and reference fields together; rerun the validator with the
   exact disposition; then emit a post-verdict hash map and a finalization
   record linking the reviewer SHA, correction record, validator result, and
   authority flags. The review artifact remains an immutable pre-verdict
   observation, so the finalization record must distinguish pre- and
   post-verdict hashes.

## Cross-record assertions

Before review, and again after disposition, assert the following from the
same packet root:

- every referenced file exists, is JSON where required, and is included in the
  role-to-path index; no reference points to a superseded date, alias, or
  missing validator;
- source SHA, source tree, binary SHA, shim SHA, public/evaluator contract
  SHA, fixed-prompt SHA, version, and exact toolchain agree across freeze,
  identity, receipts, conditions, bindings, invariants, and manifest;
- both stable conditions have the same input name, differ only in value, carry
  explicit receipt IDs, and share equivalent invariants; six holdouts are
  unique with two per stack, and the matrix has 30 unique reserved cells;
- candidate canary status, packet status, validator protocol status, and
  protocol-review status are coherent for the phase. A review timestamp is
  null while status is `PENDING` and equals the reviewer timestamp only after
  a recorded verdict;
- validator output is persisted with its exit code, `valid=true`, six handles,
  30 cells, two conditions, and the same requested protocol disposition;
- no screening, allocation, credit, acceptance, release, deployment,
  publication, or invitation flag becomes true as a side effect of review.

Missing or stale references, contradictory pending/pass prose, a stale
timestamp, a hash-map mismatch, or a validator that was only printed rather
than persisted is a concrete review finding. Stop the dependent transition,
retain the finding, make the smallest correction, rerun affected assertions,
and request scoped rereview. Never repair a current packet with blind global
string replacement; preserve historical records and write candidate-specific
aliases deliberately.

## Public boundary

Public evidence may report source/tree/binary identity, aggregate canary and
validator counts, review verdict, correction IDs, and no-credit limitations.
Keep private contracts, mappings, raw evaluator output, fixtures, child event
streams, and hidden expected values in the private lane. A protocol
`PASS_NO_CREDIT` makes a packet metadata-ready only; it does not allocate
cells, authorize screening, or complete the product card.
