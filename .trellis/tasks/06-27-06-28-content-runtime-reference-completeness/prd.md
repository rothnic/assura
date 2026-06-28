# Content Runtime Reference Graph Completeness

## Objective

Implement increment 6 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: make
repo-native content reference validation complete enough for normal project
modeling, including required, optional, many, and cross-collection references
with clear diagnostics for missing, duplicate, ambiguous, and cyclic links where
relevant.

## Scope

- Extend the content-runtime relation model only as needed to express required
  versus optional references while preserving the existing `collection.field`
  config shape.
- Keep `many: true` reference behavior and add explicit coverage for optional
  empty or absent references.
- Ensure cross-collection references work consistently across Markdown
  frontmatter, JSON, YAML, and JSONL-backed collections.
- Add duplicate-ID and ambiguous-reference diagnostics that identify collection,
  object ID, source path, and field where applicable.
- Add cycle/error reporting where relevant to configured references without
  making every graph cycle invalid by default unless the config requests it.
- Add `tests/content_runtime_references.rs` and fixtures covering valid links,
  missing links, duplicate IDs, ambiguous references, optional references, many
  references, cross-adapter references, and cycle diagnostics.

## Out Of Scope

- Authoring toolchain selection or schema generation.
- Cache/index/Deeb performance experiments.
- New CLI mutation commands.
- Broad content query APIs beyond validation snapshot/diagnostic behavior.

## Acceptance Criteria

- Reference validation is no longer limited to scalar/list field shape checks.
- Required references report missing fields and missing targets with path and
  field context.
- Optional references may be absent or empty without producing missing-target
  diagnostics.
- Many references validate every target and report item-level context as far as
  the current diagnostic model supports.
- Duplicate IDs and ambiguous references produce deterministic diagnostics.
- Cross-adapter fixtures prove Markdown/JSON/YAML/JSONL records can reference
  one another.
- Independent review finds no blocker against the goal reviewer criteria.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- `cargo test --test content_runtime_references --quiet`
- `cargo test --test content_runtime_validation --quiet`
- `cargo test --test content_runtime_adapters --quiet`
- `cargo test --test content_runtime_check_cli --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `cargo xtask evidence`
- `cargo xtask docs`
- `git diff --check`
