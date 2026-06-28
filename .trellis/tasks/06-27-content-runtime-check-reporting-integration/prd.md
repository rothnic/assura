# Content Runtime Check Reporting Integration

## Goal

Implement increment 2 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`.

This slice wires the repo-native content runtime validator into `assura check`
so configured content-object diagnostics are visible through the normal CLI
reporting surfaces.

## Scope

- Archive the merged increment 0/1 Trellis task.
- Run content runtime validation from the `assura check` path when a project
  declares `models`, `collections`, or `relations`.
- Convert `ContentFinding` values into `StructureViolation` diagnostics.
- Preserve source path, field, object type, and referenced object metadata where
  available.
- Emit content diagnostics through text, JSON, YAML, and agent-format reports.
- Add focused CLI coverage for valid and invalid content-runtime fixtures.

## Out Of Scope

- Typed create/update operations.
- YAML or JSONL content adapters.
- Optional-reference semantics beyond the existing schema-required behavior.
- Authoring toolchain generation from LinkML, TypeSpec, CUE, or other model
  sources.
- Performance/cache hardening beyond avoiding duplicate repository walks where
  the current check pipeline already has the needed project root.

## Acceptance Criteria

- `cargo run --quiet -- check --format json tests/fixtures/content_runtime/missing_reference`
  emits a content diagnostic with source path, field, object type, and
  referenced object context.
- `cargo run --quiet -- check --format agent --agent codex tests/fixtures/content_runtime/missing_reference`
  includes the same content diagnostic in the agent feedback payload.
- Text and YAML formats include content diagnostics without panics or dropped
  context.
- Valid content-runtime fixtures pass through `assura check`.
- Config/model construction errors, such as missing schema artifacts, surface as
  normal check violations.
- Independent review confirms the implementation stays within increment 2 and
  does not introduce write operations or runtime authoring-tool dependencies.

## Validation

Minimum commands for this slice:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test content_runtime_check_cli --quiet
cargo run --quiet -- check --format json tests/fixtures/content_runtime/missing_reference
cargo run --quiet -- check --format agent --agent codex tests/fixtures/content_runtime/missing_reference
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

