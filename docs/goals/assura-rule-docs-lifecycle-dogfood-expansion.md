---
id: goal-assura-rule-docs-lifecycle-dogfood-expansion
type: goal
title: Assura docs lifecycle dogfood expansion
status: completed
created: 2026-06-19
owners:
  - assura-maintainers
related:
  - docs/goals/assura-rule-docs-lifecycle-stale-claims.md
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/config-notation.md
  - docs/support-policy.md
  - docs/compatibility-and-surface.md
  - docs/release-notes.md
  - docs/release-candidate-checklist.md
  - docs/validation.md
---

# Assura Docs Lifecycle Dogfood Expansion

## Objective

Broaden Assura's own `extensions.docs_lifecycles` policy from the first narrow
proof slice into a practical dogfood rule for current release, support,
validation, roadmap, and performance-facing documentation.

The goal is to prove the merged docs lifecycle rule can keep current active
docs honest without turning historical analysis into current truth or requiring
manual broad cleanup first.

## Revalidation Result

`valid`.

PR #78 merged the first reusable docs lifecycle/stale-claim detector, PR #79
archived that implementation task, and PR #80 synced the completed roadmap
state. Live target-state evidence still classifies dense active docs/history as
partially aligned because the merged dogfood policy currently covers only the
roadmap and the completed first-slice goal.

The next valid slice is configuration and evidence expansion, not new rule
mechanics, unless implementation proves an existing rule capability is missing.

## User Certainty Bar

A maintainer should be able to edit current release/support/performance docs and
get an actionable `assura check` failure when:

- a configured active doc is missing lifecycle status;
- an active doc links to archived history without an explicit historical
  exception;
- a configured archive-name, release, or performance claim appears without a
  declared current evidence file; or
- the policy itself overreaches into historical analysis that should remain
  preserved context.

## Current Gap

The current `.assura/config.yml` policy `active_goal_docs` covers only:

- `.trellis/spec/assura/roadmap.md`; and
- `docs/goals/assura-rule-docs-lifecycle-stale-claims.md`.

Target-state evidence still identifies current docs and website-facing claims as
drift-prone:

- `docs/project-memories.md` summarizes the current product baseline and
  release claims.
- `docs/release-notes.md` lists supported commands and installable archives.
- `docs/support-policy.md` classifies supported, experimental, internal, and
  roadmap surfaces.
- `docs/compatibility-and-surface.md` maps release archive names to workflow
  evidence.
- `docs/release-candidate-checklist.md` repeats release archive expectations.
- `docs/validation.md` names required evidence and validation gates.
- Website reference docs expose performance and release-readiness claims to
  users.

## First Implementation Slice

Expand `extensions.docs_lifecycles` dogfood coverage with explicit configuration
for a bounded set of active docs:

- `.trellis/spec/assura/roadmap.md`;
- `docs/project-memories.md`;
- `docs/release-notes.md`;
- `docs/support-policy.md`;
- `docs/compatibility-and-surface.md`;
- `docs/release-candidate-checklist.md`;
- `docs/validation.md`;
- `docs/analysis/2026-06-09-assura-best-practice-target-state.md`;
- this goal file; and
- selected website reference docs only if their evidence paths are stable.

Candidate claim patterns and evidence files:

| Claim Pattern | Evidence Files |
| --- | --- |
| `assura-linux-amd64.tar.gz` | `.github/workflows/release.yml`, `docs/compatibility-and-surface.md`, `docs/release-notes.md` |
| `assura-linux-musl-amd64.tar.gz` | `.github/workflows/release.yml`, `docs/compatibility-and-surface.md`, `docs/release-notes.md` |
| `assura-macos-arm64.tar.gz` | `.github/workflows/release.yml`, `docs/compatibility-and-surface.md`, `docs/release-notes.md` |
| `assura-macos-amd64.tar.gz` | `.github/workflows/release.yml`, `docs/compatibility-and-surface.md`, `docs/release-notes.md` |
| `assura-windows-amd64.zip` | `.github/workflows/release.yml`, `docs/compatibility-and-surface.md`, `docs/release-notes.md` |
| `performance-report` | `src/cli/performance_report/mod.rs`, `docs/support-policy.md`, `docs/release-notes.md` |

