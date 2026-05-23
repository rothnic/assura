---
title: Native LS-Lint Performance Gap Review
date: 2026-05-18
status: active
---

# Native LS-Lint Performance Gap Review

## Executive Finding

The corrected benchmark changed the performance conclusion twice.

First, replacing the npm wrapper with the packaged native LS-Lint binary showed
that the full `assura check` CLI was slower than native LS-Lint on the headline
realistic fixtures. The old 100 ms LS-Lint floor was a Node wrapper artifact,
not upstream LS-Lint behavior.

Second, splitting a dedicated `assura-check` entrypoint out of the full product
CLI and feature-gating unrelated library surfaces removed unrelated startup
cost. The current checked-in release evidence now shows `assura-check-cli`
faster than native LS-Lint on all six realistic rows. That is a real
improvement, but it is not a universal 2x win. In the latest tracked report,
only the generated-heavy fixture clears 2x; the smaller fixtures are still
constrained by the local subprocess floor and remaining CLI overhead.

Latest follow-up: the measured subprocess loops now use exit-status execution
with stdout/stderr sent to null instead of `Command::output()` pipe capture.
That made the comparison fairer to both CLIs but did not change the completion
result. The tracked `benches/history/current.json` still shows
`assura-check-cli` missing the 2x target on five of six realistic rows.
The report now records both the generic `/usr/bin/true` process floor and the
smallest Assura Rust CLI status-check floor. On several realistic fixtures, the
Assura Rust CLI floor alone is above the 2x target, which makes a universal cold
Rust subprocess 2x claim unrealistic without changing the execution model.
The latest status-check floor uses Unix raw-entrypoint/no-std shims for the
daemon client and status-file reader. Those tiny binaries now use direct C FFI
and link `libSystem` on macOS instead of pulling the `libc` crate's link
surface, but the daemon-backed rows still do not produce a universal 2x CLI
result.

Additional cold-start experiments after that correction did not change the
conclusion:

- Removing `assura-check` JSON/YAML output and `--cache-dir` support shrank the
  binary to 864 KB but still passed only the generated-heavy row and regressed
  some fixture medians.
- Replacing the hot `assura-check` parser with `pico-args` preserved behavior
  but still passed only the generated-heavy row, so the extra parser dependency
  was reverted.
- Adding an exact `assura-check --quiet` pre-parser before the general parser
  was also worse in the corrected smoke and was removed.
- Testing a raw Unix entrypoint for the exact `assura-check --quiet` one-shot
  invocation was also worse and was reverted.
- Re-testing direct `_exit` after shrinking the daemon/status clients to no-std
  8.4 KB binaries was better than returning through the C runtime for those
  tiny clients, so that optimization was kept there only.

## Completion Audit Snapshot

Objective: provide a comparable binary CLI validation path that is at least 2x
faster than native LS-Lint end to end.

| Requirement | Current artifact | Status |
| --- | --- | --- |
| Comparable binary CLI execution | Native LS-Lint binary from the pinned npm package and `assura-check` release binary are measured from fixture working directories. | Satisfied |
| Established Rust CLI framework | `assura-check` uses `lexopt` instead of hand-parsing the main hot CLI surface. | Satisfied |
| Check-only startup architecture | `crates/assura-check-cli` depends on `assura` with `default-features = false`; release binary links only `libSystem` on macOS. | Satisfied |
| Fair measurement harness | Measured loops use exit status with null stdout/stderr for Assura, LS-Lint, hot client, status client, and process-floor rows. | Satisfied |
| Cold-start feasibility attribution | `benches/history/current.json` records `process_floor_*`, `rust_cli_floor_*`, `runtime_above_process_floor_ms`, and `assura_cli_overhead_ms`. | Satisfied |
| Low-risk cold-start experiments | Removing output/cache support, swapping to `pico-args`, and pre-parsing `--quiet` were each measured and reverted because they did not improve the objective. | Satisfied |
| Universal 2x realistic rows | `benches/history/current.json` shows only `ignored_generated_heavy_repo` passes the 2x target. Five realistic rows still miss. | Not satisfied |
| Remaining performance diagnosis | In-process Assura rows are well below target; cold subprocess rows are launch/startup dominated. | Satisfied |

Current tracked realistic rows:

| Fixture | `assura-check-cli` | Native LS-Lint | 2x target | Status |
| --- | ---: | ---: | ---: | --- |
| `simple_library` | 3.97 ms | 5.18 ms | 2.59 ms | Miss |
| `web_app` | 3.96 ms | 5.28 ms | 2.64 ms | Miss |
| `monorepo_packages` | 4.44 ms | 5.88 ms | 2.94 ms | Miss |
| `monorepo_policy` | 5.74 ms | 9.01 ms | 4.50 ms | Miss |
| `rule_heavy_repo` | 5.27 ms | 6.86 ms | 3.43 ms | Miss |
| `ignored_generated_heavy_repo` | 3.86 ms | 12.31 ms | 6.15 ms | Pass |

