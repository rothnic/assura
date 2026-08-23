---
name: assura-notation-review
description: Review and iterate on Assura config notation without losing scope, diagnostics, performance, or visual clarity.
---

# Assura Notation Review

Use this skill when reviewing or changing hand-authored Assura configuration.
The goal is a project-shaped YAML surface that is concise in common cases and
expands in place for complex policy without reducing configurability.

## Required Inputs

Read these before proposing a change:

1. `.trellis/spec/assura/config-notation.md`
2. `website/src/data/config-examples/*.yml`
3. `src/config/config/structure_notation.rs` and its sibling modules
4. `tests/structure_config_notation_tests.rs`
5. Current mobile and desktop screenshots of the rendered examples

Treat the checked YAML files as the source for rendered marketing examples. Do
not optimize a hand-copied display string independently from executable config.

## Review Order

Review one bounded syntax range at a time:

1. **Punctuation cost**: quotes, braces, commas, repeated sigils, and inline
   JSON-like mappings.
2. **Common-case density**: exact paths, optional paths, naming plus limits,
   reusable node rules, reusable tree rules, and inheritance.
3. **Composition**: one rule, multiple rules, built-in plus project-defined
   rules, local overrides, and readable expansion to the detailed form.
4. **Scope clarity**: direct versus recursive globs, captures, cardinality, and
   the output of `assura explain`.
5. **Failure quality**: unknown rules, type mismatch, cycles, ambiguous syntax,
   and migration guidance.

## YAML Constraints

- A plain YAML scalar cannot begin with `@` or a backtick, so either sigil
  requires quotes. Do not call quote removal a styling-only change.
- Prefer YAML-native mappings and sequences over custom tags, anchors, or a
  broad embedded expression language. Those forms hide semantic identity or
  complicate composition and diagnostics.
- A replacement rule sigil must remain visibly distinct, work unquoted as a
  mapping key and scalar value, and leave glob and capture syntax unambiguous.
- Preserve the detailed nested mapping as the escape hatch. Shorthand must
  normalize to the same internal model instead of creating a second engine.

## Candidate Evaluation

For every candidate, record:

| Dimension | Required evidence |
| --- | --- |
| Before/after | Equivalent executable YAML examples. |
| Quote and line reduction | Count changed quotes, braces, and lines in both canonical examples. |
| Composition | One rule, multiple rules, built-in plus local rule, and local override cases. |
| Scope | Direct, recursive, capture, nested, and inherited cases remain explicit. |
| Diagnostics | Unknown, mismatched, cyclic, and malformed cases name the authored syntax. |
| Configurability | Expanded notation can still express every shorthand result. |
| Performance | Same-host before/after config and accepted LS-Lint fixture evidence. |
| Visual result | Mobile and desktop screenshots with no overflow or unreadable wrapping. |

Do not retain aliases solely for compatibility before 1.0 unless the support
policy explicitly requires them. A short migration diagnostic is preferable to
two long-lived public spellings.

## Iteration Contract

1. Ask an independent reviewer to inspect the current executable examples,
   parser, and screenshots using this skill.
2. Select one bounded range. Record alternatives and why they were rejected.
3. Capture same-host baseline performance before parser or normalization edits.
4. Implement the syntax and focused positive, negative, composition, and
   explain-output tests.
5. Migrate public examples, generated configs, fixtures, built-ins, and docs in
   the same range. Remove the superseded alpha notation.
6. Run focused tests, LS-Lint parity, full regression gates appropriate to the
   changed surface, self-check, docs build, and responsive browser tests.
7. Compare performance with the baseline. A regression must be removed or
   explicitly bounded before the range can be committed.
8. Commit the completed range before requesting another independent review.
9. Repeat from the committed state.

The reviewer returns findings ordered by value, with a concrete before/after
example, affected implementation surfaces, expected tests, performance risk,
and either `retain`, `defer`, or `reject`.

## Stop Condition

Stop the review loop only when an independent reviewer finds no further bounded
improvement, or every remaining option would introduce at least one major side
effect:

- ambiguous YAML or a substantial custom DSL;
- reduced rule composition, scope control, or diagnostic precision;
- a compatibility layer without a current support requirement;
- measurable config/check regression that cannot be removed;
- conflict with an active Assura goal or a second policy engine;
- a visual-only abbreviation that no longer matches executable config.

Record the final rejected/deferred options and evidence in the active task so a
future agent does not restart the same experiments without new constraints.
