# Prototype Baseline Results

## Implemented Baseline

The first prototype implements the recommended file-native path in a test-only
`src/content_repository/` module behind `#[cfg(all(test, feature =
"full-cli"))]`. It keeps repository files canonical and builds a normalized
in-memory object graph for validation.

Implemented layers:

* Model layer: `RepositoryModel`, `CollectionSpec`, `FieldSpec`,
  `ReferenceSpec`, and `PlacementRule`.
* Adapter layer: Markdown plus YAML frontmatter and one-JSON-object-per-file.
* Repository graph layer: `RepoObject`, `RepoEdge`, and `ObjectKey`.
* Validation layer: directory/type placement, required field/type checks,
  duplicate IDs, parse errors, and missing references.
* Write path: `update_field` writes through the adapter, reparses the object,
  rejects object ID changes, validates placement and fields, rebuilds candidate
  edges, validates references, and then replaces the original file through a
  temporary sibling path.

## Executable Proof

Focused tests cover:

* loading Markdown-frontmatter and JSON record collections from a fixture repo;
* validating allowed object placement by directory scope;
* validating fields for both frontmatter and JSON data;
* resolving and reporting cross-collection references;
* updating Markdown frontmatter while preserving the body;
* updating JSON records and validating the result;
* rejecting writes that would break references;
* rejecting writes that would change an object's identity.

Command:

```bash
cargo test -p assura --lib content_repository --quiet
```

Result:

```text
7 passed; 0 failed
```

## Comparison

### File-native Assura glue

This is the best first layer to carry forward. The prototype proved that
Assura-owned model/adapters/reference validation are the product core. It also
keeps Git diffs and human editing central.

### Deeb backend

Deeb remains worth a later backend spike for JSON-native record CRUD,
transactions, and associations. It should not define the public model because
the hard parts are repo file adapters, path placement, source-preserving writes,
and Assura-specific relation semantics.

### SQLite derived index

SQLite remains the strongest general-purpose index/query candidate after the
file-native object model stabilizes. It is better suited to derived object and
edge tables than canonical content storage.

### LinkML / JSON Schema

LinkML and JSON Schema are still promising for schema interchange and generated
tooling, but neither replaces Assura's placement rules, adapters, or graph
semantics. The prototype model should be the source shape used to decide what
can be emitted to JSON Schema later.

### Keystatic / Astro-style collections

Keystatic and Astro remain the best UX references for content collections. The
prototype confirms Assura needs the same mental model, extended with writes,
multi-format adapters, and repo-wide validation.

## Recommendation

Continue with Assura-owned file-native content collections as the core. Treat
Deeb and SQLite as backend/index experiments behind the normalized object graph,
and treat LinkML/JSON Schema as export/intermediate formats rather than the
initial public authoring surface.

## Review Follow-Up

Independent review found that the first write path only revalidated the changed
object's fields. The prototype now validates the candidate graph before writing
and includes regressions for broken references and object ID changes.

Remaining known limitations:

* Markdown body bytes are preserved, but frontmatter is rewritten through
  `serde_yaml::to_string`, so comments and stylistic key formatting are not
  preserved.
* Placement validation applies to files matched by collection globs. Broader
  "unknown object file" detection should be designed with Assura's existing
  closed-world structure checks rather than added ad hoc in this prototype.
* The module is test-only. A later task should decide whether the runtime
  surface is a CLI command, internal API, generated API, or separate
  experimental feature.
