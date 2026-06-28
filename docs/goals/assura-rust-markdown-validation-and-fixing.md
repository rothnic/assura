---
id: goal-assura-rust-markdown-validation-and-fixing
type: goal
title: Assura Rust Markdown validation and fixing
status: planned
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-06-18-markdown-tooling-evaluation.md
  - docs/goals/assura-content-model-source-of-truth.md
  - docs/goals/assura-goal-11-markdown-outline-validation.md
  - src/cli/check/markdown.rs
  - src/markdown/
---

# Assura Rust Markdown Validation And Fixing

## Objective

Integrate a real Rust-native Markdown linting and fixing path into Assura while
keeping Assura-owned validation for project-aware behavior: path scoping,
modeled frontmatter, nested heading hierarchy, diagnostics, and safe fixes.

## Current Gap

Assura has custom Markdown checks for generic frontmatter presence, heading
depth, required sections, and nested outline validation. Typed frontmatter
fields now belong to content runtime models and collections. Assura does not
yet integrate a maintained Rust Markdown linter/fixer for general Markdown
formatting and lint rules.

## Candidate Direction

Start by verifying current upstream state for:

- `rumdl` for fast markdownlint-style linting and formatting;
- `mkdlint` for Rust library/CLI embeddability and custom-rule potential;
- `comrak` for parser/AST control if linter libraries do not expose enough
  internals.

The implementation should select evidence, not preference. The likely default
is to test `rumdl` first for speed and rule coverage, then compare `mkdlint`
for embeddability and custom-rule integration.

## User Certainty Bar

A user should be able to run Assura and get useful Markdown diagnostics and
safe fixes without installing JavaScript markdownlint. Agents should be able to
apply opinionated fixes that keep Markdown consistent without rewriting typed
frontmatter or body content unsafely.

## Scope

- Benchmark candidate Markdown tools on Assura docs and representative fixture
  repos.
- Compare rule coverage, span quality, autofix quality, configuration
  compatibility, GFM/frontmatter behavior, embeddability, binary impact, and
  performance.
- Choose library, CLI wrapper, fork/reference, or parser-backed Assura-owned
  implementation with evidence.
- Add Markdown lint diagnostics to `assura check` with source spans where
  available.
- Add a safe fix path for supported Markdown formatting issues.
- Keep modeled frontmatter validation in the content runtime, not in generic
  Markdown lint rules.
- Keep nested heading hierarchy validation as Assura-owned behavior.

## Non-Goals

- No JavaScript markdownlint runtime dependency.
- No arbitrary repository-defined command execution.
- No semantic search or graph store implementation.
- No code-symbol intelligence.
- No broad rewrite of Markdown files unless a fix is proven safe and
  deterministic.

## Definition Of Done

- A decision record compares `rumdl`, `mkdlint`, and `comrak` with local
  evidence.
- `assura check` can report generic Markdown lint diagnostics through the
  selected Rust-native path.
- A safe fix command or equivalent operation applies at least one class of
  Markdown formatting fixes deterministically.
- Tests cover lint-only diagnostics, successful fix, no-op fix, frontmatter
  preservation, and interaction with modeled frontmatter.
- Benchmarks record overhead on representative docs.
- Docs explain which Markdown rules Assura owns, which are delegated, and which
  remain unsupported.

## Validation Commands

```bash
cargo fmt --check
cargo test markdown --quiet
cargo test --test content_runtime_validation --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

Add candidate-specific benchmarks during implementation and record their
commands in the decision artifact.

## Review Tasks

- R1: Confirm candidate comparison used current upstream versions and measured
  local evidence.
- R2: Confirm fixes preserve frontmatter, body content outside the fix span, and
  line endings where required.
- R3: Confirm generic Markdown linting does not duplicate modeled frontmatter
  validation.
- R4: Confirm diagnostics are understandable enough for an agent to act on.

## Reviewer Blocking Criteria

Block if the implementation hand-rolls broad Markdown lint rules before
evaluating maintained Rust tools, requires JavaScript, rewrites Markdown
unsafely, or reintroduces duplicate typed frontmatter validation.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-28 | Started as the second Project Intelligence Runtime successor goal after the content-model source-of-truth slice. Created Trellis task `.trellis/tasks/06-28-rust-markdown-validation-and-fixing`, refreshed roadmap routing to this task, and corrected stale goal wording so typed frontmatter fields stay model-owned. | `python3 ./.trellis/scripts/workflow_gate.py --platform codex`; `git status --short --branch`; `.trellis/tasks/06-28-rust-markdown-validation-and-fixing/prd.md`; `.trellis/spec/assura/roadmap.md`. |
| 2026-06-28 | Refreshed Markdown tooling evidence for `rumdl`, `mkdlint`, and `comrak`; current releases exceed Assura's Rust 1.70 MSRV as direct dependencies, so the first implementation slice uses an explicit Assura-owned `markdown.lint_trailing_spaces` rule and `assura fix markdown` safe fix for blank-line trailing whitespace only. | `cargo search rumdl --limit 5`; `cargo info rumdl`; `cargo info mkdlint`; `cargo info comrak@0.52.0`; local CLI probes on a frontmatter fixture; `docs/analysis/2026-06-18-markdown-tooling-evaluation.md`; `cargo test --test markdown_lint_fix_tests --quiet`. |
| 2026-06-28 | Added CLI regression coverage proving Markdown lint diagnostics coexist with model-owned frontmatter validation without reviving `markdown_frontmatter_field`; recorded release-mode overhead for the first lint slice on a temporary copy of this repo's docs corpus. | `cargo test --test content_runtime_check_cli --quiet`; `cargo build --release --quiet`; `hyperfine --warmup 5 --runs 30 "target/release/assura check --format json $bench_root/off" "target/release/assura check --format json $bench_root/on"`; `docs/analysis/2026-06-18-markdown-tooling-evaluation.md`. |
