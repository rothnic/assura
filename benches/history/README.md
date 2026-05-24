# LS-Lint Performance History

This directory stores chart-ready Assura versus LS-Lint comparison data.

- `ls-lint-comparison.schema.json` defines one JSONL result row.
- `ls-lint-comparison-history.jsonl` is the tracked rolling baseline history.
  The report writer keeps the most recent 1,000 rows so checked-in history
  stays inside the repository structure policy.
- `current.json` is the latest checked-in full report used by the website.
  Its `claim_summary` object is the machine-readable verdict for the headline
  `assura-check-cli` versus native `ls-lint-cli` row set.

Generate a current report without changing tracked history:

```bash
env OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu \
  cargo run --quiet -- performance-report \
  --output target/performance/ls-lint-comparison.json \
  --iterations 5
```

Append a deliberate baseline update and refresh website data:

```bash
env OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu \
  cargo run --quiet -- performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
```

Baseline updates must be reviewed as normal source changes. CI creates an
artifact for each PR but does not append to this tracked history.

The current report contract is covered by
`tests/performance_report_contract_tests.rs`. That test recomputes
`claim_summary` from the checked-in result rows so the public 2x verdict cannot
silently drift from the underlying measurements.

The report installs the pinned `@ls-lint/ls-lint@2.3.0` package once, resolves
the packaged native binary, and times that binary directly. If the package or
native binary is unavailable, the report still emits Assura rows and LS-Lint
`skipped` rows with the exact blocker in `details`.
