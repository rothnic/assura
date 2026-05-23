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
harness or standalone binary harness targets, and the Assura self-check.

## Targeted Gates

Use focused commands when the change is narrow:

```bash
node --run verify:check
node --run verify:test
node --run verify:docs
node --run verify:release-size
node --run verify:release-smoke
node --run verify:release-live
```

Run the website build for docs or frontend changes. Run the release smoke for
installer, release workflow, or primary launcher changes; on Unix it builds the
local archive and installs it through `website/public/install.sh` with a local
asset override. Run the release-size gate when changing build profiles, release
packaging, install scripts, or the primary/full CLI split.

After a release tag is published, run the live release gate to verify the exact
unauthenticated URLs used by new users:

```bash
node --run verify:release-live
ASSURA_VERSION=v0.1.0 node --run verify:release-live
```

`target/` is Cargo's build cache and can be many gigabytes after local test,
benchmark, and release runs. The public artifact is the archive produced under
`target/assura-*-preview.tar.gz` or `target/assura-*-preview.zip`; the
release-size gate checks that archive instead of the cache directory. Override
the default 8 MiB archive budget only when the PR explains why:

```bash
ASSURA_MAX_RELEASE_ARCHIVE_BYTES=8388608 node --run verify:release-size
```

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
