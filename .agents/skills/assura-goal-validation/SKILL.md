---
name: assura-goal-validation
description: "Create or revalidate Assura goal docs, especially old goals or goals produced in a separate context, by checking live repo state, current specs, proof gates, and reviewer criteria before execution."
---

# Assura Goal Validation

Use this skill when creating a new `docs/goals/` file, selecting the next goal,
or executing a goal that may be stale because it is old, inherited from another
session, or produced before recent roadmap/spec changes.

## Workflow

1. Run the Trellis workflow gate and inspect `git status --short`.
2. Read the candidate goal, its related roadmap/spec files, and current Trellis
   task state. Do not trust memory or an old goal file by itself.
3. Check whether the goal is already achieved, superseded, duplicated by a
   newer goal, or still valid. Use live repo evidence: merged PRs, current
   specs, docs, tests, benchmark artifacts, and self-check output.
4. If the goal remains valid, refresh it so it has:
   - objective and current gap;
   - user certainty bar;
   - scope and non-goals;
   - definition of done;
   - validation commands;
   - review tasks and reviewer blocking criteria;
   - exact related files and artifacts.
5. For notation goals, require:
   - LS-Lint-equivalent use cases first;
   - Assura-native extension cases beyond LS-Lint;
   - public examples, generated examples, fixtures, and test-case
     `.assura/config.yml` files migrated;
   - no backwards-compatibility shims for removed alpha notation unless an
     explicit support-policy exception and removal plan exists;
   - performance evidence or bounded-cost justification.
6. For older or separate-context goals, record the revalidation result in the
   goal or a `docs/analysis/` artifact before implementation starts.

## Output

Report one of:

- `valid`: cite the evidence that the goal is still needed.
- `refresh-needed`: list the exact missing proof gates to add first.
- `already-achieved`: cite the artifact or PR that completed it.
- `superseded`: name the newer goal/spec that should be used instead.
