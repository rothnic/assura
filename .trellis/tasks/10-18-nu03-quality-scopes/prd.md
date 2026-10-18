# Goal

Correct generated Bun/Next quality scopes so established application roots
receive their declared native checks, while existing source/test/config
triggers and owner-authored policy remain intact. Prove that native checks
compose with Assura through the already-supported pre-commit runner.

This scoped task executes Beads `ae-7cd` under the active Assura execution
goal. Source evidence is the read-only NU03 report at
`/Users/nroth/workspace/.assura-review-20261017/nu03-research/report.md`
(SHA-256 `621b5655adc373eb9f16b7840d270f9b9ce331a121d12d629e5588df67c4cbad`).
Coordinate shared onboarding tests/call sites with NU02 before implementation.

## Scope

- Use only existing evidence for supported Bun/Next source roots. Retain
  `src/**`, `tests/**`, package manifest and lockfile coverage, along with the
  existing declared lint/test script selection.
- Add the smallest per-scope `exclude_paths` config capability, defaulting to
  empty so existing authored quality scopes retain their current behavior.
  Exclusions override positive matches only within the same scope. Generate
  explicit exclusions only for the Assura-owned Bun scope; do not apply
  structure-level exclusions or new exclusions to other scopes.
- Preserve owner-authored scopes and values through repeated onboarding.
- Keep implementation in the existing quality renderer and only the minimal
  necessary quality-generation call site. Use the existing quality-onboarding
  and runner lifecycle test surfaces.
- Do not add a runner, public API, dependency, module, or broad onboarding or
  hook lifecycle behavior change.

# Acceptance

- Changes under observed `app/`, `components/`, and `lib/` roots select the
  applicable declared native lint/test phases. Existing `src/` and `tests/`
  changes continue to route correctly.
- Generated output and dependency changes (including root and nested `.next/`,
  `dist/`, `coverage/`, `.turbo/`, and `node_modules/` paths) do not select Bun
  native checks. Unrelated paths remain unselected.
- Repeated onboarding preserves existing user-authored configuration and
  quality scope values.
- A disposable fixture using the existing pre-commit runner proves native
  commands run for source changes in stable order with Assura exactly once;
  independent native and Assura failures remain visible. Removing Assura
  leaves native checks running in their original order. No production project
  or global hook configuration is mutated.
- Record raw command receipts, binaries/tool hashes, fixture identities, and
  failure controls before asserting against them. Use the root-provided
  temporary directory and checkout-local Cargo target when the focused lane is
  admitted. Preserve exact commands, exits, and negative-control results.

## Phase constraint

Current phase is source/spec trace, disposable-fixture preparation, and red-test
design only. Do not run Cargo/build/test/runner batches until the coordinator
admits the focused red/green lane after NU02 releases the serialized full Rust
lane.

## Design proposal

The current planner only matches positive path patterns. Add optional
`quality.scopes.<id>.exclude_paths`, validated as relative globs with errors
contextualized to the scope field, compiled once per scope through the existing
`CompiledExclusion` and `is_excluded_rel_with` helpers, and applied only to that
scope's positive matches. Empty or omitted exclusions preserve legacy behavior.
Generate bounded exclusions for the new Bun scope, including root outputs and
nested `.next`/`node_modules` paths. Confirm matcher semantics with nested and
root controls before claiming generated-output coverage.

`QualityScopeConfig` is re-exported by `src/config/config.rs` and is not marked
`non_exhaustive`; adding a field technically breaks external Rust struct-literal
construction. The crate entrypoint and `docs/support-policy.md` explicitly
classify full-CLI Rust modules as unstable internal APIs with no compatibility
guarantee before 1.0. The only in-repository struct literal is in
`src/cli/quality.rs` tests. The supported YAML config remains backward
compatible because the field defaults empty and the quality scope already has
`deny_unknown_fields`. Record the internal literal update; do not claim a
semver-stable Rust API change.
