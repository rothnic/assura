# Close shared release publication identity predicate

## Goal

Make the shared delivery receipt predicate fail closed unless publication
evidence carries the same full tag and commit object IDs as the receipt and
the current source. This is the missing shared-lane half of the P05 release
contract; it must land as a separate, reviewed current-base correction.

## Current evidence

- Current base is `origin/master` at
  `47ca7080c84dc6349bf01f48f1c26c481bc63497`.
- `scripts/release-contract.py` now requires and normalizes publication
  `tag_oid` and `commit_oid`, but `.trellis/scripts/common/delivery_status.py`
  does not consume those nested identities.
- An otherwise valid receipt with both nested publication OIDs removed is
  currently accepted by `validate_release_receipt`.
- The P05 task explicitly assigns this predicate to the integrated P01-P04
  lane and forbids changing it in the P05 worktree; this task restores that
  cross-lane invariant without touching release publication behavior.

## Requirements

1. Extend `_release_receipt_verified` so a release receipt is valid only when
   `publish.tag_oid` and `publish.commit_oid` are full 40-character hex Git
   object IDs and exactly equal the receipt's top-level `tag_oid` and
   `commit_oid`.
2. Preserve all existing repository, version, source-tip, local-tag, archive,
   checksum, install-proof, publication-action, and verified-asset checks.
3. Add a valid control plus table-driven regressions for missing, malformed,
   and mismatched nested publication identities. Exercise the public
   `validate_release_receipt` boundary and keep fixtures aligned with
   `.trellis/spec/assura/delivery-lifecycle.md`.
4. Keep the change limited to the shared predicate and its focused delivery
   contract tests. Do not edit release scripts, workflows, installer URLs,
   task receipts, release tags, releases, deployments, or preserved worktrees.

## Acceptance

- [ ] A valid receipt with matching nested publication OIDs is accepted.
- [ ] Missing, short/invalid, and mismatched nested publication OIDs are
      rejected without accepting a terminal release status.
- [ ] Existing delivery contract tests pass with a nonzero count.
- [ ] Workflow gate, Assura structure check, and diff check pass.
- [ ] An independent reviewer returns PASS for the exact candidate SHA.
- [ ] Hosted checks pass; no GitHub approval is inferred when none is present.
- [ ] After merge, refreshed `origin/master` contains the reviewed candidate
      and exact post-merge checks pass.

## Definition of Done

The narrow correction is committed on its owned branch, reviewed, integrated
through a current-base PR, and verified after merge. The P05 worktree remains
independent until its own review resumes. No release or publication authority
is exercised.

## Out of scope

- Implementing or reworking the P05 release builder, publisher, CI workflow,
  installer, or versioning behavior.
- Reclassifying the P06 inventory, cleaning foreign worktrees, or resolving
  unrelated structure advisories.
- Creating a release tag, publishing assets, deploying, or changing durable
  download URLs.

## Technical notes

- Shared predicate: `.trellis/scripts/common/delivery_status.py`.
- Existing delivery contract coverage: `tests/agent_delivery_contract_tests.py`.
- Canonical contract: `.trellis/spec/assura/delivery-lifecycle.md`.
- Integration target is repository `rothnic/assura`, branch `master`.
