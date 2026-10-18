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
