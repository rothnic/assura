---
title: Post-Beta LS-Lint Performance Reassessment
status: active
---

# Post-Beta LS-Lint Performance Reassessment

## Conclusion

The post-beta reassessment keeps the cold CLI claim bounded and supportable:
every accepted LS-Lint-equivalent fixture row is no slower than native LS-Lint,
but the universal cold 2x claim remains `not-complete` on the local macOS
dynamic release profile. The remaining misses are explained by measured
process/Rust CLI floors on the smallest rows and by real config-load plus
walk-and-validate work on the many-scope regression fixture.

Warm/session evidence remains separate. `warm_claim_summary` proves the
persistent editor/agent session path is much faster than native LS-Lint on the
accepted cohort, but it is not cold `assura check` proof.

## Evidence

Tracked reassessment command:

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
cargo xtask performance-no-slower benches/history/current.json
```

Fresh checked report:

- commit: `82264736bae46168c1243a65bff3d41b81cbf418`
- branch: `codex/ls-lint-performance-reassessment`
- source worktree dirty: `false`
- measured iterations: 5
- LS-Lint: `ls-lint v2.3.0`
- accepted fixture count: 8
- accepted cold `assura-cli` rows no slower than native `ls-lint-cli`: 8 / 8
- cold `claim_summary.two_x_claim_verdict`: `not-complete`
- warm `warm_claim_summary.two_x_claim_verdict`: `complete`

## Accepted Row Summary

| Fixture | Assura CLI ms | Native LS-Lint ms | Assura/LS-Lint | 2x status |
| --- | ---: | ---: | ---: | --- |
| `ignored_generated_heavy_repo` | 3.45 | 9.67 | 0.36 | `meets-target` |
| `many_configured_scopes_regression` | 41.67 | 45.75 | 0.91 | `misses-target` |
| `monorepo_packages` | 3.82 | 6.04 | 0.63 | `misses-target` |
| `monorepo_policy` | 5.80 | 8.84 | 0.66 | `misses-target` |
| `multipart_extension_regression` | 7.05 | 12.75 | 0.55 | `misses-target` |
| `rule_heavy_repo` | 4.45 | 5.20 | 0.86 | `misses-target` |
| `simple_library` | 4.12 | 4.94 | 0.83 | `blocked-by-rust-cli-floor` |
| `web_app` | 4.34 | 5.06 | 0.86 | `blocked-by-rust-cli-floor` |

The no-slower merge gate passes because every accepted Assura row is at or
below the matching native LS-Lint row. Aggregate speedup is not used as the
gate; a single slower accepted row would still fail
`cargo xtask performance-no-slower`.

## Phase Attribution

| Fixture | Process floor ms | Rust CLI floor ms | Config load ms | Checker init ms | Walk/validate ms | Report sort ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ignored_generated_heavy_repo` | 1.79 | 2.27 | 0.05 | 0.01 | 0.06 | 0.00 |
| `many_configured_scopes_regression` | 1.82 | 2.46 | 8.34 | 2.56 | 8.96 | 0.00 |
| `monorepo_packages` | 2.14 | 2.79 | 0.21 | 0.03 | 0.51 | 0.00 |
| `monorepo_policy` | 2.08 | 2.32 | 0.29 | 0.19 | 0.95 | 0.00 |
| `multipart_extension_regression` | 2.17 | 2.48 | 0.05 | 0.01 | 2.53 | 0.00 |
| `rule_heavy_repo` | 1.75 | 2.35 | 0.13 | 0.07 | 0.62 | 0.00 |
| `simple_library` | 1.99 | 2.57 | 0.14 | 0.03 | 0.43 | 0.00 |
| `web_app` | 2.15 | 2.78 | 0.09 | 0.02 | 0.11 | 0.00 |

The small fixtures are startup/floor dominated: their full `assura-cli` rows
are only a few milliseconds, and their 2x targets sit close to or below the
measured Rust CLI floor. The `many_configured_scopes_regression` row is not a
floor-only miss; its attribution shows config loading and walk/validate both
near 8.4 ms because the fixture intentionally exercises hundreds of explicit
configured scopes.

## Decision

No accepted fixture is slower than native LS-Lint, so there is no remaining
merge-blocking LS-Lint parity performance defect in the current accepted
cohort. The remaining cold 2x misses should not be hidden, but they are not
the same requirement as the no-slower beta gate.

Future performance work should focus on one of these explicit tracks:

- reduce real work in `many_configured_scopes_regression` without weakening
  LS-Lint-compatible semantics;
- lower release startup/floor cost for cold dynamic builds without making
  `assura check` cumbersome;
- expand warm daemon/session proof for editor and agent workflows while
  keeping that claim separate from cold CLI proof.
