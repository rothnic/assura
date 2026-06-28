# Content Runtime Authoring Toolchain Decision

## Objective

Implement increment 7 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: select the
authoring profile and compile path that produces checked-in runtime schema
artifacts for repo-native content validation.

## Scope

- Compare LinkML, TypeSpec, CUE, and a minimal Assura-owned authoring profile
  against the current content-runtime contract.
- Decide whether to use, defer, or reject each option for the next production
  slice.
- Add `docs/analysis/<date>-content-runtime-authoring-decision.md` with a
  clear recommendation, tradeoffs, and the runtime boundary.
- Add or update generated fixture artifacts so the selected or proposed compile
  path validates the same runtime tests as the hand-authored artifact.
- Keep authoring tools optional build-time inputs. Runtime validation must
  continue consuming checked-in artifacts through Rust-native validation.

## Out Of Scope

- Making LinkML, TypeSpec, CUE, Node, Python, Go, or a server required for
  normal `assura check`.
- Broad query APIs, cache/index/Deeb decisions, or performance hardening.
- Replacing the already merged runtime config, adapters, or write operations
  unless a decision proof exposes a narrow defect.

## Acceptance Criteria

- The decision doc names the recommended authoring profile and explains why the
  other options were accepted, deferred, or rejected.
- Generated or compiled fixture artifacts are checked in, or the reproducible
  command path is documented without adding authoring tools to runtime
  validation.
- Existing content runtime validation tests still pass with generated fixture
  artifacts where applicable.
- The boundary is inspectable for TypeScript, Python, and Rust users without
  requiring them to read Assura Rust internals.
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
