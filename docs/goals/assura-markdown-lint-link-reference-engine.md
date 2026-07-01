---
id: goal-assura-markdown-lint-link-reference-engine
type: goal
title: Assura Markdown lint and link reference engine
status: completed
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

# Assura Markdown Lint And Link Reference Engine

## Objective

Make Assura's Markdown validation useful as a fast local documentation quality
gate, not only a formatter. It should lint Markdown, validate configured
document shape, validate Markdown-authored links to local files, headings, and
line anchors, and provide explicit safe fixes where the desired result is
deterministic.

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
provide a broad Markdown lint suite, GitHub-renderable Markdown link
enforcement, broken file/heading/line target detection for Markdown links,
missing-heading insertion, per-rule severity control, or a reasoned suppression
path.

This goal is a child of
[Assura Markdown Reference Intelligence Program](./assura-markdown-reference-intelligence-program.md).
Code/comment reference graph validation, daemon readiness, daemon CLI
management, VS Code integration, and agent daemon awareness are tracked as
separate child goals so this validation engine can land without prematurely
claiming repository-wide reference graph, editor, or daemon support.

## User Certainty Bar

A user should be able to run one local Assura command and learn whether the
Markdown in a repository is usable on GitHub:

- required headings exist where the project says they should exist;
- malformed internal references are caught, including code-spanned or bare
  references such as `path/to/code.py12:34` and `path/to/code.py:12-34` when
  they should be rendered links;
- links to files, headings, and line ranges resolve inside the repository;
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
- Emit Markdown-source link facts with enough source span and target detail for
  the later
  [Code and documentation reference validation](./assura-code-doc-reference-validation.md)
  goal to build inbound and changed-target analysis.
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
- No code/comment/docstring reference discovery in this goal; that belongs to
  [Code and documentation reference validation](./assura-code-doc-reference-validation.md).
- No inbound reference graph, changed-target invalidation, or daemon affected
  set calculation in this goal.
- No requirement to make `assura watch` release-grade before Markdown linting
  and Markdown-authored link validation are correct.
- No replacement of content runtime model validation with Markdown-specific
  frontmatter field checks.

## Definition Of Done

- `assura check` reports Markdown lint, outline, and internal reference
  findings through stable rule IDs and structured output.
- Markdown-authored links to docs, code files, headings, line numbers, and line
  ranges are validated where the target can be read locally.
- Markdown link facts are emitted with source span, target path, target anchor,
  and rule ID so the later reference-graph goal can consume them.
- `assura fix markdown --dry-run` previews missing-heading and deterministic
  formatting fixes without modifying files.
- `assura fix markdown --apply` applies only proven deterministic fixes and
  preserves frontmatter and unrelated body text.
- Tests cover valid GitHub-renderable links to files, headings, line numbers,
  and line ranges.
- Tests cover broken files, broken heading anchors, broken line/range anchors,
  malformed non-link references, and inline-code references that should be
  rendered links.
- Tests cover configured warning versus error behavior.
- Tests cover suppression comments with required reasons and prove unknown or
  reasonless suppressions are rejected or reported.
- Performance evidence proves the common local check remains fast on this
  repository's Markdown corpus.
- Website docs show a minimal CLI flow that catches invalid content,
  incomplete frontmatter, missing headings, malformed Markdown links, and
  broken code/file links authored from Markdown.

## Validation Commands

