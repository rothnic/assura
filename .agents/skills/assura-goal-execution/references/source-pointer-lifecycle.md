# Source-pointer lifecycle

Use this reference after a fetch or merge, after compaction, and before review,
integration or handoff whenever a task document contains a source SHA. A SHA in
a dated record is evidence as-of that audit; its heading or age does not make
it current.

## Classification

- **Current**: the exact SHA equals the freshly fetched `origin/master` from
  the same audit. Record the fetch time and the immutable ledger revision.
- **Candidate-base**: the source used to build or review an unmerged candidate.
  It is not live integration state and becomes historical when that candidate
  merges, including a squash merge.
- **Historical/superseded**: any prior snapshot after `origin/master` advances,
  or any record explicitly marked as replaced by a newer checkpoint.
- **Unproven**: a pointer whose fetch, ancestry, or ownership is missing. Do not
  route work from it; perform a read-only refresh.

## Required sequence

1. Fetch `origin/master` and record its full SHA, timestamp, branch and
   checkout status. Do not infer freshness from a task title or prompt.
2. Compare each live-looking pointer in the task, evidence and automation
   prompt with that SHA. Mark older sections historical/superseded; preserve
   their text for provenance.
3. Run the revision-pinned ledger helper against the fetched SHA and inspect
   active, implemented, verified and merge-ready candidates before pending
   rows. An empty ready queue is an intermediate route.
4. For a candidate, record both the integration base and candidate SHA. Review
   the exact candidate tree; later edits are a reviewed delta.
5. After integration, fetch again, record the PR merge SHA, rerun the ledger and
   topology audit, and reconcile the canonical checkpoint before handoff.

If source, ledger, topology and policy inputs are unchanged, reuse the same
as-of checkpoint and create no evidence-only commit. A changed pointer alone
is a refresh trigger, not a reason to rewrite unchanged reconciliation text.

## Squash-merge proof

A squash merge creates a new commit, so the original candidate head need not be
an ancestor of `origin/master`. Prove the PR is merged, verify its merge SHA is
equal to or reachable from the freshly fetched base (a later authorized merge
may have advanced that base), and compare the reviewed candidate with that PR's
merge tree. Do not use a failed `git merge-base --is-ancestor <candidate>
origin/master` by itself to call a squash merge unintegrated.

## Checkpoint shape

Keep the live checkpoint compact and explicit:

```text
source: origin/master=<full sha>; fetched_at=<timestamp>; phase=<phase>
candidate: <sha or none>; base: <sha or none>; review: <verdict/sha>
proof: <ledger/topology/gate commands and exits>
next: <owner, exact observation, smallest action>
```

When a process-only candidate is merged, label its parent snapshot
`candidate-base` or `historical` rather than leaving a misleading “current”
heading. Do not create repeated evidence-only commits solely to chase every
new merge hash; use refresh-before-use when the checkpoint is clearly
as-of-labeled. Create a reconciliation slice when stale wording could route a
fresh executor to the wrong source.

## Failure and privacy boundary

If a pointer differs from the fetched base, hold only the affected routing
action, name its owner and smallest correction, and continue independent
authorized work. Preserve unknown or user-owned dirt. Keep private evaluator
conditions, mappings, fixtures and raw results in their private lane; public
checkpoints may retain only approved aggregate or identity evidence.
