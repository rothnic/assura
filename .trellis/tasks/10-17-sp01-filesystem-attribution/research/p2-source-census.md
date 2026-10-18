# SP01 P2 source census and measurement gate

Status: source-level consumer paths are mapped; direct P2 syscall/count profiling is deferred because the fresh VPS gate failed. No P2 runtime counts, elapsed time, CPU, RSS, or bytes are claimed. This is a measurement limit, not a performance result.

## Source and fixture identity

The census was read from the SP01 checkout after P1 disposition commit `cd001a6473671d141576ef18d616b279783b0330`. Its Rust and Cargo files are byte-for-byte equal to both `49d85358` and authoritative main `ba4804bb` (`git diff --exit-code <ref> -- '*.rs' Cargo.toml Cargo.lock`, exit 0). This P2 note is source analysis only; it was not produced from an instrumented binary. The repo's current `.assura/config.yml` has one nonempty test-relationship policy and one nonempty docs-lifecycle policy. The older P1 wide-tree, per-case config hashes and binaries remain in the separate P1 evidence bundle; they do not identify a P2 measurement fixture.

## Identical-scope test and lifecycle consumers

For a full root check where `checked_path == project_root`, `validate_test_relationships` in `custom_constraints.rs` calls `test_relationship_files` twice with the same root: once for source files and once for project files. Each call creates an independent `walkdir::WalkDir`, applies the same exclusions, collects all files, then sorts and deduplicates. The second listing is used for required-test matching and ignored Rust tests; the latter opens matching `.rs` files with `read_to_string`. Fixture-family validation separately calls `read_dir` once per configured fixture root, then queries each entry's type. Test code is in `tests/test_relationship_tests.rs` and its compiled counterpart. A subtree check still walks both the checked subtree and full project root, which are different scopes and must not be treated as identical.

For the current root check, the one `docs_lifecycles` policy has both `active` and `require_frontmatter_status` selectors nonempty. `validate_docs_lifecycle_policy` in `docs_lifecycle.rs` independently calls `matching_docs_lifecycle_files` for each selector, so the identical root is walked twice, with separate match vectors and sorting/deduplication. It reads each matched status file, each active file, and each declared evidence file that exists and is inspected for a claim. `tests/docs_lifecycle_tests.rs` covers pass, configured statuses, missing status, historical link, and claim drift. Source inspection establishes two test-relationship root walks plus two docs-lifecycle root walks; actual entries visited, metadata/stat syscalls, directory reads, opens/bytes, CPU, wall time and RSS were not counted.

## NU13 discovery/config-candidate paths

The four requested cases are separable in source, but have no P2 runtime counter receipt:

| Case | Source path | Source-level behavior | Runtime candidate/stat count |
| --- | --- | --- | --- |
| No-cache, root | `fast_cli.rs::run` → `run_structure_check_with_target_mode` → `discover_project` | Direct `.assura/config.yml` existence check returns immediately for a root directory containing the config; config/project canonicalization follows. | Not measured |
| No-cache, nested target | same no-cache path → `ConfigDiscovery::find_config_path` | Starting at target directory/parent, `symlink_metadata` checks each `.assura/config.yml`; `.git.exists()` is checked at each unsuccessful candidate before moving to parent; then returned config/root are canonicalized. | Not measured |
| Warm-cache, Git | `fast_cli.rs::run` → `default_check_cache_dir` → `git_value`; then `run_structure_check_cached` → `discover_project` | Cache-root selection uses `git rev-parse --git-common-dir`; it does not use the non-Git `discover_project` fallback. Cached runner still performs normal project/config discovery once. | Not measured |
| Warm-cache, non-Git | same, `git_value` fails → `default_check_cache_dir` fallback → `discover_project`; then cached runner → `discover_project` | Discovery is repeated: once to select the default cache root, once in the cached runner. Root direct-config short circuit versus nested candidate walk applies at each call. | Not measured |

Existing NU13 validation documents discovery/cache unit-suite counts and an 11-call public CLI journey, but these are correctness receipts, not syscall attribution or a comparable performance experiment. Exact P2 output/exit parity on instrumented invocations was not run.

## Fresh resource/disk gate

Command: one bounded `ssh vps` snapshot at `2026-10-18T03:57:29Z`, capturing hostname/kernel/date, uptime, free memory, `df -h/-T` on `/home/ubuntu/data/projects` and `/var/tmp`, PSI, owned-run sizes, and a top-process sample. Host identified as `vps-9cb01956`, Ubuntu, x86_64. Results: load `11.49, 7.91, 6.90` on the 16-CPU host; memory available 34 GiB; root ext4 339 GiB, 321 GiB used, 15 GiB available (96%); memory PSI `some avg10=0.00`, CPU PSI `some avg10=20.24`, I/O PSI `some avg10=1.33`, I/O `full avg10=1.07`. Active foreign CPU work included GC/GasCity and a Bazel/Java Beads run, with additional foreign services/processes. None were stopped, modified, or treated as owned.

The CPU pressure and nearly-full disk fail the workload gate. No P2 build, tracer, fixture execution, benchmark, cache cleanup, or filesystem instrumentation was started. The older NU13 directory `/home/ubuntu/data/projects/assura-nu13-20261017.TLlZ9a` was only size-read (2.8 GiB) and was not modified; the SP01 run root size-read 2.9 GiB. Disk-backed `/var/tmp` is the same ext4 filesystem, so it does not solve the free-space constraint. Continue only after a new coordinator-authorized resource gate admits one bounded jobs=2 instrumentation build; otherwise keep the four-case runtime counts and output/exit parity explicitly unmeasured. No foreign data or caches were deleted.
