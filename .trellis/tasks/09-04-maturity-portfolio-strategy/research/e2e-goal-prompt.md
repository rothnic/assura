# Stable E2E goal: Assura maturity execution train

Resume through [recovery-plan.md](recovery-plan.md) and the repository's
`assura-goal-execution` skill. Read this task at refreshed `origin/master`;
dated direction snapshots below remain historical. Check the existing product
goal's actual lifecycle before relying on automatic continuation. Keep one
program objective with bounded card checkpoints; do not replace an unfinished
goal at each card boundary.

## Direction review

Current-state routing: read `backlog.json`, the newest `progress.md` entry and
the selected card's `evidence/<ID>.md` before using any status or SHA below.
The app goal text and older checkouts can retain superseded snapshots. Refresh
GitHub and reconcile newer goal-owned evidence in a dedicated documentation
handoff before treating a dependency as done. Preserve conflicting historical
records with explicit supersession; do not overwrite them with a stale queue.
No performance regression may merge: retain all rows and failed controls,
reject candidates that fail the prescribed comparison, and stop invalid
diagnostics instead of repeating runs until green. Root owns review judgment
and merge approval; method changes never imply relaxed acceptance thresholds.

Continuation checkpoint captured 2026-09-09: `origin/master` is `922d7f0`,
including the independently reviewed A02 plain-init handoff correction in PR
#231. A07 remains active. Its previous candidate-bound identity canary was
valid but the fresh-agent product contract failed; a later attempt is invalid
when the child falls back to an ambient Assura binary or receives private
evaluator context. Preserve both outcomes, give neither screening credit, and
resume with a fresh isolated canary after the A02 correction.

For A07 evaluation, read
[a07-candidate-binding-plan.md](a07-candidate-binding-plan.md) before launching
an initializer. A candidate must be bound by absolute executable identity inside
the initializer's actual command environment; login-shell `PATH` inheritance is
not proof. Record `command -v assura`, version, and SHA before counting a run.
Any mismatch invalidates the run and receives no screening, holdout, or
acceptance credit; fix the runner and run a fresh canary before repeating the
protocol. Use the goal skill's
[runner-isolation](../../../../.agents/skills/assura-goal-execution/references/runner-isolation.md)
reference: the child receives only a public source-only fixture and fixed task
prompt, with no evaluator contract, private harness, coordinator transcript or
hidden expected output. A forbidden-path read or context leak is invalid
protocol evidence even if the evaluator later passes.

The product direction remains sound: specialize in executable repository conventions and agent-assisted initialization, with existing language tools supplying their own checks. The execution mechanism should be a **reviewed release train**, not a long-lived mega-branch or a blind attempt to close every checkbox. The queue remains the technical source of truth; this goal supplies cross-session control, integration discipline, and cleanup rules.

On 2026-09-05, `origin/master` remains `ed093668918bc271fc98b9112acaf7c1bf3eb314`. PR #142 is still open and unstable, with macOS and Alpine failures, so installer work must be reused or repaired rather than duplicated. The worktree inventory has pre-existing and prunable entries; cleanup must be ownership-based rather than a broad deletion. The historical post-onboarding execution goal is `completed` and must not be reopened as the current program.

## Copy-paste continuation goal

