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
