---
id: goal-assura-v0-1-polished
type: goal
title: Assura v0.1 polished onboarding release
status: completed
created: 2026-05-14
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/structure-enforcement.md
  - .trellis/spec/assura/tooling-stabilization.md
  - docs/analysis/2026-05-11-ls-lint-parity-performance-regression-audit.md
---

# Assura v0.1 polished onboarding release

## Objective

Ship Assura v0.1-polished as a working pre-1.0 developer onboarding release.
Do not stop until the repository has a truthful working CLI, reproducible
LS-Lint compatibility and performance evidence, comprehensive LS-Lint feature
tests, and a website that accurately onboards developers using only supported
commands.

This goal is limited to the first polished release. Do not implement the
advanced agent-nudge system in this goal. Do document the next-goal plan for
agent nudges and quality measurement. Treat stale or overclaiming docs as
product bugs, and prefer removing or clearly marking unsupported behavior over
pretending it works.

## Before Changing Code

Read these files before implementation work starts:

- `AGENTS.md`
- `.agents/skills/assura-goal-execution/SKILL.md`
- `.trellis/workflow.md`
- `.trellis/spec/assura/index.md`
- `.trellis/spec/assura/roadmap.md`
- `.trellis/spec/assura/structure-enforcement.md`
- `.trellis/spec/assura/tooling-stabilization.md`
- `docs/analysis/2026-05-11-ls-lint-parity-performance-regression-audit.md`
- `.assura/config.yml`
- `src/cli/check.rs`
- `src/cli/commands.rs`
- `src/cli/args.rs`
- `src/config/ls_compat.rs`
- `tests/ls_lint_parity_regression_tests.rs`
- `benches/README.md`
- `benches/ls_lint_comparison.rs`
- `website/src/content/docs/guides/getting-started.md`
- `website/src/content/docs/guides/quickstart.md`
- `integrations/agents/codex/README.md`
- `integrations/agents/codex/src/index.ts`

## Agent Execution Hook

Agents executing this goal must use
`.agents/skills/assura-goal-execution/SKILL.md`.

Track one iteration as a meaningful implementation/review loop: selecting a
slice, editing files, running validation, and deciding the next slice. Every
third iteration, and before final handoff:

- Append an iteration checkpoint to this file's progress log.
- Record available context-health information. If the platform exposes token or
  context budget, include it; otherwise record `context level: not exposed` and
  summarize the relevant previous messages in 3-6 bullets.
- Review the conversation, progress log, repeated explanations, and failed
  commands for reusable repo knowledge.
- Create or update a project skill under `.agents/skills/` when the knowledge
  is reusable.
- Register new or changed project skills in `AGENTS.md` with only a lean
  trigger/purpose row; keep operational detail inside the skill.
- Run `assura check` after changing `.agents/skills/`, `AGENTS.md`, or
  `.assura/config.yml`.

`.assura/config.yml` must continue to allow `.agents/skills/` and reject skill
files that do not match the project shape contract.

## Current Repo Truth

- `assura check` is the current public structure-first validation path through
  `src/cli/check.rs` and `run_structure_check`.
- `check --format json` serializes `StructureCheckReport` with `success`,
  `project_root`, `config_path`, `checked_path`, `files_checked`,
  `dirs_checked`, and `violations`.
- `status --format json` is implemented and returns a project/config/rule
  summary from loaded config.
- `init` writes a starter `.assura/config.yml` safely and refuses to overwrite
  without `--force`.
- `watch` is a truthful thin wrapper over one-shot `check` behavior and returns
  the delegated check exit code.
- `migrate` and `info` are exposed as top-level Clap subcommands.
- Supported output formats in `OutputFormat` are `text`, `json`, and `yaml`.
- LS-Lint conversion exists in `src/config/ls_compat.rs` and has regression
  coverage in `tests/ls_lint_parity_regression_tests.rs`.
- Existing LS-Lint coverage includes core extension, `.dir`, OR, ignore,
  `exists`, direct-child count, and exact-file compatibility extension cases,
  but the audit documents remaining gaps for directory patterns and validation
  scopes implying required directories.
- `benches/ls_lint_comparison.rs` compares the current `run_structure_check`
  product path against `@ls-lint/ls-lint@2.3.0`.
- `benches/README.md` records current local baseline results and no longer
  keeps unsupported `6.8x faster` claims as current release evidence.
