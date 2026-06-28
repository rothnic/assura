# Content Runtime Validation Slice

## Goal

Implement increments 0 and 1 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`.

This slice should move the repo-native content object work out of prototype
evidence and into production-oriented runtime code while keeping the runtime
portable, file-backed, and independent of language-specific build systems.

## Scope

- Archive the merged prototype Trellis task and start the implementation branch
  from the reviewed `master` base.
- Add config support for content runtime collections.
- Load checked-in JSON Schema-like runtime artifacts.
- Discover Markdown frontmatter and JSON records from configured collections.
- Validate object shape natively in Rust without shelling out to authoring
  tools, databases, or servers.
- Assign stable object identity from configured ID fields.
- Preserve source path and field metadata for diagnostics.
- Validate cross-collection references.
- Add focused fixtures under `tests/fixtures/content_runtime/`.
- Add `tests/content_runtime_validation.rs`.
- Add focused docs explaining that authoring tools are optional and runtime
  validation is Rust-native.

## Out Of Scope

- CLI report integration beyond library-level validation tests.
- Typed write operations.
- YAML or JSONL adapters.
- LinkML, TypeSpec, CUE, Deeb, SQLite, Node, Python, Go, or server runtime
  dependencies.
- Release-readiness performance claims.

## Acceptance Criteria

- `git diff --stat origin/master...HEAD` shows only intentional increment 0
  and 1 artifacts.
- Runtime config can bind object classes to collection paths, adapters, schema
  artifacts, ID fields, and reference fields.
- A valid fixture with Markdown frontmatter and JSON records validates without
  diagnostics.
- An invalid-shape fixture reports source path and field.
- A missing-reference fixture reports source path, field, object type, and
  referenced object when available.
- `cargo test --test content_runtime_validation --quiet` passes.
- No runtime path shells out to external authoring tools, databases, or a
  server.
- Independent review is completed before opening or marking the PR ready.

## Validation

Minimum commands for this slice:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test content_runtime_validation --quiet
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```
