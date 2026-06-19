---
title: Goal 12 Support And Test Matrix Review
status: active
---

# Goal 12 Support And Test Matrix Review

## Scope Review

Goal 12 extends the existing `cargo xtask target-state` verifier instead of
creating a parallel support-matrix command. The first implementation slice adds
a joined support matrix in `xtask/src/main.rs` that connects:

- `.assura/command-surface.yml` command names;
- support-policy markers;
- compatibility matrix markers;
- CLI/public Rust source markers;
- workspace manifest policy rows;
- test coverage markers; and
- explicit exceptions for experimental or internal surfaces.

## Matrix Rows

| Surface Family | Classification | Evidence |
| --- | --- | --- |
| CLI root | Supported command root | Command surface row plus `src/cli/args.rs` root parser marker. |
| `assura check` and output formats | Supported | Support policy, compatibility matrix, CLI source markers, and CLI/real-project tests. |
| `--agent codex` | Supported adapter | Shared agent-format support policy, compatibility row, source marker, and command-surface test. |
| `assura init` | Supported | Support policy, compatibility row, source marker, and init command test markers. |
| `assura status --format json` | Supported | Support policy, compatibility row, source marker, and real-project status test markers. |
| `assura migrate` | Supported LS-Lint semantics | Support policy, compatibility row, source marker, and migration test markers. |
| `assura hooks` | Supported local workflow | Support policy, compatibility row, hook source markers, and hook behavior tests. |
| `assura quality plan` | Supported local workflow | Support policy, compatibility row, source markers, and quality-plan tests. |
| `assura performance-report` | Supported evidence command | Support policy, compatibility row, source markers, and performance contract tests. |
| `assura info` | Experimental diagnostic | Support policy and compatibility exception markers, with no supported automation-test requirement. |
| `assura watch` | Experimental | Support policy and compatibility exception markers, with no supported automation-test requirement. |
| Internal Rust APIs | Unstable internal | Support policy, compatibility matrix, `src/lib.rs` source markers, and explicit unsupported-surface language. |
| Root manifest | Public root package | Required metadata, SemVer-like version, default binary, and workspace membership checks. |
| Internal manifests | Internal support crates | Required metadata, root version/MSRV sync, and `publish=false` checks. |
| Xtask manifest | Internal maintenance crate | Required metadata, root version/MSRV sync, and `publish=false` checks through `cargo metadata`. |

## Review Tasks

- R0: Existing planned rule docs are reused through the Goal 12 task context.
- R1: Support classifications are checked against `docs/support-policy.md` and
  `docs/compatibility-and-surface.md`.
- R2: Manifest metadata is enforced by explicit target-state manifest matrix
  rows for the public root package and internal support crates.
- R3: Test relationships are now checked from the same support matrix rows as
  support classifications.
- R4: Experimental/internal rows require explicit exception markers instead of
  being silently treated as supported.
- R5: PR must link this artifact and Goal 12.
- R6: `cargo xtask target-state` now verifies the stale-goal revalidation route
  through `assura-goal-validation`.

## Review Feedback Closure

Independent review: Descartes, `trellis-check`, 2026-06-18.

| Finding | Resolution |
| --- | --- |
| New public CLI variants could bypass the matrix if omitted from `.assura/command-surface.yml`. | Added source-derived inventory checks for `Commands`, `HookCommands`, and `QualityCommands` in `src/cli/args.rs`; unknown variants fail `cargo xtask target-state`. |
| Public docs could claim `assura info` or `assura watch` as supported while policy says experimental. | Added public claim consistency scanning across README, active support/release/validation docs, and website content. |
| Manifest semantics used line-oriented TOML parsing. | Switched manifest policy rows to structural `cargo metadata --no-deps --format-version 1` data for package fields, publish policy, versions, MSRV, default run, workspace members, and default members. |
| Test relationship markers could be satisfied by source-only text. | Added external `assura hooks --help` command-surface coverage and changed matrix test markers to require test-file text. Source markers are checked separately. |
| Validation evidence was incomplete. | Expanded this artifact to include the full Goal 12 validation set. |

Gemini review on PR #54, 2026-06-19.

| Finding | Resolution |
| --- | --- |
| `metadata_package` used suffix matching and could select the wrong `Cargo.toml`. | Manifest lookup now normalizes Cargo metadata paths to exact repo-relative paths before matching matrix rows. |
| `workspace_default_members` may be absent from Cargo 1.70 metadata. | Default-member enforcement now runs only when Cargo metadata exposes `workspace_default_members`; workspace membership remains required. |
| CLI enum parsing counted braces inside comments. | Enum inventory parsing now skips line, block, and doc-comment lines before variant/depth handling. |
| Experimental support-claim scanning could flag negated statements. | Public-claim scanning now excludes `unsupported`, `not supported`, and `not yet supported` phrases before treating a line as a supported-status claim. |

## Validation Commands

```bash
cargo fmt --all -- --check
cargo test --test cli_command_surface_tests --quiet
cargo test --all-targets --quiet
cargo xtask target-state
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```
