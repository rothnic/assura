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
execution worktree is retained. Remote branches are not deleted.

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
