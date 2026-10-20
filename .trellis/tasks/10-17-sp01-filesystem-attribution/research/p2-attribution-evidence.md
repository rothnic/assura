# SP01 P2 attribution evidence correction

## Accepted boundary

The preserved `results-v5/run-manifest.json` contains **six paired cases**: `rich-root`, `rich-nested`, `naming-only-no-cache`, `cache-git-root`, `cache-git-nested`, and `cache-nongit-root`. Each baseline/instrumented pair exited 0 and the saved stdout, stderr, and exit artifacts compare byte-for-byte. The separate `tiny-positive-control` is a smoke row outside that six-case manifest; it exited 1 on both binaries with matching output and nonzero discovery probes. Do not call the matrix seven rows or fold the smoke control into its totals.

These are source-level request/entry/read counters, not OS syscall counts, timings, speedup, or optimization-retention evidence. All three cache-shaped instrumented rows report `cache.local_hits=0` and `cache.shared_hits=0`; treat these as cache-root/discovery/config-read attribution only. The zeros prove neither a warm hit nor a cache defect. No performance claim follows. P1 remains the independently reviewed no-keep/revert result.

## Six paired rows

| Case | Test relationship | Docs lifecycle | Discovery/cache attribution |
|---|---|---|---|
| `rich-root` | 2 walks; 10,472 entries / 10,256 files; 5,000 reads / 150,000 B; fixture roots 1 / entries 1 | 2 walks; 10,472 entries / 10,256 files; 256 matches; status 128 reads / 4,096 B; active 128 reads / 4,096 B | no-cache 1; project 1; direct config check 1; config loader 1 / 778 B |
| `rich-nested` | 2 walks; 5,287 entries / 5,178 files; 5,000 reads / 150,000 B; fixture roots 1 / entries 1 | 2 walks; 102 entries / 100 files; no matches or reads | no-cache 1; project 1; direct config 1; candidate metadata 4; Git markers 3; config exists 1; canonicalize 2; config loader 1 / 778 B |
| `naming-only-no-cache` | no test counters | no docs counters | no-cache 1; project 1; direct config 1; config loader 1 / 125 B |
| `cache-git-root` | no extension counters | no extension counters | Git common-dir 1; fallback 0; cached runner 1; project 1; direct config 1; cache config 1 / 125 B; local/shared hits 0 / 0 |
| `cache-git-nested` | no extension counters | no extension counters | Git common-dir 1; fallback 0; cached runner 1; project 1; direct config 1; cache config 1 / 125 B; local/shared hits 0 / 0 |
| `cache-nongit-root` | no extension counters | no extension counters | Git common-dir 1; fallback 1; cached runner 1; project 2; direct config 2; cache config 1 / 125 B; local/shared hits 0 / 0 |

`tiny-positive-control` is recorded separately under `tiny-positive-control/`; it is not an additional matrix case.

## Source callers and scope interpretation

| Consumer / caller | Source path and selected scope | What the fixture shows |
|---|---|---|
| Test-relationship source selection | `test_relationship.rs::validate_test_relationships` calls `test_relationship_files(checked_path)` and then `test_relationship_files(project_root)` | At root, both arguments are the same path: two identical-scope walks. At nested target, one walk covers the nested checked subtree and the other covers the project root; these are different required scopes, not a duplicate same-scope walk. |
| Docs-lifecycle active selector | `docs_lifecycle.rs::validate_docs_lifecycle_policy` calls `matching_docs_lifecycle_files(active_patterns, checked_path)` | One enumeration for the active-doc selector. |
| Docs-lifecycle status selector | Same caller invokes `matching_docs_lifecycle_files(status_patterns, checked_path)` separately | A second enumeration over the same root/path scope with a distinct pattern set and consumer purpose. Do not call it duplicate work solely from equal root paths. Root fixture has matching files for both selectors; nested fixture has none. |
| NU13 configuration discovery | `fast_cli.rs` / cache-root selection and cached runner, with measured probe counters at their existing discovery boundaries | Root/no-cache cases take direct config paths; nested no-cache exposes candidate metadata/Git-marker checks; Git cache cases use common-dir lookup; non-Git cache root takes fallback and repeats project/direct-config checks. These are source counters, not syscalls. |

All counts above describe the saved instrumented source boundaries: `WalkDir` loop requests and yielded entries, matching files, explicit reads/bytes, and selected discovery/cache events. WalkDir internals and kernel syscalls are outside the probe.

## Provenance and retained attempts

