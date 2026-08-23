# Repair merge diff hygiene

## Goal

Remove the two trailing blank-line errors that prevent the landing PR from
passing `git diff --check` before merge.

## Scope

- Remove only the reported trailing blank lines.
- Preserve all page content and behavior.

## Acceptance Criteria

- `git diff --check origin/master...HEAD` succeeds.
- The already-green PR checks remain green.

## Out of Scope

- Landing-page design or copy changes.
- Product, configuration, CI, or benchmark changes.
