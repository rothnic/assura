# Content Runtime Release Readiness

## Objective

Implement increment 11 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: harden
cross-platform behavior, CI coverage, diagnostics, and adoption guidance, then
complete a requirement-by-requirement audit of the full repo-native content
runtime goal.

## Scope

- Audit every increment and completion-definition bullet in the long-running
  goal against current files, tests, docs, PRs, and hosted CI evidence.
- Close any remaining small gaps in diagnostics, docs, or tests that block
  release-readiness confidence.
- Ensure adoption guidance explains the supported surface for portable projects
  and for implementation agents building a real example repo.
- Confirm cross-platform coverage exists through hosted Linux, macOS, and
  Windows jobs.
- Record final review evidence and map the completed work back to every
  increment.

## Out Of Scope

- New feature semantics beyond release-readiness fixes.
- New content storage adapters.
- New authoring tools or runtime dependencies.
- New public server, database, or UI.

## Acceptance Criteria

- A final audit artifact maps every completion-definition and program-DoD item
  to direct evidence or a closed fix.
- Any remaining diagnostics/docs/test gaps found during the audit are addressed
  or explicitly classified as out of scope with rationale.
- Independent review confirms the feature is complete against the long-running
  goal, not just increment 11.
- Local PR-boundary gates pass.
- Hosted CI passes on Linux, macOS, and Windows before merge.
- The final PR summary maps completed work back to every increment in the goal.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- Content runtime focused tests:
  - `cargo test --test content_runtime_validation --quiet`
  - `cargo test --test content_runtime_check_cli --quiet`
  - `cargo test --test content_runtime_create --quiet`
  - `cargo test --test content_runtime_update --quiet`
  - `cargo test --test content_runtime_adapters --quiet`
  - `cargo test --test content_runtime_references --quiet`
  - `cargo test --test content_runtime_dx_docs --quiet`
  - `cargo test --test artifact_authoring_paths_proof --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask docs`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `git diff --check`

Use the smart cadence: focused audit/fix checks during iteration, full gates
only at PR boundary or when a shared behavior change makes them necessary.
