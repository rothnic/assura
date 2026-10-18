# NU03 red-test design (not yet executed)

Source revision: `dafec88d507e8af31cda9306ff29d474bf938732` on
`fix/nu03-quality-scopes`. Source research: global NU03 report SHA-256
`621b5655adc373eb9f16b7840d270f9b9ce331a121d12d629e5588df67c4cbad`.

## Existing surfaces

- `src/cli/agent_onboarding_quality.rs::quality_config` renders the Bun scope;
  it currently emits `src/**`, `tests/**`, `package.json`, and `bun.lock`, and
  retains detected-script routing.
- `tests/quality_onboarding_contract.rs` already owns generated Bun config and
  `quality plan` checks. Extend the declared-script contract to assert the
  complete expected paths, not just the first element, and add a focused plan
  control for each supported root and excluded output.
- `tests/git_hook_lifecycle.rs` already uses the real CLI and pre-commit 4.6.2
  adapter surface, but its current pre-commit shim only returns status and does
  not execute/record native commands. This host currently has neither a
  `pre-commit` executable on `PATH` nor an importable Python `pre_commit`
  module (`command -v pre-commit`; `python3 -c 'import importlib.util;
  print(importlib.util.find_spec("pre_commit") is not None)'`). Do not count
  the shim as composition proof or install a runner before the coordinator
  admits an environment for that proof.

## Planned controls

Positive path matrix: `app/(chat)/page.tsx`, `components/Button.tsx`,
`lib/data.ts`, `src/index.ts`, `tests/example.test.ts`, `package.json`, and
`bun.lock`. Assert frequent and pre-push checks against declared scripts;
also preserve fallback `bun test` when no test script exists.

Negative path matrix: `node_modules/pkg/index.ts`, `.next/server/chunk.js`,
`dist/index.js`, `coverage/report.ts`, `.turbo/cache/item.ts`, nested generated
or dependency paths such as `app/.next/cache/chunk.js` and
`lib/node_modules/pkg/index.ts`, and unrelated `docs/guide.md`. Assert no Bun
checks are selected for each output or unrelated path. Keep `assura check`
behavior independently observable where the policy scope selects it.

Preservation control: seed an existing config with a distinguishable
owner-authored scope and formatting, run onboarding using the documented
preservation option, repeat onboarding, then compare exact config bytes and
parsed scope values. Coordinate with NU02 before sharing the test or call site.

Composition control: in a disposable Git fixture, install/configure the
existing pre-commit runner with recording native lint/test commands and the
Assura entry. Trigger the hook on a source edit; assert command order and one
invocation each. Repeat with native failure and Assura failure independently,
recording both outcomes. Remove Assura through the existing lifecycle and
repeat the source edit to prove native command order and results persist.
Capture raw logs/status/tool identities before assertions. Run only after
coordinator admission; no real project hooks or global configuration are
changed.

## Open implementation decision

The quality planner matches configured positive paths directly. Its special
trailing `/**` handling is a prefix match, so `app/**` admits
`app/.next/cache/chunk.js` and `lib/**` admits
`lib/node_modules/pkg/index.ts`. Top-level structural `exclude` rules do not
apply to quality planning. Explicit source roots plus top-level negative tests
cannot establish nested output/dependency exclusion.

Coordinator-authorized proposal: optional `exclude_paths` on each quality scope,
default empty, overrides only that scope's matching positive paths. Reuse
`CompiledExclusion` and `is_excluded_rel_with` from `src/cli/check/rules.rs`
through a crate-internal bridge in the existing `src/cli/check.rs`; compile the
list once per scope. Do not couple quality matching to structure exclusions or
alter authored scopes. Add nested/root output, legacy omission, invalid-pattern
context, preservation, and hook composition controls.

API boundary evidence: `QualityScopeConfig` is public and re-exported at
`src/config/config.rs`, has no `#[non_exhaustive]`, and has one in-repository
struct literal at `src/cli/quality.rs`. External Rust struct-literal callers
would need to add the field, but `src/lib.rs` and `docs/support-policy.md`
explicitly classify these full-CLI Rust modules as unstable internal APIs with
no pre-1.0 compatibility guarantee. The authored YAML behavior is additive:
the new field defaults empty under `deny_unknown_fields`, preserving all
existing files and scope behavior.

The renderer remains the lowest-overlap implementation target while NU02 owns
shared onboarding preservation work. Do not start the Cargo lane until
coordinator admission.
