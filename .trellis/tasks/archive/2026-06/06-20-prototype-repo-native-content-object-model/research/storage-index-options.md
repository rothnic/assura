# Storage And Index Options

## Option A: File-Native Canonical State Plus Custom In-Memory Index

Load repository files through adapters, normalize into `RepoObject` values, and
build an in-memory graph/index for validation and writes.

Pros:
* Minimal dependencies.
* Best Git diff and source-preservation story.
* Good first prototype baseline.

Cons:
* Query support starts basic.
* Longer-term incremental cache and concurrent writes need more design.

## Option B: SQLite Derived Index

Keep files canonical, store normalized objects and edges in SQLite for query,
joins, uniqueness checks, and UI/API use.

Pros:
* Mature, fast, embeddable, excellent query surface.
* Good fit for derived indexes and relationship checks.

Cons:
* Requires schema design and cache invalidation.
* SQL rows do not preserve source formatting; still needs file adapters.

## Option C: Deeb Backend

Deeb is a Rust embedded JSON-backed database. Its docs describe type-safe Rust
collections, programmatic entities, CRUD, transactions, and associations.

Pros:
* Rust-native and no server.
* JSON-backed storage is conceptually close to local file data.
* Associations and transactions are relevant to multi-object changes.

Cons:
* Current fit appears strongest for JSON DB documents, not arbitrary repo files
  with format-specific round-tripping.
* Assura would still own adapters, schema, graph semantics, and source-preserve
  writes.
* Adoption/maturity should be proven before making it core.

Sources:
* https://www.deebkit.com/docs/quickstart
* https://www.deebkit.com/docs/collection-trait
* https://www.deebkit.com/docs/associations-and-enrichments
* https://www.deebkit.com/docs/transactions
* https://docs.rs/deeb/latest/deeb/

## Option D: SurrealDB Embedded

SurrealDB can run embedded in Rust and has document/graph/multi-model features.

Pros:
* Strong graph/document query model.
* Embedded mode avoids a required server.

Cons:
* Much heavier than needed for validating repo files.
* More operational and dependency surface than SQLite or in-memory indexes.
* Still does not replace source adapters.

Sources:
* https://surrealdb.com/docs/build/embedding/by-language/rust
* https://surrealdb.com/docs/languages/rust/overview

## Prototype Recommendation

Compare a custom file-native in-memory index against Deeb and SQLite as
backends. Use the same normalized `RepoObject` / `RepoEdge` model for each so
the comparison measures backend fit rather than changing the product model.
