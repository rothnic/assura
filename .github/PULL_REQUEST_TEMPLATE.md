## Goal

- Goal doc:
- Active Trellis task:
- Related issue or PR:

## Change contract

- User problem:
- Behavior changed:
- Reproducer (or `not applicable` for copy-only changes):
- Exact validation (commands and exit/results, or `not applicable` with reason):
- Known gaps:

## Policy and assistance

- Policy/dependency/support-promise changes:
- Material AI assistance and how it was verified (no private prompt transcript required):
- Independent review needed? (`test deletion`, `exclusion`, `severity reduction`, `performance threshold`, or `CI scope` changes require it):

## Evidence

- Review record:
- Checked artifacts:
- Generated artifacts in `target/`:
- CI evidence links:

## Validation

<!-- List exact commands reviewers can rerun. Mark commands not run with rationale. -->

- [ ] `cargo xtask fast`
- [ ] `cargo xtask evidence`
- [ ] `cargo xtask docs`
- [ ] Goal-specific validation commands are listed above.
- Exact validation results and any nonapplicable command rationale are recorded above.

## Review Feedback

- Review-agent findings:
- Gemini or other PR review findings:
- Rejected findings and rationale:

## Completion And Handoff

- [ ] Review tasks R0-R5 are complete or explicitly marked not applicable.
- [ ] PR evidence can be understood without chat history.
- [ ] Known baseline issues are linked to `.trellis/spec/assura/tooling-stabilization.md`.
- [ ] Stable agent surface remains `assura check --format agent`, with Codex delivery only through `--agent codex`.
- Next goal:
