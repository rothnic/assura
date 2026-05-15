---
id: analysis-2026-05-15-incremental-cache-aware-checking-strategy
type: analysis
title: Incremental and cache-aware checking strategy
status: active
created: 2026-05-15
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/analysis/2026-05-15-ls-lint-good-enough-comparison-contract.md
  - docs/analysis/2026-05-15-notation-source-truth.md
  - .trellis/spec/assura/structure-enforcement.md
---

# Incremental and Cache-Aware Checking Strategy

This is the current design note for incremental `assura check` behavior. No
cache is implemented yet. Until the phases below land with tests and benchmark
evidence, the product truth remains a full structure-first check.

## Goals

- Preserve correctness before optimizing repeated checks.
- Keep cold full-check performance separately measurable from warm incremental
  performance.
- Avoid cache files that create git noise or Assura structure violations.
- Make invalidation explicit for config, binary, rule engine, schema, and root
  changes.
- Leave room for future notation features such as file pairing and package
  structure rules without baking unsafe file-local assumptions into v0.1.

## Change Detection Inputs

### Git-assisted detection

Use git as an optimization when the checked root is inside a worktree.

- `git status --porcelain=v2 -z` can identify staged, unstaged, untracked,
  renamed, copied, and deleted paths.
- Tracked file metadata from the index can narrow candidates, but the working
  tree must still be considered because Assura validates the files on disk.
- Staged and unstaged changes are both relevant. A local check should validate
  the working tree, not only the index.
- Untracked files must be included because closed-world direct-content rules
  can reject stray files and directories.
- Renames and deletes invalidate the old parent directory and new parent
  directory because direct counts, closed-world direct contents, and future
  package rules can change without a modified surviving file.
- Outside git repositories, or when git commands fail, fall back to the same
  full filesystem walk used by current `assura check`.

Git must never be the only source of truth. It is a fast path for candidate
selection; cached entries still need root/config/engine compatibility checks.

### Hash and metadata tracking

The safe cache key for a file-local result is content hash plus the rule set
that produced the result. Metadata shortcuts can reduce hashing work, but they
are not sufficient for correctness by themselves.

- Store file size, modified time, and a content hash.
- Reuse a cached file-local result only when size and modified time match the
  prior snapshot and the cache entry was produced by the same config hash,
  Assura version, rule engine version, schema version, and project root.
- If metadata changed, recompute the content hash and recheck active rules.
- If metadata is unreliable or unavailable, hash content before reuse.
- Prefer a fast portable hash with stable output across platforms. BLAKE3 is a
  good implementation candidate if a dependency is accepted; otherwise use a
  standard library or existing dependency-backed hash only after measuring the
  overhead.
- Directory-level entries should hash normalized child names, child types, and
  direct counts rather than file contents unless an active rule needs content.

Metadata shortcuts are safe only as a cache-hit prefilter. They must not hide
changed content when a file system reports coarse or stale timestamps.

## Invalidation Model

Every cache namespace must include:

- absolute or canonical project root identity;
- `.assura/config.yml` content hash and schema version;
- Assura binary version or build identity;
- rule engine version;
- cache format version;
- target platform where path normalization or metadata precision can differ.

Config changes invalidate all entries whose effective rules may change. The
first implementation should treat any config content change as a full cache
miss. Later versions can compute finer invalidation from resolved rule scopes,
but only after the resolved-rule dependency graph is explicit and tested.

Cache corruption, missing files, unreadable entries, unknown versions, or
unrecognized feature flags must fall back to a full check and rewrite the cache
only after a successful run.

## Dependency Scope

| Scope | Current checks | Incremental treatment |
| --- | --- | --- |
| File-local | File naming, size, line count, docs, markdown rules when enabled. | Recheck changed files and files whose effective rule set changed. Reuse unchanged file-local results only after metadata/hash and config identity match. |
| Directory-level | Direct child `files.exists`, `directories.exists`, `allow_extra`, allowed names/patterns, forbidden patterns. | Recompute for every changed, renamed, deleted, or untracked path's parent directory and for configured directories whose direct child set changed. |
| Subtree-level | Inherited naming and explicit child structure scopes. | Re-resolve rules for touched subtrees when config changes or when future pattern scopes are added. |
| Whole-project | Current product has no whole-project graph checks. | Full recompute for future dependency graph rules until dependency edges are stored and invalidated explicitly. |
| Future pairing/package rules | Planned notation may require matching files across directories or every package under a pattern. | Treat as directory/subtree or whole-project scope until the rule declares its dependency footprint. |

