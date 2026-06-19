# Goal 13 Install, Release, And Performance Certainty

## Goal

Make Assura's install, release, and performance claims reproducible from current
checked evidence after the Iteration 02 notation and support-policy work.

## What I Already Know

- Goal 13 is the next planned Iteration 02 goal after Goal 12 merged in PR #54.
- Goal 12 now enforces the support/test matrix through `cargo xtask
  target-state`, including current release/performance drift checks.
- The tracked performance artifacts are stale relative to the current branch:
  `benches/history/current.json` and
  `website/public/data/performance/current.json` report timestamp
  `2026-06-02T01:07:10Z`, commit
  `56937c6f688dd4182b195363569b1a4dbeb8f815`, branch
  `codex/phase-01-goal-04-incremental-check-engine`, and
  `source_worktree_dirty: true`.
- Current target-state release/performance checks already verify some archive
  names, install scripts, and performance JSON shape, but the release-sync rule
  goal is still planned and not yet a complete deterministic release contract.
- The performance-reporting skill requires target artifacts before touching
  tracked history, same-machine before/after evidence for optimizations, and
  clear separation between cold `claim_summary` and warm `warm_claim_summary`
  claims.

## Revalidation Result

`valid`: live repository state still has the Goal 13 gap. Install and release
smoke CI is green on PR #54, but current public performance artifacts are stale
and release-sync enforcement is only partial. Goal 13 should start by
refreshing evidence and tightening deterministic release/performance claim
checks before considering any optimization.

## Scope

- Refresh or explicitly supersede current performance artifacts with evidence
  generated from the current post-Goal-12 codebase.
- Use target-only performance artifacts before updating tracked benchmark and
  website data.
- Separate LS-Lint compatibility performance claims from Assura-native notation
  robustness and reusable `rules:` modularity claims.
- Add or tighten deterministic release-sync checks across release notes,
  installers, workflow asset matrices, website install copy, support policy,
  compatibility docs, package metadata, and checked performance JSON.
- Produce a release/performance review artifact that ranks bottlenecks,
  identifies stale or current claims, and records any accepted bounded costs.
- Implement only correctness-preserving optimizations backed by before/after
  evidence.

## Non-Goals

- No speculative performance rewrite without measured before/after evidence.
- No 1.0 release or semver-support commitment.
- No broad website redesign beyond release/performance copy accuracy.
- No plugin marketplace, hosted service, or dependency graph support expansion.

## Acceptance Criteria

- [ ] Current performance artifacts are refreshed or explicitly documented as
      not requiring refresh, with commit, branch, dirty-state, environment, and
      iteration evidence.
- [ ] Public performance copy and website data point at current checked
      artifacts rather than stale historical reports.
- [ ] Release docs, installers, release workflow asset names, checksums,
      support policy, compatibility docs, website install copy, and package
      metadata agree through a deterministic check.
- [ ] Install smoke evidence uses the same release/support claims published by
      docs and website copy.
- [ ] LS-Lint-compatible performance claims stay separate from Assura-native
      notation robustness and reusable-rule modularity claims.
- [ ] Any performance optimization includes same-machine before/after evidence
      and correctness tests.
- [ ] The PR links Goal 13 and an independent release/performance review
      artifact.

## Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo xtask target-state
cargo xtask docs
cargo run --quiet -- performance-report --output target/performance/current.json
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm performance-reporting work follows
  `.agents/skills/assura-performance-reporting/SKILL.md`.
- R1: Review refreshed performance artifacts for current commit, environment,
  dirty-state, and claim labels.
- R2: Review any optimization against before/after correctness and performance
  evidence.
- R3: Review release-sync checks against release notes, installers, website
  copy, package metadata, and CI/release workflows.
- R4: Confirm LS-Lint compatibility claims remain separate from
  Assura-authored notation robustness claims.
- R5: Review notation use-case matrix performance against LS-Lint-style and
  Assura-native cases when applicable.
- R6: Confirm independent review covers both efficiency and reusable-rule
  modularity claims.
- R7: Confirm the PR links this goal and release/performance review artifacts.

## Technical Notes

- Primary execution contract:
  `docs/goals/assura-goal-13-performance-and-release-evidence-governance.md`.
- Iteration context:
  `docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md`.
- Release-sync planned rule:
  `docs/goals/assura-rule-release-sync.md`.
- Target-state release/performance checks currently live under
  `cargo xtask target-state`.
- Performance workflow:
  `.agents/skills/assura-performance-reporting/SKILL.md`.
