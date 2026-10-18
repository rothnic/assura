# NU04 Getting Started first-run policy

## Goal

Make the published Getting Started first-run instructions executable against a
small real project with the released Assura CLI. The guide must show a policy
that passes after `assura init`, fails when an unexpected root entry is added,
and passes after that entry is removed. Generated agent guidance and the
project-owned recipe output remain present and modeled by the policy.

## Scope

- `website/src/content/docs/guides/getting-started.md`
- This task's Trellis contract and bounded evidence.

No CLI/API, dependency, broad configuration allowance, or generated guidance
change is in scope.

## Acceptance

1. Reproduce the current documented example against a disposable minimal
   project using the durable `/usr/local/bin/assura` v0.4.1 binary; record its
   executable SHA-256 and the exact init/config/check commands and exits.
2. Use the published installer in an isolated temporary bin, without changing
   the global executable. Run `init` with the documented recipes, retain
   generated AGENTS guidance and recipe outputs, and exercise the exact
   documented YAML/check sequence.
3. A clean modeled example exits 0, adding an unexpected root file exits
   nonzero, and removing it restores exit 0.
4. Update only the guide's misleading first-run policy example with the
   smallest coherent correction. Keep the existing generated guidance and
   recipe outputs explicitly represented in project policy.
5. Preserve focused red/green receipts under external review scratch; do not
   commit temporary projects or raw machine-specific evidence.

## Verification

- Disposable CLI journey: published installer/init, guide YAML, clean check,
  unexpected root file, cleanup and check recovery.
- Website docs build or the coordinator's shared docs build, with the exact
  source revision and root recorded.
- Scoped Assura structure/evidence checks required for PR-boundary validation.

## Context

- Base: `origin/master` at `49d85358bae85de04f1ce7faa746b377687dcaf3`.
- Native owner: `/root/nu04_quickstart`; Trellis uses the supported local
  identity alias `nu04_quickstart` for this owner.
- Candidate branch: `fix/nu04-getting-started`.
- No Rust build is needed for frozen docs using the released binary.
