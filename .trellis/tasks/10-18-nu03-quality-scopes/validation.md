# NU03 validation checkpoint

Product source and candidate binaries were verified at
`3498274b8c88c55e2fd431024ebf1125ef2910ef`, based on
`08256971aaf7490c611d58354ff871b17dae55ec`. This checkpoint changes task
documentation only; it does not claim that earlier binaries came from a later
documentation commit.

- Focused current-base checks: 70 passed, none failed or ignored.
- Structure validation: no blockers and 17 low-severity line-count advisories.
  This includes the new shared exclusion bridge taking `src/cli/check.rs` from
  495 to 516 lines against its advisory 500-line limit. The final review must
  assess that bounded debt explicitly; it is not described as pre-existing.
- Complete `cargo xtask pr`: exit 0, 342 seconds, 1,474 tests passed, none failed,
  one ignored. The pinned Node 24.19/pnpm 10.25 website build produced 48 pages.
  The tool trace omitted argv values; fixed executable wrappers, the unchanged
  build invocation and completed output supplied the independently reviewed
  binding. This is not a claim that missing argv values were captured.
- Four disposable pre-commit cases retained underlying exits 0/1/1/0 for
  positive/native failure/Assura failure/Assura removal. Native lint and test
  events remained in order in every case; Assura ran 1/1/1/0 times. Every guard
  exited 0, and all eight child Bun identities matched the admitted executable.
- The outer recorder then exited 1 while accessing nonexistent
  `CompletedProcess.pid`, after saving the controller's exit 0 and raw output.
  Its normal postflight/result records and controller PID remain unavailable.
  A separate read-only recovery snapshot verified source, fixture and hook
  state. No consumed case was rerun and the recorder failure remains a failure.

Raw receipts and independent reviews are retained under
`/Users/nroth/workspace/.assura-review-20261017/nu03/`. Their key SHA-256 digests:

| Artifact | SHA-256 |
| --- | --- |
| Four-case aggregate | `55fa761c140b963666d16656e3c9d4fd222a5dbf1f8b545dd964da73521d20e2` |
| Recovery snapshot | `254ef65ee398f8421284fcc9708b86cf2d2c21b92ccc1ea3345f537a8d797697` |
| Supplemental adjudication | `2a23a0a06549b9ab69d4d5379d3af6cc2e8c7e4cab71db00904874d887b78f0f` |
| Full-tier independent review | `2a4bd7509ecd9676cebcadaae96c102f80794f4a18b452bb56a279b775ccafc9` |
| Actual-proof independent review | `95310c13249de302e95e8d33a8d2c628d5d624474fa94a9f3e80919acbbb0176` |

The three recorder diagnoses accept the actual four-case proof with its explicit
outer failure and missing-PID limitations. This proves fixture composition,
order, failure visibility and removal behavior; it does not establish real
project-script adequacy, human adoption or release acceptance. Final independent
review, current-base integration and applicable hosted checks remain required.

## Exact focused setup correction and current-base refresh (2026-10-18)

The single focused 70-test lane was rerun on clean candidate source
`7904af3f03859fcc9aa4a3083f78f78cf9371ea9` after the selected numeric Rust
1.94.1 toolchain was supplied its missing `rustfmt` and `clippy` components.
The preflight bound Cargo/rustc to 1.94.1 and `cargo fmt`/`cargo clippy` to
rustfmt 1.8.0 and Clippy 0.1.94 under the original recorded PATH and
`RUSTUP_TOOLCHAIN=1.94.1`. The generated fixture uses the candidate's
`CARGO_BIN_EXE_assura-full` binary for `assura check`, then runs `cargo fmt`,
`cargo test --locked`, and `cargo clippy` in its fixture directory. Its full
plan and seeded-format rejection/restoration assertions completed.

The focused run exited 0 with 70 passed, none failed or ignored. Its four suite
summaries were 18, 25, 11, and 16 passed. The explicit test-name census, raw
stdout/stderr/exit, selected-toolchain preflight, resource gates, and prior
missing-component receipts are retained under
`/Users/nroth/workspace/.assura-review-20261017/nu03/fmt-environment-correction/`.
The earlier 54/0 recorder summary remains incomplete; the supplemental 69/1
census records that earlier failed attempt. This successful run supersedes that
focused-run failure only for the unchanged candidate and exact selected
component environment.

The candidate was then nonforce-merged with current base
`74d9917dd39e599bbd43c3a5760b71644b2cf91b` as merge commit
`22f1051efff7ddb8aa1654428f9cdf2bf184f1b9`. The incoming delta contains only
DX01 skill and Trellis task/workspace files. Rust source, tests, configuration,
Cargo/.cargo, xtask, `.assura`, and CI/workflow inputs are unchanged from the
focused run. All seven tracked xtask/Cargo/.cargo input hashes match the
preserved xtask binding, so its bounded `evidence` command was reused on the
merged tree and passed. The candidate `assura-full check .` structure gate also
passed: 2,027 files and 430 directories checked, 17 low-severity line-count
advisories, and no blockers. The post-merge workflow gate reports this NU03 task
ready and in progress.

The complete `cargo xtask pr` result remains historically bound to
`3498274b8c88c55e2fd431024ebf1125ef2910ef`; it was not rerun on this current
merge. This local focused proof does not resolve or explain the separate hosted
macOS hot-status failure, whose cause remains unknown. Final independent review
and the changed-source hosted diagnostic remain outstanding.

## Changed hosted diagnostic and focused lint correction

The single changed-source push at `b422c52d` ran against main `74d9917d`.
The exact external-config hot-status test passed in hosted macOS stable job
117653724888; that observation does not identify the earlier failure's cause.
Stable Clippy job 117653725092 and Rust 1.86 job 117653724818 both rejected
the diagnostic's immediately invoked closure with `redundant_closure_call`.
The MSRV language check and launcher build had passed; this was a source lint.
Official logs and all three independent read-only triages remain external.

The correction replaces only the closure delimiters with a plain block.
Its entire snapshot body is byte-identical: same 15-byte bound, read timing,
error/decoded text, `Some(0)` expectation and cleanup order; no lint allow or
policy/version change. The affected numeric Rust 1.94.1 package/test Clippy
check passed with warnings denied, offline and one compile job. This check
executes no behavior tests and does not replace the required hosted stable
and Rust 1.86 gates. Formatting/diff checks passed. The existing Rust review
reference now calls for this focused lint check on diagnostic-only changes.

Evidence is under external `nu03/diagnostic-clippy-correction/`. The focused
70-test result and original four-case composition proof retain their recorded
source and limits; neither is relabeled as freshly executed here. Independent
scoped rereview, corrected-head hosted checks, final current-source full PR
and disposition of the earlier unknown macOS failure remain outstanding.