Current cold-start attribution rows:

| Fixture | `assura-check-cli` | Process floor | Assura Rust CLI floor | Assura CLI overhead | 2x target | Status |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `simple_library` | 3.97 ms | 2.35 ms | 2.94 ms | 1.37 ms | 2.59 ms | Miss |
| `web_app` | 3.96 ms | 2.22 ms | 3.09 ms | 1.52 ms | 2.64 ms | Miss |
| `monorepo_packages` | 4.44 ms | 2.20 ms | 2.92 ms | 1.75 ms | 2.94 ms | Miss |
| `monorepo_policy` | 5.74 ms | 2.25 ms | 3.56 ms | 1.67 ms | 4.50 ms | Miss |
| `rule_heavy_repo` | 5.27 ms | 2.26 ms | 3.06 ms | 2.12 ms | 3.43 ms | Miss |
| `ignored_generated_heavy_repo` | 3.86 ms | 2.35 ms | 3.03 ms | 1.35 ms | 6.15 ms | Pass |

## Corrected Release Evidence

Command used to refresh the tracked release evidence:

```bash
cargo build --release -p assura --bin assura
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
```

Headline realistic rows from `benches/history/current.json`:

| Fixture | Full `assura check` | `assura-check-cli` | Assura in-process | Native LS-Lint CLI | Winner |
| --- | ---: | ---: | ---: | ---: | --- |
| `simple_library` | 11.58 ms | 3.97 ms | 0.25 ms | 5.18 ms | Assura 1.31x |
| `web_app` | 11.24 ms | 3.96 ms | 0.22 ms | 5.28 ms | Assura 1.33x |
| `monorepo_packages` | 12.04 ms | 4.44 ms | 0.49 ms | 5.88 ms | Assura 1.32x |
| `monorepo_policy` | 12.84 ms | 5.74 ms | 1.83 ms | 9.01 ms | Assura 1.57x |
| `rule_heavy_repo` | 11.78 ms | 5.27 ms | 0.89 ms | 6.86 ms | Assura 1.30x |
| `ignored_generated_heavy_repo` | 10.86 ms | 3.86 ms | 0.16 ms | 12.31 ms | Assura 3.19x |

Assura phase medians show the engine work is small:

| Fixture | Config discovery | Config load | Checker init | Walk + validate |
| --- | ---: | ---: | ---: | ---: |
| `simple_library` | 0.04 ms | 0.06 ms | 0.01 ms | 0.14 ms |
| `web_app` | 0.04 ms | 0.07 ms | 0.01 ms | 0.10 ms |
| `monorepo_packages` | 0.04 ms | 0.11 ms | 0.02 ms | 0.32 ms |
| `monorepo_policy` | 0.05 ms | 0.27 ms | 0.08 ms | 1.33 ms |
| `rule_heavy_repo` | 0.04 ms | 0.11 ms | 0.03 ms | 0.70 ms |
| `ignored_generated_heavy_repo` | 0.04 ms | 0.05 ms | 0.00 ms | 0.07 ms |

## What We Were Measuring Wrong

The previous LS-Lint row executed `node_modules/.bin/ls-lint`, which is a Node
shim that spawns the platform Go binary. Local timing showed:

| Command | Mean |
| --- | ---: |
| `node_modules/.bin/ls-lint --version` | 113.9 ms |
| native `ls-lint-darwin-amd64 --version` | 8.8 ms |
| `node_modules/.bin/ls-lint` on a 1-file fixture | 117.7 ms |
| native `ls-lint-darwin-amd64` on the same fixture | 10.6 ms |

The fixed benchmark now resolves
`node_modules/@ls-lint/ls-lint/bin/ls-lint-<platform>` and records
`ls_lint_execution_mode=native-binary-from-pinned-npm-package`.

## Why The Full Assura CLI Lost

The dominant issue in the original comparison was CLI process architecture, not
validation throughput.

Measured startup-only commands:

| Command | Mean |
| --- | ---: |
| `target/release/assura --version` | 15.74 ms |
| `target/release/assura --help` | 19.30 ms |
| native `ls-lint --version` | 7.87 ms |
| native `ls-lint --help` | 8.83 ms |

That startup gap is already larger than the total native LS-Lint runtime for
several headline fixtures.

Original full Assura binary characteristics:

