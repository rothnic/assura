---
id: goal-assura-post-onboarding-backlog-execution-sequence
type: goal
title: Assura post-onboarding backlog execution sequence
status: planned
created: 2026-07-03
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-performance-polish-program.md
  - ../analysis/2026-07-03-agent-ready-onboarding-final-audit.md
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Post-Onboarding Backlog Execution Sequence

## Objective

Close the agent-ready onboarding branch cleanly, then execute the next roadmap
lane as a measurable performance-polish program. This goal exists to keep the
handoff concrete: do not lose the local follow-up commits, do not reopen
completed onboarding work without a real blocker, and do not start performance
work without calibrated proof gates.

## Current Live State

- Branch: `codex/agent-ready-onboarding-backlog`.
- PR: `#139` (`docs: add agent-ready onboarding backlog`) is open and green on
  the remote head checked on 2026-07-03.
- Local branch state: ahead of `origin/codex/agent-ready-onboarding-backlog`
  by nine commits at the time this goal was written.
- Completed local onboarding follow-ups include:
  - generic research/content-authoring framing for `document-project`;
  - removal of overfit SBIR/domain-specific pack references;
  - `assura-structure-fit` skill and `STRUCTURE_FIT_CHECK` onboarding routing.
- Roadmap state: Agent-Ready Project Onboarding is completed locally; the
  separate planned lane is `docs/goals/assura-performance-polish-program.md`.

## User Certainty Bar

Nick should be able to ask:

1. "Is the current agent-ready onboarding PR actually up to date with the
   local branch and reviewable?"
2. "What is the next executable lane after onboarding, and what measurable
   proof makes it done?"

The answer must point to current PR state, local branch state, exact validation
commands, checked artifacts, and an ordered set of subgoals.

## Subgoal Sequence

### 1. Publish And Revalidate PR #139

Goal: make the remote PR match local truth before starting new product work.

Done when:

- `git status --short --branch` is clean.
- `git log --oneline origin/codex/agent-ready-onboarding-backlog..HEAD` is
  empty after pushing the local commits.
- PR #139 body or comment names the added follow-up scope:
  - research/content-authoring document-project framing;
  - overfit-domain cleanup;
  - structure-fit skill and `STRUCTURE_FIT_CHECK` routing.
- GitHub checks are green on the new PR head.
- Independent review has either no blockers or every blocker is fixed or
  logged as an explicit follow-up.

Validation:

```bash
git status --short --branch
git log --oneline origin/codex/agent-ready-onboarding-backlog..HEAD
gh pr view 139 --json state,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

### 2. Revalidate Performance Polish Against Current Evidence

Goal: refresh `docs/goals/assura-performance-polish-program.md` from live
artifacts before implementing performance code.

Done when:

- The goal's current-gap section matches live `benches/history/current.json`
  and `website/public/data/performance/current.json`.
- The starting report distinguishes:
  - LS-Lint-equivalent no-slower rows;
  - Assura-native content/query/document graph rows;
  - cold CLI rows;
  - warm daemon/session rows;
  - diagnostic rows.
- The top three implementation targets are ranked by measured cost, not
  intuition.
- Any stale performance numbers in docs are updated or explicitly marked
  historical.

Validation:

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
target/release/assura performance-report \
  --output target/performance/ls-lint-current.json \
  --history target/performance/ls-lint-current.jsonl \
  --website-dir target/performance/ls-lint-website \
  --iterations 5
cargo xtask performance-no-slower target/performance/ls-lint-current.json
cargo run --quiet -- check --format json .
```

### 3. Add Native Performance Fixture Matrix

Goal: create one shared native-performance matrix that measures Assura-only
capabilities without hiding LS-Lint parity regressions.

Done when:

- Fixtures cover `small`, `medium`, `large`, `reference-heavy`,
  `adapter-mix`, and `real-project` cases.
- Each fixture records file counts, directory counts, object counts, relation
  counts, reference counts, and accepted/diagnostic classification.
- Rows exist for content validation/querying, reference graph operations,
  context packs, Markdown lint/fix, daemon/session queries, and serialization.
- Fixture generation is deterministic and checked into the existing benchmark
  or xtask flow rather than a disconnected script.

Validation:

```bash
cargo bench --bench content_runtime -- --noplot
cargo bench --bench project_intelligence -- --noplot
cargo xtask target-state
cargo run --quiet -- check --format json .
```

### 4. Produce Checked Native Performance Reports

Goal: promote native Assura performance from local Criterion evidence to
checked JSON history and website-ready data.

Done when:

- A native report suite writes `benches/history/native-current.json` or an
  equivalent checked artifact.
- Website performance data distinguishes LS-Lint comparison rows from
  Assura-native rows.
