---
title: Beta release evidence wording cleanup
status: active
---

# Beta Release Evidence Wording Cleanup

## Objective

Remove a stale wording edge from the completed beta master goal's final release
evidence row after the release-evidence archive commit advanced `origin/master`.

## Scope

- Keep the beta completion status unchanged.
- Clarify that `v0.2.0` peels to the PR #112 merge commit.
- Preserve the final release, readiness, live URL, CI, and reviewer evidence.

## Acceptance

- The beta master goal no longer says live `origin/master` points at the
  release merge commit.
- Assura self-check, docs, evidence, target-state, and whitespace gates pass.
