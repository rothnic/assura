# Address PR 12 review comments

## Goal

Address the unresolved GitHub review comments on PR #12 by fixing comments that
block LS-Lint compatibility claims, applying small low-risk performance fixes,
and replying on each PR thread with whether the item was fixed or captured for
follow-up.

## What I Already Know

- PR #12 is `codex/ls-lint-coverage-real-repo-proof` against `master`.
- Gemini added five unresolved inline review threads on 2026-05-29.
- Two review threads claimed `.js` should match `BadName.test.js` unless a more
  specific `.test.js` rule exists. Native `ls-lint v2.3.1` does not behave that
  way; `.js` matches `BadName.js`, while `.test.js` is required for
  `BadName.test.js`. Assura should preserve the exact segment-count behavior.
- One compatibility thread is valid: multiple Assura-converted LS-Lint config
  documents need recursive nested `ls` merging to preserve sibling nested rules.
- One thread suggests using Rayon's default pool instead of creating a new
  jwalk Rayon pool per fast parallel check.
- One thread flags repeated full-tree wildcard-scope existence scans in
  `configured_structure.rs`; this needs a larger traversal-state design to fix
  without duplicating walkers or changing semantics.

## Requirements

- Preserve exact multipart-extension segment matching for LS-Lint parity.
- Make the multi-extension coverage test explicit that `.js` does not catch
  `BadName.test.js`.
- Deep-merge nested `ls` mappings across multiple LS-Lint config files.
- Add test coverage for nested multi-config merge behavior.
- Apply the low-risk Rayon default-pool change if tests and clippy accept it.
- Capture the wildcard-scope traversal concern for follow-up if it is not
  fixed in this PR.
- Reply to every review thread with the action taken.

## Acceptance Criteria

- [x] Compatibility blocker comments are fixed in code and tests, or rejected
      with native LS-Lint evidence when the review claim was incorrect.
- [x] Any intentionally deferred comment is captured in a durable goal or docs
      artifact and explained on the PR thread.
- [x] Local checks run, with exact blockers recorded if runner/tooling limits
      prevent a full gate.
- [x] Changes are committed and pushed to PR #12.
- [x] Every review thread has a PR reply.

## Out Of Scope

- Rewriting configured-structure traversal architecture unless a small safe
  patch emerges during implementation.
- Waiting for GitHub hosted runners if quota is exhausted.
- Resolving GitHub review threads directly; replies are enough unless the user
  separately asks for thread resolution.

## Technical Notes

- Relevant spec: `.trellis/spec/assura/index.md`.
- Structure contracts: `.trellis/spec/assura/structure-enforcement.md`.
- Tooling/check policy: `.trellis/spec/assura/tooling-stabilization.md`.
- PR thread ids are available through the GitHub connector for PR #12.
