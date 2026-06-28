# Content Runtime Create Operation

## Goal

Implement increment 3 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`.

This slice adds the first typed agent write operation for repo-native content
objects. The operation should validate the requested record, path policy, and
affected references before writing anything to disk.

## Scope

- Add one public content runtime create operation, such as `create_record`.
- Accept a typed operation payload that identifies the collection, object ID,
  destination path, and record data.
- Validate the destination path against configured collection/path placement
  policy.
- Validate the payload shape with the same runtime schema validators used by
  read/check validation.
- Validate outgoing references before writing.
- Reject duplicate object IDs.
- Write only after all validation passes.
- Add `tests/content_runtime_create.rs`.
- Cover successful create, invalid shape, invalid reference, duplicate ID or
  disallowed path, and unchanged tree after failure.

## Out Of Scope

- Updating existing records.
- Dry-run output and patch previews.
- YAML and JSONL storage adapters.
- Markdown body preservation for update operations.
- LinkML, TypeSpec, CUE, Deeb, SQLite, Node, Python, Go, or server runtime
  dependencies.
- CLI command design for broader mutation UX beyond the first public operation
  contract.

## Acceptance Criteria

- A valid create operation writes a new record under the configured collection
  path and the repository validates afterward.
- Invalid shape and missing-reference creates fail before any filesystem write.
- Duplicate IDs fail before any filesystem write.
- A destination outside the configured collection/path policy fails before any
  filesystem write.
- Failure tests compare the tree before and after the operation.
- The operation uses Rust-native runtime schema validation and does not shell
  out to authoring tools, databases, or a server.
- Independent review confirms the slice stays limited to create semantics and
  does not introduce update/dry-run/YAML/JSONL behavior early.

## Validation

Minimum commands for this slice:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test content_runtime_create --quiet
cargo test --test content_runtime_validation --quiet
cargo test --test content_runtime_check_cli --quiet
cargo run --quiet -- check --format json .
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask evidence
cargo xtask docs
git diff --check
```
