# Assura Claim-Complete v0.4 And v1.0 Program

## Goal

Execute `docs/goals/assura-claim-complete-v0-4-and-v1.md` through six ordered
child tasks. v0.4 closes implementation-to-marketing gaps; v1.0 remains open
until the time-based proof is complete.

## Locked Decisions

- v0.4 precedes v1.0.
- CLI-only stable contract.
- Explicit managed activation for Codex, Claude Code, OpenCode, and Pi.
- Core pages promote supported and released behavior only.
- Thirty days, 50 sessions, three repositories, four hosts, and a final 14-day
  freeze are mandatory v1.0 evidence.

## Acceptance Criteria

- [ ] Every child task has complete evidence and independent review.
- [ ] v0.4 is published only after the claim-contract, watch/runtime,
      activation, and policy-depth child tasks pass.
- [ ] v1.0 is published only after the soak task passes every count and duration.
- [ ] The goal progress log records each phase boundary and unresolved risk.

## Validation

Use each child PRD's focused checks. Run `cargo xtask full`, release smoke,
website marketing tests, Assura self-check, and `git diff --check` at release
boundaries.

## Review Blocking Criteria

Block on any unsupported core claim, unreleased promoted command, hidden manual
host step, placeholder runtime, stale release evidence, or incomplete soak.
