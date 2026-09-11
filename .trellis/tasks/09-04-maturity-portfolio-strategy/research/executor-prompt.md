# Copy-paste prompt for an execution agent

Current recovery route: [recovery-plan.md](recovery-plan.md). Resolve this task
at refreshed `origin/master`, then read the repository's `assura-goal-execution`
skill for the execution contract and validation matrix. Absolute paths below
identify the task, not the authoritative revision of an old checkout.

Resume the existing supported runtime goal; do not create a replacement goal
for a card boundary or a compaction. Refresh the `release` ref and tags as
separate availability facts, and record a missing release branch rather than
turning it into release proof.
After every merge, refresh `origin/master` and reconcile the task pointer,
ledger and owned topology before selecting or handing off; a clean merge is a
phase transition, not completion of the runtime goal.

As-of routing checkpoint (2026-09-11, refresh before use): the current source is
`origin/master=71adc2e695f4696e6a1fef7d5075aa836f4102b1`. The owned candidate-
bound build used Rust/Cargo `1.94.1`; login-shell identity checks and two
corrected source-only canaries passed the full evaluator with the expected
negative policy probe. Two omitted-contract fixture attempts remain
unfavorable no-credit evidence. The candidate canary is preparation only, not
screening or acceptance proof. A02 is complete; its old plain-init finding is
historical. The ledger at 71adc2e has 32 items, zero ready pending, five
unfinished and three held: A07 active, W03 verified, and R01/W02/F01 held.
The exact next action is to verify/rebind the current six-handle holdout and
exactly-two-condition manifest to `71adc2e`, obtain isolated protocol `PASS`,
then seek separately authorized screening. The prior af005a7 checkpoint and
all ebed/9df/692/9ad/129a249 candidate packets and protocol records are
historical no-credit metadata and must not route current work. R01's merged
raw-log recovery remains bounded negative evidence. Preserve residual fixture,
launcher, child-isolation and evaluator limitations. Route from
`recovery-plan.md`, not this snapshot, after a fresh fetch and ledger.

The historical parent-`c34f917` candidate-freeze observation used a clean
detached checkout and the exact Rust/Cargo `1.94.1` toolchain. Its non-symlink
`assura 0.4.0` binary hash is
`95c93052bd1993566d9f8209bdba5625c2b39a19287ed6358d710015d2ac59ff`, matching
`command -v`, version and target hash inside a minimal login shell. This is
historical no-credit preparation, not a current candidate binding. The six
valid holdouts are frozen; the private condition values, supplied-input
receipt, mapping, matrix and protocol `PASS` are still required and must not
be inferred from historical run names. After that redacted `PASS`, fetch
`origin/master` again, rebuild and freeze a new candidate identity before any
no-credit canary or screening allocation; never reuse this parent binary hash.

Copy the following prompt into a coding agent that can access the repository and planning task. It is designed for sequential execution with limited context. The queue and solution cards are the source of truth; no knowledge of the earlier conversation is required.

