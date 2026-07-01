---
id: goal-assura-code-doc-reference-validation
type: goal
title: Assura code and documentation reference validation
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ../project-intelligence-facts.md
---

# Assura Code And Documentation Reference Validation

## Objective

Validate repository-internal references across Markdown link facts, source
comments, docstrings, and simple string-like references so docs and code do not
silently rot when files, headings, or line targets move.

## Current Gap

The Markdown lint/link goal now emits Markdown-source link facts, but does not
own repository-wide inbound reference graph behavior. Beta is not complete
until code-to-doc and doc-to-code references are discoverable, validated, and
useful for affected-file feedback.

## User Certainty Bar

A user should be able to ask Assura why a doc, code file, heading, or line
target is still referenced before moving or deleting it. The answer should be
local, deterministic, and bounded enough for agents and daemon sessions:

- show which files contain recognizable references to the target;
- show the source span and target path or anchor for each edge;
- distinguish Markdown-authored links from lower-confidence comment or string
  references;
- report broken repository-internal references without requiring an LSP or
  remote service;
- keep ambiguous references visible as context instead of silently rewriting
  them.

## Scope

- Consume Markdown link facts emitted by the Markdown lint/link goal.
- Discover cheap repository-relative references from comments, doc comments,
  docstrings, and obvious string literals.
- Validate target files, headings, line numbers, and line ranges.
- Store inbound and outbound reference edges as project facts.
- Explain affected sources when a target is changed, moved, or deleted.
- Explain affected targets when a source reference changes.
- Keep the scanner language-agnostic by default, with optional cheap language
  helpers where they improve precision without requiring an LSP.

## Non-Goals

- No full semantic code analysis.
- No remote link checking in the default path.
- No requirement for LSP, SCIP, or hosted indexers.
- No automatic broad rewrite of ambiguous references.

## Definition Of Done

- Whole-repository checks report broken doc-to-doc, doc-to-code, code-to-doc,
  and code-to-file references where syntax is locally recognizable.
- Reference facts include source path/span, target path/anchor, rule ID, and
  target status.
- Changed-source and changed-target affected-set tests pass.
- Agent and daemon callers can request bounded reference context.
- Docs show examples of stale comments or docstrings being caught.

## Validation Commands

```bash
cargo fmt --check
cargo test --test repository_reference_graph_tests --quiet
cargo test --test markdown_link_reference_tests --quiet
cargo test project_intelligence --quiet
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm reference discovery is conservative and explains confidence.
- R2: Confirm GitHub-renderable Markdown links are enforced where applicable.
- R3: Confirm inbound references are available for changed or deleted targets.
- R4: Confirm source scanners do not require language-specific services.

## Reviewer Blocking Criteria

Block if code/comment references can rot silently, inbound edges are missing,
diagnostics lack source/target spans, remote services become required, or
ambiguous references are rewritten without explicit user action.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Revalidated this goal after completing Markdown Quality. The goal remains valid: Markdown-authored links now produce stable `MarkdownLink` facts, while repository-wide inbound edges, code/comment/docstring reference discovery, changed-target affected sets, and bounded reference context remain incomplete. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); [project-intelligence-facts.md](../project-intelligence-facts.md); `git status --short --branch`; `python3 ./.trellis/scripts/workflow_gate.py --platform codex`. |
| 2026-07-01 | Started the first Reference Graph implementation slice by deriving `RepositoryReference` edges from existing `MarkdownLink` facts. The fact store now indexes inbound repository references by target resource, and `content agent-context` ingests Markdown links so the public query surface reports repository-reference counts. Code/comment/docstring discovery remains the next incomplete slice. | `src/intelligence/facts/repository_references.rs`; `src/intelligence/facts/markdown_link_ingest.rs`; `src/intelligence/store.rs`; `src/cli/content_query/context.rs`; `tests/repository_reference_graph_tests.rs`; `tests/content_query_cli.rs`; `docs/project-intelligence-facts.md`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test content_query_cli content_query_reports_generic_agent_context --quiet`. |
| 2026-07-01 | Continued the Reference Graph slice with conservative source/comment/string reference discovery. Graph-oriented content commands now load lower-confidence `RepositoryReference` edges from common bounded source/config scans, unresolved local targets remain graph context instead of public diagnostics, and the same inbound target index covers Markdown and source-derived edges. Broader check-report integration and changed-source/changed-target query commands remain incomplete. | `src/intelligence/facts/repository_reference_ingest.rs`; `src/cli/content_query/context.rs`; `tests/repository_reference_graph_tests.rs`; `tests/content_query_cli.rs`; `docs/project-intelligence-facts.md`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test content_query_cli content_query_reports_generic_agent_context --quiet`; `cargo test --test project_intelligence_fact_model_tests --quiet`; `cargo test --test project_intelligence_store_spike_tests --quiet`. |
| 2026-07-01 | Added the first affected-set query proof for changed-source and changed-target workflows. `assura content references --source <path>` returns outbound repository targets from a changed source, and `--target <path>` returns inbound source references before moving or deleting a target. Independent review Euclid found a text-output context gap; the fix now prints rule, kind, confidence, source position, target anchor/lines, and target status in default output. Full check-report integration remains the main closure gap. | `src/cli/content_query/mod.rs`; `src/cli/args.rs`; `src/intelligence/store.rs`; `tests/repository_reference_graph_tests.rs`; `tests/content_query_cli.rs`; `cargo fmt --check`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test content_query_cli content_query_reports_repository_reference_context --quiet`; `cargo test --test content_query_cli content_query_references_requires_exactly_one_direction --quiet`; `cargo xtask target-state`; `cargo run --quiet -- check --format json .`; `git diff --check`; independent review Euclid. |