- The report attributes cold process cost, in-process load, warm query, IPC,
  serialization, and formatting cost where applicable.
- CI or `xtask` can fail material native regressions after the first calibrated
  baseline lands.

Expected final gate:

```bash
target/release/assura performance-report \
  --suite native \
  --output benches/history/native-current.json \
  --history benches/history/native-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
cargo xtask native-performance-no-regression benches/history/native-current.json
cargo xtask performance-no-slower benches/history/current.json
cargo xtask target-state
cargo xtask docs
```

### 5. Optimize The Highest-Impact Rows With Before/After Proof

Goal: improve measured bottlenecks only after attribution identifies them.

Done when:

- `many_configured_scopes_regression` has before/after phase attribution or a
  written reason it is not the first optimization target.
- Cold CLI floor rows (`simple_library`, `web_app`, or current misses) have
  before/after evidence or a bounded explanation.
- Content runtime validation identifies whether schema compilation, file
  indexing, relation validation, diagnostic construction, or serialization owns
  the cost.
- Project-intelligence incremental update behavior is measured before choosing
  any persistent store or partial-index design.
- No accepted LS-Lint-equivalent row becomes slower than native LS-Lint.

Validation:

```bash
cargo xtask performance-no-slower benches/history/current.json
cargo xtask native-performance-no-regression benches/history/native-current.json
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

### 6. Close With Independent Review And Release-Ready Evidence

Goal: make the performance lane reviewable as a product proof, not a local
benchmark experiment.

Done when:

- Docs explain how to interpret LS-Lint comparison, Assura-native rows, cold
  CLI, warm daemon/session, and diagnostic classifications.
- Website data and checked benchmark history agree.
- The PR includes before/after artifacts for optimized rows.
- Independent review confirms gates cannot pass by:
  - hiding slow accepted rows in aggregate speedups;
  - substituting warm daemon/session results for cold CLI parity;
  - reclassifying accepted rows as diagnostic without written approval;
  - relying on local-only Criterion output.

Validation:

```bash
cargo fmt --check
cargo check --workspace --all-targets --all-features --quiet
cargo test --workspace --all-targets --all-features
cargo xtask performance-no-slower benches/history/current.json
cargo xtask native-performance-no-regression benches/history/native-current.json
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Non-Goals

- Do not reopen agent-ready onboarding unless PR #139 review, CI, or a current
  product regression proves it is necessary.
- Do not relax the LS-Lint no-slower gate.
- Do not create a parallel benchmark system disconnected from `benches/`,
  `xtask`, CI, or website data.
- Do not introduce a persistent store, database, or cache dependency before
  proving it beats the in-memory baseline on the same fixtures and packaging
  constraints.
- Do not use a remote bootstrap or host-agent install story to paper over local
  CLI performance or validation gaps.

## Reviewer Blocking Criteria

Block execution if:

- PR #139 is not updated to include local follow-up commits before new product
  work starts.
- The sequence treats completed onboarding as incomplete without naming a
  current blocker.
- Performance claims rely on ad hoc local runs without checked artifacts.
- Aggregate speedups hide a failing accepted LS-Lint-equivalent row.
- Native Assura rows lack cold/warm and phase attribution.
- Docs, website data, and checked JSON disagree.

## Copy/Paste Goal Prompt

```text
Execute docs/goals/assura-post-onboarding-backlog-execution-sequence.md.

Start by revalidating live state: run the Trellis workflow gate, inspect git
status, compare local commits against origin/codex/agent-ready-onboarding-backlog,
and fetch PR #139 status. Do not start new performance work until PR #139 is
updated or explicitly parked.

Then work through the subgoals in order:
1. Publish and revalidate PR #139.
2. Revalidate docs/goals/assura-performance-polish-program.md against current
   performance artifacts.
3. Add the native performance fixture matrix.
4. Produce checked native performance report history and website data.
5. Optimize the highest-impact rows with before/after proof.
6. Close with independent review and release-ready evidence.

For each subgoal, update progress in this goal file with commands run, artifact
paths, and reviewer outcomes. Stop only when the measurable done criteria and
validation commands for that subgoal pass, or when a blocker is recorded with
the exact missing input.
```

## Progress Log

### 2026-07-03 - Iteration 1 Start

- Active task:
  `.trellis/tasks/07-03-execute-post-onboarding-backlog-sequence`.
- Context level: not exposed.
- Live state revalidated:
  - workflow gate ready before task creation;
  - PR #139 open and green on old remote head
    `c0e7b881f866027b087afc0ee6580e83f886f4e2`;
  - local branch ahead of
    `origin/codex/agent-ready-onboarding-backlog` by 10 commits.
- Next action: publish the 10 local commits to PR #139, post a PR handoff
  comment naming the follow-up scope, then wait for and verify checks on the
  new head before starting performance implementation.

