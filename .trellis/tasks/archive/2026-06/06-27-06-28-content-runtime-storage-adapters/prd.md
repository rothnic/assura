# Content Runtime Storage Adapters

## Objective

Implement increment 5 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: add YAML and
JSONL-backed content collections to the repo-native content runtime with
deterministic parse and write behavior.

## Scope

- Add runtime adapter support for YAML records and JSONL record collections.
- Extend content-runtime config parsing/modeling only as needed to declare the
  new adapter kinds.
- Ensure read validation treats YAML and JSONL records as logical objects with
  stable IDs, source metadata, schema validation, and outgoing references.
- Extend typed create/update operations to support YAML and JSONL records.
- Add adapter fixtures under
  `tests/fixtures/content_runtime/adapters/{yaml,jsonl}/`.
- Add tests covering read, create, update, validation failure, malformed input,
  and deterministic output for both adapters.

## Out Of Scope

- Reference graph completeness beyond the existing scalar/list relation model.
- Authoring toolchain generation or LinkML/TypeSpec/CUE decisions.
- Cache/index/Deeb performance experiments.
- User-facing CLI mutation commands beyond the existing public Rust operation
  contracts.

## Acceptance Criteria

- `assura check` validates YAML and JSONL collection fixtures with the same
  diagnostic context as Markdown frontmatter and JSON collections.
- YAML writes are deterministic and atomic; unsupported comment/formatting
  preservation limits are explicit in tests or docs.
- JSONL writes are deterministic and atomic, and update operations target one
  existing logical record without dropping unrelated records.
- Failed validation and malformed input leave the fixture tree unchanged.
- Independent review finds no blocker against the goal reviewer criteria.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- `cargo test --test content_runtime_adapters --quiet`
- `cargo test --test content_runtime_update --quiet`
- `cargo test --test content_runtime_create --quiet`
- `cargo test --test content_runtime_validation --quiet`
- `cargo test --test content_runtime_check_cli --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `cargo xtask evidence`
- `cargo xtask docs`
- `git diff --check`
