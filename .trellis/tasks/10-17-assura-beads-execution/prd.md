# Reviewed Beads execution goal

## Goal

Execute the reviewed Assura new-user, capability and efficiency backlog through
Beads in dependency order. Nick authorized this goal on 2026-10-17: "Create a
goal to iterate through the beads where each bead ... receive an independent
review and corrections before merging. You will merge when reviewed."
The runtime goal is active on thread `01a14bf3-bc89-7561-8853-31159fecfb86`.
This delegates reviewed, fully gated merges, including the Cloudflare website
deployment coupling discussed before this instruction. Tagged CLI releases,
external outreach and acceptance evidence still require their actual contracts.

## Ownership and authority

- Coordinator and sole Beads writer: live `/root`. All newly dispatched agents
  use GPT-6 Luna. One explicit implementation owner per candidate/worktree;
  a separate reviewer judges the frozen result and subsequent correction delta.
- Canonical intake: the reviewed 25-node/15-edge graph in
  `10-17-new-user-review-backlog/research/beads-graph.json`, from `925ad07f`.
  Its 14 new items and 10 carried outcomes retain their acceptance criteria.
  Historical terminal rows are context, not reopened work.
- Beads owns live scheduling, dependencies and issue status after verified
  import. The review JSON becomes an intake snapshot. Trellis continues to own
  specs, scoped implementation contracts and native delivery evidence; do not
  maintain a second independently mutable queue.
- Current integration is `origin/master`; goal preparation incorporates
  `49d85358bae85de04f1ce7faa746b377687dcaf3` (merged NU13 PR #355).
  The original review branch and unknown work remain preserved.

## Store adoption

Independent inspection found no project/parent `.beads`, no Beads environment
override and no Assura mapping in the inspected user registry. Leave the
separate global store untouched. Use installed `bd 1.2.2` with a project-local
store at `/Users/nroth/workspace/assura/.beads`, prefix `assura`, explicit actor
and `BEADS_DIR` for coordinator commands. Inspect actual backend/server mode
after initialization; keep issue writes serialized.

Use supported noninteractive init with `--skip-agents --skip-hooks` and
repository-local `--setup-exclude`; preserve
Assura's hooks and agent instructions. Do not upgrade, reinitialize, force,
migrate or select a shared server to bypass identity checks. Verify an empty
target and duplicates before graph import; retain dry-run and actual key/ID,
edge, parent and readiness readback. Use Dolt-native backup for recovery;
JSONL exports are interchange, not canonical sync or database backup.

The standard `.beads` path is tool-managed local database/runtime state,
excluded from source control by supported setup. Exclude that exact directory
from Assura checks, like other managed runtime outputs; no broad root
allowance or weakened source rule. The goal's tracked mapping and contracts
provide discovery; a local store is not cross-machine synchronization.

## Per-bead execution contract

1. Refresh store identity, issue, parent, blockers, current source and live
   ownership. Reuse existing Trellis work rather than duplicate the outcome.
2. Claim only dependency-ready, authorized work. Create a scoped contract and
   clean owned checkout; read the relevant specs and choose the smallest
   observable outcome. Research beads can conclude with a supported defer
   decision; they need no artificial runtime feature or status-only PR.
3. Preserve red/green controls, exact source/tool identities and unfavorable
   evidence. Use focused iteration checks, then required readiness gates.
   Admit resources and offload sustained builds to the verified development
   host. Never broaden scope or lower thresholds to manufacture a pass.
4. Independently review every bead's result with GPT-6 Luna. Adjudicate concrete
   findings, correct accepted defects, rerun affected checks and obtain scoped
   rereview of the final changes. Reviewer output is advice; coordinator owns
   the decision. A review must inspect the artifact and meaningful behavior.
5. Merge only a current-base, exact reviewed candidate with required local,
   hosted, platform and performance checks passing. Then observe configured
   post-merge workflows, actual behavior/deployment where relevant and merged
   reachability. Close only against the bead's full declared outcome.
6. Record concise evidence on the bead and native delivery receipt, verify
   readback/backup, then close the owned checkout. Keep external holds precise
   and continue independent eligible work; a single held bead does not pause
   the whole runtime goal.

## Acceptance

- The authoritative project store and writer mode are verified; the reviewed
  graph imports once without duplicates, cycles, lost prerequisites or hidden
  changes to the original scopes. Durable source-key/Beads-ID mapping exists.
- Each selected bead has an accountable owner, bounded contract, actual result,
  independent Luna review, correction dispositions and final validation proof.
- Every merged implementation meets its gates before merge and retains the
  observed post-merge outcome; no bead closes from a commit/status alone.
- External human/evaluator acceptance is observed honestly. No participant,
  sealed-evaluator, release or permission result is simulated by an agent.
- The goal completes only when every in-scope outcome is accepted and the epic
  itself passes independent outcome review, with owned merge/archive closure.
  A real unresolved prerequisite keeps its affected outcome open.

## First actions

PR #355 was reviewed and passed 30 hosted checks before merge. Its merge is
`49d85358`; post-merge Rust 39090025708, docs 39090025772 and security 39090025736
are live observations, not yet passing acceptance. Finish that reconciliation
before closing NU13. In parallel prepare store adoption and its independent
review. SP01 remains the next proposed core filesystem attribution slice;
measure repeated metadata/enumeration/discovery before retaining optimization.
