# Candidate-bound runner isolation

Use this reference for an agent-driven initializer or evaluator. It is a
phase contract, not a second queue. Keep evaluator contracts and private
fixtures outside the child agent's prompt, working directory, environment and
conversation context.

## Before launch

1. Build or select one immutable candidate from the current source SHA. Record
   its absolute path, canonical target, source SHA, version and SHA-256.
2. Create a fresh source-only fixture containing only the public inputs needed
   for the initializer. Do not copy evaluator contracts, harness directories,
   prior results, coordinator notes or hidden expected-output files into it.
3. Launch a one-shot child with the fixed public task prompt. Do not forward the
   coordinator transcript or inject acceptance assertions that reveal the
   evaluator. A fresh subprocess/session is preferred; if the harness cannot
   isolate context, the run is protocol-invalid and earns no product credit.
4. Expose the candidate through a disposable login-shell-safe launcher or the
   regular candidate executable itself. Use a minimal explicit `PATH` that
   contains the candidate (and required system/toolchain paths) but excludes
   ambient global Assura installations. Never mutate a global binary or startup
   file. A shim must be a regular executable that `exec`s one fixed absolute
   target; aliases, shell functions and opaque symlink chains are invalid.

## Identity canary

Before accepting any product observation, capture from the same command
environment used by the child:

```text
command -v assura
assura --version
sha256 of the resolved executable and of the fixed target
```

Compare those values with the pre-launch identity and record the resolved path
and fixed target separately. A login shell may replace a parent `PATH` prefix;
that is a mismatch, not a harmless detail. If the child invokes a different
absolute binary, observes a global install, or cannot prove the fixed target,
retain the event stream as invalid protocol evidence and do not allocate
screening, holdout or acceptance credit.

The canary must also prove that the selected candidate exposes the expected
surface before the initializer runs. Keep initializer events and evaluator
results separate; the evaluator is invoked only after the initializer exits.

## Product run and disposition

- Use the fixed public prompt and source-only fixture for the fresh canary.
- Run the evaluator afterward with the same absolute candidate path. Store
  private raw results separately from redacted evidence; public records contain
  aggregates and method limitations, never hidden oracle contents.
- Any private-evaluator exposure, forbidden-path read, prompt contamination,
  global-binary fallback, identity mismatch or missing identity observation
  invalidates the run. Preserve it, label it, and repair the runner before a
  fresh canary; never repeat the unchanged method until it happens to pass.
- A valid candidate-bound canary that fails the product contract is useful
  product evidence but receives no screening allocation. Route the concrete
  failure to its owning behavior card. Explicit-route controls are calibration
  only and never substitute for a blinded fresh-agent run.

## Efficient recovery

Identity checks are cheap and precede native suites, evaluator batches and CI.
Use one candidate build per source SHA and retain its hash. When a child or
evaluator is still running, retain the live handle and poll it; do not launch a
second run or end the goal at the checkpoint. If a command is rejected by a
safety guard, record the exact rejection and use a non-destructive disposable
fixture or bounded diagnostic rather than weakening the guard.
