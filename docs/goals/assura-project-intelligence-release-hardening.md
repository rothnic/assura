---
id: goal-assura-project-intelligence-release-hardening
type: goal
title: Assura project intelligence release hardening
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-onboarding-template.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - docs/goals/assura-project-intelligence-agent-cli-surface.md
  - docs/goals/assura-project-intelligence-lsp-editor-transport.md
  - docs/support-policy.md
  - website/src/content/docs/reference/release-readiness.md
  - docs/analysis/evidence-and-review-policy.md
---

# Assura Project Intelligence Release Hardening

## Objective

Prepare the usable project-intelligence slice for release by locking support
status, schema compatibility expectations, docs, evidence, and reviewer gates.

## Current Gap

The runtime is complete locally and the usability goals define adoption,
real-repo proof, onboarding templates, context packs, persistent sessions,
safe-fix workflow, `.assura/` directory organization, agent CLI surface, and
LSP-shaped local editor session. Before that can be advertised as usable,
release surfaces need to agree on what is supported, experimental,
roadmap-only, or unsupported.

## Scope

- Add release-readiness rows for project-intelligence commands, schemas, and
  transports.
- Include onboarding template and context-pack schemas in the support matrix.
- Add compatibility snapshot tests or golden examples for stable JSON schemas.
- Update support policy, website reference docs, and command-surface matrices.
- Produce release notes that call out experimental vs supported surfaces.
- Run install/adoption smoke evidence for the project-intelligence workflow.
- Record independent review evidence for the full usability program.

## Non-Goals

- No 1.0 compatibility guarantee.
- No hosted service release.
- No package-manager publication unless a separate release goal owns it.
- No promotion of incomplete transports.

## Definition Of Done

- Supported, experimental, roadmap, and unsupported project-intelligence
  surfaces are consistent across docs and support policy.
- Stable schemas have checked examples or snapshot tests.
- Release readiness docs include project-intelligence adoption and rollback
  guidance.
- Install/adoption smoke evidence proves users can run the supported workflow
  from documented artifacts or release-candidate outputs.
- The master usability program has a final audit mapping every definition of
  done item to evidence.

## Validation Commands

```bash
cargo fmt --check
cargo test --workspace --all-targets --all-features --quiet
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm release docs and support policy classify every advertised surface
  consistently.
- R2: Confirm schema examples match live command output.
- R3: Confirm install/adoption smoke evidence does not rely on local source
  checkout shortcuts unless explicitly labeled as release-candidate evidence.
- R4: Confirm the usability program final audit does not close over unresolved
  transport or safe-fix blockers.

## Reviewer Blocking Criteria

Block if docs advertise unsupported surfaces, if stable schema examples are not
checked against live output, if install evidence is missing, or if release notes
hide breaking or experimental behavior behind generic project-intelligence
language.

## Progress Log

- 2026-06-29: Started on task
  `.trellis/tasks/06-29-project-intelligence-release-hardening` after local
  completion of the `.assura/` organization, agent CLI, and editor session
  successors. Initial audit found `website/src/content/docs/reference/release-readiness.md`
  and `docs/release-candidate-checklist.md` lagged behind the supported
  project-intelligence surfaces. Added release-hardening task requirements,
  release-readiness documentation for project-intelligence commands/schemas,
  a checked final-audit artifact, and a new
  `tests/project_intelligence_release_hardening.rs` smoke/schema coverage
  target.
- 2026-06-29: Release-hardening implementation passed validation. The new
  release-hardening test caught and fixed a missing `.assura/models/**`
  compatibility classification and release-readiness wording for full LSP
  server boundaries. A full workspace validation then passed:
  `cargo fmt --check`,
  `cargo test --workspace --all-targets --all-features --quiet`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo run --quiet -- check --format json .`, `cargo xtask docs`,
  `cargo xtask evidence`, and `git diff --check`.
- 2026-06-29: Independent release-hardening review agent
  `019f14e9-a9b3-73e0-86ab-5ac72ec14d3d` found five release-readiness gaps:
  target-state support-matrix coverage for new project-intelligence commands,
  release notes that blurred current-branch Project Intelligence work with the
  already-published May 24, 2026 `v0.1.0` archives, weak docs/schema golden
  coverage, missing safe-fix apply language, and null active-task branch
  metadata. Fixed the support-matrix rows, release-note framing,
  schema-example test coverage, safe-fix apply docs, and task branch metadata.