- `target/release/assura` is 3.4 MB; native LS-Lint is 2.6 MB on this machine.
- Assura links dynamic libraries for libgit2/OpenSSL/Security/CoreFoundation
  even when running `assura check`.
- `src/main.rs` creates a Tokio runtime for all commands via `#[tokio::main]`.
- `src/main.rs` initializes tracing for every invocation, including `check` and
  `--version`.
- `Cargo.toml` pulls broad product-roadmap dependencies into the one binary:
  `git2`, `notify`, `tokio full`, `tracing-subscriber`, `validator`,
  `petgraph`, `dashmap`, `ignore`, `pulldown-cmark`, and others.

Native LS-Lint is much narrower than the original all-in-one Assura binary:

- The v2.3.0 Go module has three direct dependencies:
  `gopkg.in/yaml.v3`, `github.com/bmatcuk/doublestar/v4`, and
  `golang.org/x/sync`.
- The CLI uses Go's standard `flag` package, reads YAML, builds rule/ignore
  indexes, and walks with `fs.WalkDir`.
- The linter is focused on names only; it does not carry a broader workflow,
  watch, git, graph, markdown, or validation platform in the same startup path.

## Assura Hot-Path Observations

1. `assura check` validates quickly once inside `run_structure_check`.
   The largest realistic in-process median is 3.63 ms.

2. Per-directory rule resolution can still be improved.
   `StructureChecker::resolve_rules` walks configured structure nodes and clones
   effective rules into `rules_cache` per directory. This is not the main
   headline loss today, but it matters for larger trees.

3. The current rules model is richer than LS-Lint.
   Assura supports required files/directories, direct-content policies,
   markdown checks, size/line/doc checks, inherited directory bundles, and
   broader report metadata. The headline comparison should either:
   - use an LS-Lint-compatible fast path for LS-Lint-equivalent rules, or
   - stop claiming direct speed parity while exercising extra product scope.

4. Success output is not the main gap.
   Local repo checks were roughly the same whether output was text, JSON,
   `--quiet`, or written to a file. The startup/dependency surface is larger
   than report serialization for these cases.

## Fast Entrypoint Experiment

A first `assura-check` binary using the lightweight `lexopt` argument parser
was added as a check-only entrypoint. It avoids `#[tokio::main]`, avoids
unconditional tracing initialization, supports quiet success output, and is
measured as `assura-check-cli` in performance reports.

Short release smoke command:

```bash
cargo run --release --quiet --bin assura -- performance-report \
  --output target/performance/assura-check-cli-smoke.json \
  --history target/performance/assura-check-cli-smoke.jsonl \
  --website-dir target/performance/assura-check-cli-smoke-website \
  --iterations 5
```

Smoke result:

| Fixture | `assura-check-cli` | Native LS-Lint | Ratio |
| --- | ---: | ---: | ---: |
| `simple_library` | 14.28 ms | 7.38 ms | LS-Lint 1.94x |
| `web_app` | 14.21 ms | 7.82 ms | LS-Lint 1.82x |
| `monorepo_packages` | 14.35 ms | 8.51 ms | LS-Lint 1.69x |
| `monorepo_policy` | 18.03 ms | 12.27 ms | LS-Lint 1.47x |
| `rule_heavy_repo` | 16.88 ms | 9.44 ms | LS-Lint 1.79x |
| `ignored_generated_heavy_repo` | 13.48 ms | 15.64 ms | Assura 1.16x |

This is not enough. The binary still links libgit2/OpenSSL/Security because it
uses the full `assura` library crate. A true fast path needs a feature split or
separate crate so check-only builds do not link workflow, hook, graph, markdown,
and roadmap surfaces.

## Workspace Fast-Check Split

The next experiment moved `assura-check` into a dedicated workspace package:

- `crates/assura-check-cli` provides the `assura-check` executable.
- The package depends on `assura` with `default-features = false`.
- The full `assura` package keeps `git2` behind the default `git-signals`
  feature.
- `assura-check` links only `libSystem` on macOS after building the full CLI
  and check CLI separately.
- The `assura` crate now gates the full CLI, markdown, intelligence, and
  roadmap dependency surfaces behind the `full-cli` feature. The
  `assura-check-cli` package depends on `assura` with `default-features = false`
  so the check binary does not pull `tokio`, `clap`, `tracing-subscriber`,
  `notify`, `petgraph`, `rayon`, `bincode`, `pulldown-cmark`, `validator`, or
  `git2`.
- Release builds use LTO, one codegen unit, stripping, and `panic = "abort"` to
  keep the check binary small.
