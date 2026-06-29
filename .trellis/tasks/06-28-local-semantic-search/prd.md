# Local Semantic Search

## Goal

Execute `docs/goals/assura-local-semantic-search.md` as the seventh successor
in the Project Intelligence Runtime program. The task must add optional local
semantic search over selected project-intelligence facts without changing
deterministic validation truth.

## Revalidation Result

`valid`: the completed fact-model, embedded-store, and content-query CLI
successors provide `SearchChunk`, `EmbeddingRecord`, `InMemoryFactStore`,
keyword search, graph expansion, and public `assura content` query commands.
Live repo state still has no semantic chunk contract, no embedding provider
decision, no vector lookup path, no semantic CLI output, no embedding cache
invalidation, and docs still mark local semantic search as planned.

## Requirements

- Define a first-slice semantic chunk contract over existing facts without
  introducing remote dependencies.
- Preserve deterministic validation: semantic results are candidates only and
  must not affect `assura check`, relation resolution, diagnostics, or safe-fix
  correctness.
- Reuse existing `SearchChunk` and `EmbeddingRecord` fact concepts where they
  fit; extend them only when the stable chunk/hash/provider metadata is
  insufficient.
- Add an opt-in local semantic-search execution path that gracefully reports
  disabled/unavailable state when embeddings are not configured.
- Return candidate source facts with scores, graph-expanded context, and
  deterministic validation state where available.
- Add hash or generation-based invalidation semantics for changed chunks.
- Keep provider-backed code intelligence, daemon, LSP, MCP, and broad natural
  language planning out of this slice.

## Acceptance Criteria

- [x] Semantic chunk IDs, source facts, source locations, text hashes, provider
  metadata, and embedding records are deterministic and structurally tested.
- [x] A local provider/index decision record compares low-resource options by
  size, speed, portability, licensing, and update behavior.
- [x] Semantic search is opt-in; disabled or unavailable semantic search does
  not change `assura check` behavior or keyword search behavior.
- [x] A CLI/query surface returns semantic candidate facts with scores,
  graph-expanded related context, and validation state.
- [x] Changed chunk text is re-embedded or invalidated by stable hash or
  generation evidence.
- [x] Docs explain that semantic results are candidate context, not validation
  truth.
- [x] `cargo fmt --check`, `cargo test semantic_search --quiet`,
  `cargo run --quiet -- check --format json .`, and `git diff --check` pass or
  have explicit documented blockers.

## Out Of Scope

- Remote embedding services for normal operation.
- Semantic validation, semantic safe-fix correctness, or score-based pass/fail
  behavior.
- Provider-specific code intelligence implementation.
- Daemon, LSP, MCP, or long-running editor APIs.
- A general natural-language agent planner.

## Technical Notes

- Active goal: `docs/goals/assura-local-semantic-search.md`.
- Master program:
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- Completed dependency:
  `docs/goals/assura-content-query-and-search-cli.md`.
- Fact model: `src/intelligence/facts/`.
- First-slice store and search indexes: `src/intelligence/store.rs`.
- Query CLI: `src/cli/content_query/`.
- Product docs: `website/src/content/docs/product/query-search.md`.

## Review Tasks

- R1: Confirm semantic search remains local, optional, and disabled by default
  unless explicitly configured.
- R2: Confirm scores and nearest-neighbor results are not treated as
  correctness or validation truth.
- R3: Confirm chunk invalidation uses stable hashes or generations.
- R4: Confirm disabled semantic search leaves `assura check` and keyword
  `assura content search` unaffected.
- R5: Confirm no remote provider, code-symbol provider, daemon, LSP, or MCP
  scope creep lands in this successor.

## Progress Evidence

- 2026-06-28: Added internal semantic foundation:
  `local-hash-embedding-v1`, deterministic text hashes, embedding record
  provider/dimension metadata, explicit embedding ingestion, and store-level
  cosine candidate lookup. This does not change `assura check` or keyword
  search behavior.
- 2026-06-28: Recorded provider baseline and limitations in
  `docs/analysis/2026-06-28-local-semantic-search-baseline.md`.
- 2026-06-28: Added public `assura content semantic-search` with
  disabled-by-default behavior, `--enable-local` local hash embeddings,
  positive-score semantic candidates, related graph context, and diagnostics
  where available. Command-surface, support-policy, release-note, and website
  docs now describe the supported optional semantic-candidate surface.
- 2026-06-28: Tightened invalidation and candidate quality: store lookup skips
  embedding records whose `text_hash` no longer matches the current chunk text
  and drops zero-score candidates. Regression coverage now proves stale
  embeddings are ignored and no-signal queries return no candidates.
- 2026-06-28 validation passed:
  `cargo fmt --check`;
  `cargo test --test semantic_search_tests --quiet`;
  `cargo test semantic_search --quiet`;
  `cargo test project_intelligence --quiet`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo run --quiet -- check --format json .`;
  `git diff --check`.
- 2026-06-28 validation passed after public semantic CLI and review fixes:
  `cargo fmt --check`;
  `cargo test --test semantic_search_tests --quiet`;
  `cargo test --test content_query_cli --quiet`;
  `cargo test semantic_search --quiet`;
  `cargo test project_intelligence --quiet`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo run --quiet -- check --format json .`;
  `cargo xtask evidence`;
  `cargo xtask docs`;
  `git diff --check`.
- 2026-06-28 independent review completed by subagent `Halley`
  (`019f10ff-a3f2-7e62-937a-0ce25b4bcc56`). Findings: cross-generation stale
  semantic lookup could pair an embedding with the wrong same-ID chunk; release
  and compatibility docs overclaimed/blurred the semantic-search surface; CLI
  JSON coverage could be broader. Resolution: semantic lookup and CLI
  enrichment are now generation-aware, release/compatibility wording is
  clarified, and focused tests cover cross-generation embedding records plus
  disabled and no-signal CLI JSON behavior.
- 2026-06-28 post-review validation passed:
  `cargo fmt --check`;
  `cargo test --test semantic_search_tests --quiet`;
  `cargo test --test content_query_cli --quiet`;
  `cargo test semantic_search --quiet`;
  `cargo test project_intelligence --quiet`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo run --quiet -- check --format json .`;
  `cargo xtask evidence`;
  `cargo xtask docs`;
  `git diff --check`.