```bash
cargo fmt --check
cargo test --test markdown_lint_fix_tests --quiet
cargo test --test markdown_outline_config_notation_tests --quiet
cargo test --test markdown_link_reference_tests --quiet
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
- R2: Confirm Markdown link diagnostics enforce GitHub-renderable relative
  links and provide facts for the later repository reference graph.
- R3: Confirm safe fixes are opt-in, deterministic, and covered by dry-run and
  apply tests.
- R4: Confirm severity configuration and suppression comments are hard to
  misuse and visible in structured output.
- R5: Confirm the fast path stays local, offline, and suitable for pre-commit
  hooks.
- R6: Confirm code/comment reference discovery and inbound changed-target
  analysis are left to the separate reference-graph goal.

## Reviewer Blocking Criteria

Block if the implementation requires JavaScript or network access for the
default lint path, silently rewrites ambiguous Markdown, accepts internal
reference formats that do not render on GitHub, lacks Markdown-authored
line/range/heading target tests, fails to emit consumable Markdown link facts
for the later reference graph, or hides errors through unreasoned suppressions.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Created after live review clarified that the previous Project Intelligence follow-up was complete, but the Markdown lint/link-reference product requirement remained open and only partially covered by the earlier Markdown validation goal. | [assura-rust-markdown-validation-and-fixing.md](./assura-rust-markdown-validation-and-fixing.md), [2026-06-18-markdown-tooling-evaluation.md](../analysis/2026-06-18-markdown-tooling-evaluation.md), [.trellis/tasks/06-30-markdown-lint-link-reference-engine](../../.trellis/tasks/06-30-markdown-lint-link-reference-engine). |
| 2026-06-30 | External reviewer found that this goal and the beta Reference Graph epic overlapped too much to execute independently. This goal now owns Markdown linting, Markdown-authored links, heading validation, safe fixes, severity, suppressions, and outbound Markdown link facts only. Code/comment references, inbound edges, and changed-target affected sets moved to the separate reference-graph goal. | Reviewer agent `019f19c4-fd47-7da0-bd51-2119c7a40cfd`; [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md). |
| 2026-06-30 | Implemented the first `markdown.check_links` validation slice. Configured Markdown scopes now validate local relative links to files, Markdown heading anchors, and GitHub-style `#Lx` / `#Lx-Ly` line anchors, while reporting root-absolute internal links as non-renderable for branch/fork-safe GitHub links. Broad markdownlint compatibility, suppressions, missing-heading safe fixes, and outbound link fact ingestion remain open for later slices. | `src/cli/check/markdown/links.rs`; `tests/markdown_link_reference_tests.rs`; `docs/data/release-surfaces.json`; `website/src/content/docs/product/markdown-validation.md`; `cargo test --test markdown_link_reference_tests --quiet`; reviewer Pasteur. |
| 2026-06-30 | Added Markdown per-rule severity overrides and reasoned `assura-ignore` suppressions. Supported Markdown rules can now be downgraded or promoted with `markdown.rules.<rule_id>.severity`, valid suppressions require `<!-- assura-ignore <markdown_rule>: <reason> -->`, and invalid suppressions report `markdown_suppression`. Missing-heading safe fixes, broad markdownlint compatibility, and outbound link fact ingestion remain open. | `src/cli/check/markdown/suppression.rs`; `tests/markdown_suppression_severity_tests.rs`; `crates/assura-check-cli/tests/compiled_markdown_cli.rs`; `docs/data/release-surfaces.json`; `cargo test --test markdown_suppression_severity_tests --quiet`; `cargo test -p assura-check-cli --test compiled_markdown_cli --quiet`; independent review Jason. |
| 2026-07-01 | Added Project Intelligence `MarkdownLink` facts for outbound Markdown-authored local links. The validator, fact ingestion, and docs-lifecycle historical-link checks now share one parser/resolver path, and link facts carry source path/line/column, raw target, normalized target path, optional heading or line anchor details, target existence, and related `markdown_link_*` rule ID for the later reference-graph goal. Broad markdownlint compatibility, malformed non-link reference detection, missing-heading safe fixes, and Markdown performance evidence remain open. | `src/markdown/links.rs`; `src/intelligence/facts/markdown_links.rs`; `src/intelligence/facts/markdown_link_ingest.rs`; `tests/markdown_link_fact_tests.rs`; `docs/project-intelligence-facts.md`; `cargo test --test markdown_link_fact_tests --quiet`; `cargo test --test markdown_link_reference_tests --quiet`; `cargo test --lib docs_lifecycle --quiet`. |
| 2026-07-01 | Added malformed non-link local reference detection under `markdown.check_links`. Existing repository-local file references in prose or inline code now report `markdown_link_format` when they should be rendered as relative Markdown links, while frontmatter, fenced code, rendered links, and images stay ignored. Broad markdownlint compatibility, missing-heading safe fixes, and Markdown performance evidence remain open. | `src/markdown/links.rs`; `src/cli/check/markdown/links.rs`; `tests/markdown_link_reference_tests.rs`; `website/src/content/docs/product/markdown-validation.md`; `website/src/content/docs/reference/configuration.md`; `cargo test --lib bare_references_include_colon_and_adjacent_line_forms --quiet`; `cargo test --test markdown_link_reference_tests --quiet`. |
| 2026-07-01 | Corrected the Markdown severity config shape from a cross-cutting severity map to per-rule objects. User-facing config now uses `markdown.rules.<rule_id>.severity`, which keeps severity modular and leaves room for additional rule options without adding parallel maps; reviewer feedback also tightened check-only compilation, nested-field validation, and inherited rule merging. | `src/config/config/bundles/markdown.rs`; `src/cli/check/rules.rs`; `src/cli/check/markdown.rs`; `src/cli/check/compiled_artifact_bundles.rs`; `tests/markdown_suppression_severity_tests.rs`; `crates/assura-check-cli/tests/compiled_markdown_cli.rs`; `cargo test -p assura-check-cli --test compiled_markdown_cli --quiet`; `website/src/content/docs/product/markdown-validation.md`; `website/src/content/docs/reference/configuration.md`; independent review Plato. |
| 2026-07-01 | Added a deterministic missing-heading safe-fix slice for `markdown.required_sections`. `assura fix markdown --rule required-sections` now previews and applies bounded heading appends for configured Markdown scopes while preserving frontmatter, body text, and CRLF endings; reviewer feedback also added required-section title validation and inserted-text preview fields. Broad markdownlint compatibility and Markdown performance evidence remain open. | `src/cli/check/markdown_fix.rs`; `src/cli/check/markdown_required_sections_fix.rs`; `src/cli/check/markdown_fix_report.rs`; `tests/markdown_required_section_fix_tests.rs`; `website/src/content/docs/product/markdown-validation.md`; `website/src/content/docs/reference/configuration.md`; `cargo test --test markdown_required_section_fix_tests --quiet`; `cargo test --test markdown_lint_fix_tests --quiet`; `cargo test --test markdown_outline_config_notation_tests --quiet`; independent review Descartes. |
| 2026-07-01 | Added an opt-in `markdown.lint_common` bundle for common Rust-native Markdown checks: heading increments, heading marker spacing, duplicate headings, and multiple blank lines. Laplace review caught frontmatter-heading leakage, a missing compiled-artifact schema bump, and command-vs-config support metadata; all three were corrected. Full markdownlint-compatible coverage remains a separate future dependency or external-binary decision. | `src/cli/check/markdown/common_lint.rs`; `src/cli/check/markdown.rs`; `src/cli/check/compiled_artifact.rs`; `tests/markdown_common_lint_tests.rs`; `crates/assura-check-cli/tests/compiled_markdown_cli.rs`; `docs/data/release-surfaces.json`; `docs/analysis/2026-06-18-markdown-tooling-evaluation.md`; `cargo test --test markdown_common_lint_tests --quiet`; `cargo test --test markdown_suppression_severity_tests --quiet`; `cargo test -p assura-check-cli --test compiled_markdown_cli --quiet`; `cargo test -p assura --lib docs_claim_surfaces_reads_config_surface_rows --quiet`; `cargo run --quiet -- check --format json .`; independent review Laplace. |
| 2026-07-01 | Added release-mode common-lint timing evidence for the copied docs corpus. With 175 Markdown files, `markdown.lint_common: true` reported 25 findings and averaged 23.2 ms versus 14.0 ms with common lint disabled, keeping the local docs check in a pre-commit-suitable range while making the remaining broad markdownlint-compatible coverage question explicit. | [2026-06-18-markdown-tooling-evaluation.md](../analysis/2026-06-18-markdown-tooling-evaluation.md); `cargo build --release --quiet`; `hyperfine --warmup 5 --runs 30 --ignore-failure "target/release/assura check --format json $bench_root/off" "target/release/assura check --format json $bench_root/on"`; `jq '{files_checked, dirs_checked, success, violations: (.violations | length)}' /tmp/assura-md-off.json`; `jq '{files_checked, dirs_checked, success, violations: (.violations | length), rules: ([.violations[].rule] | group_by(.) | map({rule: .[0], count: length}))}' /tmp/assura-md-on.json`. |