```text
Execute the Assura maturity backlog, one reviewable task at a time.

Planning root:
/Users/nroth/workspace/assura/.trellis/tasks/09-04-maturity-portfolio-strategy

Read first:
1. research/orchestration-plan.md
2. research/execution-backlog.md
3. research/backlog.json
4. This project's AGENTS.md and the skills relevant to the selected card.
Read prd.md for the overall product intent. Then read only the packet section
for your selected ID and any shared contract it explicitly references.

Objective: make Assura a dependable tool for executable repository conventions
and agent-assisted setup. The professional story supports technical product /
AI systems leadership. Do not expand project intelligence, semantic search,
agent orchestration, remote plugins or generic maturity scoring.

Begin by validating B00's completion evidence; if it is current, do not rerun
or reopen it. Use latest GitHub master as the source baseline. The original planning review
used `ed093668`, but that SHA is historical and never a permanent pin. The
original local checkout is older: do not implement on it. Refresh Git/PR,
release/tag, worktree and owner state, preserve unrelated changes and work in
an isolated current-master checkout. Inspect overlapping PR #142 and the
existing NickRoth case-study branch before creating duplicate work. Record
actual cwd, SHA, binary version and toolchain. Load the worktree skill for
isolation.

First inspect unfinished active/implemented/verified candidates and actual live
owners. Then select a pending item with evidenced merged dependencies and whose
required changes are present in this checkout. A not_needed dependency requires
written evidence/approved scope disposition. If no pending item is ready, keep
one coordinator-owned recovery, review or integration action live and record
its owner, phase, proof and exact next command; do not stop at a status report
or invent a card. If an item is held on publication, people or environment,
record the narrow hold and take another independent ready or process action.
Do not run the whole backlog as one giant patch. Default batch size is one card.

Execution-continuity invariant: a checkpoint is not a stopping condition. Keep
one concrete owned action live at all times: retain an active test, CI, review,
integration, or cleanup handle; resolve a failed gate; or select the next ready
card immediately after a terminal phase. Before yielding, record the action,
owner, source SHA/worktree, required proof, and the exact next observation or
command. An empty pending queue does not satisfy this invariant while an active,
implemented, verified, merge-ready, or integrated card still has a real next
phase. If a card is held by an external prerequisite, record that narrow held
action and continue another independent authorized card; do not convert the
whole train into a status report.

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
- Run focused tests from the right cwd, then the relevant verification tier.
  cargo xtask pr already includes fast; one nested invocation proves both at
  readiness. Retain applicable final-source feature/OS/performance gates.
  Use the goal skill's matrix/VPS route; record actual exits and elapsed time.
- Use real temporary fixture repos for hook/installer tests. Restore or discard
  only your own disposable test data; never overwrite user hooks/configuration.
- Evaluate with the candidate binary, not a global older Assura installation.
  A passing check with no enforced policy, zero expected tests, generated-only
  hooks or fabricated evidence is failure.

For a novel failure not covered by the card, investigate up to two distinct
evidence-based hypotheses, record results and the smallest required plan change.
Do not repeatedly retry unchanged commands or guess a broad rewrite. Stop that
card for a real contract/authority decision; continue independent work if available.

For A07 initializer evaluation, read `a07-candidate-binding-plan.md` before
launching any run. The candidate must be bound by absolute executable identity
inside the initializer's actual command environment. Capture
`command -v assura`, `assura --version`, and the candidate SHA from that same
environment; a login-shell `PATH` mismatch invalidates the run and earns no
screening, holdout, or final-batch credit. Repair the runner and complete a
fresh identity canary before repeating the protocol. Keep private fixture and
evaluator details out of prompts, reviewer briefs, and public evidence. Read the
goal skill's `references/runner-isolation.md`: launch a fresh one-shot child
with a source-only fixture, a fixed public task prompt, and a minimal explicit
login-shell-safe `PATH`. Do not inherit the coordinator transcript or expose
evaluator contracts, private harness paths, hidden expected output, or ambient
global Assura installations. A forbidden-context read, global-binary fallback,
or missing identity observation invalidates the run and earns no allocation
credit, even when the evaluator is invoked with the intended binary afterward.
Place each fixture beneath a dedicated disposable parent with no sibling
worktrees or prior-run artifacts; recursive discovery of an out-of-scope parent
path is context contamination even when the fixture itself contains only public
source files.
Before screening allocation, read
`screening-manifest-contract.md` and validate the private manifest's exactly two
named product-input conditions, one-variable difference, blinded mapping and
complete 30-cell matrix. Obtain an isolated protocol-review `PASS` before
launching any screening cell; historical run names are not condition
definitions, and missing supplied-input evidence means the cell is not
executable. Refresh the candidate-bound canary against the current master
after the manifest review and before allocation.
Keep initializer events, private evaluator output and redacted evidence
separate. Run the evaluator only after the identity canary and initializer have
completed.

Before a complex PR, request an independent review under project rules. Review
findings critically and fix valid issues. Commit only your owned, verified changes
when the repository workflow requires it. The active supported runtime goal
authorizes pushing and merging reviewed, fully gated code/documentation slices;
without that goal authorization, prepare locally only. Tags/releases, deploys,
branch-protection changes, invitations and publication still require their
specific authority. Local implementation and draft preparation should be
finished before any remote mutation.

Update research/backlog.json and write research/evidence/<ID>.md with:
state, actual source SHA/worktree, reproduction, changed behavior/files, exact
commands and exit results, negative controls, review findings, limitations,
commit/PR/integration state and next-ready ID. Do not mark done until the card's
actual outcome is proven. If local code is verified but hosted proof or publication
is pending, use verified or blocked with that exact reason.

At each card boundary, write a checkpoint containing:
1. ID and outcome achieved (or exact blocker).
2. Changed files and why.
3. Verification evidence and known limitations.
4. Commit/PR status, the named next action, and its owner.

Only make a final handoff after an explicit user pause, completion of the
authorized scope with every owned branch/worktree terminal, or an independently
audited external prerequisite after all independent authorized work is exhausted.
Do not end with another general strategy, a merely informational status, or a
list of concerns while a concrete next action remains.
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
