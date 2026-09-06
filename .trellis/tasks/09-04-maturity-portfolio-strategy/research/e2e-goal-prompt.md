# Stable E2E goal: Assura maturity execution train

## Direction review

The product direction remains sound: specialize in executable repository conventions and agent-assisted initialization, with existing language tools supplying their own checks. The execution mechanism should be a **reviewed release train**, not a long-lived mega-branch or a blind attempt to close every checkbox. The queue remains the technical source of truth; this goal supplies cross-session control, integration discipline, and cleanup rules.

On 2026-09-05, `origin/master` remains `ed093668918bc271fc98b9112acaf7c1bf3eb314`. PR #142 is still open and unstable, with macOS and Alpine failures, so installer work must be reused or repaired rather than duplicated. The worktree inventory has pre-existing and prunable entries; cleanup must be ownership-based rather than a broad deletion. The historical post-onboarding execution goal is `completed` and must not be reopened as the current program.

## Copy-paste new-session goal

```text
Run the Assura Maturity Execution Train to completion.

Objective: turn Assura into a trustworthy, narrowly positioned repository-policy tool for agent-assisted development, while producing evidence suitable for a technical product / AI systems leadership portfolio. Execute the approved backlog end-to-end through small, reviewed, current-master integrations—not one large branch. The source of truth is:
/Users/nroth/workspace/assura/.trellis/tasks/09-04-maturity-portfolio-strategy/
Read prd.md, research/execution-backlog.md, research/backlog.json, the relevant packet for each card, and research/executor-prompt.md before work.

Begin with B00. Refresh GitHub master, CI/PR state, worktree ownership, the actual released version, and all existing planning evidence. The SHA recorded in the plan is a snapshot, never a permanent baseline. Do not implement from the older strategy checkout. If the planning artifacts are still uncommitted, first validate and preserve them in a dedicated documentation handoff; do not lose or silently fold them into unrelated product work.

Operate as a controlled release train:
1. Select only a ready card whose dependencies have evidence and are in the current branch ancestry. Work in an isolated, current-master worktree and a clearly named branch. One behavior card per PR by default; parallel work is allowed only for independent cards with separate worktrees and review capacity.
2. Follow the card's prescribed solution exactly enough to preserve its contract: reproduce or write the focused failing test first, make the smallest cohesive change, run focused checks then the required repository tier, and record the exact SHA, cwd, binary, commands, exits, negative control, limitations, and next-ready card in research/evidence/<ID>.md. Update backlog.json honestly.
3. Before any merge, obtain an independent review for behavior, CI, release, public-contract, or complex changes. Refine valid findings, rerun affected gates, and verify the reviewed SHA is the tested SHA. Never weaken policy, hide benchmark rows, claim generated hooks are active, or count skipped/zero tests as passing merely to clear a card.
4. Merge only a clean, review-resolved, current-master PR whose required hosted and local gates pass. After merging, verify the exact commit is reachable from origin/master, update the card to done only when its observable outcome exists, and remove only worktrees/branches created by this goal after confirming they are clean and merged. Inventory existing worktrees first; use prune dry-runs; never delete unknown, user-owned, or dirty paths.
5. Follow the queue's dependency graph and phase boundaries: establish baseline and support scope; repair trust/release evidence; build safe init/hooks/gates; run blinded evaluation; then release, portfolio, pilot, and feedback work. When a result contradicts the plan, repair the owning card or explicitly narrow supported scope—do not lower the evaluator or rewrite history.

Keep scope stable: invest in structure, naming, local patterns, explainable policy, bounded feedback, hooks, CI, and independent initialization proof. Do not expand project intelligence, semantic search, remote pattern execution, marketplaces, generic maturity scoring, autonomous PM, or arbitrary auto-repair.

Maintain a progress log after every meaningful iteration. Every third iteration and before final handoff, review context health, repeated failures, and whether a concise project skill would prevent rediscovery. Keep AGENTS.md as a router and put operational detail in a skill only when it is genuinely reusable.

This goal authorizes merging reviewed, fully gated Assura code/documentation PRs into main. It does not authorize tags/releases, deployments, branch-protection changes, external invitations, posts, or other public communication: prepare those artifacts and stop for the specific approval required by their backlog card.

Complete only when every applicable card is done, or is honestly blocked on a named external decision with completed local preparation and evidence. Final handoff must summarize merged commits/PRs, outstanding blocks, release/pilot evidence, remaining branches/worktrees created by the goal (normally none), and the next human decision. Do not end with a generalized status report.
```

## Start condition

Create the goal in the **new** session using the text above rather than creating one in this planning session. It intentionally binds execution to live state at start time and preserves the existing detailed per-card prompt for low-context implementers.
