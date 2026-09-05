# Initialization and feedback work packet

Read [global constraints](execution-backlog.md). These solutions use agent judgment plus small editable patterns. They do not add project-intelligence features, a remote template marketplace or a replacement test runner.

## A01

**Outcome:** A repeatable evaluator cannot confuse a permissive config with successful setup. **Create:** `tests/fixtures/agent_init/{rust,typescript,python}/`, `scripts/evaluate-agent-init.py`, `tests/agent_init_evaluator_tests.py`, `docs/analysis/agent-init-evaluation.md`. Reuse existing test-fixture conventions and register new paths.

**Interface:** `evaluate-agent-init.py --project <path> --contract <json> --assura-bin <absolute-path> --output <json>`; this evaluates one completed run, not spawns an agent. A separate orchestrating agent starts isolated runs with the fixed user prompt. Implement with Python standard library; no model SDK dependency.

Optional `--dimensions structure,policy,guidance,preservation,idempotence` supports partial development checks. Allowed dimension IDs are `structure`, `policy`, `guidance`, `hooks`, `native`, `preservation`, `idempotence`. A subset result must carry `verification_scope: partial` and `acceptance_eligible: false`; the unfiltered run checks every contract dimension. Follow-up feature execution is a separate A07 observation. Partial checks cannot satisfy the final acceptance gate.

Contract v1 fields: `schema`, `fixture_id`, `stack`, `required_paths`, `forbidden_paths`, `preserve_hashes`, `positive_probes`, `negative_probes`, `native_commands`, `required_hook_states`. Commands are trusted, fixture-owned argument arrays plus explicit cwd, never arbitrary instructions taken from evaluated repo text. Execute with timeouts and captured output.

- [ ] Freeze fixture intent before any initializing agent sees it: Rust library with `src/lib.rs`/`tests`, TS/Bun utilities with `src`/`test` plus scoped PascalCase component exception, Python package/service cases with explicit pytest/Ruff conventions. Include an existing-config and existing-hook variation per stack.
- [ ] Write evaluator tests before implementation: empty config that passes check must fail a negative probe; valid exception must pass; missing native command is unavailable/fail, not skipped-pass; changed preserved file fails; wrong cwd cannot pass a native test; zero collected tests fails where expected; a partial-dimension pass is ineligible for final acceptance.
- [ ] Implement setup evaluation using a disposable copy. First run baseline checks; then apply each trusted negative mutation separately, execute the authoritative check/hook, capture named rule/exit evidence, and restore before the next probe. Never mutate the user's source fixture or restore with broad destructive Git commands.
- [ ] Emit per-dimension states `pass|fail|unavailable`, critical failures, command evidence and costs. A run with any critical fail/unavailable is not acceptance-pass. Preserve stdout/stderr privately; redact reports intended for publication.
- [ ] Unit command: `python3 -m unittest discover -s tests -p 'agent_init_evaluator_tests.py'`. Validate intentionally broken evaluator-input cases and a known-good hand-configured fixture; only then score agents.

**Accept:** Evaluator rejects a false-green setup and catches preservation/hook/native-test failures independently of agent self-report. Contract/schema and fixed prompt hashes are recorded.

## A02

**Outcome:** Small built-in examples and local project patterns compose without overwriting intent. **Own:** `src/cli/init_support.rs`, `src/cli/agent_args.rs`, `src/cli/agent_onboarding.rs`, `agent_onboarding_templates.rs`, `agent_onboarding_rules.rs`, init arguments in `args.rs`/`commands.rs`; config merge tests. **Create:** `tests/fixtures/agent_init/patterns/`, `docs/agent-patterns.md`, `tests/agent_recipe_file.rs`.

Design: a local pattern is an ordinary Assura YAML fragment containing existing `rules`, `structure`, `exclude` and `quality` keys. No template execution or new expression language. Add `--recipe-file <path>` to init/onboard for explicit local application; existing bundled recipe behavior stays supported. Record origin/hash in `.assura/onboarding/profile-selection.json`, separate from policy semantics.

- [ ] Start with three fixture-backed bundled recommendations: Rust library, TS/Bun utility project, Python pytest project. Treat service/framework layouts as variants selected only when evidence supports them. Preserve conventional Rust special files, Python `__init__.py`, and scoped TS component naming exceptions.
- [ ] Implement one shared merge operation: absent mapping keys are added; identical values are no-op; different existing scalar/sequence values are conflicts and preserved; excludes combine by stable de-duplicated union only when requested by the selected pattern. Validate the prospective complete config before an atomic write. Never overwrite existing project files or insert inferred exclusions to hide failures.
- [ ] A conflicting required rule produces a reported conflict, path, existing/incoming value and no partial config write. Apply explicit user-selected local intent before inferred bundled recommendations; don't silently stack two incompatible naming rules.
- [ ] Tests: repeat application yields byte-identical policy; custom max_lines survives; collision is non-destructive; invalid resulting config leaves original intact; recipe path with spaces works; path outside repo is read only when explicitly supplied; no file templates execute code.
- [ ] Run `cargo test --test agent_recipe_file`, existing init/onboarding merge tests and config example validation. Add copyable local-team-pattern docs with a real passing/failing fixture.