The first cacheable implementation should support file-local result reuse and
directory-level recomputation. It should not attempt whole-project dependency
incrementality until dependency validation exists.

## Cache Placement

Preferred placement order:

1. `.git/assura/cache/` when the checked root belongs to a git worktree.
2. Platform cache directory keyed by project root when `.git` is unavailable
   or unwritable.
3. Explicit configured cache directory for CI or specialized developer setups.

Avoid repository-root `.assura/cache/` by default because it can create git
noise and must be excluded from self-validation. If an explicit cache directory
is configured inside the repository, Assura should warn unless the path is
excluded from project validation.

Required behavior:

- `--no-cache` or equivalent should force a full check and avoid writes.
- Cache cleanup can be a later `assura cache clean` command; before that,
  versioned namespaces allow stale cache directories to be removed manually.
- Non-git projects use platform cache directories or run uncached if no safe
  writable cache directory can be found.

## CI Behavior

CI should measure two modes:

- Cold full-check time from an empty cache. This remains the release baseline
  and prevents restored cache state from hiding correctness regressions.
- Warm incremental time from a restored cache plus a controlled no-change or
  small-change fixture. This measures developer-loop potential only.

PR artifacts and website history must label these modes separately. A restored
cache may speed warm benchmarks, but pass/fail correctness must still be proven
by a cold full check.

Recommended CI pattern:

1. Run `assura check` with cache disabled or empty for correctness.
2. Save the cold benchmark artifact.
3. Restore or seed cache for incremental benchmark scenarios.
4. Run no-change, small-change, and config-change incremental benchmarks.
5. Upload machine-readable results with mode labels:
   `full_cold`, `cache_cold`, `cache_warm_no_changes`,
   `cache_warm_small_change`, and `cache_config_change`.

## Correctness Risks

- Treating git tracked paths as complete would miss untracked structure drift.
- Reusing metadata-only hits can miss content changes on coarse timestamp file
  systems.
- Failing to invalidate on config or engine changes can produce stale passes.
- Directory count checks can become stale when a file is deleted or renamed,
  even if no remaining file changed.
- Pattern scopes can widen rule impact beyond a single changed file.
- Cache files inside the project can become self-check violations.
- Restored CI cache can hide bugs if cold full checks are skipped.

## Phased Implementation Plan

### Phase 1: Safe cache foundations

- Add a cache identity model covering root, config hash, Assura version, rule
  engine version, schema version, and cache format.
- Add cache placement discovery without enabling reuse by default.
- Add `--no-cache` or an internal equivalent for benchmark control.
- Add tests for cache identity mismatch, corruption fallback, and ignored cache
  placement.
- Add benchmark scenarios that distinguish full cold from cache-enabled runs,
  even if cache reuse is still disabled.

### Phase 2: File-local result reuse

- Store file-local validation results keyed by normalized path, metadata,
  content hash, and resolved file-local rules.
- Reuse only for unchanged files with matching cache identity.
- Recheck changed and untracked files.
- Recompute parent directory checks for every touched path.
- Add tests for changed files, unchanged files, deleted files, renamed files,
  untracked files, and config invalidation.

### Phase 3: Directory indexes

- Build a direct-child index during traversal and store directory fingerprints.
- Derive `exists`, `allow_extra`, allowed, and forbidden direct-content checks
  from the index instead of repeated `read_dir` work.
- Invalidate parent directory fingerprints on create, delete, rename, or child
  type changes.
- Benchmark `many_direct_counts`, `ignored_generated_heavy`, and monorepo
  fixtures separately.

### Phase 4: Advanced scopes

- Add declared dependency footprints for future pattern scopes, package rules,
  file pairing, and dependency graph checks.
- Use those footprints for finer invalidation only after regression fixtures
  prove correctness.
- Keep broad or unknown scopes as full/subtree recompute rather than risking
  stale passes.

## Benchmark Plan

Benchmark and report these modes separately:

| Scenario | Required comparison |
| --- | --- |
| Full cold check | Current full `assura check` with no cache. |
| Cache cold check | Cache-enabled first run, including cache write cost. |
| Warm no-change check | Cache-enabled repeat run with no filesystem changes. |
| Warm small-change check | One file changed in a large fixture. |
| Warm delete/rename check | Parent directory recomputation after structural change. |
| Config-change check | Full recompute after `.assura/config.yml` hash changes. |

Results should use the performance artifact schema from the good-enough
comparison contract and must not be mixed with LS-Lint comparison claims unless
the same fixture and equivalent rules are used.
