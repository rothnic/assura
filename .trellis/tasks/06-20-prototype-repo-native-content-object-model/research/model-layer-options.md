# Model Layer Options

## Option A: Assura DSL Compiled To IR And JSON Schema

Assura defines a compact DSL for object types, path scopes, fields, and
references. The DSL compiles to an internal Rust IR and JSON Schema artifacts.

Pros:
* Best fit for current structure notation and user-facing ergonomics.
* Keeps runtime Rust-native.
* Can emit JSON Schema for editors, validation libraries, and TS/Python clients.

Cons:
* Requires custom parser/model/compiler work.
* Risk of inventing too much before proving the collection model.

## Option B: LinkML As Source Model

LinkML is a language for schemas and data dictionaries. It supports classes,
slots, enums, inheritance, constraints, and generation into JSON Schema and
other artifacts. LinkML docs explicitly describe compiling to JSON Schema and
validating with JSON Schema validators.

Pros:
* Existing semantic modeling language with cross-language generators.
* Better than TypeScript-only modeling for Python/TS/Rust audiences.
* Good fit for references and rich object models.

Cons:
* Adds Python-oriented tooling to the authoring/build process.
* Likely too verbose or unfamiliar as the everyday Assura user notation.
* Still does not solve file adapters or source-preserving writes.

Sources:
* https://linkml.io/
* https://linkml.io/linkml/generators/json-schema.html
* https://linkml.io/linkml/faq/general.html

## Option C: JSON Schema First

Assura accepts or generates JSON Schema for object data validation, then adds
Assura-specific path scopes, relations, and format adapters around it.

Pros:
* Language-neutral and Rust validation libraries exist.
* Good editor ecosystem.
* Works for JSON, YAML-as-JSON, and frontmatter after normalization.

Cons:
* JSON Schema is awkward as the primary authoring surface for relationships,
  path placement, Markdown body constraints, and readable examples.
* It validates values, not the repository object graph by itself.

## Prototype Recommendation

Prototype the public shape as a small Assura YAML/DSL sketch, compile manually
to a minimal IR, and compare whether LinkML or JSON Schema can be an
intermediate artifact. Do not start by implementing a full custom DSL parser.
