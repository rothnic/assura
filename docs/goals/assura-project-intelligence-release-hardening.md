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
real-repo proof, persistent sessions, transports, and safe-fix workflow. Before
that can be advertised as usable, release surfaces need to agree on what is
supported, experimental, roadmap-only, or unsupported.

## Scope

- Add release-readiness rows for project-intelligence commands, schemas, and
  transports.
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
