# CI gate triage

Use this reference when a required check is slow, queued, failed, or has an
unclear scope. It keeps an expensive hosted run from becoming a status-only
checkpoint or a reason to weaken the merge contract.

## Classify before rerunning

Record the exact PR head, base, job/check name, scope decision, queue time,
execution time, exit, test/job count, and log URL. Classify the observation as
one of these states:

| State | Meaning | Next action |
| --- | --- | --- |
| `pass` | Required job ran at the exact reviewed head and passed | retain the proof |
| `fail` | The job ran and exposed a product, process, or environment failure | diagnose the first actionable failure; do not retry unchanged |
| `queued/running` | Hosted work has not produced a result | keep its handle and do independent authorized preparation |
| `skipped` | The changed-surface policy excludes the job | record non-applicable; never count it as pass |
| `zero/unknown` | No meaningful test or scope result was observed | unresolved; obtain the missing evidence |

An old result is not proof for a new head. A retry is justified only after a
changed candidate, repaired environment, or documented transient platform
incident. Preserve the failed row and the reason for any retry.

## Cheap-to-expensive order

For each owned checkout, run the smallest deterministic checks first:

1. verify `pwd`, repository root, branch, source SHA and clean status;
2. run workflow/context/structure/scope/target-state checks;
3. run the focused red/green contract and its negative controls;
4. run one serialized heavy local or remote command;
5. push once, then watch only the hosted jobs applicable to the changed
   surface.

Do not launch parallel Cargo commands in one checkout. `cargo xtask pr`
already includes `fast`; do not spend another unchanged `fast` run merely to
repeat a command name. `cargo xtask changed` is triage, not merge proof.

## Remote capacity decision

SSH is an optional execution venue, not a source of authority. Before using
`vps`, read `assura-local-build` and collect a bounded probe for alias,
available memory, free disk, load, existing owned jobs, OS and exact
toolchains. Use the isolated bundle procedure there for one committed
candidate only; record remote HEAD, lockfile hash, toolchain, elapsed time and
actual exit. Retain local platform-specific and hosted proof. Do not use a
remote Linux result to stand in for macOS, Windows, browser, release, or
Cloudflare checks.

Choose remote execution only when the measured disk and memory headroom can
hold the projected job plus margin and the required toolchain is present. If
capacity is marginal, keep the job local or select a smaller applicable tier;
never delete unrelated caches or worktrees to manufacture headroom. Start
with one heavy job per host and compare queue, execution, retry and success
data before proposing CI infrastructure changes.

## Merge fence

The coordinator may merge only when the exact reviewed head has resolved
review findings, all applicable local and hosted checks, required performance
rows and acceptance evidence. A queued job is not green, a skipped job is not
green, and a passing process slice is not a passing product card. After merge,
fetch again, prove the merge/tree relationship, rerun the ledger and topology,
and close only the exact clean owned branch/worktree.
