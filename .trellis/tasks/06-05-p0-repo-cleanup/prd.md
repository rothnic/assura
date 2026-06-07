# P0 Repository Cleanup And Stale Surface Alignment

## Goal

Clean the active repository state after the LS-Lint semantic migration by
removing or correcting stale public-facing guidance, aligning docs and project
configuration with the current supported command surface, and recording any
future deterministic checks that should become Assura rules.

The parked branch `codex/park-agent-governance-audit-20260605` preserves the
larger Agent Work Governance spike and the Rust modernization audit. This task
must not pull that feature spike into the cleanup branch.

## Context

- Current branch: `codex/p0-repo-cleanup`, based on `origin/master`.
- Parked branch: `codex/park-agent-governance-audit-20260605`.
- The audit report on the parked branch is
  `docs/analysis/2026-06-04-rust-modernization-quality-audit.md`.
- The active cleanup principle is small, deterministic, and reviewable changes
  before adding new feature surfaces.

## Scope

- Audit active docs, website docs, skills, Trellis specs, and examples for
  stale Assura command examples or unsupported feature claims.
- Correct stale command surfaces so documented examples match current CLI help.
- Tighten `.assura/config.yml` where existing Assura rules can express a
  deterministic cleanup invariant without broad exclusions.
- Record future generalized Assura rule opportunities when current config
  cannot express the invariant.
- Keep changes small enough for one focused cleanup PR.

## Non-Goals

- Do not merge or cherry-pick the parked Agent Work Governance implementation.
- Do not add new commands or feature behavior.
- Do not perform broad module refactors.
- Do not add new external Rust tooling configs unless directly required to
  validate this cleanup.
- Do not rewrite roadmap strategy beyond correcting stale references required
  by this task.

## Definition Of Done

- `git status --short` shows only intentional changes before commit.
- Stale command examples are corrected or explicitly marked as unsupported
  negative examples.
- Active docs and skills do not claim support for commands or flags missing
  from `assura --help` / `assura check --help`.
- Any `.assura/config.yml` changes are backed by a successful self-check.
- Future rule opportunities are recorded in a durable doc or task note rather
  than left as chat context.
- The parked branch still preserves the Agent Work Governance/audit spike.

## Validation

Run the relevant subset while editing, then before completion run:

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
node --run verify:docs
node --run verify:evidence
git diff --check
```

## Reviewer Blocking Criteria

Block the PR if it introduces new feature behavior, mixes in the parked Agent
Work Governance implementation, hides unsupported commands behind exclusions,
or leaves active docs/skills describing command surfaces that the CLI does not
support.
