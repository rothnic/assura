# Content Query And Search CLI

## Goal

Execute `docs/goals/assura-content-query-and-search-cli.md` as the sixth
successor in the Project Intelligence Runtime program. The task must expose
deterministic, local CLI query/search commands backed by the project
intelligence fact model and the Assura-owned in-memory fact store.

## Revalidation Result

`valid`: the completed fact-model and store-spike successors provide normalized
facts, generation replacement, graph traversal indexes, path-scope lookup,
keyword search, and benchmark evidence, but there is still no public CLI command
that lets users or agents query modeled instances, relations, diagnostics, or
search chunks. Existing CLI surfaces cover validation, status, migration,
Markdown fixes, quality plans, and performance reports, not project
intelligence queries.

## Requirements

- Define the smallest stable public command surface for content/project
  intelligence queries without adding an arbitrary query language.
- Build facts through the existing content runtime and `FactIngestor` path; do
  not rescan repository files ad hoc inside query commands.
- Use `InMemoryFactStore` as the local execution surface for first-slice query
  behavior.
- Support JSON output first for agent stability and concise text output for
  humans.
- Cover:
  - collection listing;
  - collection instance listing;
  - showing one instance with path, type, outbound/inbound relations,
    diagnostics, and related sections where available;
  - missing relation targets;
  - keyword search over model instances, Markdown sections, diagnostics, and
    search chunks;
  - bounded deterministic graph expansion.
- Keep semantic/vector search and code-provider enrichment out of this slice.

## Acceptance Criteria

- [ ] A CLI command can list modeled collections and collection instances from
  the fact model.
- [ ] A CLI command can show one instance with deterministic JSON containing
  source path, model type, outgoing relations, incoming relations, diagnostics,
  and related sections where present.
- [ ] A relation query can report unresolved or missing relationship targets
  from `InMemoryFactStore::missing_relationship_targets()`.
- [ ] Keyword search returns deterministic matches with enough context for an
  agent to decide which fact or resource to inspect next.
- [ ] Graph expansion is bounded, deterministic, and covered by structural JSON
  tests.
- [ ] Text output remains concise and does not become the only stable agent
  contract.
- [ ] `cargo fmt --check`, `cargo test content_query --quiet`, a focused CLI
  command fixture test, `cargo run --quiet -- check --format json .`, and
  `git diff --check` pass or have explicit documented blockers.

## Out Of Scope

- Vector embeddings or semantic ranking.
- LSP, MCP, daemon, or long-running watch/query APIs.
- Code-symbol providers beyond facts already represented by the fact model.
- Durable external graph/search database integration.
- A general-purpose query language.

## Technical Notes

- Active goal: `docs/goals/assura-content-query-and-search-cli.md`.
- Master program:
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- Completed dependency:
  `docs/goals/assura-embedded-graph-search-store-spike.md`.
- Fact model: `src/intelligence/facts/`.
- First-slice store: `src/intelligence/store.rs`.
- CLI entrypoints: `src/cli/args.rs`, `src/cli/commands.rs`, and
  `src/cli/full_entry.rs`.
- Current validation command implementation: `src/cli/check.rs` and
  `src/cli/commands.rs`.

## Review Tasks

- R1: Confirm query commands use `FactIngestor` and `InMemoryFactStore` rather
  than ad hoc file scans.
- R2: Confirm JSON output is stable, deterministic, and structurally tested.
- R3: Confirm graph expansion has explicit bounds and deterministic ordering.
- R4: Confirm keyword search does not claim semantic meaning.
- R5: Confirm semantic search, vector search, and provider-backed code
  intelligence remain out of scope.