**Accept:** Built-in and project-local approaches are reusable; policy is editable; provenance is visible; application and conflict behavior are deterministic. `--recipe-file` is a proposed new option: update CLI/spec/release inventory together as part of this card, not as an undocumented helper.

## A03

**Outcome:** The initializing agent makes low-risk structural decisions and completes specialization without a generic questionnaire. **Own:** `agent_onboarding.rs`, `agent_onboarding_handoff_templates.rs`, `agent_onboarding_report.rs`, `agent_onboarding_rules.rs`, `agent_args.rs`, generated guidance tests.

- [ ] Change generated `agent-next.md` to the following ordered procedure: inspect explicit repo instructions/manifests/layout; preserve local intent; select the smallest matching pattern; record source/test/generated boundaries and native tools; apply/validate policy; configure integration/gates; prove a negative case; report unresolved exceptions only.
- [ ] Keep the Rust CLI deterministic. It detects facts and recommends patterns; the coding agent decides project shape. For an empty project, choose a reversible documented default if stack intent exists. Ask about conflicting manifests or missing stack intent, not ten routine questions by default.
- [ ] Write `.assura/onboarding/profile-selection.json` with `schema`, `profile`, `source`, `source_hash`, `decisions` (key/value/evidence), `conflicts` and `verification`. Add a deterministic validation step for this record and the materialized config. The presence of a decision record alone cannot mark specialization verified.
- [ ] Replace generic “waiting for user answers” with actionable state: `needs_agent_specialization`, `conflict_requires_user`, `configured_unverified`, or `verified` for the specialization item. Preserve report-version rules; if changing serialized state contracts, update schema/version/consumers explicitly.
- [ ] Tests: recognizable Cargo/Bun/pytest repos yield concrete recommendations; inherited local convention beats a bundled default; conflicting instruction remains visible; no package manifest is invented just to satisfy a rule; rerun is non-destructive. Use `agent_onboarding_config_merge`, `agent_surface_cli` and new deterministic state tests.

**Accept:** No routine end-user questionnaire for supported unambiguous fixtures; run A01 with `--dimensions structure,policy,guidance,preservation,idempotence` to prove this card's specialization scope. Hook/native gate closure belongs to A04/A05, and only the full contract in A07 proves end-to-end acceptance. The agent can explain every structural decision from project evidence or a stated default.

## A04

**Outcome:** Hooks are actually active where supported, and existing integrations survive. **Own:** `src/cli/agent_integration*`, `agent_lifecycle.rs`, `hooks.rs`, onboarding handoff/report; `tests/agent_integration_cli.rs` and new hook fixture cases.

- [ ] Generated guidance must inspect existing hook ownership before installation: no hook → install Assura managed hook; existing supported hook manager → append an Assura command through that manager's project config; unknown custom hook → preserve it and report a proposed integration change, never overwrite.
- [ ] Keep `agent onboard --agent auto --activate` as the managed-host path. Inspect host status and doctor; report generated, activated and verified separately. Required host permission is an explicit unavailable state until the user grants it.
- [ ] Verify a real hook event against a disposable invalid fixture. Install files alone are not sufficient. For Git hooks, invoke from the fixture cwd on a real temporary branch/repo and capture exit code; verify pre-push advisory and opt-in blocking modes exactly as documented.
- [ ] Preserve the existing ordinary-branch warning policy unless a separately accepted profile changes it. Final enforcement comes from CI; a local nonblocking hook is not mislabeled “merge protected.”
- [ ] Test rerun/update/remove, existing custom hook preservation, paths with spaces, missing binary, host config conflicts and interrupted installation rollback. Exercise all currently claimed hosts or mark their evidence unavailable.

**Accept:** Each supported fixture has policy-backed hook behavior; no user hook is lost; permission gaps cannot be counted as verified; update/remove touch only managed content.

## A05

**Outcome:** Generated quality plans run the right native checks once, at the right stage. **Own:** onboarding templates/patterns, `src/cli/quality.rs`, `src/config/config/quality.rs`, existing planner tests; Create `tests/quality_onboarding_contract.rs` and CI recipe docs.

Use existing `quality.scopes`; do not add a general command executor. Example valid Rust fragment:

```yaml
quality:
  scopes:
    rust:
      paths: ["src/**", "tests/**", "Cargo.toml", "Cargo.lock"]
      always: ["assura check"]
      frequent: ["cargo fmt --all -- --check"]
      pre_push: ["cargo test --locked"]
      pr: ["cargo clippy --all-targets -- -D warnings"]
```

