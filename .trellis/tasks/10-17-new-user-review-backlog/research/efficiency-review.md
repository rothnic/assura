# Assura efficiency pass — October 17, 2026

The highest-leverage improvements are reducing adoption mistakes, exposing the
right configured finding, and making the next repair concrete. These two
projects do not justify another cold-start optimization campaign. Full checks
are already tens of milliseconds; repeated Review is several times slower,
and output volume and missing signal can dominate the actual agent workflow.
This is bounded diagnostic evidence, not an ecosystem benchmark or adoption ROI
measurement.

## Candidate, method and configuration

- Exact published executable: `/Users/nroth/workspace/.assura-review-20261017/bin/assura`,
  `assura 0.4.1`, SHA-256
  `05355b9639d0295c51b3ccc7f077311cdb39b90a6cbc4583aae2239e389ac0fe`.
- Adjacent full companion SHA-256:
  `4dfdb3dea035ac0fa0f8025864a356074bede59bd0c62dab09a66da93f26650c`.
  Both hashes and version were reverified before measurement.
- Host: macOS Intel x86_64; parent admission reported 32 GiB, 69% free memory,
  16 logical CPUs, load about 7, 134 GiB disk available. Ending load was
  3.36/5.45/5.50. Shared-host variability is not controlled.
- APFS disk scratch: `/dev/disk1s2`, `/System/Volumes/Data`. TMPDIR and requested
  XDG cache root were within this owned directory. No builds, dependency installs,
  services, inference, other hosts or original project changes.
- The retained TS and Python onboarded projects were copied, including their
  disposable Git history, into `typescript/` and `python/` here. All product
  commands were serial. Git hook execution was disabled in these copies.
  Python's four retained explicit negative probes were removed before sampling
  (names in measurements.json); no owner policy was changed.
- TS config SHA-256: `96a585205c38a1571adff16e8b2c74772d66ec53d6637d188d2fa9fa220a2d32`.
  Python config SHA-256: `2330529600ac76fca51b396db8600d0dc3a10377f4bd24e43ed2c14edaa458e6`.
  Checksums and Git status are identical before/after. The generated TS quality
  scope and Python onboarding failure were retained; these are different real
  configurations, not interchangeable performance workloads.
- Five fresh-process samples per command/project, 80 completed timed calls,
  timeout 10 seconds each. No warmup, cache flush or cold-disk claim. Copying and
  earlier trials already warmed OS caches. Elapsed time includes Python launch
  supervision and `/usr/bin/time -l`; stdout was captured, not rendered in a
  terminal. RSS is the OS-reported maximum, not summed simultaneous process-tree
  memory or long-running resident-service cost. No p95 claim from five samples.
- Review used explicit `--base HEAD`; history in these disposable repositories
  represents the trial snapshots, not the originals' full development histories.
  New absolute paths create fresh path-keyed Review history; samples 2–5 reuse it.
  Automatic hook delivery was not sampled. CLI probes are explicit reports.

`measure.py` records exact argv, cwd, times, exit, stdout/stderr and RSS in
`transcript.json`; `measurements.json` summarizes samples, configurations,
coverage, provenance and calculated scenarios.

## Current samples

All times are median milliseconds; parentheses show min–max. Sizes are UTF-8
stdout bytes, including wrappers/newline. RSS is the maximum across five samples
in MiB. Explicit agent output uses `--min-severity low --max-issues 5`.

