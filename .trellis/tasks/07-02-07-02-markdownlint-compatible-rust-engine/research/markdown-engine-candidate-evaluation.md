---
title: Markdown Engine Candidate Evaluation
status: active
date: 2026-07-02
---

# Markdown Engine Candidate Evaluation

This note records the live candidate evidence used to shape the
Markdownlint-compatible Rust engine goal. It complements the older analysis in
`docs/analysis/2026-06-18-markdown-tooling-evaluation.md`.

## Outcome Lens

The selected path must support a maintainer workflow, not only a faster command:

- Assura reports repository structure and coarse file policy before Markdown
  internals.
- Markdown rule configuration is rule-owned through stable rule IDs, with
  severity, suppression, and fix behavior below the rule.
- Diagnostics map into Assura finding IDs, severities, suppressions, JSON, YAML,
  text, and agent output.
- Safe fixes are deterministic, idempotent, and preserve frontmatter and line
  endings.
- Daemon, editor, and agent integrations reuse the same CLI/daemon truth.
- Performance evidence distinguishes cold CLI cost, warm daemon cost, engine
  lint time, safe-fix time, and reporting overhead.

## Current Candidate Snapshot

| Candidate | Current evidence | Fit | Risk |
| --- | --- | --- | --- |
| `rumdl` | `cargo info rumdl` reports `0.2.27`, MIT, `rust-version: 1.94.0`; project docs describe 76 rules, broad markdownlint compatibility, config import/discovery, inline disable comments, `check --fix`, `fmt`, and a Rust library crate. | Best functional match for a markdownlint-compatible Rust lint/fix engine. Evaluate first. | Direct embedding conflicts with Assura's stated Rust 1.70 MSRV unless Assura raises MSRV, vendors a compatible interface, or uses a subprocess/binary adapter. |
| `markdownlint-rs` / `mdlint` | `cargo info markdownlint-rs` reports `0.3.18`; GitHub release `v0.3.18` publishes native binaries and Python wheels. Docs describe `mdlint check --fix` and rule defaults that may diverge from markdownlint. | Viable comparison point with binary releases and fix support. | Compatibility and config behavior need local fixture proof; library stability and markdownlint parity are less clear than `rumdl`. |
| `mado` | GitHub release `v0.3.0` publishes native binaries; project docs describe a fast Rust Markdown linter. The `mado` crate on crates.io is unrelated macOS app-monitoring code. | Useful performance comparison. | Not an embeddable Cargo dependency under the expected crate name; docs indicate no auto-fix and narrower rule coverage than `rumdl`. |
| Current Assura checks | Existing Rust-native checks cover links, headings, suppressions, required-section fixes, and common lint classes. | Must remain the baseline and preserve Assura-specific reference behavior. | Not broad enough for markdownlint-compatible rule/config coverage. |
| `markdownlint-cli2` | Node ecosystem reference path for markdownlint-compatible behavior. | Compatibility and performance comparison baseline. | Not acceptable as the supported Assura runtime path because the goal excludes a Node dependency. |

## Provisional Direction

Evaluate `rumdl` first against local fixtures because it appears to provide the
best Rust-native markdownlint-compatible lint/fix surface. Treat direct
embedding as blocked until the MSRV decision is explicit. The first product
slice should therefore prove one of these paths:

1. a supported subprocess/binary adapter that preserves Assura contracts without
   adding a Node runtime dependency;
2. an explicit MSRV decision that allows a direct `rumdl` library dependency;
3. a measured rejection record plus a narrowed Assura-owned fallback for the
   highest-value safe fixes and diagnostics.

Any accepted path must include benchmark rows for current Assura checks,
selected Rust candidate behavior, and `markdownlint-cli2`.

## Probe Timing Contract

The branch `codex/markdown-engine-performance-evidence` extends
`cargo xtask markdown-engine-probe` with opt-in command timing:

```bash
cargo xtask markdown-engine-probe --run-external --measure --iterations 5
```