```text
Continue the active Assura Maturity Execution Train to completion. Use one
supported runtime goal for the program; do not create a replacement goal at a
card checkpoint or stop after a status report.

Objective: turn Assura into a trustworthy, narrowly positioned repository-policy tool for agent-assisted development, while producing evidence suitable for a technical product / AI systems leadership portfolio. Execute the approved backlog end-to-end through small, reviewed, current-master integrations—not one large branch. The source of truth is:
/Users/nroth/workspace/assura/.trellis/tasks/09-04-maturity-portfolio-strategy/
Read prd.md, research/execution-backlog.md, research/backlog.json, the relevant packet for each card, and research/executor-prompt.md before work.

Begin with B00. Refresh GitHub master, CI/PR state, worktree ownership, the actual released version, and all existing planning evidence. The SHA recorded in the plan is a snapshot, never a permanent baseline. Do not implement from the older strategy checkout. If the planning artifacts are still uncommitted, first validate and preserve them in a dedicated documentation handoff; do not lose or silently fold them into unrelated product work.

Operate as a controlled release train:
1. Select only a ready card whose dependencies have evidence and are in the current branch ancestry. Work in an isolated, current-master worktree and a clearly named branch. One behavior card per PR by default; parallel work is allowed only for independent cards with separate worktrees and review capacity.
2. Follow the card's prescribed solution exactly enough to preserve its contract: reproduce or write the focused failing test first, make the smallest cohesive change, run focused checks then the required repository tier, and record the exact SHA, cwd, binary, commands, exits, negative control, limitations, and next-ready card in research/evidence/<ID>.md. Update backlog.json honestly.

   For A07, bind the candidate by absolute executable identity inside the
   initializer's actual command environment before counting a run. Capture
   `command -v assura`, `assura --version`, and the SHA-256 there; a login-shell
   `PATH` mismatch invalidates the run and receives no allocation credit. Follow
   [a07-candidate-binding-plan.md](a07-candidate-binding-plan.md), repair the
   runner, and complete a fresh canary before repeating screening.
   Launch the initializer as a fresh one-shot child with a minimal explicit
   login-shell-safe `PATH` and a source-only fixture. Do not pass evaluator
   contracts, private harness paths, coordinator transcripts or hidden expected
   outputs into that child. If it reads forbidden context or invokes a global
   Assura binary, retain the run as invalid protocol evidence and give it no
   screening, holdout or acceptance credit.
3. Before any merge, obtain an independent review for behavior, CI, release, public-contract, or complex changes. Refine valid findings, rerun affected gates, and verify the reviewed SHA is the tested SHA. Never weaken policy, hide benchmark rows, claim generated hooks are active, or count skipped/zero tests as passing merely to clear a card.
4. Merge only a clean, review-resolved, current-master PR whose required hosted and local gates pass. After merging, verify the exact commit is reachable from origin/master, update the card to done only when its observable outcome exists, and remove only worktrees/branches created by this goal after confirming they are clean and merged. Inventory existing worktrees first; use prune dry-runs; never delete unknown, user-owned, or dirty paths.
5. Follow the queue's dependency graph and phase boundaries: establish baseline and support scope; repair trust/release evidence; build safe init/hooks/gates; run blinded evaluation; then release, portfolio, pilot, and feedback work. When a result contradicts the plan, repair the owning card or explicitly narrow supported scope—do not lower the evaluator or rewrite history.

Execution-continuity invariant: do not stop at a card checkpoint. At every
boundary, retain a named live action (test, CI, review, integration, cleanup),
resolve its failed gate, or immediately select the next independently ready
card. Record action, owner, source SHA/worktree, proof required, and the exact
next observation or command in the progress/evidence record. Treat a status
report, a passing local gate, a submitted PR, a merge-ready card, or an empty
pending queue as intermediate state whenever an active, implemented, verified,
integrated, or cleanup action remains. A held publication/authority action
holds only that action; continue independent authorized work. Final handoff is
allowed only for an explicit user pause, completed authorized scope with owned
branches/worktrees terminal, or an independently audited external prerequisite
after all independent authorized work is exhausted.

Keep scope stable: invest in structure, naming, local patterns, explainable policy, bounded feedback, hooks, CI, and independent initialization proof. Do not expand project intelligence, semantic search, remote pattern execution, marketplaces, generic maturity scoring, autonomous PM, or arbitrary auto-repair.

Maintain a progress log after every meaningful iteration. Every third iteration and before final handoff, review context health, repeated failures, and whether a concise project skill would prevent rediscovery. Keep AGENTS.md as a router and put operational detail in a skill only when it is genuinely reusable.

This goal authorizes merging reviewed, fully gated Assura code/documentation PRs into main. It does not authorize tags/releases, deployments, branch-protection changes, external invitations, posts, or other public communication: prepare those artifacts and stop for the specific approval required by their backlog card.

Complete only when every applicable card is done, or is honestly held on a named
external decision with completed local preparation and evidence after an
independent impasse review confirms that no authorized continuation remains.
Final handoff must summarize merged commits/PRs, outstanding held actions,
release/pilot evidence, remaining branches/worktrees created by the goal
(normally none), and the next human decision. Do not end with a generalized
status report while the execution-continuity invariant identifies another action.
```

## Start/continuation condition

At the first session, create this goal through the supported goal operation. On
resume, reconcile the existing active goal instead of creating a duplicate.
The text intentionally binds execution to live state at each continuation and
preserves the detailed per-card prompt for low-context implementers.
