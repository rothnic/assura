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

## Remaining boundary

The saved cache-shaped matrix has zero local/shared hits and shares mutable seed/warm tree state, so warm report reuse remains unproven and no defect is diagnosed. A single controlled cache discriminator may be proposed only after independent review of the corrected harness. Current SP01 evidence also has no public timing or syscall measurement. Keep the P1 no-keep decision, thresholds, and adverse evidence intact.