### 2026-07-03 - Iteration 1 PR Handoff Fix

- Pushed local follow-up stack to PR #139:
  `c0e7b881f866027b087afc0ee6580e83f886f4e2` ->
  `0bbc3fb91981f092770143de05f6e59755b09597`.
- Posted PR handoff comment:
  <https://github.com/rothnic/assura/pull/139#issuecomment-4877244067>.
- Independent review result: PR comment and pushed-head evidence satisfied
  Subgoal 1 scope requirements, but remote Test Suite checks failed on the
  new head and blocked Subgoal 1 completion.
- Remote failing test evidence:
  `crates/assura-check-cli/tests/compiled_requirements_traceability_cli.rs`
  failed because compiled requirements-traceability artifacts rejected content
  runtime metadata with `invalid compiled config: Found an Option discriminant
  that wasn't 0 or 1`.
- Root cause: compiled artifacts reused content-runtime config structs with
  `skip_serializing_if` serde attributes, which are unsafe for postcard's
  non-self-describing binary format.
- Local fix:
  - added portable content-runtime config structs to the compiled artifact
    boundary;
  - bumped compiled artifact schema version;
  - changed the compiled traceability fixture into a minimal valid modeled
    project that proves enforcement still works.
- Local validation after fix:
  - `cargo fmt --check` passed;
  - `cargo test -p assura-check-cli --test compiled_requirements_traceability_cli --all-features` passed;
  - `cargo test --all-features` passed.
- Next action: run Assura/Trellis docs and evidence gates, commit and push the
  CI fix, then re-check PR #139 on the new head.

### 2026-07-03 - Iteration 1 Windows CI Fixture Fix

- Pushed compiled-artifact fix to PR #139 at
  `64f0135dc2e89b21f642173cf4de7fe3fcf41ced`.
- Remote re-check fixed the original macOS compiled traceability failure, but
  Windows Test Suite failed in `tests/computed_checks.rs`.
- Remote failing test evidence: computed-check fixtures configured only `.sh`
  scripts, so Windows tried to execute them directly and returned
  `%1 is not a valid Win32 application. (os error 193)`.
- Local fix:
  - kept the runtime behavior unchanged;
  - added paired `.cmd` fixture scripts and explicit `windows_script` config
    entries so the integration tests exercise the supported Windows path.
- Local validation after fix:
  - `cargo fmt --check` passed;
  - `cargo test --test computed_checks --quiet` passed;
  - `cargo run --quiet -- check --format json .` passed.
- Next action: run the broader Rust validation, commit and push the Windows
  fixture fix, then re-check PR #139 on the new head.

### 2026-07-03 - Iteration 1 Windows Cmd Path Fix

- Pushed Windows fixture fix to PR #139 at
  `32762fc0f93447fd881a0e1cfe3b8f707704a28c`.
- Remote re-check removed the `.sh` spawn error, but Windows Test Suite still
  failed in `tests/computed_checks.rs` with computed-check `nonzero_exit`.
- Remote failing test evidence: `.cmd` fixture scripts launched through
  `cmd.exe` received canonical temp paths with a Windows extended-length
  `\\?\` prefix, and `cmd.exe` returned `The system cannot find the path
  specified.`
- Local fix:
  - kept computed-check policy semantics unchanged;
  - normalized extended-length `\\?\` and `\\?\UNC\` paths before passing
    `.cmd`/`.bat` scripts to `cmd.exe`;
  - launched batch scripts through `cmd.exe /D /C call` so literal arguments
    still flow to the fixture script.
- Next action: rerun focused computed-check and full validation gates, commit
  and push the runtime Windows path fix, then re-check PR #139 on the new head.

### 2026-07-03 - Iteration 1 Windows Cmd Argument Fixture Fix

- Pushed Windows path fix to PR #139 at
  `8d9dbd335742431bca50c3f8147e7e5e2b81c391`.
- Remote re-check proved the path fix worked: Windows computed-check tests no
  longer failed to start `.cmd` scripts and no longer reported path lookup
  failures.
- Remaining remote failure: only
  `computed_check_passes_args_literally_and_sends_versioned_stdin` failed,
  with `.cmd` fixture exit code 11 from the Windows-only argument assertion.
- Local fix:
  - kept Unix fixture and runtime behavior unchanged;
  - kept the first semicolon-containing argument exact;
  - changed the Windows `.cmd` fixture to validate the spaced argument through
    `cmd.exe`'s `%*` representation, which may preserve or drop quotes around
    the logical argument while still proving no shell expansion occurred.
- Next action: rerun focused computed-check gates, commit and push the fixture
  assertion fix, then re-check PR #139 on the new head.
