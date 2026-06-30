---
title: Project intelligence remaining usability goals
status: in_progress
---

# Project Intelligence Remaining Usability Goals

## Objective

Refresh the Project Intelligence Usability planning state after the onboarding
template and context-pack slices completed locally. The output should answer
what remains before the feature is practically usable and convert that answer
into executable goal docs.

## Scope

- Re-evaluate the latest completed slices against the usability program.
- Update the gap analysis so it no longer lists completed onboarding or
  context-pack work as open.
- Keep completed runtime, adoption, real-repo proof, onboarding, and
  context-pack work closed unless live evidence shows drift.
- Split overly broad remaining work into independently executable goals.
- Update roadmap/program routing to point to the next actionable goal.

## Non-Goals

- No runtime implementation in this task.
- No release claim that project intelligence is fully usable.
- No hosted, remote, or provider-specific dependency.

## Acceptance Criteria

- [x] The gap evaluation names completed evidence and remaining blockers.
- [x] The ordered goal set is current and executable.
- [x] Transport work is split enough for independent review and validation.
- [x] Roadmap and goal metadata route the next agent to the right next goal.
- [x] Planning validation passes with workflow gate, Assura self-check,
      docs evidence, and whitespace checks.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex --task .trellis/tasks/06-29-project-intelligence-usability-remaining-goals
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```
