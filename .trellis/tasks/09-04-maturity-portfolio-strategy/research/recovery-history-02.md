# Preserved recovery evidence tail

The topology-audit helper correction was moved here from
`recovery-evidence.md` to keep the active index within its configured line
limit. It remains historical process evidence.

## Topology-audit helper correction — 2026-09-10

- During this continuation reset, the previous orchestration helper
  `audit-topology.sh` emitted a raw Git exit `128` before its `SUMMARY` when a
  prunable worktree directory still existed but its `.git` metadata was gone.
  That failure was diagnostic-tool fragility, not evidence that the topology
  was clean. The exact old script hash was
  `654f0f826a632804f77d89977eb90a2c8306010e88b766c1739eb934a6eb1483`.
- The owned process correction is installed in
  `/Users/nroth/.codex/skills/assura-orchestration`: the helper now probes Git
  metadata and status explicitly, emits `UNREADABLE_WORKTREE` on failure,
  counts unreadable records in `SUMMARY`, and preserves a nonzero strict
  result. The final script hash is
  `67635d3f48143c5b3cf9196b192d138f50770f8d947cf2df2eb81722f4cb0ce5`; the
  accompanying skill guidance hash is
  `49544ac6b0d6493c49f962b2bfdd90ab674111a24754c6fd10b25c912f22332d`.
  No repository product, evaluator, threshold, release, deployment,
  publication, or invitation surface changed.
- `bash -n` passed. A disposable repository with an external child worktree
  whose `.git` file was removed produced `CLEAN_WORKTREE`,
  `PRUNABLE_REGISTRATION`, `UNREADABLE_WORKTREE`, and a complete
  `SUMMARY base=HEAD worktrees=2 dirty=0 prunable=1 unreadable=1
  goal_branches=0 unmerged_goal=0`; report mode exited `0` and strict mode
  exited `1`. This confirms the helper no longer aborts or misclassifies the
  stale registration as clean.
- The live Assura report now reaches
  `SUMMARY base=origin/master worktrees=34 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9` with report exit `0`; strict exits `1` for
  the preserved root/user dirt, dirty external worktree, stale registrations,
  and unmerged goal branches. No prune, deletion, ownership reassignment, or
  unknown-work mutation was performed. Independent process review returned
  `PASS` with no findings for the exact script and skill hashes above.
- This recovery slice is owned by the process coordinator in
  `/private/tmp/assura-topology-audit-robustness` on branch
  `docs/topology-audit-robustness`, based on live `origin/master=d099253`.
  The next action remains the A07 acceptance coordinator's private
  exactly-two-condition manifest and isolated protocol review; topology
  inventory may continue independently, while R01/W02/F01 retain their named
  external/diagnostic holds.
- The committed evidence candidate `e834dbbf2780e304ca77560c1f2aa39c3c393041`
  passed the process/docs tier: the workflow gate reported `Ready: yes`, the
  source check returned `success: true` with the same six unchanged low
  max-line advisories, and `cargo xtask evidence`, `cargo xtask target-state`,
  `cargo fmt --all -- --check`, `git diff --check`, JSON parsing,
  `assura check --format agent --agent codex`, and
  `scripts/ci-scope.sh --base origin/master --head HEAD` all exited `0`.
  CI scope classified `evidence=true`, `changed_count=2`, with all
  product/Rust/release/performance/rustdoc/website/security surfaces false.
  The first docs attempt exited `1` because `website/node_modules` was absent;
  the frozen `pnpm --dir website install --frozen-lockfile` bootstrap exited
  `0`, and the identical `cargo xtask docs` rerun exited `0` after building 48
  pages. The failed precondition is retained and is not counted as a pass.
