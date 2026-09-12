# Branch hygiene checkpoint

Read-only audit: 2026-09-12, `origin/master` at
`29f3207818cacd622cb881dac0bfbceb5cb50aef`. The entries below are clean
local worktrees owned by the Assura maturity train, with no matching active
Codex thread. They are historical candidates or holds, not inputs to CF01--CF04.

| branch | tip / last commit | current disposition | restore handle |
| --- | --- | --- | --- |
| `goal/a05-native-harness-lifecycle` | `5995ce13acf02ac6ea6852e8a39909bdb6ab4181`; 2026-09-07 | Archive locally; A05 acceptance is already closed through current-master PRs #200, #202, #204 and #206. | `git branch goal/a05-native-harness-lifecycle 5995ce13acf02ac6ea6852e8a39909bdb6ab4181` |
| `goal/r01-external-config-rescan` | `ff223250f3ab342a9f5782a6a3973b04c8a5bd07`; 2026-09-06 | Archive locally; current ledger marks R01 `not_needed` under the owner-approved H01 decision. Preserve failed-run and unknown-callback evidence. | `origin/goal/r01-external-config-rescan` or the tip SHA |
| `goal/r03-calibration-cohort` | `4c3c4981cf5be19e7dc49ac7d090b01d3070871f`; 2026-09-08 | Archive locally; R03 is terminal `done` through the reviewed current-master route; retain calibration artifacts as historical evidence. | `origin/goal/r03-calibration-cohort` |
| `goal/r03-calibration-collector` | `0d0f941f0418877bd6e0b6fb2ae284852644745a`; 2026-09-08 | Archive locally; historical calibration collector and failure-boundary candidate, not a CF input. | `origin/goal/r03-calibration-collector` |
| `goal/r03-count-severity-parity` | `36e01ac5850e164654f4e5ff1797c42bd484f164`; 2026-09-07 | Archive locally; old count-severity candidate held outside this goal. Reopen only with a fresh current-base review. | `git branch goal/r03-count-severity-parity 36e01ac5850e164654f4e5ff1797c42bd484f164` |
| `goal/r03-scope-rules-attribution` | `5e98e1e84218b01b7480ded9e5629807883c60b7`; 2026-09-07 | Archive locally; contains a rejected/reverted performance candidate and a historical next-slice note. | `git branch goal/r03-scope-rules-attribution 5e98e1e84218b01b7480ded9e5629807883c60b7` |
| `goal/r07-self-check` | `8bd957bd7990c93513393e6dc167b9e5350182f9`; 2026-09-05 | Archive locally; R07 is already terminal `done` in the current ledger. | `git branch goal/r07-self-check 8bd957bd7990c93513393e6dc167b9e5350182f9` |
| `goal/w02-performance-claim-containment` | `cba6628f7cf87fedbb2a761be312653c46d8262a`; 2026-09-08 | Archive locally; W02 remains explicitly held and this website candidate has no current-base merge authorization. | `origin/goal/w02-performance-claim-containment` |
| `goal/w02-release-aware-installation` | `790b0a9fa579fe4dd1feaa17a7d467572652fd42`; 2026-09-07 | Archive locally; W02 remains held until Nick grants Cloudflare approval for a current-master hosted proof. | `git branch goal/w02-release-aware-installation 790b0a9fa579fe4dd1feaa17a7d467572652fd42` |

Four additional local goal worktrees were already exact ancestors of
`origin/master` and are safe to remove as local registrations: A05 onboarding
quality (`717de56`), R03 fast rules (`c97acf2`), R03 fresh attribution
(`e6b6623`), and R03 performance variance (`c8df8cc`). The current CF
execution worktree is retained. Historical remote branches are not deleted by
this audit unless their exact tip is already reachable from the current
integration ref and the attached worktree is clean.

Owner for this checkpoint is Nick/Codex. The next action for every archived
entry is “none until explicitly reopened”; all restore handles above preserve
the exact candidate tip without treating historical evidence as current proof.

Cleanup completed after the dry-run: generated-only ignored paths were removed,
the twelve clean stale/merged worktree registrations were removed, and all
thirteen listed local branches were renamed to `archive/2026-09-12/*`. No
tracked or unknown source was removed, no remote branch was deleted, and the
active `goal/compact-feedback-execution` worktree was retained. The local
topology now has only that active `goal/*` branch; archived refs and the remote
refs preserve restoration. Disk free space increased from 17 GiB to 50 GiB.

## Execution-train cleanup

CF01 completed through PR #328 at merge commit
`75992ad4cfcba4efb1cfc8a1d3333208991bba88`. Its candidate tip
`eff94fd8d856fed163c5b5ff73072bcca2c364c9` is an ancestor of that merge and
its clean local/remote branch and worktree were removed after all post-merge
workflows passed. Restore the candidate with
`git branch goal/compact-feedback-execution eff94fd8d856fed163c5b5ff73072bcca2c364c9`.

