# Assura Benchmarks

This directory contains Criterion benchmarks for the current Assura codebase.

## Current-Product LS-Lint Comparison

`benches/ls_lint_comparison.rs` compares the public structure-first
`assura check` path, implemented by `run_structure_check`, with
`@ls-lint/ls-lint@2.3.0` on identical generated fixtures.

Run it with:

```bash
cargo bench --bench ls_lint_comparison -- --noplot
```

The benchmark uses `npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint`
so the first run may need network access to fetch the LS-Lint package. If
LS-Lint is unavailable, the benchmark still runs Assura scenarios and skips the
external LS-Lint samples.

Generated stress scenarios covered:

- `small`: representative extension and directory naming rules.
- `medium`: common source/test sized tree.
- `large`: larger file and directory count.
- `rule_heavy`: many extension patterns.
- `ignored_generated_heavy`: generated files pruned through ignore/exclude
  configuration.

The same benchmark also includes a `realistic_lslint_fixtures` group sourced
from `tests/realistic_lslint_fixtures.rs`. Those fixtures are the reusable
realistic corpus shared by compatibility tests and benchmark runs.

The `assura performance-report` command covers both the generated stress
scenarios above and realistic project-shape scenarios for `simple_library`,
`web_app`, `monorepo_packages`, `rule_heavy_repo`, and
`ignored_generated_heavy_repo`. In addition to top-level Assura, LS-Lint, and
traversal rows, it emits per-fixture Assura phase rows for config discovery,
config load, checker initialization, configured-structure validation,
walk-and-validate, and deterministic report sorting.

Record local release evidence with the date, branch or commit, operating
system, exact command, LS-Lint version, and Criterion median estimates. Do not
claim a speedup unless this current-product benchmark supports it.

For CI artifacts, website charting, and tracked history, use the lightweight
report command instead of scraping Criterion output:

```bash
cargo run --quiet -- performance-report \
  --output target/performance/ls-lint-comparison.json \
  --iterations 5
```

The report schema is tracked at
`benches/history/ls-lint-comparison.schema.json`. PR CI uploads the current
report as an artifact and writes a concise GitHub Step Summary. To intentionally
refresh the tracked baseline history, run:

```bash
cargo run --quiet -- performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
```

Review the resulting `benches/history/` and `website/public/data/performance/`
diffs in the PR. Do not update tracked history as an automatic side effect of
normal benchmark or CI runs. The tracked JSONL history is capped to the most
recent 1,000 rows to keep generated evidence inside the repository structure
policy.

### Local Baseline: 2026-05-14

- Branch: `codex/assura-v0-1-polished`
- Environment: WSL on a locked-down machine
- LS-Lint: `ls-lint v2.3.0`
- Command:
  `env OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo bench --bench ls_lint_comparison -- --noplot`

Criterion median estimates:

| Scenario | Assura median | LS-Lint 2.3 median | Result |
| --- | ---: | ---: | --- |
| `small` | 241.19 us | 511.59 ms | Assura faster |
| `medium` | 3.8721 ms | 496.50 ms | Assura faster |
| `large` | 18.620 ms | 516.61 ms | Assura faster |
| `rule_heavy` | 21.793 ms | 527.76 ms | Assura faster |
| `ignored_generated_heavy` | 54.267 us | 488.42 ms | Assura faster |

The first sandboxed `npm exec` attempt failed with DNS `EAI_AGAIN` for
`registry.npmjs.org`. Re-running with approved network access confirmed
`ls-lint v2.3.0` and produced the full comparison above.

## Structure-First Profiling

`benches/profiling.rs` includes `structure_check/...` groups for the current
`assura check` implementation plus attribution slices for config load,
traversal, exclusion pruning, directory count reads, and glob pattern matching.

Run the main structure-check profile with:

```bash
cargo bench --bench profiling structure_check -- --noplot
```

## Legacy Context

Other benchmark files remain useful for internal engine and graph comparison,
but release performance claims should be based on the current-product
`ls_lint_comparison` and `profiling structure_check` commands above.