| Command | TS ms (range) | Python ms (range) | TS/Python bytes | TS/Python RSS MiB |
| --- | ---: | ---: | ---: | ---: |
| `check --format json .` | 37.8 (34.6–39.2) | 56.1 (52.9–63.0) | 7,898 / 68,332 | 2.55 / 2.99 |
| `check .` | 36.9 (33.2–41.0) | 58.2 (56.3–62.6) | 5,264 / 49,378 | 2.55 / 3.02 |
| `check --format agent --agent codex ... .` | 36.7 (35.0–41.0) | 59.9 (55.3–62.3) | 3,272 / 3,189 | 2.70 / 3.04 |
| `check --quiet .` | 33.5 (31.8–34.9) | 60.9 (59.5–65.2) | 0 / 49,378 | 2.41 / 3.03 |
| `review --format json --base HEAD .` | 212.9 (202.5–222.5) | 530.8 (505.1–555.0) | 11,110 / 12,018* | 6.49 / 16.72 |
| `review --base HEAD .` | 210.0 (203.6–220.5) | 507.9 (494.4–518.2) | 439 / 536 | 6.45 / 14.52 |
| `doctor --format json .` | 38.0 (37.3–39.2) | 58.4 (58.1–62.0) | 2,463 / 2,847 | 5.39 / 5.79 |
| `explain --format json <existing-source>` | 17.2 (16.5–17.7) | 19.6 (19.0–20.8) | 4,726 / 4,675 | 4.76 / 4.76 |

*Review JSON bytes shown for samples 2–5; initial path-keyed history samples were
11,069/11,971. Explain selected `components/message.tsx` and
`tools/agent-workflow/src/agent_workflow/cli.py` respectively.

TS Check covered 273 files/68 directories, 18 low findings, exit 0. Python Check
covered 449 files/140 directories, 129 findings (128 low/1 medium), exit 1. Every
sample retained these counts and exits. Python has 112 agent-guidance findings;
that policy mismatch is an adoption cost already captured by NU02, not evidence
that the engine takes longer on Python syntax. Review remains advisory exit 0
for the blocking Python project; Doctor exits 1. Quiet suppresses success text,
while preserving Python failure diagnostics; that is the documented safety
contract, not an output bug.

Repeated Review correctly changes history from new=7/8 to unchanged=7/8, and
does not invent worsening on repeated identical inputs. It still reports
advisory=0 despite 18/128 configured nonblocking findings. Its compact text puts
optional content-model setup before the TS project's active source health
findings. Extra speed would not fix this missing signal (NU01).

## Workflow efficiency is more than elapsed time

The bounded agent option reduces TS Check JSON output by 58.6%, Python by 95.3%.
It truthfully reports 13/124 omitted findings; blocking comes first. This is an
existing capability users can use now. However, the five items repeat the same
references and generic instructions. In particular, custom folder-health text
replaces the actual count/limit, root paths render empty, and closed-scope
diagnostics omit offending children. The user needs another discovery pass
before deciding the repair. NU07 should retain counts/thresholds and name bounded
offenders, and deduplicate repeated generic guidance without hiding evidence.
This explicit output is about 3.2 KiB; it must not be confused with the already
implemented 256-byte routine automatic-statistics contract.

The following is arithmetic only, not a 100-call experiment, token estimate or
real task duration. It multiplies sampled medians and observed median bytes by
100, assumes unchanged behavior, and excludes native tests, user reading,
model processing, setup, repair and output rendering:

| Calculated workload | TS wall seconds / bytes | Python wall seconds / bytes |
| --- | ---: | ---: |
| 100 full JSON checks | 3.78 / 789,800 | 5.61 / 6,833,200 |
| 100 explicit agent checks (five findings) | 3.67 / 327,200 | 5.99 / 318,900 |
| 100 JSON Reviews | 21.29 / 1,111,000 | 53.08 / 1,201,800 |
| 100 cycles of Check+Review+Doctor+Explain JSON (400 calls) | 30.58 / 2,619,700 | 66.50 / 8,787,200 |

Review was about 5.6x/9.5x the Check JSON median in these conditions. This does
not establish a regression against July's synthetic warm-loop budgets: those
used another commit, dirty build, workload and direct companion invocation.
It supports operation selection and a bounded profiler question, not a caching
rewrite. A documented efficient loop should gate with Check, use Review at
meaningful checkpoints, use Doctor for setup/troubleshooting and Explain for
one confusing path. Existing changed-path/session modes can serve repeated
events, subject to their proven coverage; no fresh installed-hook measurement
was made here.

