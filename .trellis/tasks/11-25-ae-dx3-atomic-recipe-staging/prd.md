# Atomic built-in recipe staging

Beads issue `ae-dx3` under `ae-e6t` owns priority, live state and closure.

## Acceptance

The built-in `config add-recipe` command uses the existing atomic config writer and a separate staging path per attempt. A failed stage preserves the complete original config and leaves starter files untouched. Dry run, conflict and force semantics and starter-file materialization order stay intact. Verify the recipe command and onboarding with existing tests, plus a deterministic staging failure followed by a successful attempt. Any concurrency claim is limited to staging.
