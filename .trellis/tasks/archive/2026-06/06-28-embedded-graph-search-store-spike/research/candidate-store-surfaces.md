# Candidate Store Surfaces

Date: 2026-06-28

## Question

What current upstream surfaces should the embedded graph/search store spike
verify before implementing candidate prototypes?

## Sources

- Grafeo GitHub README: https://github.com/GrafeoDB/grafeo
- Grafeo crates.io package: https://crates.io/crates/grafeo
- grafeo-engine crates.io package: https://crates.io/crates/grafeo-engine
- redb docs.rs: https://docs.rs/redb/latest/redb/
- Tantivy docs.rs: https://docs.rs/tantivy/latest/tantivy/
- rusqlite docs.rs: https://docs.rs/rusqlite/
- SQLite docs: https://sqlite.org/docs.html

## Findings

### Grafeo

Grafeo is a current published candidate, not only an old research note. The
repository describes an embeddable or standalone graph database with in-memory
and persistent storage, ACID transactions, LPG/RDF support, multiple query
languages, BM25 text search, vector search, and hybrid graph/vector queries.
The crates.io package points at current `0.5.x` docs and the companion
`grafeo-engine` crate.

Implication for this spike: Grafeo should be tested first, but not accepted by
README claims. The implementation task must prove that the Rust API can load
Assura `ProjectFact` and `ProjectEdge` data, run the required graph/search
queries locally, and report memory/update behavior on Assura fixtures.

### redb

redb is a current pure-Rust embedded key-value store. Its docs describe ACID
transactions, MVCC readers, crash safety by default, savepoints, and a typed
`BTreeMap`-style API.

Implication for this spike: redb is a good lean durable-fact fallback candidate
when paired with Assura-owned adjacency indexes and Tantivy for text search. It
does not provide graph or text search semantics by itself.

### SQLite / rusqlite

SQLite remains the mature embedded relational option, and rusqlite is the
current ergonomic Rust wrapper. SQLite official docs cover disk-backed and
in-memory databases, atomic commit, memory behavior, and portability.

Implication for this spike: SQLite is a viable fallback if SQL indexes and
join/query ergonomics beat a redb table layout. It adds C dependency concerns
unless bundled carefully, so the comparison should measure binary/build
complexity in addition to runtime behavior.

### Tantivy

Tantivy `0.26.1` is the current Rust full-text search library. Its docs show
schema-driven indexing, explicit writer commits before documents become
searchable, reader/searcher APIs, segment-based storage, and configurable
directories.

Implication for this spike: Tantivy should be treated as the text-search
component for lean fallback, not as the graph/fact store. Benchmarks need to
measure commit/update behavior because incremental fact replacement maps to
delete/reindex or generation-scoped document replacement.

## Recommended Spike Shape

1. Build one fact fixture from the completed `src/intelligence/facts` contract.
2. Implement a read/write candidate for Grafeo if the current Rust API supports
   the required embedded operations without a standalone service.
3. Implement a lean fallback candidate with either:
   - redb for serialized fact/edge tables plus Assura-owned adjacency maps and
     Tantivy text indexes; or
   - rusqlite/SQLite tables plus Tantivy if SQL queries are simpler and the C
     dependency is acceptable for a spike.
4. Use the same query/benchmark harness for both candidates.
5. Record a decision that separates current production readiness from
   experimental promise.

