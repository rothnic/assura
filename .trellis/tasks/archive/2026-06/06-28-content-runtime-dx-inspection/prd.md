# Content Runtime DX And Cross-Language Inspection

## Objective

Implement increment 8 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: provide
inspectable schema artifacts and examples for TypeScript, Python, and Rust
users.

## Scope

- Show the same small project/task content model in Markdown frontmatter, JSON,
  and checked runtime schema form.
- Document how TypeScript, Python, and Rust users can inspect the model shape
  and references without reading Assura Rust internals.
- Include JSON Schema-aware editor or tooling hints where they are already
  practical with checked artifacts.
- Include commands for validating the example repo or fixture with existing
  Assura commands.
- Link back to the increment 7 authoring decision and keep LinkML/TypeSpec/CUE
  out of the normal validation hot path.

## Out Of Scope

- Implementing the production content-model compiler command.
- Adding new runtime adapters, write operations, cache/index layers, or Deeb
  decisions.
- Requiring Node, Python, LinkML, TypeSpec, CUE, or Rust build steps for normal
  `assura check`.

## Acceptance Criteria

- Docs/examples show the same object shape in Markdown frontmatter, JSON, and
  generated or checked runtime schema form.
- A TypeScript, Python, or Rust user can understand allowed fields and
  references without reading Rust code or installing authoring tools.
- Validation commands use existing public Assura command surfaces only.
- Existing content runtime and artifact authoring tests still pass.
- Independent review finds no blocker against the goal reviewer criteria.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- `cargo test --test artifact_authoring_paths_proof --quiet`
- `cargo test --test content_runtime_validation --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `cargo xtask evidence`
- `cargo xtask docs`
- `git diff --check`
