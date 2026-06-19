---
id: goal-assura-rule-docs-lifecycle-stale-claims
type: goal
title: Assura docs lifecycle and stale-claim rule
status: planned
created: 2026-06-19
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/config-notation.md
  - docs/analysis/evidence-and-review-policy.md
  - docs/goals/assura-rule-public-surface-support-matrix.md
  - docs/goals/assura-rule-release-sync.md
  - docs/goals/assura-rule-module-topology.md
---

# Assura Docs Lifecycle And Stale-Claim Rule

## Objective

Create a reusable docs lifecycle and stale-claim detector so active analysis,
goal, support, release, performance, and website docs cannot keep presenting
historical or superseded claims as current truth.

The rule should generalize Assura's current hard-coded docs governance without
replacing proven target-state checks until the reusable config surface covers
the same risk.

## Revalidation Result

`valid`, with narrowed first-slice scope.

The command-surface, release-contract, public-surface support matrix, Cargo
manifest semantics, test-relationship, and module-topology first slices are
complete. The current target-state analysis still classifies dense active
docs/history as misaligned because active docs can accumulate stale roadmap,
support, release, and performance claims.

The valid next product question is whether Assura can provide explicit
docs-lifecycle and stale-claim notation that catches deterministic drift
without pretending to understand arbitrary prose.

## User Certainty Bar

A maintainer should be able to declare which docs are active, historical,
archived, or claim-bearing, then get an actionable `assura check` finding when
an active doc lacks lifecycle metadata, a historical doc appears in an active
surface, or a configured claim token appears without a current owner/evidence
row.

## Current Gap

- `docs/analysis/**` and `docs/goals/**` contain useful durable history, but
  the active/archive boundary is not a reusable Assura rule.
- `cargo xtask target-state` catches selected Assura-specific stale claims, but
  other repositories cannot configure the same pattern through Assura.
- Existing command-surface and support-matrix rules classify structured
  commands and exports; they do not govern active document lifecycle.
- Natural-language stale-doc classification is too broad for a safe first
  slice, but explicit claim terms, lifecycle status, and evidence owners are
  deterministic enough to validate.

## Detector Hypothesis

Validate configured document groups against explicit lifecycle statuses and
configured claim patterns. Report active docs missing lifecycle metadata,
archived/historical docs referenced from active surfaces without an exception,
and configured stale claim tokens that appear without a current evidence row.

## First Slice Config Notation

The first implementation should prefer explicit configuration over prose
inference:

```yaml
extensions:
  docs_lifecycles:
    - id: project_docs
      severity: medium
      active:
        - docs/*.md
        - website/src/content/docs/**/*.mdx
      historical:
        - docs/archive/**
      require_frontmatter_status:
        - docs/analysis/*.md
        - docs/goals/*.md
      allowed_statuses:
        - active
        - planned
        - completed
        - archived
        - historical
      claim_patterns:
        - id: release_assets
          pattern: "assura-*.tar.gz"
          evidence_files:
            - docs/release-notes.md
            - .github/workflows/release.yml
        - id: performance_current
          pattern: "2x"
          evidence_files:
            - benches/history/current.json
            - website/public/data/performance/current.json
      historical_exceptions:
        - docs/archive/**
```

The notation is intentionally explicit:

- `active` and `historical` define lifecycle groups.
- `require_frontmatter_status` names docs that must declare lifecycle status.
- `allowed_statuses` defines the accepted local lifecycle vocabulary.
- `claim_patterns` names deterministic claim tokens and their evidence files.
- `historical_exceptions` prevents archived material from failing because it
  preserves old claims as history.

## First Slice Scope

- Add explicit docs-lifecycle config notation.
- Validate lifecycle status/frontmatter for configured active analysis and goal
  docs.
- Validate that active docs do not link to historical/archive docs unless the
  link is covered by a configured exception or label.
- Validate configured stale-claim patterns in active docs against declared
  evidence files.
- Add fixtures independent of Assura's docs for active docs, historical docs,
  missing lifecycle metadata, stale claim tokens, and exception handling.
- Dogfood the first slice on a narrow Assura surface without weakening
  existing `cargo xtask target-state`, evidence, support, release, performance,
  command-surface, or module-topology checks.

## Non-Goals

- No broad natural-language classifier for arbitrary stale prose.
- No automatic archival or deletion of existing docs.
- No replacement of current Assura-specific `cargo xtask target-state` checks
  in the first slice.
- No remote URL, GitHub API, or release inspection.
- No guarantee that every old claim is semantically false; the rule only
  governs configured lifecycle and claim contracts.

## Required Examples

- Passing: active goal with allowed lifecycle status and current evidence row.
- Passing: archived analysis preserving old claims under a historical
  exception.
- Failing: active analysis doc missing lifecycle status.
- Failing: active website doc mentions a configured release or performance
  claim token without the declared evidence file.
- Failing: active roadmap links to an archived/historical doc without an
  exception label.

## Definition Of Done

- Docs-lifecycle notation is documented before implementation.
- Passing fixtures cover active, completed, archived, and historical docs.
- Failing fixtures cover missing lifecycle status, forbidden active-to-archive
  references, and configured stale claim tokens without evidence.
- `assura check --format json` reports actionable docs-lifecycle violations
  with file, policy id, claim id or lifecycle group, and expected evidence.
- Assura self-check dogfoods the rule on a narrow current surface without
  weakening existing target-state/evidence checks.
- Independent review confirms the rule is reusable outside Assura and does not
  pretend to infer arbitrary prose meaning.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

## Review Tasks

- R0: Confirm the rule uses explicit configured document groups and claim
  patterns rather than hard-coded Assura paths.
- R1: Review lifecycle status vocabulary and frontmatter behavior for reuse
  outside this repository.
- R2: Review stale-claim pattern matching for false positives and bounded
  behavior.
- R3: Review historical/archive exception handling so durable history can remain
  in the repo without being treated as current truth.
- R4: Confirm diagnostics identify the file, policy id, claim id or lifecycle
  group, and missing evidence/exception.
- R5: Confirm existing `cargo xtask target-state` docs/release/performance
  checks are not weakened or removed in the first slice.

## Reviewer Blocking Criteria

Block the PR if the lifecycle boundary is implicit, if the rule hard-codes only
Assura docs, if archived material cannot preserve historical claims, if claim
matching behaves like unbounded prose inference, if existing target-state or
evidence checks are weakened, or if diagnostics do not identify the exact file
and owning policy.

## Tests

Add independent fixture docs trees plus CLI integration coverage for lifecycle
status, active-to-archive references, configured stale claim tokens, and
historical exceptions.

## Progress Log

- 2026-06-19: Created after Module Topology Rule first slice merged in PR #72,
  archive PR #73 merged, and planning sync PR #74 routed the roadmap to docs
  lifecycle/stale-claim detection. Result: valid with a narrowed first slice
  based on explicit lifecycle metadata, configured claim patterns, and evidence
  owners rather than broad natural-language inference.
