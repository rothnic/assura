---
title: Validation Command Tiers
date: 2026-05-23
status: active
---

# Validation Command Tiers

Use the narrowest tier that proves the change.

## Fast Local Gate

Run the changed-file gate during normal editing:

```bash
cargo xtask changed
cargo xtask changed -- --phase pre-push
```

This asks `assura quality plan` for the checks selected by
`.assura/config.yml` under `quality.scopes`, then executes only local shell
commands selected for the requested phase. It is the default local loop for
docs, Trellis, website, release, and Rust changes because it avoids rebuilding
or retesting unrelated surfaces. When a broader local gate such as
`cargo xtask pr` is selected, narrower checks already covered by that
gate are skipped. Use `--dry-run` or `--files-from` to inspect a deterministic
plan:

```bash
printf 'docs/validation.md\n' \
  | cargo xtask changed -- --files-from - --dry-run
```

Run the full fast gate when the changed-file plan is not enough or when you
want a broad local confidence sweep:

```bash
cargo xtask fast
```

This runs formatting, whitespace checks, focused compile checks for the primary
`assura` launcher and `assura-full` companion, Rust tests without benchmark
harness or standalone binary harness targets, the Assura self-check, and the
Trellis/goal status sanity check. The status check fails when completed tasks
remain under `.trellis/tasks/` instead of the archive, when goal frontmatter
uses a status outside `planned`, `active`, `completed`, or `archived`, or when
the Phase 01 ledger and `assura-goal-01..08` frontmatter statuses disagree.

## Targeted Gates

Use focused commands when the change is narrow:

```bash
cargo xtask check
cargo xtask test
cargo xtask evidence
cargo xtask docs
cargo xtask release-size
cargo xtask release-smoke
cargo xtask release-live
```

Run the website build for docs or frontend changes. Run the release smoke for
installer, release workflow, or primary launcher changes; on Unix it builds the
local archive, installs it through `website/public/install.sh` with a local
asset override, and runs the first-run adoption smoke against the installed
binary. That smoke proves `assura --version`, `assura init`,
`assura status --format json`, `assura check --format json`, a failing
validation case, and an LS-Lint migration walkthrough. Run the release-size gate
when changing build profiles, release packaging, install scripts, or the
primary/full CLI split.

After a release tag is published, run the live release gate to verify the exact
unauthenticated URLs used by new users:

```bash
cargo xtask release-live
ASSURA_VERSION=v0.2.0 cargo xtask release-live
```

`target/` is Cargo's build cache and can be many gigabytes after local test,
benchmark, and release runs. The public artifact is the archive produced under
`target/assura-*-preview.tar.gz` or `target/assura-*-preview.zip`; the
release-size gate checks that archive instead of the cache directory. Override
the default 8 MiB archive budget only when the PR explains why:

```bash
ASSURA_MAX_RELEASE_ARCHIVE_BYTES=8388608 cargo xtask release-size
```

Run the evidence gate when changing goal docs, PR templates, review records,
Trellis roadmap state, or public agent feedback wording:

```bash
cargo xtask evidence
```

This checks review evidence templates, goal frontmatter metadata, local
markdown links in goal/review/spec docs, and stale forbidden user-facing command
surfaces such as per-agent feedback CLIs or per-agent check formats.

## CI Scope Gate

Assura owns the high-level quality policy in `.assura/config.yml` under
`quality.scopes`. Use `assura quality plan` to ask which checks apply to a
changed-file set and workflow phase:

```bash
printf 'docs/validation.md\nsrc/main.rs\n' \
  | assura quality plan . --files-from - --phase merge --format json
```

Phases are cumulative for normal development: `frequent` is the local loop,
`pre-push` adds branch-push checks, `pr` adds pull-request checks, and `merge`
adds final merge confidence. `release` adds release-specific checks.
`scheduled` is separate for background audits.

GitHub Actions uses `scripts/ci-scope-github.sh` as the lightweight bootstrap
classifier before running expensive jobs. That wrapper calls
`scripts/ci-scope.sh` and records two scopes when possible:

- full scope: the full PR or push diff;
- effective scope: the scope used by jobs in the current workflow run.

For `pull_request` `synchronize` events, effective scope can be the commit
delta from the previous PR head to the new PR head. If the delta would skip a
heavy full-scope job family, the wrapper first checks the previous PR head for
successful check runs in that family. This lets docs/planning-only follow-up
commits avoid rerunning heavy Rust, release, performance, coverage, rustdoc,
security, and install-smoke jobs after those jobs are already green. If the
previous-head evidence is missing or not green, the wrapper falls back to full
scope.

Opened/reopened PRs, pushes, schedules, workflow/classifier changes, and
unavailable delta state use full scope.

The wrapper mirrors the same policy shape but does not invoke
`assura quality plan` yet because compiling Assura inside the first scope job
would erase the speed win for docs-only changes. Classifier policy is covered by
`cargo xtask evidence`; test it directly with:

```bash
scripts/check-ci-scope.sh
```

Use representative file lists for quick manual probes:

```bash
printf 'AGENTS.md\n.trellis/workflow.md\n' | scripts/ci-scope.sh --files-from -
printf 'src/main.rs\nCargo.toml\n' | scripts/ci-scope.sh --files-from -
printf 'website/public/install.sh\n' | scripts/ci-scope.sh --files-from -
printf '.github/workflows/ci.yml\n' | scripts/ci-scope.sh --files-from -
```

Workflow or classifier changes intentionally force every CI scope. Docs,
Trellis, skill, Assura config, and agent-policy-only changes keep evidence
validation and Assura self-check active without scheduling Rust compile/test,
release, coverage, rustdoc, or performance jobs.

Branch names are useful routing hints, but they are not the CI authority. Use
`docs/*` or `planning/*` branch names for documentation and backlog-only work
when practical, and prefer a separate PR for planning-only follow-ups. CI still
uses changed paths, not branch names, to decide which jobs are required.

Security Audit uses the same classifier. It runs for Cargo metadata changes and
scheduled audits; source-only Rust changes are covered by Rust compile/test
gates without scheduling `cargo audit`.

## PR Gate

Run this before pushing broad Rust or mixed changes:

```bash
cargo xtask pr
```

This adds Clippy and the website build to the fast local gate.

## Full Gate

Reserve the full gate for benchmark-adjacent code, benchmark harness changes,
or final release confidence:

```bash
cargo xtask full
```

This intentionally runs `cargo test --all-targets`, which executes benchmark
harness targets as test binaries. It is broader and noisier than the fast gate,
so it should not be the default iteration command.
