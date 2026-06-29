# Project Intelligence Runtime Completion Audit

## Goal

Audit `docs/goals/assura-project-intelligence-runtime-program.md` against live
repo evidence after completing the ninth successor goal. The task decides
whether the master Project Intelligence Runtime program can be marked complete,
or records exact remaining blockers.

## Requirements

- Check every successor in the master program execution sequence.
- Verify each successor goal has current completion evidence, status metadata,
  and archived or otherwise resolved Trellis task state.
- Verify every Program Definition Of Done item in the master goal against
  current code, docs, command output, tests, and support matrices.
- Fix documentation or metadata drift discovered during the audit when the fix
  is mechanical and evidence-backed.
- Do not mark the master goal complete unless all requirements are proven by
  current evidence.

## Acceptance Criteria

- [x] Audit table maps each successor to status, evidence, validation, review,
  and task/archive state.
- [x] Program Definition Of Done table maps each item to current proof or an
  explicit blocker.
- [x] Any stale successor metadata found during audit is fixed or recorded as a
  blocker with exact file paths.
- [x] Required final validation commands pass or exact blockers are recorded.
- [x] Master goal status is changed only if the audit proves completion.

## Validation Commands

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm the audit does not redefine the master program scope around the
  work just completed.
- R2: Confirm each successor completion claim cites live repo evidence.
- R3: Confirm every Program Definition Of Done item has direct proof, not only
  indirect test coverage.
- R4: Confirm any unresolved blocker keeps the master program open.

## Progress Evidence

- 2026-06-29: Started after completing and archiving the Project Intelligence
  Agent Surfaces successor. Initial live scan found at least one likely
  metadata drift case: an earlier successor is described as completed in the
  roadmap but still has `status: planned` in its goal frontmatter. The audit
  will verify all successor goal metadata before making any master completion
  claim.
- 2026-06-29: Fixed the metadata drift in
  `docs/goals/assura-content-model-source-of-truth.md`, changing its
  frontmatter status from `planned` to `completed` based on its progress log
  and archived Trellis task evidence.
- 2026-06-29: Marked the master runtime program complete after final validation
  passed: `cargo test --workspace --all-targets --all-features --quiet`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, `git diff --check`,
  `cargo run --quiet -- check --format json .`,
  `cargo run --quiet -- check --format agent --agent codex .`,
  `cargo xtask docs`, and `cargo xtask evidence`.

## Successor Audit

| # | Successor | Status | Evidence |
| --- | --- | --- | --- |
| 1 | `assura-content-model-source-of-truth.md` | Completed | Goal progress log records implementation, independent review `019f0f27-9499-7d62-9da7-d429ebd24bbf`, broad validation, and archived task `.trellis/tasks/archive/2026-06/06-28-06-28-content-model-source-of-truth`. |
| 2 | `assura-rust-markdown-validation-and-fixing.md` | Completed | Goal status is completed; progress log records no-blocker review `019f0f80-4f70-7350-9756-83152ac99fb8`, Markdown lint/fix validation, Clippy, docs, evidence, and archived task `.trellis/tasks/archive/2026-06/06-28-rust-markdown-validation-and-fixing`. |
| 3 | `assura-documentation-ia-project-intelligence.md` | Completed | Goal status is completed; progress log records product-layer docs, no-blocker review `019f0f8b-9ccf-7fc3-941a-0693f7d05a23`, docs/evidence validation, and archived task `.trellis/tasks/archive/2026-06/06-28-documentation-ia-project-intelligence`. |
| 4 | `assura-project-intelligence-fact-model.md` | Completed | Goal status is completed; progress log records normalized facts, deterministic IDs, diagnostics, safe fixes, relation edges, review agents `019f0fb7-1933-7c20-b422-23899f9a1566` and `019f0ff1-b758-70c2-b8d6-cf2ea5b4db2f`, validation, and archived task `.trellis/tasks/archive/2026-06/06-28-project-intelligence-fact-model`. |
| 5 | `assura-embedded-graph-search-store-spike.md` | Completed | Goal status is completed; progress log records benchmarked in-memory fallback, external backend MSRV deferral, no-blocker review `019f1056-1fa3-75c2-8710-46b80e7a955f`, project-intelligence benchmark validation, and archived task `.trellis/tasks/archive/2026-06/06-28-embedded-graph-search-store-spike`. |
| 6 | `assura-content-query-and-search-cli.md` | Completed | Goal status is completed; progress log records content query/search commands, diagnostic search chunks, command-surface/support registration, independent review `019f10a5-8ca4-7a03-8cfb-55f8b10ca563`, validation, and archived task `.trellis/tasks/archive/2026-06/06-28-06-28-content-query-and-search-cli`. |
| 7 | `assura-local-semantic-search.md` | Completed | Goal status is completed; progress log records optional local embeddings, semantic CLI behind `--enable-local`, stale-record fixes from review, validation, and archived task `.trellis/tasks/archive/2026-06/06-28-local-semantic-search`. |
| 8 | `assura-code-symbol-enrichment.md` | Completed | Goal status is completed; progress log records modeled code-symbol fields, provider evidence, local Rust baseline, resolved/unresolved refs, review blockers fixed, validation, and archived task `.trellis/tasks/archive/2026-06/06-28-06-28-code-symbol-enrichment`. |
| 9 | `assura-project-intelligence-agent-surfaces.md` | Completed | Goal status is completed; progress log records agent context/query schemas, safe-fix dry-run schema, review agent `019f11ef-0d70-7a52-816d-e08a5a59c336`, docs-status blocker fixed, validation, and archived task `.trellis/tasks/archive/2026-06/06-28-06-29-project-intelligence-agent-surfaces`. |

