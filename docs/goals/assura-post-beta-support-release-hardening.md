---
id: goal-assura-post-beta-support-release-hardening
type: goal
title: Assura post-beta support and release hardening
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ../support-policy.md
  - ../compatibility-and-surface.md
  - ../data/release-surfaces.json
---

# Assura Post-Beta Support And Release Hardening

## Objective

Close the post-beta program with release-grade support classifications,
compatibility notes, target-state checks, documentation, and evidence for all
newly supported code-agnostic capabilities.

## Current Gap

The next program spans daemon process support, document graph support,
Markdown lint/fix integration, agent hooks, VS Code packaging, performance
gates, and extension API boundaries. A final hardening goal should reconcile
their support levels before any beta or post-beta release claim is made.

## Scope

- Audit public docs, release notes, support policy, compatibility matrix,
  command surface, release surfaces, and website roadmap for consistent
  support wording.
- Ensure supported, experimental, internal, planned, and unsupported surfaces
  are classified consistently.
- Add target-state checks for the new supported surfaces and for known
  overclaim risks.
- Confirm all child-goal validation evidence is current.
- Prepare release readiness notes and a rollback/support plan for daemon,
  agent, VS Code, document graph, Markdown, and performance surfaces.

## Non-Goals

- No new feature implementation unless needed to fix a release blocker.
- No support promotion for a surface that lacks tests, docs, and review.
- No hosted service or marketplace claim without corresponding release proof.

## Definition Of Done

- Support policy, compatibility docs, release surfaces, website docs, and public
  roadmap agree on the status of every post-beta capability.
- Target-state checks prevent the most likely unsupported claims.
- All child goals have completion evidence or explicit deferral notes.
- Release readiness commands pass on a clean branch.
- Independent review confirms no public overclaim or missing support caveat.

## Validation Commands

```bash
cargo fmt --check
cargo test --workspace --all-targets --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm support classifications are consistent across docs and data.
- R2: Confirm release evidence covers every promoted surface.
- R3: Confirm unsupported or deferred items remain clearly marked.
- R4: Confirm target-state checks cover the highest-risk overclaims.

## Reviewer Blocking Criteria

Block if public docs claim unsupported daemon, editor, agent, graph, Markdown,
or extension behavior; if release evidence is stale; if target-state permits a
known overclaim; or if a child goal is marked complete without independent
review.
