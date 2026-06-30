---
id: goal-assura-markdown-lint-link-reference-engine
type: goal
title: Assura Markdown lint and repository reference engine
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ../analysis/2026-06-18-markdown-tooling-evaluation.md
  - ./assura-rust-markdown-validation-and-fixing.md
  - ../../.trellis/spec/assura/config-notation.md
  - ../../src/markdown/
  - ../../tests/markdown_lint_fix_tests.rs
  - ../../tests/markdown_outline_config_notation_tests.rs
---

# Assura Markdown Lint And Repository Reference Engine

## Objective

Make Assura's Markdown and repository-reference validation useful as a fast
local documentation quality gate, not only a formatter. It should lint
Markdown, validate configured document shape, discover references from Markdown
and code comments, verify internal file/code/heading targets, and provide
explicit safe fixes where the desired result is deterministic.

## Current Gap

Assura already has a narrow Rust-native Markdown slice:

- modeled frontmatter validation through content runtime models and
  collections;
- generic frontmatter presence validation;
- heading depth, required section, and nested outline validation;
- `markdown.lint_trailing_spaces` and `assura fix markdown` for blank-line
  trailing whitespace;
- Project Intelligence facts that can connect modeled content, diagnostics,
  and some code symbols.

That is not yet the full product the docs workflow needs. Assura does not yet
provide a broad Markdown lint suite, GitHub-renderable internal reference
enforcement, whole-repository reference discovery, broken file/code/heading
target detection, missing-heading insertion, incremental affected-file
feedback, per-rule severity control, or a reasoned suppression path.

This goal is a child of
[Assura Markdown Reference Intelligence Program](./assura-markdown-reference-intelligence-program.md).
Daemon readiness, daemon CLI management, VS Code integration, and agent daemon
awareness are tracked as separate child goals so this validation engine can
land without prematurely claiming editor or daemon support.

## User Certainty Bar

A user should be able to run one local Assura command and learn whether the
Markdown in a repository is usable on GitHub:

- required headings exist where the project says they should exist;
- malformed internal references are caught, including code-spanned or bare
  references such as `path/to/code.py12:34` and `path/to/code.py:12-34` when
  they should be rendered links;
- links to files, headings, and line ranges resolve inside the repository;
- comments, docstrings, and string-like references in source code that point to
  repository docs are checked, so references such as `docs/some-doc-name.md`
  do not silently rot after files move or get deleted;
- warnings versus errors are configurable per rule;
- intentionally unusual references can be suppressed with a reason.

## Scope

- Keep the default implementation hyper fast and Rust-native. Re-evaluate
  maintained tools from
  [Markdown tooling evaluation](../analysis/2026-06-18-markdown-tooling-evaluation.md),
  but do not require JavaScript, network access, or arbitrary project commands
  for the default path.
- Add or integrate a broad Markdown lint layer beyond blank-line trailing
  whitespace, with explicit rule IDs and source spans.
- Extend existing outline support so configured missing headings can be added
  by a safe fix path when the config opts in and insertion is unambiguous.
