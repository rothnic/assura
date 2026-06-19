---
id: goal-assura-rule-module-topology
type: goal
title: Assura module topology rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - .trellis/spec/assura/roadmap.md
  - docs/goals/assura-rule-public-surface-support-matrix.md
  - docs/goals/assura-rule-test-relationship.md
  - docs/support-policy.md
  - src/lib.rs
---

# Assura Module Topology Rule

## Objective

Generalize module topology validation beyond explicit directory allowlists so
Assura can detect abandoned module families and overly broad public surfaces.

## Revalidation Result

`valid`, with narrowed first-slice scope.

The release-contract, public-surface support matrix, Cargo manifest semantics,
and test-relationship first slices are complete. `cargo xtask target-state`
passes on current `origin/master`, so this is not a live repository repair.

The target-state analysis still identifies module boundaries as
misaligned/contained: `src/` is broad, experimental/internal modules are public
through `src/lib.rs`, and current file-size/directory constraints do not prove
cohesion or ownership. The valid next product question is whether Assura can
offer a configurable module-topology rule that classifies module families and
public exports without deleting contained experimental code prematurely.

## User Certainty Bar

A Rust maintainer should be able to declare supported, experimental, internal,
roadmap, and unsupported module families, then get an actionable `assura check`
finding when a public module/export appears without an owning topology row or
when a configured module family drifts from its declared status.

## Current Gap

- Directory allowlists and line limits keep files bounded but do not prove that
  module families are cohesive or intentionally exposed.
- `extensions.support_matrices` classifies configured command/export families,
  but it does not inspect Rust module declarations or nested module topology.
- Experimental/internal modules can remain public through `src/lib.rs` as long
  as support policy marks them, but there is no reusable rule tying module
  trees, support rows, and explicit topology ownership together.
- Broad refactors or deletions should wait until a detector distinguishes
  supported product modules from contained experimental/internal evidence code.

## Detector Hypothesis

Parse Rust module declarations and directory trees, compare them to configured
ownership/status categories, and flag public modules whose status conflicts
with support-policy rows.

## First Slice Scope

- Add explicit config notation for one or more module-topology policies.
- Accept configured module-family rows with:
  - stable status vocabulary aligned with support matrices;
  - owner/purpose text;
  - root files or directories;
  - allowed public export names or explicit internal-only markers.
- Build a bounded Rust module inventory from configured files, starting with
  `mod`, `pub mod`, and top-level `pub use` forms already relevant to
  `src/lib.rs` and current module roots.
- Report unclassified public module families, public exports whose status
  conflicts with configured topology, and configured module roots that no
  longer exist.
- Add independent fixtures for public, private, experimental, internal, and
  abandoned module families.
- Dogfood the first slice on Assura's current public/internal module split
  without deleting or renaming modules.

## Non-Goals

- No broad Rust parser beyond the bounded declaration/export forms needed for
  configured module roots.
- No public API semver guarantee for pre-1.0 exports.
- No module deletion, movement, or large refactor in the first slice.
- No replacement for support-matrix classification; compose with it.
- No natural-language docs lifecycle or stale-claim detector in this slice.

## Required Examples

- Passing: current-product modules under `src/cli/check`.
- Passing: experimental modules explicitly labeled unstable.
- Failing: public unsupported module without an experimental/internal marker.

## Definition Of Done

- Module-topology notation is documented before implementation.
- Passing fixture covers supported/current-product modules and explicitly
  internal or experimental modules.
- Failing fixtures cover an unclassified public module, a missing configured
  module root, and a public export/status conflict.
- `assura check --format json` reports actionable module-topology violations
  with file/module/policy context.
- Assura self-check dogfoods the rule without weakening support-matrix,
  manifest-semantics, test-relationship, release-contract, or target-state
  checks.
- Independent review confirms the rule is reusable outside Assura and does not
  force module deletion or overclaim API stability.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

## Review Tasks

- R0: Confirm the rule uses explicit configured module-family rows rather than
  hard-coding Assura module names.
- R1: Review the bounded Rust module/export scanner for false positives and
  missed top-level declaration forms.
- R2: Review status vocabulary alignment with `extensions.support_matrices`.
- R3: Review fixtures for reusable supported, internal, experimental, and
  abandoned module examples.
- R4: Confirm the first slice does not imply public API semver guarantees or
  require broad module deletion/refactoring.
- R5: Confirm diagnostics identify the specific file, module/export, policy id,
  and expected topology status.

## Reviewer Blocking Criteria

Block the PR if topology ownership is implicit, if the rule only hard-codes
Assura's current modules, if public modules can silently bypass classification,
if the scanner recurses into arbitrary Rust bodies without a clear bound, if
support-matrix checks are weakened, or if diagnostics do not identify the
specific module/export and owning policy.

## Tests

Add Rust fixture trees with public, private, experimental, and abandoned module
families plus CLI integration coverage.

## Progress Log

- 2026-06-19: Revalidated after Test Relationship Rule first slice merged in
  PR #68 and archived in PR #69. Result: valid with narrowed first-slice scope.
  The next slice should implement explicit module-family topology notation,
  bounded Rust module/export inventory, public export/status conflict checks,
  and Assura dogfood policy while leaving module deletion/refactoring and docs
  lifecycle detection to later work.
- 2026-06-19: Started implementation under Trellis task
  `06-19-module-topology-rule-implementation`. Added
  `extensions.module_topologies` config, semantic validation, runtime root and
  public-export checks, compiled-config portability, Assura dogfood policy,
  notation/support-policy docs, and focused parser/runtime/compiled CLI tests.
  Local gates run so far: `cargo fmt --all -- --check`,
  `cargo test --all-targets --quiet`, `cargo clippy --all-targets
  --all-features -- -D warnings`, and
  `cargo run --quiet -- check --format json .`.
- 2026-06-19: Independent review found two blockers: conflicting export keys
  across `family` and `public_exports` could silently overwrite at runtime, and
  `status: unsupported` public exports were not covered as status conflicts.
  Added semantic conflict rejection, unsupported-public-export runtime
  diagnostics, and regression coverage. Re-ran focused tests plus the full
  local gate set successfully.
