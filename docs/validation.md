---
title: Validation Command Tiers
date: 2026-05-23
status: active
---

# Validation Command Tiers

Use the narrowest tier that proves the change.

## Fast Local Gate

Run this during normal editing:

```bash
node --run verify:fast
```

This runs formatting, whitespace checks, focused compile checks for the primary
`assura` launcher and `assura-full` companion, Rust tests without benchmark
harness targets, and the Assura self-check.

## Targeted Gates

Use focused commands when the change is narrow:

```bash
node --run verify:check
node --run verify:test
node --run verify:docs
node --run verify:release-smoke
```

Run the website build for docs or frontend changes. Run the release smoke for
installer, release workflow, or primary launcher changes.

## PR Gate

Run this before pushing broad Rust or mixed changes:

```bash
node --run verify:pr
```

This adds Clippy and the website build to the fast local gate.

## Full Gate

Reserve the full gate for benchmark-adjacent code, benchmark harness changes,
or final release confidence:

```bash
node --run verify:full
```

This intentionally runs `cargo test --all-targets`, which executes benchmark
harness targets as test binaries. It is broader and noisier than the fast gate,
so it should not be the default iteration command.
