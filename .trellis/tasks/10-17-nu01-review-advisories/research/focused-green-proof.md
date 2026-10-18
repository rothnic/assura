# NU01 focused green proof

- Checkout: `/Users/nroth/.codex/worktrees/assura-nu01-review-advisories/assura`
- Branch: `fix/nu01-review-advisories`
- Base source: `49d85358bae85de04f1ce7faa746b377687dcaf3`
- Tested working tree was based on `a0d7bc403a371d06c0dea722d153311aca431e32`; the final code delta was subsequently limited to moving the private next-action adapter into the existing `project_review.rs` command assembly module and formatting the touched assertions/imports.
- Rust/Cargo binary: the repository toolchain selected by `cargo`; no VPS was used.
- Resource gate immediately before final focused run: 71% system memory free, 128 GiB disk available, no existing Cargo/rustc process. Cargo was bounded to two jobs and scratch used `/Users/nroth/workspace/.assura-review-20261017/nu01-tmp`.

## Commands and outcomes

1. `env TMPDIR=/Users/nroth/workspace/.assura-review-20261017/nu01-tmp CARGO_BUILD_JOBS=2 cargo test --test project_review_cli --no-run` — exit 0; focused integration executable built.
2. `env TMPDIR=/Users/nroth/workspace/.assura-review-20261017/nu01-tmp CARGO_BUILD_JOBS=2 cargo test --test project_review_cli` — exit 0; 16 passed, 0 failed, 0 ignored.
3. The suite verifies low-severity Check remains exit 0, Review remains exit 0, configured finding identity and fingerprint remain stable, 501 to 700 lines becomes worsened, 499 lines resolves, unchanged compact text hides the finding, JSON retains complete history, text reports `+12 more` for 15 configured findings, and agent output stays at 12 while summary/history count all findings.
4. `target/debug/assura check --format json .` — exit 0, `success: true`. It reports 15 existing low-severity `max_lines` findings across unrelated files and none in edited source/test files.
5. `git diff --check` — exit 0.
6. `rustfmt --edition 2024 --check` was run on the touched Rust files. The current installed rustfmt still reports pre-existing formatting differences elsewhere in `tests/project_review_cli.rs`; newly added assertions were corrected to its output. The touched `doctor.rs` import order was corrected. Use the repository-pinned formatting gate at PR boundary.

## Remaining gates

The source is not independently reviewed yet. The compact-review spec and task acceptance were updated; broader repository/PR tiers and final review remain for the coordinator. The VPS lane was intentionally skipped because SP01 owns it; this is not evidence of a VPS validation failure.
