# Complete Deslopify Plan

## Goal

Complete the full Assura deslopify plan, not just the P0 stale-surface slice:
remove or contain misleading public surfaces, tighten repository policy where
Assura can already express the invariant, add external Rust hygiene gates where
specialist tooling is the right detector, and open durable follow-up tasks for
generalized Assura rule families that require product implementation.

The work must preserve performance. Any Rust, benchmark, release, workflow, or
performance-evidence change must run the relevant local gate and CI
Performance Report before merge. If a cleanup removes code or public exports,
it must be proven by tests and support-policy alignment rather than by
assumption.

## What Is Already Done

- PR #35 completed the first stale-surface slice:
  - fixed `.agents/skills/custom/assura-validation/SKILL.md`;
  - scanned active skills through `node --run verify:evidence`;
  - rejected unreleased `assura/assura-action` references;
  - rejected unsupported `assura check` flags in direct and
    `cargo run -- ... check` examples;
  - tightened `.assura/config.yml` under `src/cli` for `check` and
    `performance_report` module families.
- PR #36 archived the completed P0 repository cleanup task.
- `docs/analysis/2026-06-05-p0-repo-cleanup-findings.md` records the completed
  stale-surface cleanup and future command-surface rule opportunity.

## Source Inputs

- Parked audit:
  `docs/analysis/2026-06-04-rust-modernization-quality-audit.md` on
  `codex/park-agent-governance-audit-20260605`.
- Active cleanup findings:
  `docs/analysis/2026-06-05-p0-repo-cleanup-findings.md`.
- Support truth:
  `docs/support-policy.md` and `docs/compatibility-and-surface.md`.
- Assura policy:
  `.assura/config.yml`.
- Tooling policy:
  `.trellis/spec/assura/tooling-stabilization.md`.

## Requirements

### R1 Public Surface Alignment

- Audit `src/lib.rs` public exports against the support matrix.
- For unsupported domains such as dependency graph validation, maturity
  detection, and broad validation APIs, choose and implement one of:
  - make internal;
  - gate behind explicit experimental/internal feature names; or
  - document as unstable internal-only surface with a deterministic check.
- Update tests, benchmarks, and docs so they use the chosen contract.
- Add a deterministic evidence check or durable follow-up when the current repo
  cannot express the support matrix check directly.

### R2 Assura Config Tightening

- Add line/size policy for `.agents/skills/**/SKILL.md`; long skills should
  move detailed examples or references into subdirectories where practical.
- Replace or narrow the broad `tests/fixtures/**` exclusion with explicit
  fixture-family policies where current Assura config can express them.
- Tighten active docs/goal limits or archive/split near-limit docs when they no
  longer need active-goal status.
- Preserve self-check success:
  `cargo run --quiet -- check --format json .`.

### R3 External Rust Hygiene Gates

- Evaluate and add the practical subset of:
  - `cargo-deny` for advisories/licenses/banned crates/source policy;
  - `cargo machete` for unused dependency signal;
  - `cargo-semver-checks` for release/advisory API drift;
  - `cargo nextest` only if it measurably helps and does not replace
    `cargo test` before parity is proven.
- Prefer advisory or scheduled gates until baseline behavior is proven.
- Do not reimplement compiler/dependency analysis inside Assura.

### R4 Generalized Assura Rule Backlog

Open one durable task or goal per generalized rule family that cannot fit this
implementation safely:

- command-surface documentation rule;
- Cargo manifest semantics rule;
- module topology rule;
- test relationship rule;
- release sync rule;
- public surface support-matrix rule.

Each task must define a deterministic detector, a config shape hypothesis,
passing/failing examples, and required tests.

### R5 Dead/Abandoned Path Audit

- Review the live tree areas called out by the audit:
  - `src/cli/check`;
  - `src/cli/performance_report`;
  - `src/intelligence/**`;
  - `src/maturity/**`;
  - `src/validation/**`;
  - watch/daemon surfaces;
  - `crates/assura-check-cli`.
- For each area, classify it as current product, experimental, internal test
  support, roadmap-only, or removal candidate.
- Remove or contain abandoned paths only after deterministic evidence and tests
  prove they are not needed by supported surfaces.

### R6 Performance Preservation

- For docs/config-only slices, use the scoped validation policy and CI
  classifier evidence.
- For Rust/workflow/release/performance slices, require:
  - `cargo fmt --all -- --check`;
  - `cargo test --all-targets --quiet`;
  - `cargo clippy --all-targets --all-features -- -D warnings`;
  - `cargo run --quiet -- check --format json .`;
  - `node --run verify:pr`;
  - CI Performance Report pass before merge.
- Any benchmark-data or performance-copy change must use the
  `assura-performance-reporting` skill and regenerate checked evidence.

## Out Of Scope

- Do not merge the parked Agent Work Governance implementation as-is.
- Do not add broad feature behavior just to satisfy a cleanup finding.
- Do not hide unsupported surfaces through exclusions.
- Do not declare the whole plan complete while known backlog categories remain
  unimplemented or untracked.

## Acceptance Criteria

- [ ] Public exports and support-policy docs no longer conflict, or remaining
      conflicts are explicitly labeled experimental/internal and checked.
- [ ] `.assura/config.yml` is stricter for skills, fixtures, active docs, and
      module topology where current Assura rules support it.
- [ ] External Rust hygiene gate configs/scripts exist for the selected tools,
      with baseline evidence and scoped CI/local integration.
- [ ] Durable follow-up tasks/goals exist for every generalized Assura rule
      family not implemented in this task.
- [ ] Dead/abandoned path classifications are recorded in a checked analysis
      document and any removals have tests.
- [ ] Independent review finds no issues after fixes.
- [ ] Local validation passes for the changed surfaces.
- [ ] PR CI is green, including Performance Report for any workflow/Rust or
      performance-sensitive changes.

## Validation Plan

Run the narrow changed-file gate while iterating:

```bash
node --run verify:changed
```

Before merging any Rust/workflow/tooling slice:

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
node --run verify:pr
git diff --check
```

For docs/config-only slices, follow `.trellis/spec/assura/tooling-stabilization.md`
and still require Assura self-check plus evidence gates.

## Review Blocking Criteria

Block if the work:

- makes unsupported features look supported;
- adds broad exclusions instead of deterministic policy;
- removes public exports without updating tests/benchmarks/docs;
- introduces a new long-lived compatibility shim without support-policy
  justification;
- leaves performance checks unrun for Rust/workflow/performance changes;
- leaves dirty or uncommitted work after a task boundary.
