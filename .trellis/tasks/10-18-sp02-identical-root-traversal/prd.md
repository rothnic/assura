# SP02: Reuse identical-root test-relationship traversal

## Objective

Evaluate the smallest possible reuse of test-relationship file enumeration when `checked_path` and `project_root` identify the same root during one check. SP01 source attribution found the two calls at `test_relationship.rs:18-19`; its accepted root fixture observed both walks visiting the same 10,472 entries and returning the same sorted file collection. This is a bounded hypothesis only. No public speedup, OS syscall reduction, or cache defect is established.

The eventual candidate may reuse a collection only when both roots are identical within a single `validate_test_relationships` call. A nested checked subtree and the project root have different scopes and both traversals must remain. Keep the candidate in `src/cli/check/test_relationship.rs` and necessary adjacent existing tests. Do not add public APIs, modules, dependencies, persistent/shared indexes, daemons, or incremental eligibility changes.

## Owner, base, and phase

- Candidate: `10-18-sp02-identical-root-traversal`.
- Owner: `sp02-identical-root-traversal`.
- Branch: `perf/sp02-identical-root-traversal`.
- Base: `refs/remotes/origin/master` at `da23fa7dab9600463cf234fceea060df2c8f5ccc`.
- Current phase: task contract and read-only source/consumer/measurement-plan preparation only. No source optimization, build, test, benchmark, remote work, runner install, or native lifecycle action is admitted in this phase.
- Scratch root when later admitted: `/Users/nroth/workspace/.assura-review-20261017/sp02`.

## Proposed correctness contract

If the checked root equals the project root, the check may enumerate that root once and supply the same canonical sorted/deduplicated file set to the existing source and project consumers. If the checked root is nested, preserve the nested source set and the separate full project set. In both cases retain current exclusions, error propagation for failed walks and entries, relative-path conversion, sorting/deduplication, findings and their order/severity, and CLI exit/output behavior. Missing/unreadable paths, symlinks, source/test matches, ignored Rust tests, fixture-family validation, and policy behavior remain covered by existing semantics.

Naming-only fast-path behavior and configured size-policy behavior are controls for unrelated work and must remain unchanged. Do not reuse SP01's failed metadata guard: it failed the public keep bar, was reverted, and must not be relitigated. The SP01 no-hit cache result establishes no cache defect and gives no basis for cache changes.

## Consumer evidence and measurement plan

Before any implementation or run, identify the exact ordinary public consumer command, current source/tree, effective policy, fixture tree, binary hash/path, and toolchain. Use SP01's accepted `rich-root` fixture and harness identities only after verifying their availability and identities from the saved evidence; do not edit SP01 receipts. Establish consumer cost on an admitted safe host before deciding whether to retain the candidate. Source counts alone do not establish wall-time benefit.

Any later comparison must use uninstrumented exact ordinary quiet commands on the same admitted host, same fixture/policy/profile, with two balanced runs in reversed order. Preserve every raw sample and exit. Both public rows must improve by at least 5% and 0.75 ms; exact-quiet tie-breaker must improve at least 3%; accepted spillover must be no more than 5% or 0.25 ms. Only after this initial keep bar passes may strict no-slower/native/warm gates be considered. No retry-until-green or cherry-picking. If costs or gains do not meet the bars, document a no-keep/defer result with unfavorable evidence.

Required identities and receipts: source SHA and tree, policy/config hash, fixture manifest/tree/generator hashes, executable path/hash, toolchain, exact argv/cwd/exits, raw samples, host/resource admission and teardown. Instrumented counts are diagnostic only and cannot substitute for public timing. Before each admitted batch, after retries/timeouts, every five minutes during prolonged work, and at teardown, use the infrastructure policy's resource and ownership gate. Prior VPS pressure and disk evidence are stale and do not authorize a run. NU03 owns the compiler lane; do not build until the coordinator explicitly admits a lane.

## Acceptance

1. Focused source and consumer evidence identifies the identical root case and distinguishes it from nested scope; no speedup is presumed.
2. Any later candidate is limited to identical-root reuse within one check in the existing module and preserves all stated behavior and negative controls.
3. Any retained change meets every public and secondary keep bar, all required strict no-slower/native/warm gates, and independent source/evidence review with accepted corrections and scoped rereview.
4. If a bar fails or the candidate cannot yield meaningful improvement, preserve unfavorable measurements and record a no-keep/defer outcome.
5. Final hosted, merge, post-merge, native closure, and owned cleanup are performed only by the coordinator under existing authority; Beads remains root-owned.