The implementation should first run the candidate policy locally and record any
false positives. If a claim token is too broad for the current tokenizer, narrow
the configured active docs or choose a more specific deterministic token rather
than weakening the rule. Do not configure phrase-style claim patterns until the
rule explicitly supports phrase matching; the current contract is literal tokens
or glob-style token patterns.

The `2x` performance claim remains out of the first executable slice. Current
performance JSON evidence stores that concept under `two_x_*` keys, so a literal
`2x` policy would fail under the token matcher even when current evidence exists.
Cover `2x` only after adding explicit alias/phrase evidence semantics or after
choosing a committed evidence artifact that contains the exact configured token.

## Scope

- Update `.assura/config.yml` docs lifecycle policy to cover the bounded active
  doc set.
- Add or adjust frontmatter status only for docs included in the policy.
- Add deterministic claim patterns only when evidence files are committed and
  current.
- Add historical exceptions for `docs/archive/**` and any explicit current-doc
  historical reference patterns needed to preserve durable history.
- Update support/notation docs only if the policy expansion exposes a confusing
  rule contract.
- Add regression tests only if implementation finds a missing reusable behavior
  in `extensions.docs_lifecycles`.

## Non-Goals

- No broad natural-language stale-claim detection.
- No automatic archival, deletion, or rewriting of historical analysis.
- No wholesale cleanup of `docs/analysis/**`.
- No new release-sync, support-matrix, manifest-semantics, or module-topology
  rule mechanics unless current docs lifecycle expansion proves a blocker.
- No remote GitHub or release API checks.

## Definition Of Done

- `.assura/config.yml` includes a broader docs lifecycle dogfood policy for the
  selected active docs.
- Included active docs have accepted lifecycle statuses.
- Configured claim tokens have current evidence files and produce actionable
  `assura check --format json` diagnostics when broken in a fixture or focused
  regression.
- Historical docs can remain in `docs/archive/**` without becoming current
  truth.
- Existing `cargo xtask target-state`, evidence, release, support-matrix,
  release-contract, test-relationship, and module-topology gates are not
  weakened.
- Roadmap and target-state docs are updated after merge to route the next
  candidate based on the new self-check output.
- Independent review confirms the dogfood policy is bounded and does not create
  broad cleanup debt.

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

Docs-only changes may use the scoped gate policy, but any Rust/test change must
run the full Rust gates above before PR.

## Review Tasks

- R0: Confirm the expanded policy names explicit active docs and evidence files
  rather than broad `docs/**` globs.
- R1: Review claim patterns for token-boundary false positives, especially
  `2x` and release archive names.
- R2: Confirm historical exceptions preserve archived analysis without masking
  stale current docs.
- R3: Confirm diagnostics name policy id, active file, claim id, and expected
  evidence files.
- R4: Confirm target-state, release-sync, support-matrix, and evidence gates are
  not weakened or duplicated.
- R5: Confirm the implementation leaves a concrete next route after merge.

## Reviewer Blocking Criteria

Block the PR if the policy uses broad unbounded active globs, if release or
performance claim patterns can match unrelated prose, if evidence files are not
current committed artifacts, if historical material becomes impossible to keep,
or if existing target-state/evidence checks are weakened.

## Progress Log

- 2026-06-19: Created after docs lifecycle first slice merged in PR #78, task
  archive PR #79 merged, and completion sync PR #80 routed follow-up work to
  broaden dogfood coverage from live target-state evidence.
- 2026-06-19: Started implementation task
  `.trellis/tasks/06-19-docs-lifecycle-dogfood-expansion-implementation` on
  branch `codex/docs-lifecycle-dogfood-expansion`; revalidated scope against
  current `.assura/config.yml`, roadmap, target-state, and config-notation
  token semantics before editing the dogfood policy.
- 2026-06-19: Expanded `.assura/config.yml` with explicit active docs,
  frontmatter status requirements, release archive claim tokens, and the
  `performance-report` claim token. Left website docs and `2x` out of the first
  implementation slice because their exact evidence boundaries remain
  conditional or deferred by the goal.
- 2026-06-19: Completed first dogfood expansion slice in PR #83
  (`7a56a2c6f6a3ca8b03fa07ed180919211d3638ba`). Hosted scope and evidence
  checks passed. Follow-up remains limited to explicit website docs or `2x`
  coverage only after exact evidence/token boundaries are defined.
