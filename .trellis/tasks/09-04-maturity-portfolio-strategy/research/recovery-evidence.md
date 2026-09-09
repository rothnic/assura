# Recovery process verification

Date: 2026-09-09. Scope: process artifacts, agent instructions and validation
routing only. Product acceptance is unchanged; no card is promoted by this file.

## Candidate and proof

- Base: `da773bce8315cbb9e69941f0fe89d0a91e42829d`.
- Process implementation: `ceb3029009cc573f9cd6a37c84a8270193c7111b`.
- Checkout: `/private/tmp/assura-execution-recovery-plan`.
- Branch/owner: `goal/execution-recovery-plan`, this thread's process coordinator.
- `git diff --check`: exit 0.
- `cargo xtask evidence`: exit 0; CI scope controls and evidence policy passed.
- `cargo run --quiet -- check --format json .`: exit 0, success true; four
  existing low-severity line-count advisories in unchanged Rust/test files.
  No new advisory in changed artifacts. Source output retained locally in
  `/private/tmp/assura-recovery-structure.json`.
- `target/debug/xtask target-state`: exit 0 using the source-built xtask.
- Workflow gate with canonical task: exit 0, Ready yes, clean owned checkout.
- `scripts/ci-scope.sh --base origin/master --head HEAD`: exit 0;
  evidence=true, Rust/release/performance/rustdoc/website/security=false.
  Classifier and workflow configuration are unchanged.
- `git diff --quiet origin/master HEAD -- src tests Cargo.toml Cargo.lock
  xtask .github/workflows scripts .assura/config.yml`: exit 0; those product,
  policy and CI surfaces are unchanged. Full Rust/platform/performance reruns
  are not claimed for this process-only diff.

AGENTS shrank from 296 to 93 lines; its managed Trellis block is retained.
The stale hardcoded minimum Rust version was replaced by the manifest/CI
source route. Goal execution now routes Trellis tasks and layered references.

## Independent audit

Read-only agent `recovery_audit` identified five concrete process issues at the
base: broad `fast`/`pr` duplication, catch-all environment recovery, cwd drift in
the old recipe, unsuitable dirty-diff VPS transport, and evidence-policy checks
mistaken for outcome proof. All five were accepted and corrected in the skills
and recovery plan. The audit found existing hosted proof-aware scope reuse;
it remains unchanged. It did not establish external provider deployment rules.

Independent review of `ceb3029` accepted the recovery sequence and exercised
seven decisions: current-base state supersedes stale checkout; failed required
performance holds merge; live timeout resumes its handle; process scope does
not authorize product edits; VPS capacity/toolchain must be checked; unknown
dirt is preserved during owned branch closure; passing slice does not close
failing A07. These are instruction simulations, not operational outcome proof.

Three medium findings were accepted: P1 restored unconditional `Ready: no`
routing; P2 qualified remote compiler/Cargo identity with the selected
toolchain; P3 retained `cargo xtask docs` for any docs/website path instead of
narrowing it to build inputs. Scoped rereview remains required after these
corrections. No hosted-green claim is made before that run completes.

## Topology and remote observations

Global `audit-topology.sh --strict`: exit 128 on pre-existing missing worktree
registration. This is not a passing strict gate. Read-only fallback inspected
all 14 registered goal worktrees: clean, with reachability classified against
origin/master. Eight pre-existing branches plus this process branch are
unmerged; five point to commits reachable from master. These are reachability
facts, not permission to remove others' work. The root's untracked research
file is untouched. `git worktree prune --dry-run -v` reported three missing
registrations; no prune/removal was performed.

VPS probes succeeded through configured alias `vps` after `vps-dev` resolution
failed. CPU/RAM, disk and toolchain observations are in the recovery plan.
No remote builds, cleanup, runner installation or benchmark speedup is claimed.
The bundle validation procedure is documented, not yet operationally timed.

The existing heartbeat was updated through the app to follow the latest user
scope and revision-aware task evidence. Its response reported ACTIVE. The
goal tool returned no active goal; automatic goal continuation is not assumed.

## Next phase

Resolve candidate review, rerun affected checks, then submit the reviewed final
candidate for applicable hosted checks. Merge only after those pass. Verify
integration and remove only this process slice's clean owned branch/worktree.
The wider recovery plan remains for the train coordinator to execute.
