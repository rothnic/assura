---
title: Markdownlint-compatible Rust engine
status: planning
priority: P0
---

# Markdownlint-Compatible Rust Engine

## Goal

Execute the next child goal from
`docs/goals/assura-post-beta-capabilities-program.md`: select and integrate,
or explicitly reject with evidence, the fastest practical Rust Markdown
linter/fixer path that is consistent with markdownlint.

## Current State

- The true-daemon child goal is complete in PR #117 as merge commit
  `745455215d757e49fb4614a170e48f046cb829ad`.
- Assura already has Rust-native Markdown checks for links, headings,
  suppressions, required-section fixes, and common lint classes.
- Assura does not yet provide broad markdownlint-compatible rule/config
  coverage or prove the selected Markdown lint/fix path is the fastest
  available Rust option.
- Markdown linting must remain below structure and coarse file-level policy in
  user-facing validation order.

## Requirements

- Revalidate `docs/goals/assura-markdownlint-compatible-rust-engine.md` before
  implementation.
- Evaluate `rumdl` first, then compare against `mado`,
  `markdownlint-rs`/`mdlint`, current Assura Markdown checks, and Node
  `markdownlint-cli2`.
- Build a local markdownlint compatibility matrix for rule IDs, config keys,
  suppressions, severities, fixability, and expected diagnostics.
- Preserve Assura-specific link/reference graph checks and daemon/reference
  contracts.
- Preserve staged validation order: structure and coarse file policy first,
  then Markdown internals, then content-model/reference/language-specific
  findings.
- Prove safe fixes are deterministic, idempotent, bounded, and preserve
  frontmatter and line endings where required.
- Add benchmark fixtures for small docs, large docs, many-file repos,
  generated docs, frontmatter-heavy docs, and link-heavy docs.
- If no candidate is adopted, produce a decision record with measured reasons
  and a fallback plan.

## Acceptance Criteria

- [ ] Candidate research records version, license, MSRV, API shape, binary size,
      dependency impact, rule coverage, fix coverage, and performance evidence.
- [ ] Markdownlint-compatible fixtures cover accepted rule/config behavior and
      expected diagnostic mapping.
- [ ] The chosen path integrates with Assura finding IDs, severity overrides,
      reasoned suppressions, JSON/agent output, and `assura fix markdown`, or a
      rejection record explains why integration is deferred.
- [ ] Assura-specific Markdown link/reference graph checks remain covered by
      tests.
- [ ] User-facing docs preserve validation hierarchy and do not imply Markdown
      linting sits above structure validation.
- [ ] Performance evidence shows the accepted Rust path is no slower than
      current Assura Markdown checks for accepted fixtures and materially faster
      than `markdownlint-cli2` on representative repos.
- [ ] Independent review finds no unsafe fixes, unmeasured linter adoption, or
      regression to reference graph behavior.

## Validation

```bash
cargo fmt --check
cargo test --test markdown_link_reference_tests --quiet
cargo test --test markdown_suppression_severity_tests --quiet
cargo test --test markdown_required_section_fix_tests --quiet
cargo test --test markdown_lint_fix_tests --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
python3 ./.trellis/scripts/task.py validate 07-02-07-02-markdownlint-compatible-rust-engine
```

## Reviewer Blocking Criteria

Block if implementation skips `rumdl` evaluation, adopts a linter without local
compatibility fixtures, applies unsafe fixes, regresses Assura reference checks,
omits benchmark comparison against current Assura, Rust candidates, and
`markdownlint-cli2`, or lets docs imply Markdown linting is above structure
validation.
