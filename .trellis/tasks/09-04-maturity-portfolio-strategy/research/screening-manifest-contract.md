# A07 screening manifest contract

Status: required before any A07 screening credit. This public contract defines
the record shape and privacy boundary; the condition values, fixture identities,
contract contents and raw results remain in a private evidence store.

## Why this exists

The packet requires two product-input conditions but did not define their names,
values, or evidence. Do not infer them from old run filenames, model prompts, or
holdout layout names. The coordinator must freeze exactly two conditions before
the first screening run and have the manifest independently reviewed.

## Required private manifest

Use a versioned record with these fields for each of exactly two conditions:

- `condition_id`: stable blinded identifier and a separate unblinded mapping;
- `product_input`: the one changed product input, its value/state, and evidence
  that the initializer actually received it;
- `invariants`: prompt, candidate, stack/toolchain, fixture rules and all other
  inputs held constant;
- `fixture_ref` and `contract_ref`: private handles, not repository paths;
- `source_digest`, `contract_digest`, `prompt_digest` and candidate identity;
- one exact canonical `toolchain` identity (including compiler and Cargo
  commit/date when available), reused byte-for-byte in the candidate freeze,
  both condition invariants, every supplied-input receipt and every holdout
  row;
- `public_summary`: a redacted description suitable for aggregate reporting.

The two conditions must differ in one specified product-input variable only.
The source fixture, evaluator contract, acceptance predicate and thresholds must
otherwise remain equivalent. A condition with an ambiguous value, missing
supplied-input proof, or an unreviewed mapping is not executable.

## Independent protocol-reviewer role

This manifest has two distinct reviews. The ordinary product/code reviewer sees
the public contract and redacted evidence. A separate protocol reviewer runs in
an isolated session with narrowly scoped access to the private manifest's
schema, condition rows, blinded/unblinded mapping, supplied-input evidence and
matrix metadata. The protocol reviewer may verify the one-variable difference,
exactly-two rule, complete 30-cell allocation and privacy fields, but must not
read raw evaluator output, hidden expected results, child transcripts, private
fixture contents, or unrelated worktrees. It returns only `PASS` or redacted
findings with location, failure scenario, contract and smallest verification.
The coordinator records that disposition without copying private values into
the repository. A missing or conflated protocol review leaves the matrix
unexecutable.

## Matrix and run record

Freeze 30 unique cells: three stacks × two conditions × five repetitions. Each
cell receives a fresh source-only fixture and one-shot child under a dedicated
sibling-free disposable parent. Record privately: run ID, condition ID,
candidate identity, command/cwd, environment, elapsed/token cost when exposed,
initializer disposition, evaluator result, follow-up feature result, and any
invalid/no-credit reason. No run is credited when identity, context, fixture,
condition, or evaluator provenance is missing or mismatched.

Before any cell is allocated, the private manifest must reference an immutable
holdout-binding record. That record must map exactly six opaque handles (two
valid unseen layouts for each stack) to their source-tree and full-contract
digests, current candidate source/tree/binary/contract/prompt/toolchain
identity, private hand-verification evidence, an immutable per-handle
`created_at` plus `creation_evidence_ref`, and a second read-only confirmation
that repeats and compares those six creation records and the canonical
toolchain. It must list every disqualified or unfrozen layout outside the
six-handle set. A historical binary hash, a prose claim that a layout is
frozen, an unbound handle, a missing creation record, or a shorthand toolchain
string is not current-candidate holdout proof.

## Gate order

1. Refresh and freeze the current-master candidate identity; this observation
   is not screening credit.
2. Confirm the immutable six-handle holdout-binding record is frozen privately;
   verify all six per-handle creation records and exact toolchain identity;
   exclude every disqualified or unfrozen construction draft.
3. Validate the manifest schema, exactly-two condition rule, one-variable
   difference, private mapping, and complete 30-cell matrix in an isolated
   protocol review. Keep the separate product/code review boundary intact. This
   is the smallest resolution for the missing-condition finding; a review that
   does not return `PASS` leaves the matrix unexecutable.
4. Run a fresh candidate-bound canary against the frozen current-master
   identity and confirm the full contract passes; the canary remains no-credit.
5. Run cheap identity/context checks before each child, then evaluate and run
   the separate follow-up feature. Preserve failures and stop invalid runs;
   never repeat an unchanged method until it happens to pass.

The manifest does not change the 30-run screening, 18-run untouched holdout,
or final ten-per-stack (at least 9/10) acceptance thresholds. It only makes the
existing protocol executable and auditable.

## Publication boundary and efficiency

Public evidence contains only condition-level counts, sample sizes, approved
candidate identity, redacted outcomes, and limitations. Do not publish private
paths, fixture/contract/event/identity hashes, raw commands, or hidden labels.
Build one candidate per source SHA and run cheap checks first. Use VPS capacity
only after a measured quiet-host/disk/toolchain check; remote Linux supplements
but never replaces required macOS/Windows/hosted proof.

## Current next action

On each resume, the A07 coordinator must first refresh `origin/master`, rerun
the revision-pinned ledger, and freeze the candidate identity. The 9b, 8be, 9df
and 692 manifests, canaries and protocol `PASS` records are historical
no-credit evidence after the later process merges and must not route current
work. The current as-of ebedb3e candidate has fresh identity controls and two
full-contract source-only canaries with a meaningful negative control; these are
no-credit preparation. Its six-handle binding, supplied-input receipts,
two-condition manifest and 30 reserved cells are rebound with isolated protocol
rereview pending. On each next resume, complete that rereview, record PASS or
concrete corrections, refresh source/ledger, release/tag, PR/CI and topology
state, then run a fresh no-credit canary before the separately authorized
screening gate. Prior canaries, process PRs and metadata-only evidence never
satisfy screening or acceptance gates.
