# Preserved progress entries — iterations 139 and 136

These dated entries moved from `progress.md` on 2026-09-11 to keep the active
progress index below the configured 1000-line limit. They remain historical
evidence and do not change the current ac3 route.

## Iteration 139 — 2026-09-11 — post-merge current-source reconciliation

- Owner/phase: process coordinator `/root` / `post-merge-reconcile`; clean
  owned worktree `/private/tmp/assura-a07-postmerge-961dced` is based on
  `origin/master=961dced162c8fe67ad87d64e0be750249edd5681`. PR #280 merged the
  reviewed process-only route; applicable Documentation, CI Scope, Security
  Scope, Evidence Gates and GitGuardian checks passed. Product/Rust/
  performance/release jobs were scope-skipped and are not acceptance proof.
- The post-merge Rust CI run `34564937464` completed `success`: CI Scope and
  Evidence Gates passed; the product, Rust, performance, release, MSRV,
  Rustfmt and test jobs were skipped by scope and remain non-applicable. The
  revision-pinned ledger is `items=32; ready_pending=0; unfinished=5; held=3`:
  A07 active, W03 verified, R01/W02/F01 held.
- The first candidate build command accidentally compiled the dirty root
  checkout; its `c7815c5` binary and permissive wrapper output are retained as
  failed no-credit evidence. The explicit-workdir rebuild from 961dced
  completed in `/private/tmp/assura-a07-current-961dced` with Rust/Cargo
  `1.94.1`, Assura `0.4.0`, source tree
  `749d51fdb33c9d46a6c45b7a84521274cb4f43b3` and binary SHA
  `be33c5018feb55ea9b1ab6f8d4766bb3a50c71a5cba1773bcfbc6d0f3968f9e7`.
- Fresh login identity and two source-only receipts agree on that candidate;
  both full evaluators pass all seven dimensions. Isolated review found stale
  5b03c7f refs and contradictory pending/pass prose; both are corrected, the
  cross-artifact assertion passes and scoped protocol rereview returns `PASS`.
  Context level: not exposed. Next: refresh source/ledger and prove packet
  match before separately authorized screening; preserve topology exceptions.

## Iteration 136 — 2026-09-11 — current candidate canary and packet rebind

- Owner/phase: process coordinator `/root` / `candidate-bound-canary`; the
  clean candidate worktree is `/private/tmp/assura-a07-current-5b03c7f` at
  `origin/master=5b03c7f41df1fe8ecf9ae5168eaa11f550b8721f`. The root checkout
  remains dirty with the unknown user-owned A04 note and was not modified;
  topology still reports only the preserved foreign dirt and three stale
  registrations.
- The candidate was built once with Rust/Cargo `1.94.1` using an isolated
  target directory. The private freeze records `assura 0.4.0`, the absolute
  binary SHA, source tree SHA, login-shell shim SHA and exact toolchain. An
  explicit wrong-path control returned exit `97` with `identity_status=fail`.
- Two fresh sibling-free source-only Rust fixtures were launched through the
  composed Codex initialization route with the two frozen private
  `--content-template` conditions. Both initializer receipts and full private
  evaluator runs exited `0`; all seven dimensions passed, the seeded negative
  naming probe rejected with the expected rule, and native `cargo test
  --offline` collected and passed a test. The first evaluator invocation used
  an unsupported `negative` dimension token; it is retained as an explicit
  failed no-credit attempt and the runner guidance now requires checking the
  evaluator's declared dimension set before invocation.
- The private six-handle construction record, current binding and second
  read-only confirmation are rebound to `5b03c7f`; immutable creation times,
  source digests, exact toolchain and raw-hook exclusion are preserved. The
  exactly-two-condition manifest has 30 reserved cells, no allocation credit,
  and a separate blinded mapping. Independent protocol review is pending;
  no screening, holdout, follow-up-feature, acceptance, release, deployment,
  publication or invitation authority changed.
- Context level: not exposed. Repeated failure review found stale checkpoint
  pointers and evaluator invocation drift as the rediscovery risks; the
  canonical task/checkpoint and runner-isolation reference now point to the
  current candidate and exact dimension rule without expanding `AGENTS.md`.
  Next owner/action: finish the independent isolated protocol review of the
  frozen packet; resolve any concrete finding and rereview, or if it passes,
  seek the separately authorized screening decision. Do not allocate a cell.
