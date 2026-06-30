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

- [x] A CLI command can list modeled collections and collection instances from
  the fact model.
- [x] A CLI command can show one instance with deterministic JSON containing
  source path, model type, outgoing relations, incoming relations, diagnostics,
  and related sections where present.
- [x] A relation query can report unresolved or missing relationship targets
  from `InMemoryFactStore::missing_relationship_targets()`.
- [x] Keyword search returns deterministic matches with enough context for an
  agent to decide which fact or resource to inspect next.
- [x] Graph expansion is bounded, deterministic, and covered by structural JSON
  tests.
- [x] Text output remains concise and does not become the only stable agent
  contract.
- [x] `cargo fmt --check`, `cargo test content_query --quiet`, a focused CLI
  command fixture test, `cargo run --quiet -- check --format json .`, and
  `git diff --check` pass or have explicit documented blockers.

## Completion Evidence

- Implemented `assura content collections|instances|show|search|missing-relations|expand`.
- Query execution uses `RepositoryModel`, `ContentRepository`, `FactIngestor`,
  and `InMemoryFactStore`; no query command performs ad hoc repository file
  scans.
- Structural CLI tests in `tests/content_query_cli.rs` cover JSON outputs for
  collection/instance listing, instance show, graph expansion, keyword search,
  missing relations, and diagnostic search.
- Website examples in `website/src/content/docs/product/query-search.md` target
  the checked content runtime fixtures after independent review caught stale
  root-path examples.
- Public surface policy now classifies `assura content` subcommands in
  `.assura/command-surface.yml`, `.assura/config.yml`,
  `docs/compatibility-and-surface.md`, `docs/support-policy.md`, and
  `docs/release-notes.md`.
- Review agent `019f10a5-8ca4-7a03-8cfb-55f8b10ca563` found no remaining
  blockers after the docs example fix.

Validation passed on 2026-06-28:

```bash
cargo fmt --check
cargo test --test content_query_cli --quiet
cargo test content_query --quiet
cargo test project_intelligence --quiet
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

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
