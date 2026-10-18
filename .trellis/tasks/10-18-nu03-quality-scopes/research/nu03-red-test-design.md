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
  not execute/record native commands. Use an installed available pre-commit in
  a disposable fixture if present; keep any shim outside the proof used to
  claim real composition.

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

The quality planner matches configured positive paths directly and has no
per-scope denylist. Its special trailing `/**` handling is a prefix match, so
`app/**` admits `app/.next/cache/chunk.js` and `lib/**` admits
`lib/node_modules/pkg/index.ts`. Top-level structural `exclude` rules do not
apply to quality planning. Therefore explicit source roots plus top-level
negative tests cannot establish the acceptance that nested generated and
dependency paths stay excluded.

Do not claim that the renderer-only expansion satisfies this acceptance or
silently apply structural exclusions to all authored quality scopes. A
per-scope quality exclusion contract is the smallest general capability that
appears to express the requirement, but it changes planner/schema behavior and
is outside the current bounded renderer slice. Hold implementation of
generated-safe broad roots pending coordinator scope decision. The renderer
remains the lowest-overlap target while NU02 owns shared onboarding
preservation work.