Setup cost remains unquantified. These generated configs have 145/132 lines;
the previous trials required evidence-based specialization and preservation
checks, and Python onboarding failed twice against pre-existing agent policy.
The TS closed-scope example needed an explicit extension cardinality allowance
before the intended baseline passed. The 65/80 previous receipts are experiment
instrumentation, not normal-user command-count estimates. A 0.329-second TS
onboard call is not time to a dependable working policy. User minutes,
agent tokens/cost, actual false-positive disposition, repair-loop count,
continued use/retention and total trust/approval overhead were not measured.

## Published benchmark claims and fairness

The July 5 decision matrix is useful for its stopping rule, but its numeric
snapshot is stale. It says cold strict 2x=2/8. Checked-in
`benches/history/current.json` and website's copied current report instead have:

- Date September 30, 2026; source `3d3c37b04d842f9e73e8686ed92486890cde4d73`,
  clean worktree, macOS Intel i9-9880H, Rust 1.94.1, five iterations, release
  binaries. This is dated evidence, not today's hosted gate or the 80 calls here.
- Eight accepted generated naming/count/scope/ignore fixtures, migrated shared
  LS-Lint policy, native parity metadata. Cold Assura faster 8/8, strict 2x 1/8,
  aggregate 1.308x (sum of fixture medians). Warm session 2x 8/8, aggregate 24.798x.
  Eight generated fixtures are not eight independently evaluated user projects.
- Pinned `@ls-lint/ls-lint@2.3.0` resolves its native Go binary before measurement;
  rows say `native-binary-from-pinned-npm-package`. Measured loops do not pay npm,
  Node wrapper or package resolution. Assura public cold command is ordinary
  release `assura check --quiet`, and both sides include new-process startup,
  configuration/traversal/validation/exit with output discarded. Source alternates
  order per iteration; odd five-sample cohorts have Assura first one extra time,
  explicitly labeled. More decisive marginal results need balanced even cohorts
  and repeated same-host order reversal, not a threshold relaxation.
- Warm headline is a persistent session client plus already started daemon:
  full CHECK warmup occurs outside the measured loop, then a selected changed
  path is marked and DIRTY-PROJECT-PATH timed. Headline session rows declare
  `proves_whole_project_success=true` with one changed path after this baseline.
  Separate socket dirty-path diagnostic rows declare false; do not conflate them.
  Session startup, initial full validation and steady background resource cost
  are outside those per-request medians. This is a different execution model
  versus repeated native CLI, not an intrinsic 24.8x engine advantage or an
  installed-host-hook latency claim. Website already explicitly labels this
  persistent-session comparison and distinguishes hook cost.
- Rich Markdown/reference/guidance/content checks are not part of the
  naming-equivalent cold comparison. These configured-project samples exercise
  different policy surfaces, so their 38–56 ms cannot be compared to fixture
  rows as a product regression or competitor speed result.
- Release linking matters: the skill requires a lightweight launcher built
  with `--no-default-features --features json-output,yaml-config`, separately
  built full companion/check-only binaries, release LTO/one codegen unit/strip/
  abort. A monolithic all-bins development build is not equivalent. Linux
  static-CRT evidence has a separate scope from the macOS rows used here.
- `warm-loop-current.json` is July 10, commit `dba8d08...`,
  `source_worktree_dirty=true`, direct `target/release/assura-full`, 20 synthetic
  samples (no-change Review median 108.2 ms). It must retain that provenance and
  cannot certify current released real-project or actual-hook performance.
- The prior review's failed current-master CI no-slower comparison remains a
  separate unresolved gate (11.812 vs 11.650 ms on many scopes). Historical 8/8
  success cannot clear it; these samples neither reproduce nor dismiss it.
  The root orchestrator is responsible for refreshing hosted state.

