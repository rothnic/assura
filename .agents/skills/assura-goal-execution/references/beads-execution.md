# Beads-backed execution

Use this route only for a user-authorized Beads goal. Trellis continues to own
specs, candidate contracts and native delivery receipts. Beads is the sole live
queue for readiness, ownership, dependencies, progress and issue closure. A
review `backlog.json` and graph are dated intake artifacts, never parallel
status stores. Keep a source-key/Beads-ID map and evidence pointers in the goal
task; query Beads for current state rather than copying a second queue.

## Identity and adoption

Read the applicable Beads skill and installed help. Record executable/version,
`bd context --json`, `bd where --json`, `bd info --json`, environment overrides,
primary Git checkout and any existing registry/redirect. A missing local folder
alone does not authorize another database. Reuse the verified project store;
leave unrelated/global stores and registrations intact.

Before initializing, inspect the Git remote's `refs/dolt/data` as well as
local stores/registries: `bd 1.2.2` automatically bootstraps that remote from
cwd even with an explicit `BEADS_DIR`. Existing remote history must be reused,
not replaced by an independent empty tracker. Inspect a preserved copy using
compatible native Dolt tooling if the embedded engine cannot decode it.

Use installed help and exact source before selecting backend/recovery flags.
In observed 1.2.2 behavior, `--skip-agents --skip-hooks` preserved those files,
but `--setup-exclude` added broad patterns to tracked `.gitignore`; do not
assume help guarantees repository-local info/exclude only. Preserve/review
any owned side effect. No force, reinit, upgrade, schema repair or migration
as a substitute for identity and compatibility proof.

Verify actual backend/server mode and issue prefix after recovery. Keep one
coordinator writer and pass verified `BEADS_DIR` and actor across worktrees.
Read-only workers do not claim or mutate issues. A local copy of a configured
remote is distinct from a successfully synchronized write; verify native
backup/sync and remote identity under the installed version's contract.

## Import and retries

1. Freeze the reviewed input revision and graph SHA-256; inspect all source
   keys, titles, acceptance, labels, `parent_key` and edges. Check duplicate
   source identities and current store issues before creation.
2. Use the installed graph dry run. In the reviewed 1.2.2 plan, an edge's
   `from_key` is the dependent and `to_key` its prerequisite; confirm with
   actual readback rather than relying on an arrow's appearance.
3. Retain the exact creation output and key-to-ID map. Read all created issues,
   parents, blocking edges and ready/blocked sets. Verify expected counts,
   uniqueness, absence of cycles and original dependency meaning. Parent
   membership is not a replacement for blocking edges.
4. On an interrupted/unknown creation outcome, query the same store and
   reconcile source keys, IDs and edges before another write. Never blindly
   import again. Prefer completing the original mapping to duplicate issues.
5. Use the version's Dolt-native backup/sync path and read back the result.
   JSONL export is useful interchange/view evidence, not database backup or
   cross-machine synchronization. No configured remote means local state;
   do not claim it was published or synced elsewhere.

## Per-bead loop

Read the selected issue and actual dependency readiness, latest notes, linked
contract and current source. Review descriptions can contain historical states;
the live issue and its latest evidence decide the next action. Verify real
ownership/liveness and isolate source edits in a clean owned checkout. Reuse
the existing Trellis task for the outcome when one exists.

Record a bounded outcome, source/base, owner, worktree and next proof. Implement
the smallest complete change and preserve meaningful failing/passing controls.
Before merge, obtain an independent GPT-6 Luna review of the exact candidate
and connected behavior; adjudicate findings, apply accepted corrections, rerun
affected checks and obtain scoped rereview. Source, dependency, configuration
or policy changes invalidate affected proof; metadata-only changes get a
proportionate review/gate delta. Never lower required thresholds.

Merge reviewed current-base candidates only after all applicable gates pass
and the action plus any known coupled deployment has authority. Observe
configured post-merge workflows and actual behavior, prove merged reachability,
then close against full acceptance with review/check/outcome evidence. Clean
only the exact owned checkout after native delivery closure. Verify issue
readback and database backup/sync after meaningful transitions.

Research/non-code beads receive independent outcome review too. A supported
keep/defer decision may be the declared result; do not manufacture code or an
empty PR to make research look delivered. An epic needs its own outcome review.
Human, participant, evaluator and release acceptance must be observed under
their real contracts. Hold only the affected action and continue other
dependency-ready authorized work. Preserve failures and real external holds;
never simulate acceptance or close from status alone.
