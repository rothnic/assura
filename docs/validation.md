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
node --run verify:changed
node --run verify:changed -- --phase pre-push
```

This asks `assura quality plan` for the checks selected by
`.assura/config.yml` under `quality.scopes`, then executes only local shell
commands selected for the requested phase. It is the default local loop for
docs, Trellis, website, release, and Rust changes because it avoids rebuilding
or retesting unrelated surfaces. When a broader local gate such as
`node --run verify:pr` is selected, narrower checks already covered by that
gate are skipped. Use `--dry-run` or `--files-from` to inspect a deterministic
plan:

```bash
printf 'docs/validation.md\n' \
  | node --run verify:changed -- --files-from - --dry-run
```

Run the full fast gate when the changed-file plan is not enough or when you
want a broad local confidence sweep:

```bash
node --run verify:fast
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
node --run verify:check
node --run verify:test
node --run verify:evidence
node --run verify:docs
node --run verify:release-size
node --run verify:release-smoke
node --run verify:release-live
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

Run the evidence gate when changing goal docs, PR templates, review records,
Trellis roadmap state, or public agent feedback wording:

```bash
node --run verify:evidence
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

GitHub Actions currently uses `scripts/ci-scope.sh` as the lightweight bootstrap
classifier before running expensive jobs. It mirrors the same policy shape but
does not invoke `assura quality plan` yet because compiling Assura inside the
first scope job would erase the speed win for docs-only changes. Classifier
policy is covered by `node --run verify:evidence`; test it directly with:

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

Security Audit uses the same classifier. It runs for Cargo metadata changes and
scheduled audits; source-only Rust changes are covered by Rust compile/test
gates without scheduling `cargo audit`.

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
