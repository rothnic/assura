# A07 candidate-binding correction plan

Status: active. Owner: this thread's A07 acceptance coordinator. This plan
does not embed a permanent routing baseline: fetch `origin/master` and rerun
the revision-pinned ledger before every canary. The 2026-09-10 post-merge
refresh resolved `origin/master=373fb01ac268285d9b21cd6948862050bd7d8c6e`
(PR #259), whose reviewed process candidate was based on
`8c198dcb4c94095e7db1b017908eaadb813cb9e7`. A clean detached checkout built
the parent candidate with Rust/Cargo
`1.94.1`; `assura 0.4.0` and binary SHA-256
`95c93052bd1993566d9f8209bdba5625c2b39a19287ed6358d710015d2ac59ff` matched
inside a `zsh -lic` initializer-like environment. This identity observation
is preparation only and earns no screening allocation. The six valid unseen
holdout layouts remain frozen in the private owner-controlled store; the
versioned exactly-two-condition manifest, supplied-input receipt, blinded
mapping, complete 30-cell matrix and isolated protocol-review `PASS` are still
absent. Do not infer conditions from historical run names or holdout labels.
The next action is ordered: create and validate the private condition manifest
from an explicitly supported product input, obtain an isolated protocol-review
`PASS`, then refresh the candidate identity again before any no-credit canary
or screening batch. The prior
`2eda17e82d9dab12338805479a33a5774560451f` checkpoint is historical after PR
#251; earlier candidate SHAs remain historical evidence/archive references,
not the active baseline. This plan does not change the A07 contract, scoring
thresholds, fixture allocation, or product acceptance.

The canary's private event, evaluator, fixture, and identity provenance remains
outside the repository. Public evidence retains only the candidate identity and
redacted aggregate outcomes needed to route the next action.

## Historical supersession — 2026-09-09 (superseded)

The earlier status below correctly records a candidate-bound identity canary
and the A02 discovery correction that followed it, but it is superseded for
execution routing. PR #231 merged the A02 correction as `922d7f0`; that SHA is
retained as historical evidence and a clean candidate archive, not as the
current execution baseline. A later initializer attempt was invalid because it
selected an ambient Assura binary and inspected evaluator-only context; it
receives no screening, holdout or acceptance credit. The fresh canary against
`77b41fed7ee625333ea97ef2791da609f0ed5cc4` then proved candidate identity,
context separation, and the full product contract, but that source is now
historical after PR #235. The live baseline is
`5329abd880fb26b0eeded5176e2d308b654d2cc9`; PR #237 is now merged, so it is
historical. The `8cabc536` baseline is also historical after PR #248; the
`755c28d` baseline is historical after PR #249, and the `2eda17e` baseline is
historical after PR #251. The dated execution baseline in this narrative was
`062f6c3`; it is now superseded by the live `373fb01` section above. The next authorized action is to
create and validate the exact two product-input conditions in the private
screening manifest, obtain an isolated protocol-review `PASS`, and refresh the
candidate-bound canary against the then-current master before any 30-cell screening
batch; no prior canary result is allocated to it.

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
`init --agent <host> --activate` surface. The historical `77b41fe` canary
passed those identity, surface, context, and full-contract checks; a fresh
candidate-bound canary against the freshly fetched then-current
`origin/master` (the live post-merge checkpoint is `373fb01`, subject to a
fresh fetch) is still required before
allocation. The historical `f1595fc` and `922d7f0` runs remain labeled below
for provenance; none is a screening allocation. An explicit-route control may
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
- The prior candidate-bound canary is complete for historical `77b41fe` only.
  Before the first screening run, fetch `origin/master`, rerun the
  revision-pinned ledger and freeze the candidate identity, then record the
  authoritative names and contract references for both product-input
  conditions in a private screening manifest. Validate its supplied-input
  evidence and complete matrix, and obtain an isolated protocol-review `PASS`;
  do not infer or rename conditions in public evidence. Then refresh the live
  `373fb01` source pointer and ledger and run a fresh candidate-bound canary
  against that frozen identity; all embedded checkpoints remain historical.
  launch one fresh source-only fixture and child under a sibling-free
  disposable parent per run. If identity, context, or contract checks fail,
  retain the result as invalid/no-credit evidence, repair the owning method, and
  obtain a fresh canary before resuming.
- The earlier plain-route product failure and context-contaminated attempt are
  historical protocol evidence. They remain no-credit records and must not be
  replayed as an unchanged method or treated as a replacement for the passed
  current canary.

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
