# Code Symbol Enrichment

## Goal

Execute `docs/goals/assura-code-symbol-enrichment.md` as the eighth successor
in the Project Intelligence Runtime program. The task must make code
intelligence optional while letting modeled content preserve unresolved code
references and expose resolved symbol relationships when local evidence exists.

## Revalidation Result

`valid`: live repo state after the completed local semantic-search successor
already has the first fact-model placeholders: `CodeSymbol`, `SymbolRef`,
`ProjectEdge::SymbolRef`, `FactIngestor::add_symbol_ref`, store indexing for
symbol edges, and tests proving manually added unresolved symbol refs survive.
The goal is not complete because current ingestion does not derive symbol refs
from modeled fields, does not record provider evidence quality, does not import
or extract baseline symbols from repository code, and does not expose
symbol-to-model or model-to-symbol query output.

## Requirements

- Preserve core validation independence: `assura check` must not require CKB,
  LIP, LSP, SCIP, Codanna, CQS, Serena, MCP, or any standalone service.
- Reuse existing `CodeSymbol` and `SymbolRef` facts where they fit; extend the
  fact model only for missing provenance, span, or impact metadata needed by the
  goal.
- Derive unresolved symbol refs from modeled collection fields in a
  deterministic, schema/config-backed way.
- Add at least one local baseline resolver/import path that can resolve symbols
  in fixtures without a remote or daemon dependency.
- Expose query output for model-to-symbol and symbol-to-model relationships.
- Keep provider-backed precision optional and provenance-tagged.

## Acceptance Criteria

- [ ] Code-symbol fact metadata captures symbol identity, source location or
  span when available, provider name, and evidence quality/provenance.
- [ ] Modeled collection instances can produce unresolved symbol refs from
  configured fields without changing validation truth.
- [ ] A fixture repository proves one local baseline path can resolve at least
  one symbol reference to a `CodeSymbol` fact.
- [ ] Missing or unavailable providers leave explicit unresolved refs rather
  than failing normal validation.
- [ ] A CLI/query surface shows symbols referenced by a model instance and
  model instances related to a symbol.
- [ ] Docs explain optional provider boundaries and distinguish unresolved,
  baseline-resolved, and provider-resolved evidence.
- [ ] `cargo fmt --check`, `cargo test code_symbol --quiet`,
  `cargo test project_intelligence --quiet`,
  `cargo run --quiet -- check --format json .`, and `git diff --check` pass or
  have explicit documented blockers.

## Out Of Scope

- Required standalone code-intelligence services.
- Language-semantic correctness guarantees from a rough local baseline.
- Code-modifying safe fixes.
- Mandatory CKB, LIP, LSP, SCIP, MCP, Codanna, CQS, or Serena integration.
- Replacing the content runtime model with a code-provider model.

## Technical Notes

- Active goal: `docs/goals/assura-code-symbol-enrichment.md`.
- Master program:
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- Completed dependencies:
  `docs/goals/assura-project-intelligence-fact-model.md`,
  `docs/goals/assura-content-query-and-search-cli.md`, and
  `docs/goals/assura-local-semantic-search.md`.
- Existing fact surface: `src/intelligence/facts/types.rs`,
  `src/intelligence/facts/ingest.rs`, and `src/intelligence/store.rs`.
- Query CLI surface: `src/cli/content_query/`.
- Product docs: `website/src/content/docs/product/code-intelligence.md` and
  `website/src/content/docs/product/query-search.md`.

## Review Tasks

- R1: Confirm code intelligence remains optional and normal validation does not
  require external providers.
- R2: Confirm unresolved references remain queryable and do not disappear when
  no provider resolves them.
- R3: Confirm provider/baseline evidence is recorded with resolved facts.
- R4: Confirm query output covers model-to-symbol and symbol-to-model flows.
- R5: Confirm no daemon, LSP, MCP, or broad code-planner scope lands in this
  successor.

## Progress Evidence

- 2026-06-28: Revalidated as `valid` after completing and archiving local
  semantic search. Existing code already defines `CodeSymbol`, `SymbolRef`,
  manual `FactIngestor::add_symbol_ref`, store symbol-edge indexing, and
  unresolved-symbol tests, but lacks modeled-field extraction, provider
  evidence, local baseline resolution/import, and public code-symbol queries.
