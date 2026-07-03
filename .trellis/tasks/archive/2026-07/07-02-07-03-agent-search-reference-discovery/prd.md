# Agent Search Reference Discovery

## Goal

Implement child goal 5 of the agent-ready onboarding program: make Assura
useful for discovery before a project has perfect content models by adding raw
repository search fallback, frontmatter repository references, direct
all/unresolved reference enumeration, and machine-readable agent-query
capability discovery.

## What I Already Know

- Parent goal: `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- Child goal: `docs/goals/assura-agent-search-reference-discovery.md`.
- Child goals 1-4 are complete locally: `assura agent onboard`, reusable
  dynamic project contracts, doctor/explain feedback, and AGENTS/SKILL guidance
  contracts.
- Existing supported document graph work already exposes modeled collection
  queries, keyword search over fact chunks, repository-reference facts,
  affected-source and affected-target `content references` queries, and context
  packs.
- Existing `assura content search` searches modeled facts; it does not provide
  an explicit raw repository text fallback for unmodeled day-one projects.
- Existing `assura content references` requires exactly one of `--source` or
  `--target`; there is no direct all-reference or unresolved-reference listing
  command even though the graph can contain unresolved references.
- Existing repository-reference ingestion covers Markdown, source comments,
  docstrings, and string literals. Configurable Markdown frontmatter reference
  fields are not yet first-class graph edges.
- Existing `assura content agent-query` exposes deterministic capabilities as
  enum values and documentation, but there is no JSON capability listing or
  unresolved-reference agent-query capability.

## Revalidation Result

`valid`: the child goal is still needed. The current repo passes self-check and
the supported document graph is real, but a new agent-ready project can still
need external grep for raw discovery, cannot enumerate all/unresolved
repository-reference edges directly, cannot turn frontmatter path lists into
reference graph facts, and cannot ask the CLI for the agent-query capabilities
it should use next.

## Requirements

- Add raw repository text search with JSON output and deterministic bounded
  results.
- Make modeled keyword search expose an explicit raw-search fallback mode so
  agents can continue when modeled facts are missing or inactive.
- Add configurable frontmatter repository-reference extraction for fields such
  as `source_documents`, `related`, `evidence`, and `requirements`.
- Include frontmatter references in repository-reference graph output, context
  packs, affected-reference answers, and check diagnostics when configured.
- Add all-reference and unresolved-reference listing modes with JSON output.
- Add agent-oriented reference summaries that include follow-up commands when
  unresolved references exist.
- Add machine-readable agent-query capability listing.
- Add deterministic agent-query capabilities for gaps, next actions, and
  unresolved references without introducing a natural-language parser.
- Keep semantic search and code-symbol provider behavior as optional candidate
  enrichment, not validation truth.

## Acceptance Criteria

- [ ] A fixture with no active modeled content can return raw text search
      matches through Assura.
- [ ] `assura content search` or an adjacent content command exposes an
      explicit raw fallback mode with stable JSON.
- [ ] Configured frontmatter path fields become `RepositoryReference` graph
      edges and are visible in `content references` and object-mode context
      packs where relevant.
- [ ] All-reference and unresolved-reference listings can enumerate the same
      edges counted in summaries.
- [ ] Agent-query capability listing returns deterministic capability names,
      descriptions, required arguments, and suggested follow-up commands.
- [ ] Agent-query unresolved-reference output enumerates unresolved
      repository references directly.
- [ ] Website/API docs show the raw search, reference discovery, and
      agent-query discovery flow.

## Definition Of Done

- `cargo fmt --check`
- `cargo test --test content_query_cli --quiet`
- `cargo test --test repository_reference_graph_tests --quiet`
- `cargo test --test project_intelligence_context_pack --quiet`
- Focused tests for raw search fallback, frontmatter references, all/unresolved
  reference listing, and agent-query capability discovery.
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`
- Independent review confirms agents do not need external grep for day-one
  discovery, frontmatter references are graph-visible, unresolved edges can be
  enumerated directly, and docs do not overclaim semantic search.

## Out Of Scope

- Semantic search correctness claims.
- Natural-language agent-query parsing.
- Hosted indexing, hosted search, or remote providers.
- Full content initialization or content doctor behavior; that belongs to child
  goal 6.
- Lifecycle hook nudge/warn/gate behavior; that belongs to child goal 7.
- Domain-specific presets.

## Technical Approach

Extend the existing content-query and repository-reference fact path instead of
adding a parallel discovery engine. Raw search can be a bounded local fallback,
but frontmatter and repository-reference behavior should flow through
Project Intelligence facts so context packs, `content references`, and
agent-query envelopes share the same evidence.

## Technical Notes

- Existing agent-query envelope: `src/cli/content_query/agent_query.rs`.
- Existing content commands: `src/cli/content_args.rs` and
  `src/cli/content_query/mod.rs`.
- Existing repository-reference ingestion:
  `src/intelligence/facts/repository_reference_ingest.rs`.
- Existing context-pack repository-reference output:
  `src/cli/content_query/context_pack.rs`.
- Existing tests to extend: `tests/content_query_cli.rs`,
  `tests/repository_reference_graph_tests.rs`, and
  `tests/project_intelligence_context_pack.rs`.