The planning-only ref `codex/compact-feedback-plan` was also an exact clean
ancestor of `origin/master` at `48f18fab7527150b384603ca64cec8b374295842`.
Its remote/local ref and attached clean worktree were removed after the plan
was carried into CF01. Restore it with
`git branch codex/compact-feedback-plan 48f18fab7527150b384603ca64cec8b374295842`.
CF02 was the sole active execution candidate: owner Nick/Codex, worktree
`/Users/nroth/.codex/worktrees/assura-compact-feedback-cf02`, and it completed
through PR #329 at merge commit `9383acc2fd9a47d63553fd32768aef7428c7b34c`.
Its candidate tip `ea634c5401a27578c62b4b0de2ba6db30dddd583` is an ancestor of
that merge; focused tests, independent review, current-base gates, exact-head
PR checks, and post-merge Rust CI/Documentation/Security workflows passed.
The clean CF02 worktree, local branch, and remote branch were removed after
reachability proof. Restore it with
`git branch goal/compact-feedback-cf02 ea634c5401a27578c62b4b0de2ba6db30dddd583`.
Owner Nick/Codex; handle none; next action none unless CF02 is deliberately
reopened. The current CF03 candidate is owner Nick/Codex at
`/Users/nroth/.codex/worktrees/assura-compact-feedback-cf03`, branch
`goal/compact-feedback-cf03`, base `9383acc2fd9a47d63553fd32768aef7428c7b34c`,
next action implement CF03 and preserve this restore decision.
The detached historical
worktree at `/Users/nroth/.codex/worktrees/2f5f1792-b405-41ee-8885-bbb1ad693526/assura`
has no branch owner and remains untouched.

CF02 review follow-up: the independent review found seven boundedness and
reconciliation risks. Accepted fixes in the active candidate cover Git
timeouts, honest status unknowns, untracked-path counts, hostile diff
isolation, retained-branch tree matching, bounded/schema-checked cache reads,
and no cache-hit reuse for partial snapshots. Two reviewer-created dirty
snapshots remain recoverable as stash commits
`851de4adc7050437d2f53e1e7006801e6879d84a` and
`f220df766396106ba2b4e01960a707feb4a65761`; they are preserved foreign work,
not candidate commits.

CF02 completed through PR #329. Candidate tip
`ea634c5401a27578c62b4b0de2ba6db30dddd583` merged into `origin/master` as
`9383acc2fd9a47d63553fd32768aef7428c7b34c`. The exact-tip focused tests,
independent review, current-base gates, and hosted PR checks passed before the
merge. Post-merge Rust CI run `34713601124`, Documentation run `34713601114`,
and Security Audit run `34713601129` all passed for the merge SHA. The clean
CF02 worktree and local/remote candidate branch were then removed. Owner is
Nick/Codex; next action is CF03 implementation. Restore the archived candidate
with `git branch goal/compact-feedback-cf02 ea634c5401a27578c62b4b0de2ba6db30dddd583`.

## CF03 ownership checkpoint

CF03 remains one owned candidate at `/Users/nroth/.codex/worktrees/assura-compact-feedback-cf03`,
branch `goal/compact-feedback-cf03`, based on CF02 merge `9383acc2fd9a47d63553fd32768aef7428c7b34c`.
The product candidate is Nick/Codex-owned; commits `3e82ea8` and `0f78e42`
implement the bounded automatic delivery contract, `3148110` scopes its
delivery state to the canonical worktree, and `aaabba4` excludes the foreign
note from the net candidate diff. The exact current tip is retained until
CF03 review and merge gates finish.

An unrelated harness-matrix note was observed in the shared candidate while
another validation owner was active. It is deliberately excluded from the
candidate net diff and preserved as foreign work: owner unknown/foreign;
handle `stash@{0}` / `389641009075d6eec126adc6a9fca8a225a37f23`; next action is
owner review before any reapplication; restore with
`git stash apply 389641009075d6eec126adc6a9fca8a225a37f23`.
No unknown or foreign source was deleted.

During the final CF03 freeze, another owner reintroduced a foreign harness
matrix note and an alternate `bound_line` implementation into this shared
worktree while the current-base gate was starting. They were not reviewed as
candidate work and were parked intact: owner unknown/foreign; handle
`stash@{0}` / `f826b855eb7d76bcb4e1fa3d6736d98e54d9aa3e`; next action is owner
review before reapplication; restore with `git stash apply f826b855eb7d76bcb4e1fa3d6736d98e54d9aa3e`.

A second overlapping harness note was observed later and is also deliberately
excluded from the candidate net diff: owner unknown/foreign; handle
`stash@{0}` / `8205a4a92167ece55276da24cd17b1576ed62fe2`; it contains the
additional `SKILL.md` and `references/harness-hook-matrix.md` edits. The next
action is owner review before any reapplication; restore with
`git stash apply 8205a4a92167ece55276da24cd17b1576ed62fe2`.

During CF03, a foreign harness-contract overlay was briefly committed as
`0e815a9`; it is excluded from this candidate and remains recoverable from
that commit. Owner unknown/foreign; next action is owner review; restore with
`git show 0e815a9 -- .agents/skills/assura-agent-harness-hooks` (or apply the
existing preserved stash for the same owner work).
