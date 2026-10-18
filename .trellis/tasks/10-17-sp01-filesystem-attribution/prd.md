# SP01: Attribute core file-size metadata work

## Objective

Attribute filesystem work in ordinary policy-rich checks. P1 tested whether `src/cli/check/file_limits.rs::validate_file_size` performs an avoidable metadata lookup when the effective full-engine `FileBundle` policy has no applicable `max_size` rule. Three independent retention reviews found no-keep; that guard is reverted. Continue the same task with bounded attribution-only P2. No source optimization is currently retained.

This is the first discriminator from the systemic filesystem review, not a general filesystem optimization project. The existing source hypothesis is copied into `research/`. The work does not explain or replace the separate NU11/R02/R03 many-scope no-slower obligation.

## Scope and owner

- Candidate: `sp01-filesystem-attribution`.
- Delivery owner: `/root/filesystem_performance` (Trellis creator/assignee: `filesystem_performance`).
- Base: `origin/master` at `49d85358bae85de04f1ce7faa746b377687dcaf3`.
- Implementation scope, if admitted: `src/cli/check/file_limits.rs` and only necessary adjacent existing tests or benchmark fixture. Temporary instrumentation and raw samples live in disk-backed scratch outside the repository.
- No shared inventory/index, daemon, API, dependency, persistent cache, broad traversal change, or incremental eligibility expansion. No Beads/issue-store writes in this task.

## Measurement discriminator

Use the same admitted idle host, exact candidate/base source trees, same release profile, same fixture/policy, and ordinary public CLI. Contrast:

1. A full-engine policy-rich wide tree with `FileBundle` rules active but no applicable `max_size` default or pattern rule.
2. The same tree with a controlled default/pattern `max_size` policy.
3. A naming-only fast-path negative control, which must remain behaviorally and operationally unchanged.

First use disposable scratch instrumentation to attribute eligible-file counts and stat-family calls specifically to size validation, plus consumer-attributed walk, open/read, byte, CPU, wall, and peak-RSS work where the profiler supports it. Instrumentation counts mechanism only; do not use instrumented timings as the performance result. Then compare uninstrumented `assura check --quiet` on the ordinary fixture in balanced alternating baseline/candidate order, with two order-reversed runs on that same host. Record exact commands, exits, cwd, source/tree and policy hashes, executable path/hash, toolchain, raw samples, and resource-gate evidence. Do not infer causation from phase medians.

After admission, remote sustained builds and runs must follow the infrastructure maintenance resource/ownership gate. Use SSH alias `vps` only after verifying hostname `vps-9cb01956` and Ubuntu; create an owned fresh project directory under `/home/ubuntu/data/projects`, use verified disk-backed `/var/tmp` scratch with a short `TMPDIR`, and limit to two jobs/one compiler lane. Do not touch the previous NU13 source/build directory. At teardown verify owned processes are reaped.

Public timing comparison uses exact quiet commands on both sides and the existing exact-quiet tie-breaker. Keep a source change only when both runs support at least 5% and 0.75 ms improvement on the public row, the exact-quiet tie-breaker improves at least 3%, accepted-fixture spillover is at most 5% or 0.25 ms, and strict no-slower/native/warm gates pass. A keep decision requires both public thresholds, not either one. If noise prevents a decision, preserve samples and repeat only within the bounded two-run protocol; do not cherry-pick favorable samples.

## Correctness acceptance

The no-size-policy arm must issue zero size-metadata requests from this validator. With size rules, preserve default and pattern boundaries/overrides. Preserve missing and unreadable behavior, symlink handling, zero-byte files, complete findings/order/severity, and CLI exit status. Verify the naming-only fast path remains unchanged. Include positive, negative, and valid-exception controls in the focused proof after admission.

No fix is required if the measurable public gain is below the keep bar. The accepted outcome may be a documented defer with the exact measurements and preserved unfavorable evidence.

## P1 disposition and bounded P2 attribution

P1 is complete as a no-keep disposition. Preserve its raw timings, syscall counts, correctness receipts and binaries as historical evidence. Do not rerun P1 public A/B or strict keep-gate comparisons, and do not present instrumented timings or syscall reduction as public speed. The exact quiet tie-breaker fixture was `/home/ubuntu/data/projects/manual-many-scopes-common`; its historical config/tree hash was not captured and must remain marked unavailable.

P2 continues within this task and is attribution-only. Enumerate the identical-scope test and docs-lifecycle traversals, including the paths and lifecycle/callers that trigger each traversal. Separately attribute NU13 discovery/config-candidate metadata for four cases: no-cache root, no-cache nested target, warm-cache Git repository, and warm-cache non-Git repository. Preserve the naming-only negative control and full output/exit parity. Count, by consumer where practical, `stat`/metadata, walk, readdir, open, read, and bytes; record CPU, wall time, and peak RSS where meaningful. Use only provenance-bound rich-policy and controlled-wide fixtures. Instrumented timing is diagnostic and is never a public speed result.

P2 may use temporary instrumentation in the exact existing consumer files and an isolated one-lane build only when required. It must not add a dependency or alter production source, shared inventory/index, daemon, API, traversal model, or incremental eligibility. No optimization is authorized by attribution alone. Record baseline SHA/tree, scratch diff, binary and fixture hashes, exact commands/exits, resource/disk gate, raw counts/samples, and teardown. Before any VPS work, refresh the host and disk-backed scratch gate; use the existing owned toolchain/cache only if safe, jobs=2, and do not mutate the separate NU13 source/build run. Avoid new full-target builds or deletion of foreign data. If the gate is unsafe, retain that as a measurement limit and finish independent source/test enumeration without timing claims.

## Saved P2 attribution evidence

The v5 data set is accepted as a six-paired-row source-count/output-parity result, with the tiny positive control separately labeled. It does not establish OS syscall counts, cache hits, performance improvement, or optimization retention. All three cache-shaped rows have local/shared hit counts of 0/0; neither a warm hit nor a cache defect is established. The exact source callers/scopes, counter results, identities, preserved v1-v5 attempts, and both prepared harness revisions are documented in `research/p2-attribution-evidence.md`. One later isolated default-cache seed/repeat protocol used the pinned binaries only; it exited successfully with no local/shared hits. Warm reuse and a cache defect remain unproven. No build, performance claim, or further retry was made.

## Admission and current phase

The coordinating root verified Beads claim/admission for `ae-xsm`; Trellis/native ownership remains `/root/filesystem_performance` (`filesystem_performance`). The P1 source disposition is committed. Continue P2 under the same canonical task; do not create another task or write to Beads. Root coordinates independent review and any later delivery transitions.

## Acceptance

1. Task remains uniquely owned by `/root/filesystem_performance` and isolated on `perf/sp01-filesystem-attribution` from the specified current base.
2. P1 records the no-keep/revert disposition, preserves unfavorable raw evidence, and does not claim a retained speedup.
3. P2 provides provenance-bound identical-scope test/lifecycle enumeration and separates NU13 discovery/config-candidate attribution across the four required root/nested and Git/non-Git cases, with consumer counts and output/exit controls where available.
4. No P2 optimization is retained absent a separately authorized scope and evidence. Record exact provenance, resource gates, measurement limitations and owned-process teardown.
5. Independent review checks the final P1/P2 evidence and any source disposition before root makes delivery decisions.
