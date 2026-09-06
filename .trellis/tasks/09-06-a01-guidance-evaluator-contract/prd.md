# Guidance evaluator contract

## Goal

Extend the independent initialization evaluator so A03 can prove generated
specialization guidance through a fixture-owned, deterministic assertion,
without ever treating absent guidance evidence as a pass.

## Requirements

* Keep the evaluator schema at v1 unless a compatibility-breaking format is
  necessary; add an optional, declarative `guidance_assertions` contract field.
* Each assertion must use a repository-relative safe path and an expected text
  fragment. The evaluator must record pass, fail, or unavailable evidence.
* A requested guidance dimension with no assertions remains unavailable and
  acceptance-ineligible; an assertion with a missing file or fragment fails.
* Evaluate guidance only from the trusted contract and disposable project copy;
  do not execute repository-provided guidance or commands.
* Update the frozen A01 fixtures and evaluator documentation, then rerun A03's
  partial specialization evaluation using the merged candidate binary.

## Acceptance Criteria

* [ ] Focused tests prove a matching guidance assertion passes.
* [ ] Focused tests prove missing and mismatched guidance assertions fail.
* [ ] Focused tests prove uncontracted guidance remains unavailable.
* [ ] Contract validation rejects unsafe or malformed guidance assertions.
* [ ] A03's requested partial dimensions include concrete guidance evidence;
      the result remains partial and not final-acceptance evidence.

## Technical Approach

Add an optional `guidance_assertions` array of `{id, path, contains}` records.
Validate its shape with the existing safe-relative-path helper, inspect files in
the disposable fixture copy, emit `kind: guidance` evidence, and map failed
records to the guidance dimension. This preserves the existing distinction
between unavailable (no contractable evidence) and failed (asserted evidence
does not hold).

## Decision (ADR-lite)

**Context:** Contract v1 names guidance as an evaluator dimension but has no
executable assertion. A03 cannot honestly claim the packet's partial evaluator
proof while that dimension is unavailable.

**Decision:** Add a fixture-owned textual assertion rather than an arbitrary
command or a weak path-only check.

**Consequences:** The evaluator can prove bounded generated guidance facts and
retain its false-green safeguards. It does not evaluate instruction quality,
agent behavior, hooks, or native tools; those remain owned by their cards.

## Out of Scope

* Arbitrary command execution or interpretation of repository instructions.
* Changing A03 behavior beyond its evidence closure.
* Full A07 acceptance, hook lifecycle proof, native gate proof, releases, or
  public communication.

## Technical Notes

* A01 packet: `research/initialization-packet.md#a01`.
* A03 acceptance: `research/initialization-packet.md#a03`.
* Evaluator: `scripts/evaluate-agent-init.py`.
* Existing evaluator tests: `tests/agent_init_evaluator_tests.py`.
