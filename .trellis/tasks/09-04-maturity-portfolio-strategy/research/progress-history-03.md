# Maturity execution train progress — preserved history

These entries were moved from `progress.md` when the live journal reached its
configured line limit. They remain historical evidence and do not supersede
the newer entries at the top of `progress.md`.

## Iteration 44 — 2026-09-07 — W03 ownership impasse

- Refreshed Assura `origin/master` to `98bd187` and confirmed B00/P01 are done.
  W03 is the sole dependency-ready pending card, but its only identified
  NickRoth candidate is clean, local, unpushed, Nick-authored work at `113febc`
  with no Assura-ledger owner or closure record.
- Independent read-only impasse review accepted two findings: do not infer
  ownership or mutate that worktree, and do not treat its unrun local test code
  as W03 acceptance evidence. The exact held action, resolution choices, and
  non-ready alternatives are recorded in `evidence/W03.md`.
- Context level: not exposed. This is a first ownership classification impasse,
  not a rediscovered procedure gap; existing orchestration and worktree skills
  provide the required control. Next: Nick records transfer or preservation of
  `113febc`; then run W03's prescribed checks in one clean owned worktree.

## Iteration 45 — 2026-09-08 — W03 ownership transfer and PR readiness

- Nick explicitly transferred `goal/w03-case-study-evidence` at `113febc` to
  the execution goal. Its branch already contained current NickRoth `main`; no
  duplicate article or rebase was needed.
- Independent review found no correctness issues. Astro check, Vitest (18),
  focused Playwright (12 across Chromium/WebKit and mobile/desktop), and the
  static production build passed. The first Playwright attempt failed before
  test execution because browsers were absent; pinned browser installation and
  the succeeding run are retained distinctly.
- PR #61 is current-base, clean, and hosted-green (Lighthouse, Cloudflare Pages
  preview, GitGuardian). W03 is `verified`, not `done`: no production
  publication/deployment authority was exercised. Next: merge or publish only
  when Nick explicitly authorizes production visibility.

## Iteration 46 — 2026-09-08 — A05 current execution proof and ownership rotation

- PR #200 merged the independently reviewed first A05 native execution slice as
  `f1f3312`. A closed-world fixture test runs the generated Rust `pr` plan in
  its project cwd, observes a seeded formatter failure, and proves recovery
  after restoration; it does not introduce a general command executor.
- The exact execution-branch allowlist moved from the completed A04 branch to
  `goal/a05-current-audit`, preserving target-state's exact branch-plus-PRD
  requirement. The full local `cargo xtask pr` gate and the hosted matrix,
  including Performance Report and platform/install lanes, passed. Security
  Scope passed; the conditional Security Audit was scope-skipped and is not
  treated as test evidence.
- The A05 ledger now records the integrated slice while preserving the separate
  user-owned historical worktree. A05 remains active because Bun/Python
  detection, broader-phase/config coverage, released-binary CI recipe behavior,
  and full acceptance evidence remain. Next: select its smallest current-master
  contract without importing the preserved work.
