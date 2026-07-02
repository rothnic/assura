---
title: Markdownlint-compatible Rust engine
status: active
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
- Anchor implementation to the goal's outcome verification use case: a
  documentation-heavy Rust CLI maintainer must be able to validate staged
  structure/coarse policy first, then Markdown internals, then preserve the same
  findings through CLI, daemon, editor, and agent surfaces.
- Evaluate `rumdl` first, then compare against `mado`,
  `markdownlint-rs`/`mdlint`, current Assura Markdown checks, and Node
  `markdownlint-cli2`.
- Build a local markdownlint compatibility matrix for rule IDs, config keys,
  suppressions, severities, fixability, and expected diagnostics.
- Keep rule severity modular and stable: severity, suppression, and fix policy
  belong under stable rule identities instead of unique concern keys nested
  below a top-level severity map.
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

## Final Output Verification Story

This child goal should be judged against the parent program's final
documentation-heavy Rust CLI fixture, not only a candidate linter benchmark.
The reviewer should be able to picture and eventually run this workflow:

1. A maintainer starts with a repository that has goals, ADRs, analysis notes,
   generated references, release docs, and agent-written Markdown.
2. The repository contains intentional drift: a misplaced generated file,
   coarse line-policy violations, markdownlint-style formatting drift,
   duplicated and skipped headings, stale anchors, invalid suppression comments,
   and content-model frontmatter that later document-graph checks will own.
3. `assura check` reports structure and coarse file policy before Markdown
   internals. Markdown findings then map into stable Assura rule IDs with
   rule-owned severity, suppression, and fix policy.
4. `assura fix markdown --dry-run` previews only deterministic safe fixes,
   including whether the selected Rust engine or an Assura-owned focused fixer
   is responsible. `--apply` must be idempotent and preserve frontmatter, line
   endings, and prose semantics.
5. JSON, YAML, text, agent, daemon, editor, and later content-graph surfaces
   reuse the same finding IDs and severity/suppression model so the user does
   not see competing truths.
6. Benchmark evidence separates cold CLI startup, config loading, file walking,
   candidate engine lint/fix time, reporting overhead, warm daemon behavior,
   and comparison rows for current Assura, selected Rust candidates, and
   `markdownlint-cli2`.

For this goal, the minimum useful increment is therefore not "run a linter on
Markdown." It is executable evidence that the selected or rejected Rust engine
can fit this staged Assura workflow without breaking the final parent
verification package.

## Acceptance Criteria

- [ ] Candidate research records version, license, MSRV, API shape, binary size,
      dependency impact, rule coverage, fix coverage, and performance evidence.
- [ ] A concrete verification fixture or documented fixture plan proves the
      maintainer workflow from staged `assura check` output through safe-fix
      preview/apply, CLI/daemon/editor/agent consistency, and benchmark
      evidence.
- [ ] Markdownlint-compatible fixtures cover accepted rule/config behavior and
      expected diagnostic mapping.
- [ ] Severity mapping is rule-owned under stable rule IDs and compatibility
      mapping is documented for markdownlint rule names/config keys.
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
cargo check -p xtask --quiet
cargo test -p assura --lib rumdl_adapter --quiet
cargo test --test markdown_engine_candidate_fixture_tests --quiet
cargo test --test markdown_rumdl_adapter_tests --quiet
cargo test --test markdown_link_reference_tests --quiet
cargo test --test markdown_suppression_severity_tests --quiet
cargo test --test markdown_required_section_fix_tests --quiet
cargo test --test markdown_lint_fix_tests --quiet
PATH="$PWD/target/markdown-engine-tools/bin:$PATH" cargo xtask markdown-engine-probe --run-external
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