The default probe remains dependency-light and does not require third-party
tools. Timing is emitted only when `--measure` is provided. `assura-current`
receives timing whenever measurement is enabled; external candidates receive
timing only when they are available and `--run-external` actually runs them.
Each timed candidate receives a `timing` object with sorted samples, median,
p95, min, max, successful run count, failed run count, and errors. External
candidates are still measured against isolated copies under
`target/markdown-engine-probe/` so candidates that mutate files cannot write
into source fixtures.

For `assura-current`, the probe uses `target/debug/assura` when that binary is
present and records `execution_mode: target-debug-binary`; it falls back to
`cargo run` only when the binary has not been built. Local timing evidence
should therefore build the binary first:

```bash
cargo build --bin assura --quiet
PATH="$PWD/target/markdown-engine-tools/bin:$PATH" \
  cargo xtask markdown-engine-probe --run-external --measure --iterations 5 \
  > .trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/markdown-engine-probe-2026-07-02-measured.json
```

Timing is wall-clock command time for the configured probe command. It is not
yet a final release benchmark, and it does not replace the later performance
floor work. It exists to stop engine adoption from relying on qualitative
"fast" claims.

## 2026-07-02 Measured Candidate Probe

Local environment:

- `rustc 1.94.1 (e408947bf 2026-03-25)`
- `cargo 1.94.1 (29ea6fb6a 2026-03-24)`
- `node v25.6.0`
- cached candidate binaries under `target/markdown-engine-tools/bin`

Command:

```bash
cargo build --bin assura --quiet
PATH="$PWD/target/markdown-engine-tools/bin:$PATH" \
  cargo xtask markdown-engine-probe --run-external --measure --iterations 5 \
  > .trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/markdown-engine-probe-2026-07-02-measured.json
```

Result summary:

| Candidate | Status | Median ms | p95 ms | Interpretation |
| --- | --- | ---: | ---: | --- |
| `assura-current` | `ran_with_findings` using `target-debug-binary` | 8.804 | 10.241 | Baseline for the current Rust-native Markdown checks on the small invalid fixture. |
| `rumdl 0.2.27` | `ran_with_findings` | 22.132 | 22.442 | Functional leader and much faster than Node `markdownlint-cli2`, but not no-slower than current Assura on this fixture. Do not adopt as the default supported path yet. |
| `mdlint 0.3.18` | `ran_with_findings` | 8.466 | 9.543 | Competitive raw command timing, but prior probes showed unrequested fixture mutation risk. Needs stronger sandbox/fix-safety investigation before adoption. |
| `mado 0.3.0` | `ran_with_findings` | 10.719 | 11.667 | Competitive raw command timing, but prior evidence showed narrower rule/fix surface than `rumdl`. |
| `markdownlint-cli2 v0.23.0` | `ran_with_findings` | 396.341 | 413.412 | Node compatibility baseline is much slower than all Rust candidates on this fixture. |

Decision impact: this slice proves the benchmark plumbing and confirms that
`rumdl` cannot be declared the accepted default engine on the small fixture
yet. It remains the best feature-fit candidate, but the next slices must add
larger/frontmatter/link-heavy fixtures, fix-cost measurement, and candidate
configuration before claiming a selected Rust path is no slower than current
Assura checks.

Raw output is checked in at
`./markdown-engine-probe-2026-07-02-measured.json`.

## Executable Fixture Probe

The branch `codex/markdown-engine-fixture-probe` adds
`tests/fixtures/markdown_engine_candidates/` and
`cargo xtask markdown-engine-probe` as the first executable evidence slice.
Normal tests only require the current Assura binary. External candidate tools
are optional: the probe reports `unavailable` when `rumdl`, `mdlint`, `mado`,
or `markdownlint-cli2` are not installed instead of making CI depend on them.

The fixture matrix locks these expectations before engine adoption:

- structure and coarse file-policy findings sort before Markdown internals;
- markdownlint-style rules map to stable Assura rule IDs;
- Assura-owned link/reference rules remain separate from markdownlint
  candidates;
- severity and fix policy are owned by stable rule entries;
- safe-fix preview covers bounded trailing-space and required-section fixes
  without writing files.

## 2026-07-02 Isolated Candidate Probe

