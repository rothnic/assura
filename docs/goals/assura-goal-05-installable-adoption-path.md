---
id: goal-assura-roadmap-05-installable-adoption-path
type: goal
title: Assura roadmap 05 installable adoption path
status: planned
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - website/src/content/docs/guides/installation.md
  - website/src/content/docs/guides/getting-started.md
---

# Goal 05: Installable Adoption Path

## Objective

Make Assura adoption work from install to first useful feedback without a source
checkout, hidden local setup, or stale website instructions.

This is a two-week team chunk for release, website, CLI, and docs owners.

## Scope

- Validate install scripts on Linux, macOS, and Windows.
- Make `assura init`, `assura migrate`, `assura check`, and `assura status`
  form a coherent first-run path.
- Add fresh-machine smoke scripts or CI jobs that use release artifacts rather
  than source builds.
- Rewrite getting started docs around supported commands only.
- Include one realistic adoption walkthrough that starts from an empty project
  and one that starts from an LS-Lint config.
- Document failure recovery for missing config, invalid config, unsupported
  migration rules, and hook prerequisites.

## Non-Goals

- No package feedback CLI.
- No cloud service.
- No promise of dependency graph validation.

## Definition Of Done

- A reviewer can install Assura from documented artifacts and run a check
  without cloning this repository.
- Install scripts are smoke-tested from release-style artifacts on this platform
  matrix: `ubuntu-latest` x86_64, `macos-14` arm64, `macos-13` x86_64, and
  `windows-latest` x86_64. If one platform is unavailable in CI, the PR must
  include dated manual terminal evidence for that platform and a follow-up issue
  to restore automation.
- The artifact source is explicit: smoke tests must install from a release
  workflow archive, package dry-run output, or documented release candidate URL.
  Smoke tests must not satisfy the goal by running `cargo run` from a source
  checkout.
- Each platform smoke must prove `assura --version`, `assura init`,
  `assura status --format json`, `assura check --format json`, and one failing
  validation case with a nonzero exit status.
- Getting started docs use current command signatures.
- LS-Lint migration docs clearly label supported and unsupported rules.
- The website build passes and links the adoption walkthrough from an existing
  discoverable path.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo run --quiet -- check --format json .
cd website && npx pnpm@10.25.0 build
git diff --check
```

## Review Tasks

- R0: Confirm docs are based on release artifact behavior, not local source
  convenience.
- R1: Review install script contracts and platform assumptions.
- R2: Review CLI first-run errors for clarity.
- R3: Reproduce a fresh install smoke path and confirm the platform matrix and
  artifact source are included in PR evidence, with any manual substitution dated
  and justified.
- R4: Review website flow on desktop and mobile when pages changed.
- R5: Confirm PR evidence includes commands a reviewer can run.

## Reviewer Blocking Criteria

Block the PR if docs require an unmentioned source checkout, if install scripts
overwrite user files without opt-in, or if first-run examples use unsupported
feedback command surfaces. Also block if the platform matrix is reduced without
an explicit owner-approved exception.
