---
id: goal-assura-agent-proposal-sbir-domain-pack
type: goal
title: Assura agent proposal SBIR domain pack
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-document-project-preset.md
  - ./assura-agent-requirements-evidence-traceability.md
  - ./assura-agent-script-backed-computed-checks.md
---

# Assura Agent Proposal SBIR Domain Pack

## Objective

Package proposal/SBIR-specific modeling, scoring, traceability, review, and
submission checks as a domain pack that composes with the generic agent-project
and document-project foundations.

## Scope

- Add a proposal/SBIR pack after the generic onboarding, document-project,
  traceability, and computed-check foundations are usable.
- Model gates, weighted scores, confidence, review actions, source documents,
  requirements, evidence, scorecards, and final package readiness.
- Use script-backed computed checks for scoring and rollups before native
  computed fields exist.
- Validate final package manifests, source-document custody, review findings,
  and portal/submission checklist records.
- Clearly label this as a domain pack, not part of the core agent-project
  preset.

## Non-Goals

- No proposal-specific behavior in the core agent-project preset.
- No unsupported PDF/DOCX content parsing as a core requirement.
- No claim that scoring replaces human proposal judgment.

## Definition Of Done

- A proposal repo can apply the pack and receive deterministic checks for
  requirements/evidence coverage, review status, scoring inputs, and package
  readiness.
- The pack composes with document-project and source-document custody.
- Website docs explain the pack as an optional domain workflow.
- Agent next-actions prioritize missing evidence, unresolved source documents,
  incomplete review findings, and final package gaps.

## Validation Commands

```bash
cargo fmt --check
cargo test content_runtime --quiet
cargo test computed_checks --quiet
cargo test --test project_intelligence_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if proposal/SBIR logic leaks into generic presets, if scoring cannot be
audited from source records, if final package checks require unsafe binary
reads, or if agent output suggests the pack is a substitute for expert review.
