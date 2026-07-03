# Agent Lifecycle Hooks And Next Actions

## Goal

Connect the existing Assura agent feedback, nudge, integration, and severity
contracts into the first-run agent-ready onboarding experience so a new
repository can see explicit nudge, warn, and gate lifecycle profiles and a
ranked next-action list without implying hidden host-agent mutation.

## What I Already Know

- Parent program: `docs/goals/assura-agent-ready-project-onboarding-program.md`
  is the active P0 adoption lane.
- Child goal: `docs/goals/assura-agent-lifecycle-hooks-next-actions.md`
  remains planned and targets nudge/warn/gate semantics plus ranked next fixes.
- Related lifecycle goal `docs/goals/assura-agent-integration-lifecycle.md`
  is completed; `assura agent integration install|update|remove|status|doctor`
  already generates reviewable bundles for Codex, OpenCode, Claude, and Pi.
- The beta severity contract is completed. `low` is advisory while
  `medium`, `high`, and `critical` are blocking unless `--warn` is used.
- Current support docs mark `assura check --format agent`, `--agent codex`,
  `assura hooks`, `assura agent nudge`, and `assura agent integration` as
  supported or experimental surfaces with no per-agent validation engines.
- Current onboarding installs a broad baseline and optional host-agent bundle,
  but its lifecycle state is a prose integration detail and `next_actions` is a
  simple string list.
- Current agent nudge output already sorts findings by severity and path, and
  includes suggested commands for bounded follow-up diagnostics.

## Revalidation Result

Status: valid.

Live evidence says the missing slice is not a new hook engine. The valid gap is
to make the existing lifecycle contracts explicit in onboarding artifacts and
agent-facing output:

- `cargo run --quiet -- check --format agent --agent codex .` passes.
- `.trellis/spec/assura/roadmap.md` still lists Agent-Ready Project Onboarding
  as the active roadmap iteration and points at the parent goal.
- `docs/support-policy.md`, `docs/compatibility-and-surface.md`, and
  `website/src/content/docs/reference/agent-feedback.md` already constrain the
  current command-surface truth.
- `src/cli/agent_onboarding.rs`, `src/cli/agent_onboarding_templates.rs`,
  `src/cli/agent_nudge.rs`, and `src/cli/agent_integration_bundle.rs` contain
  the reusable implementation seams for this slice.

## Scope

- Add explicit lifecycle profiles for:
  - agent working-loop nudges;
  - pre-commit warning/advisory checks;
  - pre-push or CI gate checks.
- Surface those profiles in `assura agent onboard` JSON/YAML/text output and
  generated `.assura/onboarding/agent-next.md` or adjacent onboarding packet
  files.
- Preserve manual host-agent wiring for integration bundles. Installing a
  bundle must remain reviewable and reversible.
- Add ranked next-action objects with priority, action text, affected paths
  when known, and follow-up commands.
- Prove advisory mode exits successfully while gate mode blocks on configured
  errors by testing existing check severity behavior through the agent/lifecycle
  contracts.

## Non-Goals

- No hidden mutation of Codex, Claude, OpenCode, Pi, or global agent configs.
- No new per-agent validation command or `--format <agent>-hook` value.
- No daemon requirement for baseline onboarding.
- No domain-specific or domain-specific lifecycle pack.

## Acceptance Criteria

- [ ] `assura agent onboard --format json` includes lifecycle profiles with
  explicit `nudge`, `warn`, and `gate` modes and commands.
- [ ] Generated onboarding files tell agents when to use nudge, warn, and gate
  checks without presenting future CLI behavior as implemented.
- [ ] Ranked next actions are structured objects, not only prose strings.
- [ ] Tests prove warning/advisory feedback can report configured violations
  without failing the process.
- [ ] Tests prove gate-mode checks fail on configured blocking errors.
- [ ] Tests prove generated host-agent integration remains reviewable local
  bundle behavior and does not silently mutate host-agent config.

## Validation Commands

```bash
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo test --test project_intelligence_onboarding --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if lifecycle modes are implicit, if hook profiles hide side effects, if
agent output lacks ranked next actions, if advisory mode can block normal draft
work unexpectedly, or if the implementation revives per-agent feedback
surfaces instead of reusing the shared `assura check --format agent`,
`assura agent nudge`, and integration lifecycle contracts.