- Website quickstart/getting-started docs use supported commands and include
  install/source build, `assura init`, `assura check`, intentional failure
  flow, CI usage, LS-Lint migration, JSON examples, and future agent-roadmap
  labeling.
- `integrations/agents/codex` is currently a skeleton only. It does not install
  hooks or provide runtime validation feedback.

## Acceptance Criteria

### 1. CLI Truth and Polish

- `assura check` is the primary supported validation command.
- `cargo run --quiet -- check --format json .` returns valid JSON and succeeds
  on the repository baseline.
- `assura check` returns a nonzero exit code on known failing fixtures.
- `assura status --format json` works and reports project/config/rule summary.
- `assura init` is implemented safely or removed from current onboarding docs.
  Prefer implementing it.
- `assura watch` is implemented as a thin wrapper over `check` or removed from
  current onboarding docs. Do not leave it as a successful stub.
- `migrate` and `info` are exposed as real Clap commands with tests or removed
  from current docs. Prefer exposing `migrate` because LS-Lint migration is
  part of the adoption story.
- Docs only mention supported output formats.

### 2. LS-Lint Compatibility

Add comprehensive fixture coverage for LS-Lint 2.3 behavior:

- Extension rules.
- Wildcard extension rules.
- Subextension rules where applicable.
- `.dir` directory naming.
- Explicit nested directory scopes.
- Glob or alternative directory scopes such as `packages/*`, `**`, and
  `{src,tests}`, or documented unsupported behavior with tests proving current
  error behavior.
- OR syntax such as `kebab-case | snake_case`.
- Ignore/exclude behavior.
- `exists`, `exists:0`, `exists:1`, and `exists:N-M` for file and directory
  counts.
- Direct-child-only `exists` semantics.
- Validation scope must not imply required directory unless an explicit
  existence rule requires it.
- Exact filename `exists` remains documented as an Assura compatibility
  extension, not native LS-Lint parity.

### 3. Performance

- Add or update benchmarks so they compare the current product path, not only
  older internal `ConstraintEngine` paths.
- Measure current structure-first `assura check` behavior against
  `@ls-lint/ls-lint@2.3.0` on identical fixtures.
- Include small, medium, large, rule-heavy, and ignored/generated-heavy
  scenarios.
- Assura must beat LS-Lint on median runtime in the main supported scenarios.
- Do not keep or add a `6.8x faster` claim unless the new current-product
  benchmark supports it.
- Save benchmark instructions and current baseline results in docs or benches
  documentation.

### 4. Website Onboarding

- Website build passes.
- No current docs mention unsupported `assura validate`, unsupported
  completions, unsupported formats, stale V1 quickstart flows, or wrong
  repository identity.
- A developer can follow one guide from install/source build to `assura init`
  to `assura check` to intentional failure to fix to CI usage.
- Add or update LS-Lint migration docs showing `.ls-lint.yml` to
  `assura migrate` to `assura check`.
- Add real JSON output examples matching the current `StructureCheckReport`
  fields.
- Add a clearly labeled future roadmap section for agent nudges. Do not imply
  the Codex integration is complete.

### 5. Required Validation Commands

