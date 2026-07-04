---
id: goal-assura-agent-project-preset-dynamic-contracts
type: goal
title: Assura agent project preset and dynamic contracts
status: completed
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
- No one-off document-heavy domain model.
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

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Implemented the first dynamic-contract slice for the generated `agent-project` baseline. The compact notation normalizer now treats exact children under captured directory scopes as direct children, the full rule plan consistently prefers deeper/more-specific scopes, and `assura agent onboard` generates one reusable `@assura-skill-dir` contract applied through `.agents/skills/{skill}/`. | `nested_captured_directory_use_expands_tree_rule_fragments`; `dynamic_directory_scopes_apply_normalized_captured_rule_fragments`; `agent_onboard_generated_config_validates_dynamic_directory_skill_contracts`; `cargo test --test project_intelligence_onboarding --quiet`. |
| 2026-07-02 | Addressed independent review findings for mixed capture-plus-brace scopes and same-depth precedence. Scope matching now expands brace alternatives before or after captures, and both full and fast rule plans share deterministic specificity so literal or constrained pattern scopes beat broad captures. | `scope_patterns_expand_brace_alternatives_after_captures`; `scope_patterns_expand_brace_alternatives_before_captures`; `literal_scopes_override_captured_scopes_at_same_depth`; `constrained_patterns_override_long_capture_names_at_same_depth`; `fast_plan_literal_scopes_override_captured_scopes_at_same_depth`; `fast_plan_constrained_patterns_override_long_capture_names_at_same_depth`. |
| 2026-07-02 | Added CLI-level proof for reusable dynamic contracts beyond skills. The normalizer now avoids generating implicit counterpart relationships for exact structural children inside captured directories, so package, docs-section, example, and fixture contracts can require local children without ambiguous relationship errors while explicit `needs:`/`provides:` relationships still work. | `agent_project_dynamic_contracts_validate_repeated_project_structures`; `nested_captured_directory_use_expands_tree_rule_fragments`; `captured_directory_exact_child_providers_remain_explicit_relationships`; `counterpart`; `cargo test --test project_intelligence_onboarding --quiet`. |
| 2026-07-02 | Closed child goal 2 after independent review and PR verification. The generated onboarding baseline now uses reusable dynamic contracts for skills, merge mode preserves the reusable contract path, repeated non-skill structures are covered by CLI integration tests, and PR #139 reached a clean merge state with all checks green. | PR #139 `mergeStateStatus=CLEAN`; `cargo fmt --check`; `cargo test --test project_intelligence_onboarding --quiet`; `cargo test --lib counterpart --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; independent review findings addressed in `8566abd`. |
