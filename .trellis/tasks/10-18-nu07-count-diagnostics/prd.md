# NU07 count diagnostics

## Objective

Fix count violations so users see a bounded, deterministic set of matching paths and the exact number omitted, while retaining configured guidance and actual count/limit facts. Aggregate failed count rules into one finding per rule without changing policy allowance, exit, severity, or serialized parent path behavior.

Acceptance authority: active Bead `ae-dc7`, owned by `/root/nu07_count_diagnostics`. The bead is read-only to this implementation; do not edit its receipt.

## Scope

- `src/cli/check/direct_contents.rs`
- New focused integration test module `tests/count_diagnostics_tests.rs`

Use names already collected for direct children; do not enumerate again. Keep the existing count matcher, allowance/refinement behavior, thresholds, severities, parent path, and output schema. Root remains serialized as an empty path; diagnostic messages render root as `.`. Preserve authored custom folder-health guidance and show actual child count and configured maximum. Keep one shared corrective context per failed count constraint. Do not change generic renderer, Fix/Next routing, Doctor, Review, history, public API/schema, or dependencies.

## Acceptance

- Root and nested file and directory count failures include a deterministic sorted sample capped at five matching paths and the exact omitted count when applicable.
- Allowed/refinement patterns remain excluded from zero-count offender paths according to `matches_count_target`.
- Count failures emit one aggregate finding per failed rule, including zero-count directory rules.
- A zero-count directory rule failed by the onboarding fixture's `tmp` child keeps one `exists_count` at serialized parent path `.agents/skills/missing-skill-md`; its message includes `expected 0` and the unexpected path `.agents/skills/missing-skill-md/tmp`, with no duplicate child-path count finding.
- Diagnostic content is consistent across text, JSON, and agent output; all preserve current severity, ordering, and exit behavior.
- Child-limit custom guidance remains present alongside actual child count and configured maximum.
- Missing/minimum count failures report measured count and expectation without inventing offending paths.
- Focused integration proof covers root/nested file and directory counts, allowed refinements, more than five offenders, custom child-limit guidance, text/JSON/agent parity, complete findings/order/severity/exits, and an unrelated policy negative control.

## Validation and evidence

Run only the admitted focused lane on this candidate until the integration owner refreshes the stable base. Record cwd, HEAD, toolchain, exact argv, executable, raw logs and exits including negative controls. Capture the extra discovery step(s) the diagnostic removes; do not claim speed improvement. Run tests with jobs capped at two and isolated disk-backed TMPDIR. Commit the coherent contract before independent review. Full PR validation, source push, Beads closure, and native closure belong to the integration owner.
