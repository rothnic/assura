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

## Source Links

- `rumdl`: https://github.com/rvben/rumdl
- `rumdl` comparison: https://rumdl.dev/comparison/
- `rumdl_lib` docs: https://docs.rs/rumdl/latest/rumdl_lib/
- `mado`: https://github.com/akiomik/mado
- `mdlint`: https://github.com/swanysimon/mdlint
- `markdownlint`: https://github.com/DavidAnson/markdownlint
