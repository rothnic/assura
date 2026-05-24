---
title: Filesystem Validation Throughput Research
date: 2026-05-17
status: current
---

# Filesystem Validation Throughput Research

## Sources Checked

- `walkdir` supports sorted traversal through `sort_by_file_name` and pruning
  through `filter_entry`:
  <https://docs.rs/walkdir/latest/walkdir/struct.WalkDir.html>
  <https://docs.rs/walkdir/latest/walkdir/struct.FilterEntry.html>
- `jwalk` is explicitly designed for Rayon-backed parallel walking and exposes
  `process_read_dir` for sort/filter/skip decisions before child traversal:
  <https://docs.rs/jwalk/latest/jwalk/>
  <https://docs.rs/jwalk/latest/src/jwalk/lib.rs.html>
- ripgrep's public guide documents recursive search with ignore handling across
  `.gitignore`, `.ignore`, repository excludes, and global git excludes:
  <https://github.com/BurntSushi/ripgrep/blob/master/GUIDE.md>
- ESLint's CLI supports cache reuse with `--cache` and `--cache-location`,
  which is relevant to future incremental Assura work but outside the current
  no-cache CLI-to-CLI comparison:
  <https://eslint.org/docs/latest/use/command-line-interface>

## Findings

Ignore pruning should happen during traversal, not after validation queues are
built. This matches Assura's current `jwalk` `process_read_dir` path and the
new `walkdir` `filter_entry` strategy path.

Deterministic output is a separate requirement from traversal order. Assura can
sort final violations, but fail-fast semantics need a deterministic traversal
path. The current implementation keeps fail-fast on the serial sorted path.

Parallel traversal is not automatically a full-check win. In the current
release report, `traversal:*` rows remain diagnostic only, and full-check
`strategy:*` rows are the decision evidence.

Caching is a known throughput lever for lint-style tools, but the current goal
explicitly measures warm CLI execution without incremental cache. ESLint-style
cache semantics should be handled as a separate feature because they change the
meaning of the measured loop.

## Assura Decision Impact

The report now measures full-check CLI strategy rows for:

- `strategy:jwalk-serial-cli`
- `strategy:walkdir-cli`
- `strategy:jwalk-parallel-cli`

Current release totals for realistic-equivalent fixtures:

| Strategy | Total median runtime |
| --- | ---: |
| `strategy:jwalk-serial-cli` | 87.563752 ms |
| `strategy:walkdir-cli` | 87.592170 ms |
| `strategy:jwalk-parallel-cli` | 92.004143 ms |

The fastest individual strategy varies by fixture, but the bundle differences
are small. The current decision is to use walkdir as the default non-fail-fast
full-check path because it is effectively tied with serial `jwalk` in the
15-iteration committed report, while preserving sorted traversal and exclusion
pruning through the simpler walkdir baseline. Serial `jwalk` remains the
deterministic fail-fast path and an opt-in diagnostic strategy; parallel
`jwalk` remains opt-in until a larger traversal-heavy fixture or adaptive
heuristic clearly justifies it.

Parallel rule application and indexed/rule-planned execution are explicitly
deferred from this slice. They require new rule-planning data structures,
deterministic merge semantics, and additional correctness tests for preserving
text/JSON ordering and fail-fast behavior. That work can improve the execution
architecture later, but it is not required to decide the current public
CLI-to-CLI claim because the measured row families already compare equivalent
end-to-end CLI subprocess contracts.

## Follow-Up Architecture Questions

- Can Assura precompute per-directory rule plans for exact filename,
  extension, glob, `.dir`, and direct-child `exists` checks without increasing
  setup cost on small repositories?
- Can rule application run in parallel while preserving deterministic violation
  ordering through local buffers plus a final stable sort?
- Does a larger traversal-heavy fixture show a durable win for parallel
  `jwalk`, or do the current realistic-equivalent fixtures represent the
  expected v0.1 workload closely enough to keep the single serial default?
- Would an incremental cache change the product contract enough that it should
  become a separate command mode rather than hidden behavior in `assura check`?