- [ ] Detect existing scripts/tools before recommending commands. Bun project → declared package scripts plus Bun tests; Python → actual pytest/Ruff/mypy configuration, not guessed installed tools. Missing optional tool is a recommendation, not an invented runnable gate.
- [ ] Keep frequent checks cheap; native suite/type checks belong later. Config/lockfile/toolchain changes trigger broader checks; docs-only changes avoid unrelated native suites. Add an explicit config/guidance scope and full CI `assura check` so unclassified changed paths cannot evade the authoritative policy gate.
- [ ] Verify cumulative phases through `checks_for_phase`: frequent ⊆ pre_push ⊆ pr ⊆ merge; de-duplicate identical commands in stable order. Test missing quality config returns honest not-configured feedback; onboarding should provide useful supported scopes rather than leave a failing suggestion.
- [ ] Add an opt-in CI recipe using the released binary and native project commands. Do not overwrite workflow files; show a patch when a project already has CI. Branch protection activation is separately verified by the maintainer.
- [ ] Run `cargo test --test quality_onboarding_contract`; fixture tests assert expected commands for docs/source/config changes, deduplication and a seeded failure in actual CI-equivalent execution.

**Accept:** Planned commands exist and pass/fail meaningfully in their fixture cwd; no duplicate expensive suite; hooks/CI explicitly distinguish configured from active. Policy changes themselves remain gated.

## A06

**Outcome:** Useful feedback has a bounded runtime and context cost. **Own:** `agent_nudge*`, watch/daemon feedback rendering and existing performance/nudge tests.

- [ ] Measure representative idle, single-file, burst-edit and full-gate cases with the exact binary. Record cold/warm path, hardware, project size, output bytes, event count and p95 latency. Establish budgets before tuning.
- [ ] Initial budget proposal for supported toy/small fixtures: idle event emits no repeated finding body; rendered nudge body ≤2 KiB with a pointer to full output; one actionable nudge per identical finding set per cooldown; quiet-state work avoids full native test execution. Reuse existing configurable cooldowns rather than adding a second timer.
- [ ] Preserve first critical finding, config changes and recovery notices even if a cooldown exists. Deduplicate by stable finding identity plus relevant policy generation; a new policy must not inherit stale suppression.
- [ ] Add tests for unchanged repeated events, distinct new failures, resolved/reintroduced failures, changed configuration, multibyte output boundaries and daemon fallback. No absolute latency promise until reference hardware measurements define the supported budget.
- [ ] Run existing nudge/integration tests and warm-loop benchmark/gate. Report cost and missed/late findings, not only reduced bytes.

**Accept:** Idle loops stay quiet, new actionable issues appear, output limit is enforced without invalid encoding, and warm/cold correctness agrees. R01 safety fallbacks remain observable.

## A07

**Outcome:** Candidate initialization meets the expected configuration and supports later development. **Own:** evaluation run artifacts and findings; fixes go back to A02–A06, not into evaluator expectations.

- [ ] Freeze A01 contracts, source SHA, binary hash, model/harness version and two input conditions. Screening: 3 stacks ×5 repetitions ×2 conditions =30 runs, changing one product-input variable at a time. Limit concurrency to available budget and isolate every repo/session.
- [ ] Initializing agent receives only “Initialize this project with Assura,” normal repo context and candidate tool access. It does not see evaluator contracts. No parent coaching, inherited trial memory or manual post-run repairs.
- [ ] Evaluate each run with A01. A separate fresh agent adds a small specified feature; assess structure preservation, native test success, bounded repair and no disabled policy.
- [ ] Final candidate batch: 10 fresh runs per stack, at least 9/10 acceptable per stack, zero destructive overwrites and zero missed critical seeded violations. Unavailable required host/tool evidence is not pass.
- [ ] Freeze a separate holdout set before screening: two unseen fixture layouts per stack, three fresh runs per layout =18 holdout runs. One layout per stack contains a valid, explicitly configured naming/layout exception and existing local policy; the other contains an already working supported hook manager and a custom hook action whose behavior must survive. Verify each original fixture by hand with native commands before freezing its hash. Keep holdout contracts/layouts away from initializing agents and input-tuning work; agents see only the ordinary project contents at run time.
- [ ] Holdout acceptance requires all 18 runs to pass the full A01 contract and the follow-up feature check, including preservation of the valid exception and actual existing-hook behavior. Required unavailable evidence fails acceptance; no silent fixture substitutions or excluded failed runs. A failed holdout returns to the owning card; after changes, rerun the complete holdout batch and disclose that it is now regression evidence, not an untouched holdout. A new generalization claim requires a newly frozen unseen set with the same allocation. Record the run budget before starting; exhausted budget means incomplete evidence, not a relaxed gate.
- [ ] Publish a redacted result table with counts, sample sizes, median/p95 time/tokens where observable and failure examples. Compare baseline and candidate; do not claim a population success rate from the small batch.

**Accept:** Per-stack acceptance and holdout criteria met. Otherwise return specific failures to owning cards or narrow support through P01; never lower the evaluator's expected configuration merely to improve the score.
