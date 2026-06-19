---
id: goal-assura-roadmap-13-performance-and-release-evidence-governance
type: goal
title: Assura roadmap 13 install release and performance certainty
status: planned
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
  - docs/goals/assura-rule-release-sync.md
  - docs/analysis/2026-06-15-notation-clean-start-roadmap.md
  - benches/history/current.json
  - website/public/data/performance/current.json
---

# Goal 13: Install, Release, And Performance Certainty

## Objective

Make install, release, and performance claims reproducible after the notation
and policy depth work, then implement only the optimizations and release-sync
checks proven by current evidence.

This is a two-week team chunk for performance reporting, release-sync
validation, checked artifacts, docs, website data, and reviewer evidence.

## Current Gap

This goal is not complete today because Assura has release docs, install
scripts, and performance artifacts, but there is no post-notation certainty
gate proving that install instructions, release assets, support copy, website
copy, and performance claims still agree after Iteration 02 policy work. Current
artifacts are valuable evidence; they are not a permanent guarantee.

## User Certainty Bar

A new user should be able to install the documented release artifact, run the
documented first check, and trust any performance or support claim because it is
linked to current checked evidence, not stale historical reports.

## Scope

- Refresh performance baselines after canonical notation and relationship
  hardening work.
- Measure the notation use-case matrix from Goal 09 against LS-Lint-equivalent
  cases and Assura-native extension cases.
- Profile parsing, shorthand normalization, relationship validation traversal,
  compiled artifacts, and fast-path disabling.
- Add or enforce a notation-change performance gate that every future notation
  goal can cite.
- Produce a ranked report that separates confirmed bottlenecks from suspected
  opportunities.
- Implement only correctness-preserving performance improvements backed by the
  analysis.
- Advance release-sync checks across version, installer docs, asset names,
  support policy, release notes, website install copy, and release workflows.
- Keep LS-Lint compatibility performance and Assura-authored notation
  robustness as separate claims.
- Require independent review to confirm the notation is effectively as
  efficient as LS-Lint for LS-Lint-style cases while remaining more modular for
  broader Assura policies through reusable `rules:`.

## Non-Goals

- No speculative performance rewrites without before/after evidence.
- No public 1.0 release commitment.
- No plugin marketplace or hosted service launch.
- No broad website redesign beyond evidence and release-copy accuracy.

## Definition Of Done

- Current performance artifacts are regenerated or explicitly documented as not
  requiring refresh.
- LS-Lint-equivalent notation cases meet the agreed performance target, or any
  miss has a bounded, justified, and explicitly accepted follow-up.
- A performance analysis ranks bottlenecks with evidence and identifies any
  deferred opportunities.
- Any implemented optimization has before/after evidence and correctness tests.
- Any notation-related performance limit is explicitly recorded with a bounded
  cost, user-value justification, and follow-up threshold.
- Release docs, installers, support policy, website install copy, and workflow
  asset names agree through a deterministic release-sync check.
- Public performance and release claims point at current checked artifacts.
- The install path used by new users is smoke-tested against the same release
  and support claims that the docs publish.
- An independent review artifact confirms the notation-performance result and
  the modularity claim from reusable rules.

## Required Validation

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

- R0: Confirm performance-reporting work follows the project performance skill.
- R1: Review refreshed performance artifacts for current commit, environment,
  and claim labels.
- R2: Review any optimization against before/after correctness and performance
  evidence.
- R3: Review release-sync checks against release notes, installers, website
  copy, and workflows.
- R4: Confirm LS-Lint compatibility claims remain separate from
  Assura-authored notation robustness claims.
- R5: Review the notation use-case matrix performance against LS-Lint-style and
  Assura-native cases.
- R6: Confirm independent review covers both efficiency and modularity claims.
- R7: Confirm the PR links this goal and performance/release review artifacts.

## Reviewer Blocking Criteria

Block the PR if performance claims cite stale artifacts, if an optimization
lacks before/after evidence, if release docs mention assets the workflow does
not publish, or if release-sync checks rely on manual review only. Also block if
LS-Lint-equivalent notation cases fail the performance target without an
accepted bounded-cost record, or if the reusable-rule modularity claim is not
independently reviewed.

## Progress Log

- 2026-06-19: Revalidated after Goal 12 merged in PR #54. Goal remains valid:
  tracked performance artifacts still point at the older
  `56937c6f688dd4182b195363569b1a4dbeb8f815` run with
  `source_worktree_dirty: true`, and release-sync enforcement is still partial
  even though PR #54 CI proved the current install/release smoke matrix.
