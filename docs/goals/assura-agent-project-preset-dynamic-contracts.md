---
id: goal-assura-agent-project-preset-dynamic-contracts
type: goal
title: Assura agent project preset and dynamic contracts
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-onboarding-bootstrap-command.md
---

# Assura Agent Project Preset And Dynamic Contracts

## Objective

Implement the broad `agent-project` baseline and reusable dynamic directory
contracts so project-local skills and repeated structures do not require
hardcoded config entries.

## Scope

- Add the broad agent-project preset used by first-run onboarding.
- Require or recommend `AGENTS.md`, `.assura/config.yml`, `.agents/skills/`,
  `docs/process/`, `docs/learnings/`, `README.md`, and `.gitignore` according
  to the parent-goal baseline.
- Add reusable repeated-directory contracts for skill directories and similar
  project-local structures.
- Add default safe rules for root clutter, Markdown links, line limits,
  binary-read exclusions, skill folder children, scripts, references, and
  assets.
- Ensure dynamic contracts work for skills, package/app folders, examples,
  fixtures, and docs sections without listing every child.

## Non-Goals

- No language-specific Rust/Node/Python rules unless a pack explicitly opts in.
- No proposal or SBIR domain model.
- No performance benchmark changes.

## Definition Of Done

- A generated agent-project config can validate multiple skill directories
  through one reusable contract.
- Passing and failing fixtures cover repeated skill directories, unexpected
  child folders, missing `SKILL.md`, allowed `references/`, `scripts/`, and
  `assets/`.
- The preset can be merged into an existing config without duplicating large
  blocks per skill.
- Docs describe the baseline as broad and safe, not domain-specific.

## Validation Commands

```bash
cargo fmt --check
cargo test dynamic_directory --quiet
cargo test skill --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if users still need to enumerate every skill directory manually, if the
preset overfits a language or domain, or if dynamic contracts make inheritance
harder to explain.
