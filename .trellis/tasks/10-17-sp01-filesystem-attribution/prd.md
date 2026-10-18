# SP01: Attribute core file-size metadata work

## Objective

Measure whether `src/cli/check/file_limits.rs::validate_file_size` performs an avoidable metadata lookup for files whose effective full-engine `FileBundle` policy has no applicable `max_size` rule. Make at most one smallest local change, and retain it only if ordinary public `assura check --quiet` meets the existing performance keep bar with equivalent findings and exit behavior.

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

## Admission and current phase

This task is intentionally left in Trellis `planning` until the coordinating root confirms the verified Beads claim/admission. No build, remote workload, runtime edit, or test execution is authorized in this preparation phase. Once admitted, follow the repo Trellis implementation/check workflow and obtain a separate independent Luna review before any PR or merge; root coordinates those transitions.

## Acceptance

1. Task remains uniquely owned by `/root/filesystem_performance` and isolated on `perf/sp01-filesystem-attribution` from the specified current base.
2. After admission, provenance-bound attribution distinguishes the no-size-policy and size-policy arms, includes the naming-only negative control, and preserves raw unfavorable outcomes.
3. Any retained code change is limited to the listed scope and passes the correctness controls plus all existing keep/no-slower thresholds. Otherwise provide an explicit evidence-backed defer decision.
4. Candidate is reviewed by a separate Luna reviewer before PR/merge, with concrete findings resolved or dispositioned and scoped rereview after fixes.