- Source HEAD: `49d85358bae85de04f1ce7faa746b377687dcaf3`; baseline and instrumented source roots derive from this commit. The instrumentation exists only in the scratch copy/patch.
- Temporary patch SHA-256: `3264a00c52dab8b8d40b9188c9f063187ac20772f6a11fde82cff82bd020e2d9`.
- Baseline executable SHA-256: `126a3bdfb232031cd90aa50886320c87cb47834c8fdadcda1ce2e094e014dba1`.
- Instrumented executable SHA-256: `8011b0f7aa74992c5b2f8a0de2b10e956f13ea6e4972cf2a67bb72ea0b431205`.
- Fixture generator SHA-256: `4145e224dfda62cde583fe292ad2695a1cf7839105fee90b707cd6201950fac6`; v3 fixture manifest SHA-256: `bbe6a2b887e0c6af25c0fcbd95122ce5bedfdbae818b2dfaab07cedc04e7749e`; full fixture-tree digest: `61e3f2cc24f10ad285f888df4aa4fbfcb0d33be4eddbddf00c13e99e84f74723`.
- v5 runner SHA-256: `e20581f6ed8cc3fa54a18d723c44566cf67da1f4dd5b6752e69e8670c5a0b5ae`; command receipt SHA-256: `df4e97a0e3a6a90fdb49c8697743482b861506208be8acc821792b6f2b643a04`; six-row manifest SHA-256: `b7496be3e90b379bd397f9ab7605111c405d2cff8d9b6b64d2434c159f8e27cc`.
- v1 is preserved but excluded for the failing policy fixture; v2 is preserved but excluded for the extra positional operand; v3 is preserved as an earlier corrected attempt; v4 is explicitly parity-only because it emitted no counter files; v5 is the accepted six-pair source-count/parity dataset. The initial instrumented build that reused the baseline binary and the later mtime-triggered instrumented rebuild are both preserved and distinguished in the raw run directory.
- Raw files remain at `/Users/nroth/workspace/.assura-review-20261017/sp01-p2-mac-preparation/run-20261018-mac-p2-counts/`. Earlier attempts were not replaced.

## Harness correction (prepared, not run)

`/Users/nroth/workspace/.assura-review-20261017/sp01-p2-mac-preparation/run_cases_v6.py` is a proposed fail-fast successor, **not a measurement and not executed against Assura**. Before its first subprocess, it pins source HEAD, patch, fixture generator/manifest/full fixture-tree digest, and both executable hashes to the identities above. It requires every intended-success invocation to exit zero; rich and naming reports must have the exact expected canonical `project_root` and `checked_path`. Every counter artifact must contain the complete 34-key schema, exact integer non-negative values, and entry-subset plus zero-read/zero-byte invariants (allowing valid zero-byte reads). Its usage string is labeled v6, correcting v5’s stale `run_cases_v4.py` error label. This only corrects future harness gates; it does not modify or retroactively upgrade v5 receipts. The proposed runner SHA-256 is `a1c269e8d4c25773908f052879df3b5ca9454d8cf3c2057ebd9cc45bbe94cd74`. Syntax-only `py_compile` passed; no CLI, build, or new matrix attempt was run.

## Single controlled default-cache discriminator (2026-10-18)

One bounded seed/repeat protocol was run after a fresh Mac resource/disk gate. It used the existing pinned baseline and instrumented binaries only; no build, source change, timing, or public-performance claim. The 125-byte naming-only config is LS-Lint-fast-plan/cache eligible and has SHA-256 `3023fea37a4569ec8d8d69da20d5d40242532fd8b999ec012193f4cc6fc74959`. Each of the three fixture shapes was copied into distinct baseline and instrumented trees before launch. Each arm ran a cold check and one unchanged repeat using the default `--cache` selector (no `--cache-dir`), for a hard maximum and actual total of 12 Assura CLI calls. This preserved default Git/non-Git cache-root selection separately from report reuse.

The run receipt is `/Users/nroth/workspace/.assura-review-20261017/sp01-p2-mac-preparation/cache-discriminator-20261018/run-01/protocol-run.json` (SHA-256 `cce29ed7e05b5ef1f26dd245f470915f59a1141fd612f96930e3a12a3e247ccc`). It records pinned source/patch/binary/fixture/config identities, each initial isolated-tree digest, full argv/cwd/exit and output hashes, counter paths/hashes, per-step cache-file snapshots, path assertions, and all 12 parity comparisons. The focused runner is `/Users/nroth/workspace/.assura-review-20261017/sp01-p2-mac-preparation/cache_discriminator_v1.py` (SHA-256 `f6a2a79f1387966d47ac1379e35a63356dcfd10794e53ac1726cf42749257ba7`). Each invocation's stdout, stderr, exit, counter JSON and cache snapshot were saved before its assertions.

