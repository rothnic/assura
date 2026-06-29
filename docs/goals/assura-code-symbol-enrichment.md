---
id: goal-assura-code-symbol-enrichment
type: goal
title: Assura code symbol enrichment
status: planned
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-fact-model.md
  - docs/goals/assura-content-query-and-search-cli.md
  - docs/goals/assura-local-semantic-search.md
---

# Assura Code Symbol Enrichment

## Objective

Add optional code-symbol facts and provider interfaces so modeled repository
objects can link to relevant code without making code intelligence mandatory
for core Assura validation.

## Current Gap

Modeled content can reference strings that are intended to represent code, but
Assura does not yet have a normalized code-symbol model, unresolved symbol
edges, native baseline extraction, SCIP import, LSP lookup, or optional
provider boundary for richer tools.

## User Certainty Bar

A user should be able to model a `UseCase`, `Scenario`, `Component`, or
`Requirement` that references a code symbol. If a provider is unavailable,
Assura should preserve the unresolved reference. If a provider is available,
Assura should resolve it and expose affected docs/specs/model instances.

## Scope

- Define `CodeSymbol`, `CodeReference`, `CodeSpan`, `CodeImpact`, `SymbolRef`,
  and `CodeProviderEvidence` facts.
- Add unresolved symbol references from modeled collection fields.
- Add a native baseline candidate using tree-sitter where practical for rough
  declarations, imports, and spans.
- Add optional SCIP import for precise symbols when an index exists.
- Define optional provider contracts for LSP, CKB, LIP, Codanna, CQS, Serena,
  or similar tools without requiring any of them.
- Add graph edges from model instances and Markdown sections to unresolved or
  resolved code symbols.
- Add query support for "what docs/specs are related to this symbol?" and
  "what symbols are referenced by this model instance?"

## Non-Goals

- No required standalone code-intelligence service.
- No deep language-semantic correctness guarantee from tree-sitter alone.
- No code-modifying safe fixes.
- No mandatory CKB, LIP, LSP, SCIP, or MCP integration.
- No replacement for the content runtime model.

## Definition Of Done

- Code-symbol facts are represented in the project intelligence fact model.
- Modeled collection instances can create unresolved symbol edges.
- At least one baseline import or extraction path can resolve symbols in a
  fixture repository.
- Missing or unavailable providers leave explicit unresolved facts rather than
  failing core validation.
- Query output can show model-to-symbol and symbol-to-model relationships.
- Provider contracts document evidence quality and provenance.

## Validation Commands

```bash
cargo fmt --check
cargo test code_symbol --quiet
cargo test project_intelligence --quiet
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R1: Confirm code intelligence is optional.
- R2: Confirm unresolved references remain queryable.
- R3: Confirm provider evidence is recorded with resolved facts.
- R4: Confirm the implementation does not vendor a large provider into core
  without a separate support and dependency decision.

## Reviewer Blocking Criteria

Block if normal validation requires CKB, LIP, LSP, SCIP, or another provider;
if unresolved symbols disappear; if provider results lack provenance; or if
code intelligence starts defining the content model.

## Progress Log

- 2026-06-28: Revalidated as `valid` after completing and archiving local
  semantic search. Live repo already has `CodeSymbol`, `SymbolRef`,
  `ProjectEdge::SymbolRef`, `FactIngestor::add_symbol_ref`, store symbol-edge
  indexing, and tests for manually added unresolved symbol refs. The goal still
  needs modeled-field symbol extraction, provider evidence metadata, at least
  one local baseline resolution/import path, public symbol query output, and
  docs that distinguish unresolved, baseline-resolved, and provider-resolved
  evidence.
