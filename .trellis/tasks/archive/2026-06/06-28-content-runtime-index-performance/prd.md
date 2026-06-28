# Content Runtime Index And Performance Hardening

## Objective

Implement increment 9 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: decide
whether the repo-native content runtime needs an internal file index/cache
layer now, and record performance evidence for normal `assura check` use.

## Scope

- Add benchmark evidence for content-runtime validation against a comparable
  no-content-runtime baseline.
- Measure cold and warm paths where practical, keeping the repo files as the
  canonical source of truth.
- Add or explicitly reject an internal index/cache layer based on measured
  evidence.
- Include a Deeb comparison if Deeb remains a candidate after the first
  benchmark pass.
- Update checked performance history or analysis docs when runtime behavior or
  release-readiness evidence changes.
- Keep any cache/index internal and optional; it must not become the public
  content model source.

## Out Of Scope

- User-facing write-operation changes.
- New YAML or JSONL adapter behavior.
- Public server, database, or Deeb-backed source-of-truth workflows.
- Requiring Deeb, SQLite, LinkML, TypeSpec, CUE, Node, Python, or Go for normal
  validation.
- Release-readiness hardening beyond the performance/index decision.

## Acceptance Criteria

- Benchmark evidence compares content-runtime validation with and without
  content-object validation on the same machine and similar build profile.
- Evidence records cold and warm medians, or explains why one path is not
  meaningful for the current implementation.
- Any accepted index/cache remains internal, optional, and derived from ordinary
  repository files.
- Any rejected index/cache path, including Deeb if evaluated, has a concise
  evidence-backed rationale.
- Existing content-runtime validation, adapter, reference, write, and DX tests
  still pass.
- Independent review finds no blocker against the goal reviewer criteria.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- `cargo test --test content_runtime_validation --quiet`
- `cargo test --test content_runtime_create --quiet`
- `cargo test --test content_runtime_update --quiet`
- `cargo test --test content_runtime_adapters --quiet`
- `cargo test --test content_runtime_references --quiet`
- `cargo test --test content_runtime_dx_docs --quiet`
- `cargo run --quiet -- check --format json .`
- Performance benchmark command added or selected for this increment.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `cargo xtask evidence`
- `cargo xtask docs`
- `git diff --check`