## Program Definition Of Done Audit

| Requirement | Proof |
| --- | --- |
| Exactly one supported typed frontmatter model path | `markdown.required_fields` is rejected with model guidance in `tests/markdown_config_deprecation_tests.rs`; typed missing Markdown fields are reported by content runtime model validation in `tests/content_runtime_validation.rs` and `tests/content_runtime_check_cli.rs`; configuration docs direct typed fields to `models`, `collections`, and `relations`. |
| Rust-native Markdown lint/fix behavior is benchmarked and documented | `tests/markdown_lint_fix_tests.rs` covers diagnostics, dry-run, and write behavior; `assura fix markdown --dry-run --format json` emits `assura.safe-fix.markdown.v1`; benchmark/tooling evidence is recorded in `docs/analysis/2026-06-18-markdown-tooling-evaluation.md`; docs and support matrices classify the safe-fix surface. |
| Heading hierarchy remains Assura-owned with required/optional nested headings | `tests/markdown_outline_config_notation_tests.rs` covers required, optional, escaped, skipped, and ambiguous outline behavior; configuration docs document `markdown.outline` and optional headings. |
| Modeled collection instances become graph facts with stable IDs | `tests/project_intelligence_fact_model_tests.rs` covers model/resource/Markdown/instance/diagnostic/safe-fix/relation facts, deterministic IDs, and generation replacement. |
| Graph/search storage approach is benchmark-justified | `docs/analysis/2026-06-28-project-intelligence-store-spike.md`, `benches/project_intelligence.rs`, and `tests/project_intelligence_store_spike_tests.rs` justify the in-memory fallback and defer external backends due MSRV/runtime fit. |
| Query commands answer missing-target, relation, path-scope, diagnostic, keyword, and graph-expansion questions | `tests/content_query_cli.rs` and live probes cover `collections`, `instances`, `show`, `search`, `missing-relations`, and `expand`, including diagnostic search matches. |
| Semantic search is optional candidate context only | `tests/semantic_search_tests.rs` and `tests/content_query_cli.rs` cover disabled-by-default behavior, `--enable-local`, positive-score candidate output, graph expansion, and stale same-generation filtering. Docs state scores do not decide validation correctness. |
| Code intelligence is optional and provider-based | `tests/code_symbol_tests.rs` and `tests/content_query_cli.rs` cover provider evidence, `rust-token-baseline-v1`, resolved and unresolved symbol refs, and query output. Docs state Assura works without external code-intelligence services. |
| Docs teach the layered path from structure validation through project intelligence | Website product pages cover structure validation, Markdown validation, content models, query/search, code intelligence, and agent/editor surfaces; `cargo xtask docs` builds the site. |
| Each goal has independent review evidence and clear validation commands | Every successor goal progress log records review evidence and validation commands; archived Trellis tasks exist for all nine successors. |