All 12 invocations exited 0. Each JSON report had `success: true`, and exact `project_root` and `checked_path` equal to that arm's intended project target; the two arms did not share mutable project trees or cache directories. Output bytes were compared between baseline and instrumented and cold and repeat executions after normalizing only each isolated absolute project-root prefix; all 12 saved comparisons passed. The complete 34-key counter schema and non-negative integer/entry/byte invariants passed for all six instrumented invocations.

| Fixture | Default cache root observed | Instrumented cold and repeat attribution | Result |
|---|---|---|---|
| Git root | `<input>/.git/assura/check-cache` | `root_git_common_dir_queries=1`, fallback 0, project/direct-config checks 1; each run reads config 1 / 125 B; local/shared hits 0 / 0 | Cache marker and one `worktrees/...json` report were present after cold and unchanged after repeat; no hit observed. |
| Nested project inside Git repo | `<input>/.git/assura/check-cache` (the parent repository's common directory) | `root_git_common_dir_queries=1`, fallback 0, project/direct-config checks 1; each run reads config 1 / 125 B; local/shared hits 0 / 0 | Cache marker and one `worktrees/...json` report were present after cold and unchanged after repeat; no hit observed. |
| Non-Git root | `<input>/.assura/cache/check` | `root_git_common_dir_queries=1`, fallback 1, project/direct-config checks 2; each run reads config 1 / 125 B; local/shared hits 0 / 0 | Cache marker and one `worktrees/...json` report were present after cold and unchanged after repeat; no hit observed. |

This run ends as `completed_no_hit_hold`. The candidate wrote a cache report, and an unchanged repeat found the file in the default root, but both hit counters stayed zero in every fixture. This does **not** prove report reuse and does **not** diagnose a cache defect. The sole protocol is complete; there was no retry. Any follow-up requires a separate scoped review of the retained cache files/source before any further run.

### Counter coverage and source-level interpretation

Read-only inspection of the pinned instrumented source confirms that the tested `assura check --cache <single-project>` path selects the default cache root in `fast_cli.rs` and delegates to `run_structure_check_cached`. In that function, there are three cached-report return sites: two guarded by `fresh_cached_report` (local cache) and one guarded by `fresh_shared_report` plus shared-path validation (shared cache). The instrumentation increments the corresponding local/shared hit counter immediately before each of those returns. The remaining `Ok(report)` returns in this function return the just-computed engine report (including the pre/post snapshot-change early return); they are not cached-report returns. The source search for `run_structure_check_cached`, `fresh_cached_report`, `fresh_shared_report`, and the two hit counters found no other CLI fast-cache return path in this source tree. Thus the observed 0/0 means no hit at the three known cached-report return sites reached through this tested CLI path. It does not establish that reuse is impossible in other entry points or configurations, and it does not explain why these eligible unchanged repeats missed.

The source-count data also motivates only a follow-up hypothesis: when the checked target is exactly the project root, the test-relationship consumer appears to collect the same root-scoped file set once for the checked target and again for the project root. At a nested target these scopes differ and must remain distinct. A future slice could test reuse only for the identical-path case, with nested-scope and naming-only controls; it is not an established benefit or an authorized implementation. Any such slice still needs correctness/output/exit parity and public measurements that meet the existing retention and no-slower gates before a keep decision.

The pre-run gate at `cache-discriminator-20261018/host-gate-before-run.json` (SHA-256 `9a9d0c62db5d2d0ef33865d2247a5cebf24f37a7fa747d922a22533652d3c9ad`) recorded a 16-CPU Mac, 10.62 load, 11.91 GiB free/inactive/speculative pages of 32 GiB, and 118 GiB free on the disk-backed APFS data volume. Active Node/Vitest and Codex processes were present, so no host exclusivity claim is made. Post-run gate `host-gate-after.json` (SHA-256 `cdf37d6164a41a7b34cfb1f15127014dc9d57b5c07776e9ce52c9b276374519a`) showed 9.91 load, 11.12 GiB free/inactive/speculative pages and the same 118 GiB disk headroom; no process matching the owned runner or binaries remained. Evidence and run trees were retained without cleanup.

## Remaining boundary

Both the original v5 warm-shaped rows and the later isolated default-cache protocol have zero local/shared hits. The controlled protocol retained separate arm trees and caches, but warm report reuse remains unproven and no defect is diagnosed. The counter coverage is scoped to the three cached-report return sites in the tested `run_structure_check_cached` CLI path. No public timing or syscall measurement exists. Keep the P1 no-keep decision, thresholds, and adverse evidence intact; the no-hit result does not authorize retries or a source change.
