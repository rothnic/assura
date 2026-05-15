# LS-Lint Performance History

This directory stores chart-ready Assura versus LS-Lint comparison data.

- `ls-lint-comparison.schema.json` defines one JSONL result row.
- `ls-lint-comparison-history.jsonl` is the tracked baseline history.
- `current.json` is the latest checked-in full report used by the website.

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

If `npm exec --package @ls-lint/ls-lint@2.3.0` is unavailable, the report still
emits Assura rows and LS-Lint `skipped` rows with the exact blocker in
`details`.
