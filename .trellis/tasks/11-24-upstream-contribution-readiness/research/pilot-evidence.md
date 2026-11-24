# pipx and Codex pilot evidence

## Comparison

Both workflows used the same installed Assura Git hooks, generated and activated
Codex hook commands, event sequence, and real Git checkpoints. The fifth
checkpoint replayed the exact published three-file diff from pipx PR #2502 on
top of the synthetic size controls. The current workflow is Assura's shipped
default at `7ab92cb61b78d714d264227535e1191fe612aad6`. The proposed workflow
adds the [contribution guide](../../../../docs/upstream-contribution-workflow.md)
at the review boundaries and retains the current advisory settings.

| Measure | Current default | Proposed workflow | Threshold probe |
| --- | ---: | ---: | ---: |
| Development commits accepted | 5/5 | 5/5 | 5/5 |
| Development commits interrupted | 0 | 0 | 0 |
| Explicit staged enforcement rejections | 1/1 | 1/1 | 1/1 |
| Nonempty Codex hook context responses | 6 | 6 | 7 |
| Total context UTF-8 bytes, including wrappers | 947 | 947 | 1,345 |
| Context on ordinary successful commit events | 0 | 0 | 0 |
| Automatic patch-size responses | 0 | 0 | 4 |
| Exercised boundary controls | — | 7 | — |

These are response bytes in actual `additionalContext` fields from the installed
hook command under replayed Codex events. Token use and live model behavior were
outside this replay. The six default responses are existing file-length findings,
some repeated by pre- and post-tool events. The probe reduced the interval for
observation and explicitly enabled the patch signal. It repeated the 1,861-line
signal after the patch shrank from 2,761 lines. It surfaced no review problem
missed by the explicit preflight. Keep automatic PR-size feedback opt-in.

The native file check accepted the 1,629 → 340 rewrite (+286/−1,575).
Committing and undoing 900 temporary additions returned the same final diff.
Growth to 1,740 failed the explicit snapshot check against the inherited 1,629
limit. The default hook still allowed that checkpoint, and the staged opt-in
rejected the next line while preserving HEAD. Additional controls distinguished
a fork tracking ref from the intended target, rejected missing and ambiguous
bases, rejected stale saved snapshots on target and head changes independently,
rejected an untracked worktree file, and reported unknown binary line counts.
Existing Git and Assura inspection covered these cases. No added-line policy was
needed from `ae-whr.10`.

Reproduce from the Assura repository with Python 3, Git, an absolute Assura
binary built from the source above (`cargo build --locked --bin assura`), and a
local pipx checkout containing the candidate and merge-base objects below.
Pass a fresh disk-backed output directory. The replay records every observed
context payload in `results.json`.

```sh
python3 .trellis/tasks/11-24-upstream-contribution-readiness/research/replay.py \
  "$ASSURA_BIN" "$PIPX_CHECKOUT" "$NEW_RUN_DIR"
```

Measured with local Git 2.47.1, Python 3.14.2 and rustc 1.94.1. Assura binary
SHA-256 `4786c9af7003db2591ffed99c48ba263305d495d2d093e02622fe01a69a3dc18`.

## Issue draft check

**Title** — `pipx run` misses the entry point for an ordinary local project path.

**Expected and observed** — A project at `dist\my-app` with a sole console
script named `renamed` should run that script when the path is passed without
`--spec`. On Windows 11 with PowerShell, Python 3.13.15 and pipx 1.19.1, the
reported `--verbose` output shows the pip backend looking for the path as an
executable, and the uv backend delegating to `uv tool run` and failing to find
the executable. This is the reporter's environment and observation.

**Disposition** — Search found an exact issue and an existing PR. Use the
original report, which supplies full verbose traces, and review its linked PR
instead of filing this draft. [Issue #2501](https://github.com/pypa/pipx/issues/2501),
[PR #2502](https://github.com/pypa/pipx/pull/2502)

## PR draft check

**Title** — Infer the entry point when `pipx run` receives a plain local path.

**Summary** — Local paths use entry-point discovery with either backend. VCS,
explicit `--spec` and option-like operands retain their existing routes.
Regression coverage exercises both backends using the existing local-project
fixture. A one-line bugfix news fragment describes the restored behavior.
There are no temporary interpreter overrides or new public commands. The
patch keeps the documented command form and adds a news fragment for users.

**Pre-open result** — The patch fits the [pipx brief](pipx-brief.md) at
+62/−8 across three files (70 changed lines versus a sample median of 134).
`run.py` grows 748 → 768 lines, `test_run.py` grows 1,866 → 1,899, and the new
fragment is one line. `git diff --check` passed on the committed patch. Link the
open issue and keep the review in the existing PR. The template's requested
validation and hosted freshness evidence belong in its readiness report.

**Ready result** — Inspected source head `78eaea56aadea0d786680246024396c12b6982a8`,
current `pypa/pipx:main` at `9bd278888b88af870e7c6d5da2e87fa49d1a5b59`, and unique
merge base `dbb0e420a831220edabf53819b7a1b3870de0d52`. Local validation ran on
GitHub's integration commit `81c33d67e30c732b345b1353e00fb75684ac3771`, whose
parents exactly match the target and head. `prek run --all-files` passed, and
the focused tox run passed 8 tests each on Python 3.11.9 and 3.15.2.

The green hosted run for the source head finished at 02:39 UTC, before the
current target commit at 19:04 UTC on 2026-11-24. This does not establish hosted
validation of the current integration tree. The pilot's decision is to hold
readiness pending fresh hosted evidence. No upstream artifact was submitted
or transitioned. [Hosted run](https://github.com/pypa/pipx/actions/runs/45312144337),
[current target commit](https://github.com/pypa/pipx/commit/9bd278888b88af870e7c6d5da2e87fa49d1a5b59)
