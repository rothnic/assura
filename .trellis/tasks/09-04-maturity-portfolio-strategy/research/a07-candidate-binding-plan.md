# A07 candidate-binding correction plan

Status: active. Owner: this thread's A07 acceptance coordinator. The runner
binding correction is now canary-proven; the remaining action is the smallest
product discovery correction. This plan does not change the A07 contract,
scoring thresholds, fixture allocation, or product acceptance.

## Finding

The first post-merge source-only probe was launched with the frozen candidate
first on `PATH`, but the initializer's command tool starts `/bin/zsh -lc`
login shells. Those shells resolved `/usr/local/bin/assura` instead of the
frozen candidate. The event stream's help output lacked the merged `init
--agent` surface, proving that the initializer and evaluator did not use the
same executable. The full evaluator result is consequently invalid for
screening, even though the evaluator itself was invoked with the frozen
candidate. The result and its redacted output remain retained as unfavorable
protocol evidence; they receive no allocation credit.

## Contract to enforce

Every initializer run must bind one immutable candidate executable and prove
the binding inside the run before any product result is counted:

1. Resolve an absolute executable path, source SHA, version, and SHA-256 before
   launch.
2. Expose that executable to command tools through a login-shell-safe launcher
   or an isolated command shim; inherited `PATH` order is insufficient. The
   preferred launcher is the regular candidate executable itself. If a shim is
   needed, it must be a regular executable that `exec`s one fixed absolute
   candidate path; aliases, shell functions, and unrecorded symlink chains are
   not valid bindings.
3. Capture `command -v assura`, `assura --version`, and the candidate SHA from
   the same command environment used by the initializer. Record both the
   resolved command path and the fixed target path. Canonicalize the target
   before hashing and compare `source_sha`, `version`, and `binary_sha256` with
   the pre-launch identity; a shim that delegates elsewhere is a mismatch.
4. Compare those observations with the pre-launch identity. A missing,
   mismatched, or ambiguous identity invalidates the run and prevents it from
   entering screening, holdout, or final acceptance.
5. Evaluate the resulting project with the same absolute candidate path. Keep
   initializer output, evaluator output, and redacted publication output
   separate.

The canary must also prove that the candidate exposes the expected merged
`init --agent <host> --activate` surface. This binding canary now passes those
identity and surface checks. Its fresh agent still chose plain `assura init .`
and failed the full contract, so that result is valid product evidence but a
canary only; it receives no screening allocation. An explicit-route control may
verify the composed implementation, but it is calibration evidence and never
counts as a blinded run.

## Smallest implementation and verification

- Prefer a disposable, absolute-path launcher whose command shim survives
  login-shell startup; do not mutate global `/usr/local/bin/assura`. When a
  shim is used, inspect its fixed absolute `exec` target and hash that target;
  do not rely on an alias, function, or opaque symlink chain.
- Add focused runner regressions for PATH reset, wrong-shim-target mismatch, and
  successful exact-target binding. Keep private fixture and evaluator
  identities out of source, prompts, reviewer briefs, and public evidence.
- Run one fresh Rust canary with the fixed prompt only after the identity
  assertions pass. Compare the event stream and evaluator input before
  authorizing any screening repetition.
- The identity canary passed, but the product contract did not: the fresh agent
  selected the intentionally config-only plain route. Route that failure to the
  A02 discovery owner and add a focused normative handoff regression before
  any screening. After the product correction, freeze its exact source/binary
  identity and run a new candidate-bound canary. If it fails, retain the result,
  repair the owning behavior, and do not repeat the unchanged method.

## Gates and ownership

The runner correction is a separate, reviewable process/evaluation slice. Its
owner must commit one coherent change before review and record the exact branch,
source SHA, executable identity, command/cwd, exit status, and redacted result
in `evidence/A07.md`. Independent review must cover the binding invariant,
login-shell behavior, private-data separation, and the no-credit rule. Resolve
accepted findings and obtain scoped rereview before integration.

For a runner/script-only change, use the workflow gate, focused positive and
negative controls, `git diff --check`, the source structure check,
`cargo xtask evidence`, and applicable documentation checks. If Rust behavior
or release surfaces change, add the focused red/green test and the full
`cargo xtask pr`/hosted gates required by the changed surface. VPS execution is
optional: check measured capacity, disk headroom, and exact toolchain first;
remote speed or memory does not replace hosted platform proof.

Only a clean, independently reviewed, current-master, fully gated correction
may merge. After merge, verify ancestry and rerun the identity canary against
the merged candidate. A07 remains active until its independent screening,
untouched holdout, follow-up feature checks, and final per-stack threshold are
all satisfied.
