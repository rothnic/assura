---
title: Supported Document Graph Closure
status: active
date: 2026-07-02
---

# Supported Document Graph Closure

## Decision

Close the supported document graph child goal for this beta increment. Assura's
supported graph contract is the local, deterministic content/query path:
modeled content validation, collection and instance queries, lexical search,
relation diagnostics, bounded graph expansion, repository-reference queries,
object-mode context packs, and local JSON-line sessions.

Semantic search, code-symbol queries, repository-reference validation checks,
and daemon/editor integrations remain separate surfaces. Semantic and
code-symbol results are candidate enrichment only; they do not decide content
validation truth and are not required for the supported graph workflow.

## Supported Contract

The supported graph is built from checked repository files and local facts:

- content model instances from configured collections;
- typed frontmatter and record validation diagnostics;
- relation edges and missing-relation diagnostics;
- Markdown sections and deterministic lexical search chunks;
- conservative repository-reference facts from Markdown links, comments,
  docstrings, and string-literal path references;
- bounded context packs that include diagnostics, relations, search matches,
  repository-reference context, and safe-fix preview metadata.

The supported user workflow is:

```bash
assura check --format json .
assura content collections . --format json
assura content instances <collection> . --format json
assura content show <collection> <id> . --format json
assura content search "query text" . --format json
assura content missing-relations . --format json
assura content references . --target docs/guide.md --format json
assura content references . --source docs/guide.md --format json
assura content expand <collection> <id> . --format json
assura content context-pack . --collection <collection> --id <id> --text "query text" --format json
```

This workflow does not require a hosted service, daemon process, editor
extension, semantic ranking provider, code-symbol provider, or remote agent
transport.

## Parent Verification Story Fit

The parent program's final user is a maintainer deciding whether a branch with
renamed architecture docs, moved code, Markdown drift, frontmatter issues, and
agent-written references is safe to merge. This graph closure supports the
parts of that decision that ask:

- which goals, ADRs, analysis notes, generated references, source files, tests,
  headings, benchmark rows, and release docs are affected by this rename or
  move;
- which frontmatter records and relations are invalid or missing;
- which repository-reference facts explain inbound and outbound doc/code
  impact;
- what bounded context an agent should read before editing an affected file.

The graph child is therefore complete for this beta increment only as the
local deterministic graph/query layer. Daemon freshness, agent event timing,
VS Code parity, Markdown fix coverage, and LS-Lint fixture speed remain owned
by their separate child goals and must still prove their portions of the
parent verification story.

## Evidence

Current tests and docs prove the contract:

| Requirement | Evidence |
| --- | --- |
| Content validation, frontmatter, IDs, relations, and missing targets | `cargo test content_runtime --quiet`; `tests/content_runtime_validation.rs`; `tests/content_runtime_references.rs` |
| Collection, instance, show, search, missing relation, reference, graph, and agent-query CLI outputs | `cargo test --test content_query_cli --quiet` |
| Repository-reference facts and inbound/outbound graph queries | `cargo test --test repository_reference_graph_tests --quiet` |
| Object-mode context packs with bounded repository-reference context | `cargo test --test project_intelligence_context_pack --quiet` |
| Realistic non-Assura project workflow | `cargo test project_intelligence --quiet`; `tests/project_intelligence_real_repo_proof.rs`; `tests/fixtures/project_intelligence_real_repo/beacon_crm/` |
| Public supported/experimental classification | `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `docs/content-runtime.md`; `website/src/content/docs/product/query-search.md` |
| Drift guard against unsupported graph/search/provider claims | `cargo xtask target-state`; `check_document_graph_support_claims` in `xtask/src/main.rs` |

## Remaining Follow-Up

This closure does not finish the whole parent post-beta program. The next child
goal should move to the performance floor and fixture gate so accepted
LS-Lint-equivalent rows cannot hide slower fixture cases behind aggregate pass
criteria.
