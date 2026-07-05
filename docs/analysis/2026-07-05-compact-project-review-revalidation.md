---
title: Compact Project Review Revalidation
status: active
---

# Compact Project Review Revalidation

## Result

`valid`

The compact project review goal remains the right next product slice after the
performance stop-policy decision. The current Assura repo can pass structure
validation while still requiring a human or agent to run several separate
commands to understand inactive capabilities, recommended missing paths, and
noisy content/reference gaps.

## Live Evidence

- Workflow gate is clean and no prior active task remains after archiving the
  performance-polish execution task.
- Roadmap still names the performance lane and adjacent compact project review
  proof; the performance decision matrix now says to stop cold micro-tuning and
  move effort back to core structure validation quality.
- `cargo run --quiet -- check --format json .` passes with zero violations.
- `cargo run --quiet -- doctor . --format json` reports a passing structure
  check, active content models/collections, inactive repository-reference and
  structure-relationship policy, inactive onboarding packet, and recommended
  missing `docs/process` and `docs/learnings` paths.
- `cargo run --quiet -- content agent-query gaps . --format json` reports
  `unresolved_repository_references = 1086`, which is still too noisy to expose
  as a hard blocker without classification.

## First Slice

Implement the smallest product slice that turns those separate surfaces into
one compact review path:

- reuse existing `check`, `doctor`, and content agent-query truth rather than
  building a parallel validator;
- return bounded JSON and concise text/agent output;
- classify findings as blocking, advisory, inactive, or informational;
- include structure-fit guidance for new files/directories;
- filter or explicitly omit generated/archive/log/benchmark noise from
  reference-gap summaries.

## Keep Out

Do not auto-edit `.assura/config.yml`, do not add another search engine, and do
not make unresolved-reference counts blocking until the noisy source classes are
classified or filtered.