- Validate internal Markdown links that target repository files:
  - ``[label](relative/path.md)``
  - ``[label](relative/path.md#heading-slug)``
  - ``[label](relative/path.rs#L12)``
  - ``[label](relative/path.rs#L12-L34)``
- Validate GitHub-style heading anchors for Markdown targets and line/range
  anchors for code or text targets where the file can be read locally.
- Detect internal file or code references that are not rendered links when they
  appear in prose or inline code spans and look like repository-relative
  references.
- Discover repository-relative references outside Markdown files, including
  code comments, doc comments, docstrings where the language has a cheap
  scanner, and simple string literals when they look like internal file paths.
- Store discovered references as source-to-target facts with enough detail to
  answer both directions:
  - source file changed: which outbound references need rechecking;
  - target file changed, moved, or deleted: which inbound references are now
    stale.
- Make the same reference graph usable by whole-repository `assura check` and
  by warm session, watch, or future daemon paths that can update only affected
  references after a file event.
- Keep incremental feedback context-efficient for agent workflows: report the
  changed file, the small affected source/target set, rule IDs, target status,
  and suggested next action without dumping unrelated repository context.
- Keep internal references relative so rendered GitHub links work in branches,
  forks, and PR previews.
- Add config controls for per-rule severity, including warning-level findings
  that can be reported without failing the whole check when configured that
  way.
- Add a suppression escape hatch for legitimate exceptions, requiring a rule ID
  and human-readable reason. The default shape should be compact, for example
  `<!-- assura-ignore markdown_link_format: generated fixture text -->`.
- Keep diagnostics structured enough for agents to fix the issue without
  guessing the rule, path, target, configured severity, and suggested fix.

## Non-Goals

- No remote link checking in the default path.
- No hosted service or MCP requirement.
- No automatic broad Markdown rewrite without explicit apply/fix intent.
- No semantic code analysis beyond cheap local reference discovery and
  validation of file, heading, and line targets for this goal.
- No requirement to make `assura watch` release-grade before the one-shot
  reference graph is correct; daemon/session integration may land as a later
  slice over the same graph.
- No replacement of content runtime model validation with Markdown-specific
  frontmatter field checks.

## Definition Of Done

- `assura check` reports Markdown lint, outline, and internal reference
  findings through stable rule IDs and structured output.
- Whole-repository checks discover Markdown-to-code, Markdown-to-doc,
  code-to-doc, and code-to-file references where the syntax is locally
  recognizable.
- The reference graph records inbound and outbound edges so a changed source or
  changed target can produce a bounded recheck set for warm session, watch, or
  daemon workflows.
- `assura fix markdown --dry-run` previews missing-heading and deterministic
  formatting fixes without modifying files.
- `assura fix markdown --apply` applies only proven deterministic fixes and
  preserves frontmatter and unrelated body text.
- Tests cover valid GitHub-renderable links to files, headings, line numbers,
  and line ranges.
- Tests cover broken files, broken heading anchors, broken line/range anchors,
  malformed non-link references, and inline-code references that should be
  rendered links.
- Tests cover stale references from source comments or docstrings to deleted or
  renamed docs files.
- Tests cover incremental affected-reference calculation for source changes
  and target changes.
- Tests cover configured warning versus error behavior.
- Tests cover suppression comments with required reasons and prove unknown or
  reasonless suppressions are rejected or reported.
- Performance evidence proves the common local check remains fast on this
  repository's Markdown corpus.
- Website docs show a minimal CLI flow that catches invalid content,
  incomplete frontmatter, missing headings, malformed internal references, and
  broken code/file links.

## Validation Commands

```bash
cargo fmt --check
cargo test --test markdown_lint_fix_tests --quiet
cargo test --test markdown_outline_config_notation_tests --quiet
cargo test --test markdown_link_reference_tests --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo test markdown --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

Add a release-mode timing command before implementation is called complete and
record the result in the Markdown tooling analysis or a successor evidence
artifact.

## Review Tasks

- R1: Confirm existing Markdown behavior is accurately reused instead of
  reimplemented under a parallel path.
- R2: Confirm internal reference diagnostics enforce GitHub-renderable relative
  links in Markdown and catch stale code/comment references to docs.
- R3: Confirm safe fixes are opt-in, deterministic, and covered by dry-run and
  apply tests.
- R4: Confirm severity configuration and suppression comments are hard to
  misuse and visible in structured output.
- R5: Confirm the fast path stays local, offline, and suitable for pre-commit
  hooks.
- R6: Confirm the reference graph can explain the bounded affected set for a
  changed source file and a changed target file without full-context output.

## Reviewer Blocking Criteria

Block if the implementation requires JavaScript or network access for the
default lint path, silently rewrites ambiguous Markdown, accepts internal
reference formats that do not render on GitHub, lacks line/range/heading target
tests, misses stale code/comment references to repository docs, cannot identify
inbound references for a changed or deleted target, or hides errors through
unreasoned suppressions.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Created after live review clarified that the previous Project Intelligence follow-up was complete, but the Markdown lint/link-reference product requirement remained open and only partially covered by the earlier Markdown validation goal. | [assura-rust-markdown-validation-and-fixing.md](./assura-rust-markdown-validation-and-fixing.md), [2026-06-18-markdown-tooling-evaluation.md](../analysis/2026-06-18-markdown-tooling-evaluation.md), [.trellis/tasks/06-30-markdown-lint-link-reference-engine](../../.trellis/tasks/06-30-markdown-lint-link-reference-engine). |
| 2026-06-30 | Expanded scope from Markdown-only links to a repository reference graph after review clarified that code comments and similar source references to docs can rot too. The goal now requires whole-repo checks plus an incremental affected-reference model for warm session, watch, or future daemon workflows. | User review; `rg -n "daemon|watch|content session" docs src`; `rg -n "code_symbols|references" src tests docs/goals`. |
