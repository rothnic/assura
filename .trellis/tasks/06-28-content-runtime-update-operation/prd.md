# Content Runtime Update Operation

## Goal

Implement increment 4 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`.

This slice adds the first bounded update operation for existing repo-native
content records. The operation should preserve file-backed authoring ergonomics
while proving update writes are safe enough for agents to use.

## Scope

- Add one public content runtime update operation, such as `update_record`.
- Accept a typed operation payload that identifies the collection, object ID,
  target path or resolved existing record, and field-level data changes.
- Validate the updated object shape with the same runtime schema validators
  used by read/check/create validation.
- Validate affected outgoing references before writing.
- Reject updates that change object identity or target a non-existing record.
- Preserve Markdown body bytes for frontmatter-only updates.
- Write updates atomically so failed writes do not leave partial record
  contents.
- Add dry-run output that describes the target path and deterministic proposed
  record bytes or patch-equivalent summary without mutating the tree.
- Add `tests/content_runtime_update.rs`.
- Cover successful update, dry-run unchanged tree, invalid shape, invalid
  reference, missing record, identity-change rejection, atomic write behavior,
  and Markdown body byte preservation.

## Out Of Scope

- YAML and JSONL storage adapters.
- Broad CLI mutation UX beyond the public operation contract.
- Batch operations.
- LinkML, TypeSpec, CUE, Deeb, SQLite, Node, Python, Go, or server runtime
  dependencies.
- Preserving comments or formatting for JSON records beyond deterministic
  serialization.

## Acceptance Criteria

- A valid update rewrites only the targeted record and the repository validates
  afterward.
- Markdown frontmatter updates preserve the original Markdown body bytes
  exactly.
- Dry-run output is deterministic and leaves the tree unchanged.
- Invalid shape and missing-reference updates fail before filesystem mutation.
- Missing records and identity-changing updates fail before filesystem
  mutation.
- Atomic write tests prove a failed write cannot leave partial target content.
- The operation uses Rust-native runtime schema validation and does not shell
  out to authoring tools, databases, or a server.
- Independent review confirms the slice stays limited to update/write-safety
  semantics and does not introduce YAML/JSONL or broader CLI mutation UX early.

## Validation

Minimum commands for this slice:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test content_runtime_update --quiet
cargo test --test content_runtime_create --quiet
cargo test --test content_runtime_validation --quiet
cargo test --test content_runtime_check_cli --quiet
cargo run --quiet -- check --format json .
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask evidence
cargo xtask docs
git diff --check
```
