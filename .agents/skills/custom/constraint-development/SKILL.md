---
name: constraint-development
description: "Develop Assura first-party validation policies. Use when adding or changing structure rules, configured extensions, computed checks, or custom constraint behavior."
---

# Constraint Development

Assura's current extension surface is config-driven and first-party. Prefer
`structure:` captures, `exists`, `needs`, and `provides` for common repository
relationships before adding `extensions.*` policy.

## Workflow

1. Identify whether the policy belongs in `structure:` or a first-party
   `extensions.*` family.
2. Read `.trellis/spec/assura/config-notation.md` before changing notation or
   extension contracts.
3. For new or changed `extensions.*` fields, update source config structs,
   validation, generated onboarding/templates, docs, fixtures, and compiled
   artifact coverage as one change.
4. Add passing and failing tests for the policy. Include CLI coverage when the
   behavior reaches `assura check`.
5. Run `cargo fmt --check`, focused tests, `cargo run --quiet -- check --format
   json .`, and `cargo xtask target-state` when public docs or examples change.

## Read as needed

| When | Read first |
| --- | --- |
| Changing config notation or first-party extension contracts | `.trellis/spec/assura/config-notation.md` |
| Changing closed-world structure checks | `.trellis/spec/assura/structure-enforcement.md` |
| Checking public extension boundaries | `docs/extension-api-boundaries.md` |

## Outputs

- Updated config model/check implementation, or a decision that existing
  notation already covers the case.
- Tests proving valid and invalid cases.
- Docs or generated examples when the user-facing surface changes.

## Guardrails

- Do not describe or build a public plugin API; it is deferred.
- Do not add arbitrary shell execution. Use `extensions.computed_checks` only
  for configured local script-backed findings.
- Do not revive the old trait-registration examples as user guidance.
- Keep common relationships in `structure:` unless a first-party extension is
  clearly the right boundary.
