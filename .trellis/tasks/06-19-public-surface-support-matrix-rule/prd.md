# Public Surface Support Matrix Rule

## Goal

Implement the first reusable support-matrix rule slice so Assura can report
public commands and Rust exports that are not classified as supported,
experimental, internal, roadmap, or unsupported.

## What I Already Know

- The command-surface documentation rule is completed and already dogfooded
  through `.assura/command-surface.yml`.
- Release Contract Rules first slice merged in PR #59 and the Trellis task was
  archived in PR #60.
- The current target-state analysis lists public-surface support matrix as the
  first remaining P0 detector.
- `src/lib.rs` exposes internal/experimental modules with comments, but there
  is no reusable Assura rule that checks those exports against a support
  matrix.

## Requirements

- Add explicit support-matrix configuration notation.
- Support a stable status vocabulary:
  `supported`, `experimental`, `internal`, `roadmap`, and `unsupported`.
- Check command families from an existing command-surface contract.
- Check configured Rust source files for public module/export surfaces.
- Report unclassified surfaces with file, surface, and matrix context.
- Dogfood the rule in this repository without weakening existing
  command-surface docs validation.

## Acceptance Criteria

- [ ] Passing fixture covers a supported command and an internal/experimental
      Rust export.
- [ ] Failing fixture covers an unclassified command family.
- [ ] Failing fixture covers an unclassified public Rust export.
- [ ] `assura check --format json` includes actionable support-matrix
      diagnostics.
- [ ] Assura self-check passes with the new rule enabled.

## Definition Of Done

- Rust format, tests, clippy, target-state, self-check, evidence, and
  whitespace gates pass.
- Review agent checks notation, diagnostics, false-positive risk, and
  reusability.
- PR is opened, hosted checks pass, review comments are addressed, PR merges,
  and the task is archived.

## Out Of Scope

- Semver stability guarantees for pre-1.0 exports.
- Full Rust AST parsing unless source scanning proves inadequate.
- Manifest metadata enforcement.
- Broad stale-doc natural language classification.
- Replacing the completed command-surface docs rule.

## Technical Notes

- Canonical goal: `docs/goals/assura-rule-public-surface-support-matrix.md`.
- Target-state analysis: `docs/analysis/2026-06-09-assura-best-practice-target-state.md`.
- Existing command contract: `.assura/command-surface.yml`.
- Public export starting point: `src/lib.rs`.
