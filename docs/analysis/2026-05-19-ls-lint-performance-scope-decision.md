---
title: LS-Lint Performance Scope Decision
date: 2026-05-19
status: accepted
---

# LS-Lint Performance Scope Decision

## Decision Needed

The current implementation can claim a universal cold-subprocess 2x win against
native LS-Lint for the Linux static-CRT release artifact on the generated
realistic-equivalent fixture set.

The retained implementation now provides a check-only Rust CLI, LS-Lint-native
binary comparison, LS-Lint-compatible fast validation, compiled-config
artifacts, hot/editor-session validation, dirty-path validation, status-file
checks, and explicit completion gating. The remaining gap is no longer
validation throughput. The latest tracked report shows the Linux static-CRT
`assura-check-cli` faster than native LS-Lint on all six realistic-equivalent
fixtures and meeting the 2x target on all six.

Source of truth:

```text
benches/history/current.json
claim_summary.two_x_claim_verdict = complete
claim_summary.measured_iterations = 5
claim_summary.assura_faster_count = 6
claim_summary.two_x_pass_count = 6
claim_summary.two_x_fail_count = 0
claim_summary.aggregate_speedup_ratio = 2.8980855186211874
assura-check-cli.assura_binary_profile = release-static-crt
```

The matching completion audit is
`docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md`.

## Why This Is A Scope Decision

The in-process validation engine is already below every 2x target:

| Fixture | In-process Assura | 2x target |
| --- | ---: | ---: |
| `simple_library` | 0.21 ms | 2.74 ms |
| `web_app` | 0.21 ms | 2.55 ms |
| `monorepo_packages` | 0.43 ms | 2.64 ms |
| `monorepo_policy` | 1.44 ms | 4.43 ms |
| `rule_heavy_repo` | 0.81 ms | 3.21 ms |
| `ignored_generated_heavy_repo` | 0.13 ms | 5.10 ms |

The misses come from cold subprocess and startup overhead, not from the
validation algorithm. The latest audit also records that these likely
implementation families were already measured and either retained as
diagnostic architecture or rejected as completion paths:

- parser swaps, quiet pre-parsing, and raw Unix entrypoints;
- smaller/no-output/no-cache check binaries;
- default compiled-artifact probing;
- compiled-config source fingerprinting and path matching;
- daemon/status-file protocol reductions;
- alternative YAML parser candidates;
- rule-heavy suffix matching and lazy filename work;
- Linux cross-host cold validation.

Another unscoped cold-start tweak is unlikely to turn five misses into a
universal 2x pass without changing what is being measured.

## Honest Product Options

### Option A: Keep The Current Universal Cold CLI Gate

Status: complete for Linux static-CRT release artifacts.

Claim allowed: Assura is faster than native LS-Lint on all current
realistic-equivalent generated fixtures, but it is not universally 2x faster.

Completion condition remains:

```text
claim_summary.two_x_claim_verdict = complete
claim_summary.assura_row_family = assura-check-cli
claim_summary.two_x_pass_count = claim_summary.fixture_count
claim_summary.measured_iterations >= 3
```

This now requires preserving the static-CRT release build and avoiding
unqualified claims for default dynamic macOS builds.

### Option B: Scope The Claim To Editor/Daemon Sessions

Status: implemented as a separate warm gate in the performance report.

This matches the compiled/prepared-config direction: config validation happens
only when the config changes, project state is kept warm, and changed-path
checks avoid full traversal when the dirty set is small and rule dependencies
are local.

Claim shape:

```text
For editor-session validation with a warm Assura daemon and unchanged config,
Assura can answer project status or dirty-path checks without reparsing config
or traversing the whole project.
```

This is now measured by `assura-check-dirty-project-session-cli`, with
`assura-check-dirty-project-cli` and `assura-check-dirty-project-socket`
retained for one-shot client and daemon/socket attribution. It must not be
mixed with the cold `assura-check-cli` row.

### Option C: Scope The Claim To Amortized CLI Validation

Status: requires benchmark and product-surface decision.

This would compare batch or repeated validation where startup is amortized over
multiple paths or multiple edits. The existing batch path already reuses loaded
config and checker state for paths in the same project, so the remaining work
is defining a fair comparison contract, not inventing a simple batch shortcut.

### Option D: Scope The Claim To Larger Workloads

Status: partially supported by existing external fixture evidence, but not by
the universal generated-fixture gate.

The existing external fixture smoke showed strong aggregate performance on
pinned larger projects, but small generated fixtures still fail the universal
2x rule. A larger-workload claim must not be presented as a universal claim.

## Recommendation

Keep `claim_summary.two_x_claim_verdict` as the public cold CLI completion
gate and label the release profile. The current completed scope is Linux
static-CRT release artifacts.

For the next PR/goal, productize Option B: document, harden, and package the
persistent session contract. That path aligns with the architecture Assura now
has: config fingerprinting, compiled artifacts, prepared checks, daemon state,
dirty-path validation, status files, and the persistent `assura-check-session`
CLI. It also matches the real user value during editing: avoid config
validation and full traversal unless the config or broad project state is
dirty.

Do not change website or PR language to say "2x faster than LS-Lint" without
including the row family and Linux static-CRT release scope that proves the
claim.