The branch `codex/markdown-engine-candidate-runs` corrected the probe command
arguments and runs external tools against isolated copies under
`target/markdown-engine-probe/<candidate>/invalid`. This isolation is required:
`mdlint check --output-format json` printed `Fixed:` messages and rewrote the
temporary fixture even though the probe did not pass `--fix`.

Local tool versions:

| Tool | Version evidence |
| --- | --- |
| `rumdl` | `rumdl 0.2.27`; crates.io metadata reports MIT and `rust-version: 1.94.0`. |
| `mdlint` | `mdlint 0.3.18`; crates.io package `markdownlint-rs` is Unlicense. |
| `mado` | `mado 0.3.0`; macOS x86_64 GitHub release asset. |
| `markdownlint-cli2` | `markdownlint-cli2 v0.23.0 (markdownlint v0.41.0)`. |

Probe command:

```bash
PATH="$PWD/target/markdown-engine-tools/bin:$PATH" \
  cargo xtask markdown-engine-probe --run-external \
  | tee target/markdown-engine-probe-2026-07-02-isolated.json
```

Result summary:

| Candidate | Status | Fixture findings observed | Integration implication |
| --- | --- | --- | --- |
| `assura-current` | `ran` | Stable Assura rules for structure, suppressions, required sections, headings, links, and trailing spaces. | Remains the contract baseline and owns repository semantics. |
| `rumdl` | `ran_with_findings` | JSON diagnostics for `MD001`, `MD009`, `MD012`, `MD018`, `MD024`, `MD025`, `MD051`, and `MD057`, with fix metadata for several rules. | Best fit for a Rust markdownlint-compatible lint/fix adapter, but direct embedding still requires an MSRV decision or subprocess boundary. |
| `mdlint` | `ran_with_findings` | JSON diagnostics for `MD001`, `MD003`, `MD009`, `MD012`, `MD018`, `MD022`, `MD024`, `MD041`, and `MD051`; also wrote fixes to the isolated fixture. | Needs strict sandboxing or deeper investigation before any adapter; unrequested writes are a blocker for direct in-place probe use. |
| `mado` | `ran_with_findings` | Markdownlint-style text for `MD001`, `MD009`, `MD012`, `MD018`, `MD024`, and `MD041`. | Useful comparison point, but narrower than `rumdl` and no JSON/fix surface was observed in this probe. |
| `markdownlint-cli2` | `ran_with_findings` | Node baseline reported `MD001`, `MD009`, `MD012`, `MD018`, `MD024`, and `MD025`. | Remains compatibility baseline only; not acceptable as Assura's supported runtime dependency. |

Current decision: continue evaluating `rumdl` first. It produced the richest
machine-readable Rust output, surfaces fix metadata without mutating files in
check mode, and overlaps the target markdownlint-compatible fixture rules. The
next implementation slice should measure `rumdl` lint/fix cost and decide
between a subprocess adapter and an explicit MSRV increase for direct library
integration.

The adapter decision is now recorded in
`docs/analysis/2026-07-02-rumdl-adapter-decision.md`: keep Assura's current
Rust 1.70 MSRV for the next slice and prototype `rumdl` as an optional
subprocess adapter before considering a direct dependency or MSRV increase.

Raw default checks against the `valid` fixture were intentionally not recorded
as passing compatibility evidence. `rumdl`, `mdlint`, `mado`, and
`markdownlint-cli2` all reported findings against the current Assura-valid
fixture. The repeated causes were frontmatter being interpreted as heading
content, single-H1/title expectations that conflict with this fixture shape,
default 80-column line-length behavior, and link/reference semantics that
Assura handles separately. `mdlint` also rewrote the valid fixture when run
directly. This confirms the integration must provide an explicit candidate
configuration/mapping layer before any candidate can be called compatible with
Assura's valid fixture set.

## Source Links

- `rumdl`: https://github.com/rvben/rumdl
- `rumdl` comparison: https://rumdl.dev/comparison/
- `rumdl_lib` docs: https://docs.rs/rumdl/latest/rumdl_lib/
- `mado`: https://github.com/akiomik/mado
- `mdlint`: https://github.com/swanysimon/mdlint
- `markdownlint`: https://github.com/DavidAnson/markdownlint