No competitor was executed, installed or rebenchmarked in this pass. Upstream
LS-Lint currentness and ecosystem research belong to the separate source-backed
ecosystem pass; these are audit findings about Assura's retained artifacts.

## Incremental coverage cross-check

The orchestrator supplied `../capability-pass/transcript.json` for an independent
environment/argv/source inspection. Its exact same pinned published executable
returns exit 1 for full Check with missing `src/components/Button.test.tsx`, but
exit 0/no violations for `check --format json src/components/Button.tsx`, with
no `--ls-lint-target-semantics` flag. The file-target report acknowledges only
one checked file/zero directories. This should not count as evidence that the
relationship passed or that a faster per-file invocation provides equivalent
policy coverage.

Current `src/cli/check/custom_constraints.rs::relationship_entries` walks the
checked path and unconditionally skips the entry equal to that path. For a file
target, this leaves no entries to match the captured producer. This supplies a
specific source explanation for the retained receipt, although no further run
or patch was made by this worker. Its intended file-target relationship contract
needs explicit acceptance; do not assume the omission is deliberate merely
because traversal is scoped. Separately, prepared-check API docs state that
repository-wide extensions and relationships require full fallback; cached full
checks in the orchestrator's retained receipts correctly catch counterpart
deletion and configuration changes. The explicit target issue therefore does
not establish stale-cache or persistent-session failure. The backlog should
require captured source/provider cases and negative controls for whichever
incremental operation is advertised, and truthful coverage metadata for limited
paths. Do not trade that proof for a smaller latency number.

## Backlog and stopping recommendations

1. Keep NU01/NU02/NU03 highest: truthful advisory selection, preservation of
   existing agent policy intent, and real source-root coverage prevent missed
   checks and unnecessary repair loops. NU08 adds explicit Python/Rust coverage.
   Evaluate outcomes, not merely setup exit status or count of supported modes.
2. Expand NU07's acceptance to quantify the one-repair journey: report actual
   count and limit, root label, bounded offending paths and omitted count in
   one deterministic result; retain project-specific guidance; deduplicate
   per-item generic boilerplate. A new abstraction is unnecessary.
3. NU11 should reconcile dated matrix/public claims with the checked report and
   current failed hosted gate, then retain comparable release/profile/platform
   labels. Do not create a parallel cold-optimization epic for historical 2x.
4. Reuse F02 for a small external adopter scorecard after blockers are repaired:
   time to first correctly configured negative control; manual decisions and
   repair commands; false positives versus genuine findings; task outcome;
   total output bytes/model-token costs when observable; retained use after
   repeated real changes. Bind latency and coverage to the same policy, binary,
   project and operation. This is pilot acceptance, not an indefinite benchmark
   program or a new dashboard. Participant authority remains separate.
5. CF01–CF04 are already terminal/merged (#328–331, CF02 #329, CF03 #330, CF04
   #331), according to their current skill backlog. They already cover bounded
   query loading/batch fallback, cached Git facts, cadence/async delivery and
   installed-hook/performance/opt-in trial evidence. Do not recreate those as
   open efficiency work. Check any new claim against their actual evidence;
   these explicit CLI samples do not establish a CF automatic-cadence failure.
6. Reuse the July stop policy: retain cold optimization only with public-row
   improvement >=5% and >=0.75 ms, exact command >=3%, no fixture spillover >5%
   or 0.25 ms, no-slower passing, and two same-host directionally consistent
   runs. Stop after three failed attempts/category, sub-ms floor/noise misses,
   auxiliary-only improvements, or user-visible command complexity for a score.
   A profiler-backed public bottleneck or real user budget breach can reopen
   it. The now-observed larger Review cost merits profiling only after proving
   the intended command cadence and preserving configured finding coverage.

Skipped builds, full perf cohorts, competitor execution, hook/runtime delivery,
original project edits and model-token/retention measurement. Diagnostic samples
on a shared Mac cannot certify release readiness, external adoption or causal
speed improvements.