Run and make these pass, or document the blocker with exact failing output and
next action:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --quiet`
- `cargo test --test ls_lint_parity_regression_tests`
- `cargo run --quiet -- check --format json .`
- `cargo bench --bench profiling structure_check -- --noplot`
- The new current-product LS-Lint comparison benchmark.
- `cd website && corepack pnpm install --frozen-lockfile && pnpm build`
- `cd integrations/agents/codex && npm install && npm run lint && npm run build`

## Test Matrix

| Area | Valid fixture | Invalid fixture | Expected Assura violation rules | Classification |
| --- | --- | --- | --- | --- |
| Extension rules | Files matching configured extension naming | Misnamed file with configured extension | `file_naming` | Native LS-Lint parity |
| Wildcard extension rules | `.*` and `.*.js` matching allowed naming | Wildcard-matched bad filename | `file_naming` | Native LS-Lint parity |
| Subextension rules | Subextension fixture if LS-Lint behavior applies | Bad subextension filename | `file_naming` or documented unsupported error | Native parity or documented gap |
| `.dir` naming | Directories matching configured `.dir` rule | Bad directory name | `directory_naming` | Native LS-Lint parity |
| Explicit nested scopes | `src` or `packages/core` files obey nested rules | Nested file violates scoped rule | `file_naming` or `directory_naming` | Native LS-Lint parity |
| Glob directory scopes | Explicit supported directory scopes convert | `packages/*`, `**`, `{src,tests}` return a migration error | Unsupported directory-scope conversion error | Documented gap |
| OR syntax | `kebab-case` and `snake_case` both pass | Name matching neither alternative | `file_naming` or `directory_naming` | Native LS-Lint parity |
| Ignore/exclude | Ignored bad files are skipped | Non-ignored bad file fails | No violation for ignored path; normal violation outside ignore | Native LS-Lint parity |
| `exists` shorthand | Required direct count is present | Required direct count missing | `exists_count` | Native LS-Lint parity |
| `exists:0` | No matching direct child exists | Matching forbidden child exists | `exists_count` | Native LS-Lint parity |
| `exists:1` | Exactly one matching direct child exists | Zero or multiple matching direct children | `exists_count` | Native LS-Lint parity |
| `exists:N-M` | Count inside inclusive range | Count below or above range | `exists_count` | Native LS-Lint parity |
| Direct-child exists | Nested descendant does not satisfy direct count | Only nested descendant exists | `exists_count` | Native LS-Lint parity |
| Scope not required | Scoped directory may be absent when no explicit existence rule requires it | Missing scope currently reports required directory if still unsupported | No violation, or documented unsupported behavior | Required for polished release |
| Exact filename exists | Exact file count works in Assura | Missing exact file reports count violation | `exists_count`, never `required_directory` | Assura compatibility extension |

Each compatibility feature must have a valid fixture, invalid fixture, expected
Assura violation rules, and a native parity versus Assura extension label.

## Benchmark Matrix

| Scenario | Fixture shape | Assura command/path | LS-Lint command/path | Required evidence |
| --- | --- | --- | --- | --- |
| Small | Small tree with representative extension and directory rules | Current structure-first `assura check` path | `@ls-lint/ls-lint@2.3.0` on equivalent `.ls-lint.yml` | Median runtime comparison |
| Medium | Medium tree with common source/test layout | Current structure-first `assura check` path | Same fixture and equivalent LS-Lint config | Median runtime comparison |
| Large | Large tree with many files and directories | Current structure-first `assura check` path | Same fixture and equivalent LS-Lint config | Median runtime comparison |
| Rule-heavy | Many wildcard/extension/path rules | Current structure-first `assura check` path | Same fixture and equivalent LS-Lint config | Median runtime and hotspot notes |
| Ignored/generated-heavy | Large ignored generated directories | Current structure-first `assura check` path with exclusions | Same ignored/generated fixture | Proof ignored output is pruned |

Benchmark documentation must name the machine/date, exact commands, LS-Lint
version, Assura commit or branch, and whether results are local evidence or CI
thresholds.

## Website Onboarding Requirements

- Replace stale `assura validate` instructions with `assura check`.
- Remove or clearly mark unsupported shell completions unless implemented.
- Remove stale V1-first quickstart content from current onboarding.
- Ensure install/source build instructions use the correct repository identity.
- Include a single supported first-run path:
  install or source build, `assura init`, `assura check`, intentional failure,
  fix, and CI usage.
- Include LS-Lint migration docs:
  `.ls-lint.yml` input, `assura migrate`, generated `.assura/config.yml`, and
  `assura check`.
- Include real JSON examples matching `StructureCheckReport`.
- Clearly label agent nudges as future roadmap, and state that Codex
  integration is currently not complete unless implemented in this goal.

## Known Gaps

- LS-Lint directory pattern scopes were closed by the 2026-05-26 rule coverage
  audit. `packages/*`, `**`, and `{src,tests}` now migrate as validation
  scopes instead of unsupported literal directories.
- Codex integration is a skeleton and must not be described as complete.

## Product Direction

Assura should preserve LS-Lint's useful mental model: one config describes the
allowed project shape, and anything outside that shape is rejected or reported
with clear context. The future direction is broader than filename casing:

The upstream LS-Lint PRs listed below should be treated as prototype evidence,
not as the final Assura design. At the start of Assura, those ideas were
further refined: some function and concept names changed, and the config moved
toward a more scalable structure-first notation. The notation should still stay
aligned with LS-Lint for simple use cases, but Assura's native configuration is
allowed to be more explicit where that improves reuse, closed-world structure
contracts, content checks, or agent-facing guidance.

- Keep structure rules and high-level organization checks in one readable
  configuration surface.
- Support exact basename and direct count constraints so projects can require
  specific files, allow optional-at-most-one files, and deny unexpected classes
  of files.
- Keep closed-world/deny-by-default project-shape policies practical for large
  monorepos.
- Add context-aware feedback that tells a developer or agent why a rule exists,
  whether the current hook is warning or blocking, who can approve policy
  changes, and what references to load before trying a fix.
- Prefer reusable directive-style notation for repeated rule sets over YAML
  anchors or duplicated rule strings.
- Treat content checks as high-level organization checks, not language-format
  linting: examples include max lines, markdown frontmatter, required heading
  outline, or simple path/name-aware gross mismatch checks.
- Keep specialized code-format and language semantics in external tools unless
  Assura is only orchestrating or referencing them through a safe, named
  validator path.
- Provide failure messages that reduce agent exploration and discourage
  bypassing constraints. Messages should point to skills, scripts, or docs that
  explain the intended remediation.

Historical upstream LS-Lint PR discussion that informs this direction:

- `rothnic/ls-lint#1`: exact basename `exists`, direct count semantics,
  deny-by-default examples, ignore performance, and large-monorepo measurement.
- `rothnic/ls-lint#2`: self-contained context/policy feedback with warn/fail
  mode, hook, environment, override, change approval, references, and
  formatted failure output.
- `rothnic/ls-lint#3`: content rule direction, `groups:`-style reusable rule
  sets, per-rule messages, real-project timing, markdown structure examples,
  and agent-facing failure-message examples.
- `rothnic/ls-lint#4`: TypeScript/docs best-practice examples, failure
  messages that point agents to analysis scripts and policy docs, explicit
  owner approval boundaries, composable naming/size groups, and removal of
  `rule-groups:` in favor of `groups:` with no backwards compatibility.

## Non-Goals

- Do not implement advanced agent nudges in this release goal.
- Do not claim future Assura-native agent/plugin behavior as complete before it
  is implemented and tested.
- Do not keep unsupported CLI commands in current onboarding.
- Do not preserve pre-1.0 compatibility layers solely for internal backwards
  compatibility.
- Do not add performance claims without current product-path benchmark
  evidence.

## Validation Commands

Run these commands before marking the goal complete:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --quiet
cargo test --test ls_lint_parity_regression_tests
cargo run --quiet -- check --format json .
cargo bench --bench profiling structure_check -- --noplot
```

If Cargo fails for platform, OpenSSL, or network reasons, load
`.agents/skills/assura-local-build/SKILL.md` before changing product code.

Run the new current-product LS-Lint comparison benchmark:

```bash
cargo bench --bench ls_lint_comparison -- --noplot
```

```bash
cd website
corepack pnpm install --frozen-lockfile
pnpm build
```

```bash
cd integrations/agents/codex
npm install
npm run lint
npm run build
```

If a command cannot pass, document the blocker in the progress log with:

- Exact command.
- Exact failure.
- Likely cause.
- Smallest next step.
- Whether the blocker should be fixed in this goal or deferred.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-05-14 | Goal file created from release objective and current repo inspection. | `docs/goals/assura-v0-1-polished.md` |
| 2026-05-14 | Added agent execution hook for every-third-iteration context review, repo-local skill maintenance, and AGENTS.md progressive-disclosure registration. | `.agents/skills/assura-goal-execution/SKILL.md` |
| 2026-05-14 | Added product-direction lineage from upstream LS-Lint prototypes to Assura's refined structure-first notation. | `Product Direction` section |
| 2026-05-14 | Iteration 1 completed CLI surface polish: `init` writes config safely, `watch` delegates to check, `migrate` and `info` are real Clap commands, and CLI command-surface tests pass. | `cargo test --test cli_command_surface_tests` |
| 2026-05-14 | Iteration 2 completed LS-Lint compatibility fix so scoped validation directories are not required unless explicit existence rules require them. | `cargo test --test ls_lint_parity_regression_tests` |
| 2026-05-14 | Iteration 3 checkpoint: context health available from goal tool, `tokensUsed=282543`, no completion token budget exposed. Reviewed prior messages and created repo-local skills for long-goal execution and WSL/local build issues. Current reusable-skill need is covered; no new skill required at this checkpoint. | `get_goal`; `.agents/skills/assura-goal-execution/SKILL.md`; `.agents/skills/assura-local-build/SKILL.md` |
| 2026-05-14 | Updated current-product performance benchmark and recorded local LS-Lint 2.3 comparison baseline. Initial sandboxed npm fetch failed with `EAI_AGAIN`; approved network access confirmed `ls-lint v2.3.0` and benchmarked all scenarios. | `benches/ls_lint_comparison.rs`; `benches/README.md` |
| 2026-05-14 | Website onboarding was rewritten around supported commands, LS-Lint migration, real JSON report fields, and a future-only agent nudge roadmap. Search audit found no stale `assura validate`, unsupported completions, unsupported formats, stale migrate flags, unsupported agent flags, wrong repo identity, or current `6.8x` claims. | `rg` stale-doc search; `website/src/content/docs/guides/getting-started.md`; `website/src/content/docs/guides/ls-lint-migration.md` |
| 2026-05-14 | Required release validation passed. Network-dependent npm/pnpm installs needed approved network access after sandbox DNS `EAI_AGAIN`; website build disables Astro telemetry to avoid writing `/home/nickroth/.config/astro` in sandboxed validation. | `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-targets --quiet`; `cargo test --test ls_lint_parity_regression_tests`; `cargo run --quiet -- check --format json .`; `cargo bench --bench profiling structure_check -- --noplot`; `cargo bench --bench ls_lint_comparison -- --noplot`; `corepack pnpm install --frozen-lockfile`; `pnpm build`; `npm install`; `npm run lint`; `npm run build` |
| 2026-05-14 | PM/end-user audit found two acceptance gaps that were not proven by checked boxes: LS-Lint glob directory scopes were previously converted too literally, and migration to `.assura/config.yml` failed in a fresh project when `.assura/` did not exist. Fixed both, added regression coverage, and manually verified fresh init/check/fail/fix and LS-Lint migration flows. | `cargo test --test cli_command_surface_tests`; `cargo test --test ls_lint_parity_regression_tests`; fresh temp-project smoke commands |
| 2026-05-14 | PM/end-user docs audit found stale public pages describing unsupported library APIs, plugin APIs, role/maturity surfaces, and long-running watch behavior. Rewrote those pages to current CLI/report/config surfaces and marked agent/plugin work as future-only. | `website/src/content/docs/why-assura.md`; `website/src/content/docs/reference/api.md`; `website/src/content/docs/docs/api.md`; `website/src/content/docs/examples/*.md`; `pnpm build`; stale-doc `rg` search |
| 2026-05-14 | Final iteration checkpoint: context health from goal tool showed `tokensUsed=356415`, no completion token budget exposed, and goal status already marked complete. Re-reviewed PM/end-user evidence, updated reusable docs for local LS-Lint/npm behavior, and no additional project skill was needed beyond `assura-goal-execution` and `assura-local-build`. | `get_goal`; `.agents/skills/assura-goal-execution/SKILL.md`; `.agents/skills/assura-local-build/SKILL.md` |

## Next Goal Definition

After v0.1-polished ships, define the next goal as the Codex/agent nudge MVP:

- Build the smallest runtime nudge path that surfaces Assura validation results
  to Codex or another agent without blocking normal developer work by default.
- Measure instructions-only behavior versus `AGENTS.md`/skills behavior versus
  Assura runtime nudges.
- Track quality metrics for modularity, instruction adherence, structural
  violations, correction loops, and nudge precision.
- Keep nudge precision explicit: count useful nudges, noisy nudges, missed
  violations, and whether the agent corrected the issue after the nudge.
- Include context-aware policy feedback in Assura reports: hook mode, severity,
  approval boundary, references, and remediation skill/script pointers.
- Include a comparison of how many tool calls or correction loops are avoided
  when failures include actionable policy context instead of rule text alone.

## Final Release Checklist

- [x] CLI supported commands are implemented, removed from docs, or clearly
  marked future-only.
- [x] `assura check` baseline succeeds on this repository with JSON output.
- [x] Known failing fixtures produce nonzero exit codes.
- [x] LS-Lint feature matrix has valid and invalid fixtures.
- [x] LS-Lint unsupported behavior is documented with tests.
- [x] Current-product benchmark compares Assura against
  `@ls-lint/ls-lint@2.3.0`.
- [x] Benchmark results and instructions are saved.
- [x] Website onboarding uses only supported commands.
- [x] LS-Lint migration guide is accurate and tested manually.
- [x] Codex integration status is accurate and future roadmap is labeled.
- [x] All required validation commands pass or have documented blockers.
- [x] Stop condition is satisfied.

## Stop Condition

Stop only when all acceptance criteria are met and the validation checklist
passes, or when a blocker is documented with:

- Exact command.
- Exact failure.
- Likely cause.
- Smallest next step.
- Whether the blocker should be fixed in this goal or deferred.