- Diagnostic `jwalk` traversal strategies are gated behind the full CLI feature
  so `assura-check-cli` does not link `jwalk`, `rayon`, or `crossbeam` for the
  default `walkdir` validation path.
- Simple suffix naming patterns such as `*.ts` are matched directly instead of
  compiled into glob matchers during checker initialization.
- A conservative LS-Lint-compatible fast path bypasses configured-structure
  and richer Assura validation when the config contains only naming,
  direct-count, and ignore rules. It is disabled for `--fail-fast` so the
  existing sorted fail-fast behavior is preserved.
- Common single naming conventions such as `kebab-case` and `snake_case` now
  avoid the allocation-heavy alternative-splitting path; OR-composed and regex
  conventions still use the existing parser.
- The LS-Lint-compatible fast path compiles suffix/glob naming patterns and
  naming validators once per scope, so rule-heavy rows do not re-resolve
  string-based rules for every file.
- The `assura-check-cli` benchmark row now runs from the fixture working
  directory with no path argument, matching native LS-Lint's invocation shape.

Build commands:

```bash
cargo build --release -p assura --bin assura
cargo build --release -p assura-check-cli
```

Do not use a single `cargo build --release --bins` for this evidence. Cargo
feature unification can build the check package with the full package's default
features in the same invocation, which reintroduces `git2`/OpenSSL into the
check executable.

Smoke command:

```bash
target/release/assura performance-report \
  --output target/performance/assura-check-cli-smoke-6.json \
  --history target/performance/assura-check-cli-smoke-6.jsonl \
  --website-dir target/performance/assura-check-cli-smoke-6-website \
  --iterations 5
```

5-iteration smoke result:

| Fixture | `assura-check-cli` | Native LS-Lint | Winner |
| --- | ---: | ---: | --- |
| `simple_library` | 6.38 ms | 8.21 ms | Assura 1.29x |
| `web_app` | 6.57 ms | 7.54 ms | Assura 1.15x |
| `monorepo_packages` | 6.65 ms | 7.78 ms | Assura 1.17x |
| `monorepo_policy` | 9.54 ms | 12.53 ms | Assura 1.31x |
| `rule_heavy_repo` | 7.82 ms | 9.18 ms | Assura 1.17x |
| `ignored_generated_heavy_repo` | 7.24 ms | 15.17 ms | Assura 2.10x |

An earlier checked-in 15-iteration report was refreshed after the batch-support,
traversal, compiled naming fast-path, and working-directory measurement
changes:

| Fixture | `assura-check-cli` | Native LS-Lint | Winner |
| --- | ---: | ---: | --- |
| `simple_library` | 4.49 ms | 5.79 ms | Assura 1.29x |
| `web_app` | 4.53 ms | 6.36 ms | Assura 1.40x |
| `monorepo_packages` | 5.10 ms | 5.64 ms | Assura 1.10x |
| `monorepo_policy` | 6.85 ms | 10.83 ms | Assura 1.58x |
| `rule_heavy_repo` | 5.16 ms | 6.45 ms | Assura 1.25x |
| `ignored_generated_heavy_repo` | 5.06 ms | 11.12 ms | Assura 2.20x |

The split fixes the embarrassment-class issue: the current checked-in evidence
is no longer timing a Node wrapper, and the check-only native CLI is competitive
with native LS-Lint instead of being dominated by unrelated OpenSSL/git startup
cost. The current 5-iteration tracked report wins all six realistic headline
rows and clears 2x on `monorepo_policy` and `ignored_generated_heavy_repo`.
It still does not meet a universal 2x claim. The small fixtures are now running
close to the observed Rust subprocess floor, so a 2x per-fixture target remains
below what this benchmark shape can honestly demonstrate.

There is also a hard measurement-floor problem for the stated 2x target on
small fixtures:

| Command | Mean |
| --- | ---: |
| `/usr/bin/true` | 4.95 ms |
| minimal optimized Rust no-op binary | 6.57 ms |
| native LS-Lint on `simple_library` | 7.38 ms |

For `simple_library`, 2x faster than native LS-Lint means an end-to-end CLI
runtime at or below about 3.69 ms. That is below the observed process-spawn
floor on this machine. The 2x target is therefore not achievable for the
smallest per-fixture subprocess benchmark unless the comparison changes to a
larger workload, a single process validating multiple roots, or a persistent
server/client model.

Post-split startup-only timing still shows the same constraint:

| Command | Median | Min | Mean |
| --- | ---: | ---: | ---: |
| `/usr/bin/true` | 5.66 ms | 4.47 ms | 5.86 ms |
| `target/release/assura-check --version` | 8.11 ms | 6.79 ms | 8.04 ms |
| `target/release/assura-check --help` | 7.78 ms | 6.72 ms | 8.02 ms |

