---
id: goal-assura-roadmap-05-installable-adoption-path
type: goal
title: Assura roadmap 05 installable adoption path
status: active
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
  matrix: `ubuntu-latest` x86_64, `macos-14` arm64, `macos-15-intel` x86_64,
  and `windows-latest` x86_64. If one platform is unavailable in CI, the PR must
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

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-02 | Started Goal 05 from updated `master` after Goal 04 merged; moved the active Trellis task to `codex/phase-01-goal-05-installable-adoption-path`. | `gh pr view 21 --json state,mergedAt,mergeCommit,url`; `git status --short --branch`; `python3 ./.trellis/scripts/task.py set-branch 06-01-roadmap-phase-01-execution codex/phase-01-goal-05-installable-adoption-path`. |
| 2026-06-02 | Added release-archive adoption smoke scripts and CI matrix wiring for Ubuntu x86_64, macOS arm64, macOS x86_64, and Windows x86_64. The local macOS archive smoke installed from `target/assura-macos-amd64-preview.tar.gz` and proved version, init, status JSON, passing check JSON, failing check JSON, and LS-Lint migration. | `scripts/smoke-install-adoption.sh`; `scripts/smoke-install-adoption.ps1`; `.github/workflows/ci.yml`; `node --run verify:release-smoke`; `target/adoption-smoke/local/empty-check-pass.json`; `target/adoption-smoke/local/empty-check-fail.json`; `target/adoption-smoke/local/lslint-check-pass.json`. |
| 2026-06-02 | Completed local Goal 05 validation and review-agent pass before PR creation. Full platform smoke remains delegated to PR CI because Windows and macOS arm64 require GitHub-hosted runners. | `cargo test --all-targets --quiet`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --release --bin assura --no-default-features --features json-output,yaml-config`; `cargo run --quiet -- check --format json .`; `cd website && npx pnpm@10.25.0 build`; `git diff --check`; review agent `019e8604-7d65-7842-b89b-ef24e6356a12`. |
| 2026-06-02 | Replaced stale Intel macOS CI label `macos-13` with the current GitHub-hosted Intel label `macos-15-intel` after the x86_64 adoption smoke remained queued with no runner assigned. | `gh api repos/rothnic/assura/actions/jobs/78985721463`; GitHub-hosted runners reference for Intel macOS labels. |
