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

Local reassessment command:

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output target/performance/ls-lint-reassessment.json \
  --history target/performance/ls-lint-reassessment.jsonl \
  --website-dir target/performance/website-data \
  --iterations 3
cargo xtask performance-no-slower target/performance/ls-lint-reassessment.json
```

Fresh local report:

- commit: `b7dd0f276d866ede6e2c60806084d3c8dd102fdc`
- branch: `codex/ls-lint-performance-reassessment`
- LS-Lint: `ls-lint v2.3.0`
- accepted fixture count: 8
- accepted cold `assura-cli` rows no slower than native `ls-lint-cli`: 8 / 8
- cold `claim_summary.two_x_claim_verdict`: `not-complete`
- warm `warm_claim_summary.two_x_claim_verdict`: `complete`

## Accepted Row Summary

| Fixture | Assura CLI ms | Native LS-Lint ms | Assura/LS-Lint | 2x status |
| --- | ---: | ---: | ---: | --- |
| `ignored_generated_heavy_repo` | 3.50 | 9.01 | 0.39 | `meets-target` |
| `many_configured_scopes_regression` | 45.48 | 48.98 | 0.93 | `misses-target` |
| `monorepo_packages` | 4.41 | 5.55 | 0.80 | `misses-target` |
| `monorepo_policy` | 6.38 | 9.41 | 0.68 | `misses-target` |
| `multipart_extension_regression` | 6.77 | 10.83 | 0.63 | `misses-target` |
| `rule_heavy_repo` | 4.06 | 5.71 | 0.71 | `misses-target` |
| `simple_library` | 4.49 | 4.79 | 0.94 | `blocked-by-rust-cli-floor` |
| `web_app` | 4.27 | 5.09 | 0.84 | `blocked-by-rust-cli-floor` |

The no-slower merge gate passes because every accepted Assura row is at or
below the matching native LS-Lint row. Aggregate speedup is not used as the
gate; a single slower accepted row would still fail
`cargo xtask performance-no-slower`.

## Phase Attribution

| Fixture | Process floor ms | Rust CLI floor ms | Config load ms | Checker init ms | Walk/validate ms | Report sort ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ignored_generated_heavy_repo` | 1.83 | 2.61 | 0.08 | 0.01 | 0.10 | 0.00 |
| `many_configured_scopes_regression` | 2.14 | 2.69 | 8.40 | 2.63 | 8.43 | 0.00 |
| `monorepo_packages` | 2.23 | 2.75 | 0.20 | 0.05 | 0.43 | 0.00 |
| `monorepo_policy` | 2.29 | 2.81 | 0.37 | 0.24 | 1.13 | 0.00 |
| `multipart_extension_regression` | 2.17 | 2.75 | 0.05 | 0.01 | 2.42 | 0.00 |
| `rule_heavy_repo` | 1.91 | 2.29 | 0.16 | 0.08 | 0.76 | 0.00 |
| `simple_library` | 1.86 | 2.65 | 0.13 | 0.02 | 0.25 | 0.00 |
| `web_app` | 2.52 | 3.41 | 0.10 | 0.03 | 0.12 | 0.00 |

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