The current `simple_library` native LS-Lint median is 5.79 ms, so a literal
2x Assura target for that row would require 2.90 ms or less. That is below the
measured `/usr/bin/true` median on this host.

## Batch CLI Experiment

`assura-check` now accepts multiple path arguments:

```bash
target/release/assura-check --quiet path-a path-b path-c
```

This is a valid CLI shape for amortizing startup over more validation work. A
same-config batch smoke used six roots, each with 10 package directories and
200 TypeScript files, and compared one `assura-check` invocation with one
native LS-Lint invocation from the shared config directory:

| Command | Median | Min | Mean |
| --- | ---: | ---: | ---: |
| `assura-check --quiet root-*` | 21.97 ms | 16.79 ms | 46.64 ms |
| `ls-lint root-*` | 21.44 ms | 17.25 ms | 22.35 ms |

Batch mode is a useful ergonomic and measurement improvement, but this fair
same-config batch comparison still does not produce a 2x win. The latest local
smoke was effectively tied and noisier on Assura after config/checker reuse, so
it should not be used as current headline evidence. LS-Lint also
accepts repeated `-config` flags, but local timing of that mode was over 1.5 s
for the same roots, so it should not be used as headline evidence without a
separate upstream-behavior review.

## Recommended Rearchitecture

### PR 1: Keep the Corrected Benchmark Honest

Ship the native LS-Lint correction and website copy that computes the current
winner from data. This prevents public embarrassment while performance work
continues.

Completion checks:

- `ls-lint-cli` rows use `tool_name=ls-lint-native-cli`.
- `ls_lint_execution_mode=native-binary-from-pinned-npm-package`.
- The website does not say Assura is faster unless the current data says so.

### PR 2: Keep the Dedicated Fast Check Entrypoint

Keep hardening the lightweight check-only binary or feature split that excludes
unrelated runtime surfaces:

- no `#[tokio::main]` on the check path,
- no `tracing-subscriber` initialization for check unless requested,
- no `git2` / OpenSSL / hook dependencies,
- no `notify` watch dependencies,
- no graph/agent/plugin roadmap dependencies,
- minimal argument parser for check.

Target experiment:

```bash
target/release/assura-check --version
target/release/assura-check <fixture> --format json
```

Gate:

- startup-only median within 1.25x native LS-Lint version/help startup,
- headline fixtures faster than native LS-Lint or clear remaining attribution.

### PR 3: Make `check` Internals a Compiled Plan

Move from per-run config traversal to a compiled validation plan:

- compile inherited directory rules once into path-scope entries,
- precompile glob/regex/extension matchers into compact structures,
- separate LS-Lint-compatible naming rules from Assura-only structural checks,
- store direct directory policy separately from inherited file naming policy,
- avoid cloning `EffectiveRules` for every directory resolution.

Gate:

- in-process `monorepo_policy` and `rule_heavy_repo` improve materially from
  the current 3.63 ms and 1.63 ms medians,
- no parity regression in realistic LS-Lint fixtures.

### PR 4: Add an LS-Lint-Compatible Fast Path

For configurations produced by LS-Lint migration or tagged as native parity,
run a specialized name-lint path:

- build an index keyed by directory scope and extension,
- walk once,
- validate basename/stem against compiled rule lists,
- skip Assura-only checks entirely.

This path should be allowed to coexist with the richer structure-first engine.
The product can be materially faster than LS-Lint on LS-Lint's own problem only
if it solves the same problem with a similarly narrow execution path.

Gate:

- fast path matches LS-Lint parity fixtures,
- fast path is the row used for LS-Lint-equivalent headline claims,
- full Assura structure rows remain available as richer-product evidence.

### PR 5: Revisit Distribution Shape

If the project needs one binary, use feature gates to keep optional surfaces out
of the default release artifact. If not, publish:

- `assura-check` for fast validation,
- `assura` for full workflow/status/watch/hook commands.

The current one-binary design is convenient, but it makes every invocation pay
for roadmap features that are irrelevant to lint-speed comparisons.

## Immediate Positioning

Do not claim Assura is 2x faster than LS-Lint right now.

The defensible claim after the correction and fast-check split is:

> `assura-check` is faster than native LS-Lint on the current realistic
> LS-Lint-compatible fixture set, and the in-process engine is much faster.
> The universal 2x CLI claim is not achieved because small fixture timings are
> dominated by subprocess startup. Further 2x work requires either a larger
> workload, a persistent process/batch contract, or a much narrower
> LS-Lint-compatible execution path with a documented process-floor caveat.
