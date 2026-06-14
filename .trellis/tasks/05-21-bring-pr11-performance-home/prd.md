# Bring PR 11 Performance Work Home

## Goal

Finish the local PR #11 performance work so the branch is coherent, validated,
and ready for PR update. The work must preserve the narrow evidence story:
Linux static-CRT `assura-check-cli` completes the cold 2x gate, the persistent
session path completes a separate warm/editor-session gate, and local macOS
dynamic rows remain diagnostic.

## What I Already Know

- The branch is `codex/ls-lint-realistic-parity-core-performance`.
- There is a large local dirty layer on top of the pushed PR branch.
- Fresh 2026-06-12 reruns turned green again: `cargo test --all-targets
  --quiet` now passes.
- Both named exact repro commands still pass in isolation: `cargo test
  --quiet cli::check::prepared::tests::prepared_check_reloads_when_config_changes
  -- --exact` and `cargo test --quiet
  cli::check::compiled_artifact_tests::source_fingerprint_detects_same_size_rewrite_on_unix
  -- --exact`.
- The remaining blocker is now the docs / handoff / review batch needed to turn
  the broad dirty branch into one narrow truthful PR-update slice.
- The branch still needs shaping because the current work is a broad docs /
  handoff batch, not yet a narrow reviewable PR update slice, even though the
  canonical verification checkpoint is green again.
- `git diff --check`, `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo run --quiet -- check --format json .` passed in the prior review pass.
- `benches/history/current.json` reports:
  - `claim_summary.two_x_claim_verdict = complete`
  - cold row family `assura-check-cli`
  - 6 / 6 2x passes
  - aggregate speedup `2.8980855186211874`
  - `assura_binary_profile = release-static-crt`
  - `warm_claim_summary.two_x_claim_verdict = complete`
  - warm row family `assura-check-dirty-project-session-cli`
  - warm aggregate speedup `25.213361575198398`
- Some docs still contain stale "not complete" language from earlier macOS
  dynamic and pre-static-CRT experiments.
- The website performance page previously had visual alignment issues; this
  slice must verify desktop and mobile rendering, not just build.

## Requirements

- Narrow the branch to a reviewable PR-update batch while the refreshed green
  verification checkpoint stays truthfully documented.
- Reconcile performance docs into one consistent scoped claim:
  Linux static-CRT cold release evidence is complete; macOS dynamic evidence is
  diagnostic; persistent warm/editor-session evidence is separately complete.
- Keep rejected experiments documented as learnings, but compress or clarify
  stale language so old interim states do not read as current truth.
- Update website performance copy and component logic so the public page uses
  `assura-check-cli` versus native `ls-lint-cli` and clearly labels the Linux
  static-CRT scope.
- Fix performance page layout so summary facts, run context, tables, and
  diagnostic rows align on desktop and stack cleanly on mobile.
- Run a review agent before PR-facing handoff.

## Acceptance Criteria

- [x] `cargo test --all-targets --quiet` passes.
- [x] `cargo test --quiet cli::check::prepared::tests::prepared_check_reloads_when_config_changes -- --exact` passes.
- [x] `cargo test --quiet cli::check::compiled_artifact_tests::source_fingerprint_detects_same_size_rewrite_on_unix -- --exact` passes.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `git diff --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo run --quiet -- check --format json .` passes.
- [ ] `pnpm --dir website build` passes.
- [ ] Website desktop and mobile visual review is recorded with screenshot
      paths or notes.
- [ ] Docs and website do not contain contradictory current claims about the
      cold 2x gate.
- [ ] Review agent findings are considered and either fixed or explicitly
      rejected with rationale.

## Out Of Scope

- New performance experiments.
- Changing the completed cold claim away from Linux static-CRT release
  artifacts.
- Treating warm/editor-session rows as cold CLI headline evidence.
- Pushing or updating PR #11 before local validation and review are complete.

## Technical Notes

- Use `.agents/skills/assura-performance-reporting/SKILL.md` for performance
  report workflow and claim-routing rules.
- Key evidence files:
  - `benches/history/current.json`
  - `website/public/data/performance/current.json`
  - `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md`
  - `docs/analysis/2026-05-19-ls-lint-performance-progress-ledger.md`
  - `docs/analysis/2026-05-19-ls-lint-performance-scope-decision.md`
  - `docs/goals/assura-native-ls-lint-performance-rearchitecture.md`
- Key website files:
  - `website/src/components/performance-evidence.astro`
  - `website/src/content/docs/reference/performance.mdx`
  - `website/src/content/docs/reference/performance-implementation.mdx`
  - `website/src/styles/custom.css`
