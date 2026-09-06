# Copy-paste prompt for an execution agent

Copy the following prompt into a coding agent that can access the repository and planning task. It is designed for sequential execution with limited context. The queue and solution cards are the source of truth; no knowledge of the earlier conversation is required.

```text
Execute the Assura maturity backlog, one reviewable task at a time.

Planning root:
/Users/nroth/workspace/assura/.trellis/tasks/09-04-maturity-portfolio-strategy

Read first:
1. research/execution-backlog.md
2. research/backlog.json
3. This project's AGENTS.md and the skills relevant to the selected card.
Read prd.md for the overall product intent. Then read only the packet section
for your selected ID and any shared contract it explicitly references.

Objective: make Assura a dependable tool for executable repository conventions
and agent-assisted setup. The professional story supports technical product /
AI systems leadership. Do not expand project intelligence, semantic search,
agent orchestration, remote plugins or generic maturity scoring.

First task is B00 unless its completion evidence already exists and is current.
Use latest GitHub master as the source baseline. ed093668 was the reviewed SHA,
not a permanent pin. The original local checkout is older: do not implement on
it. Refresh Git/PR/worktree state, preserve unrelated changes and work in an
isolated current-master checkout. Inspect overlapping PR #142 and the existing
NickRoth case-study branch before creating duplicate work. Record actual cwd,
SHA, binary version and toolchain. Load the worktree skill for isolation.

Select the first pending backlog item whose dependencies are done and whose
required changes are present in this checkout. A not_needed dependency requires
written evidence/approved scope disposition. If an item is blocked on publication,
people or environment, record the blocker and take another independent ready item.
Do not run the whole backlog as one giant patch. Default batch size is one card.

For the selected card:
- State ID, expected outcome, owned files and acceptance checks before editing.
- Follow its prescribed solution. Use existing patterns and commands. Proposed
  new files/options in the card must be implemented/documented/tested together.
- For behavior changes, first reproduce the defect or write the focused failing
  contract test. A test that merely checks the new code exists is not sufficient.
- Implement the smallest cohesive fix. Keep config, generated artifacts, tests,
  CLI help and public behavior aligned where the card requires it.
- Never widen excludes, disable rules/tests, remove benchmark rows, reduce
  severity, change CI scope or claim a skipped check passed to finish a task.
  If such a policy change is justified, record the evidence and request a
  separate maintainer decision.
- Run the focused tests from the right cwd, then the repository's relevant
  verification tier. For Rust PRs run cargo xtask fast and cargo xtask pr plus
  applicable feature/OS gates. Do not rerun unrelated heavy suites repeatedly.
- Use real temporary fixture repos for hook/installer tests. Restore or discard
  only your own disposable test data; never overwrite user hooks/configuration.
- Evaluate with the candidate binary, not a global older Assura installation.
  A passing check with no enforced policy, zero expected tests, generated-only
  hooks or fabricated evidence is failure.

For a novel failure not covered by the card, investigate up to two distinct
evidence-based hypotheses, record results and the smallest required plan change.
Do not repeatedly retry unchanged commands or guess a broad rewrite. Stop that
card for a real contract/authority decision; continue independent work if available.

Before a complex PR, request an independent review under project rules. Review
findings critically and fix valid issues. Commit only your owned, verified changes
when the repository workflow requires it. Prepare a PR-ready summary; creating a
remote PR, pushing/merging, tagging, releasing, deploying, changing branch protection,
sending invitations or publishing posts requires explicit authorization covering
that action. Local implementation and draft preparation should be finished first.

Update research/backlog.json and write research/evidence/<ID>.md with:
state, actual source SHA/worktree, reproduction, changed behavior/files, exact
commands and exit results, negative controls, review findings, limitations,
commit/PR/integration state and next-ready ID. Do not mark done until the card's
actual outcome is proven. If local code is verified but hosted proof or publication
is pending, use verified or blocked with that exact reason.

Your final handoff must contain:
1. ID and outcome achieved (or exact blocker).
2. Changed files and why.
3. Verification evidence and known limitations.
4. Commit/PR status and next-ready ID.
Do not end with another general strategy or merely a list of concerns.
```

## Starting with a particular card

Append `Execute only R01 after verifying B00 evidence and dependencies.` to select a card. Replace the ID only with an existing queue ID. For a longer session, explicitly authorize a batch size and integration approach; the default avoids accumulating unreviewed patches.

## Status meanings

- `pending`: not started.
- `active`: one agent owns the card; record its checkout.
- `implemented`: patch exists, proof incomplete.
- `verified`: local/card checks pass but required review/hosted/public outcome may remain.
- `done`: all required outcomes and evidence, including external ones if any, exist.
- `blocked`: exact required input/environment/authority unavailable; completed work recorded.
- `not_needed`: proven existing equivalent or explicit scoped exclusion, with rationale.

No automatic scheduler is created by this backlog. The coding agent uses the queue; human-dependent cards remain visible instead of being invented as completed.
