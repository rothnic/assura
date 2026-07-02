---
id: goal-assura-post-beta-capabilities-program
type: goal
title: Assura post-beta capabilities program
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-self-config-doc-variance-hardening.md
  - ./assura-supported-document-graph.md
  - ./assura-true-daemon-mode.md
  - ./assura-performance-floor-and-fixture-gate.md
  - ./assura-agent-integration-lifecycle.md
  - ./assura-markdownlint-compatible-rust-engine.md
  - ./assura-vscode-supported-extension.md
  - ./assura-extension-api-clarification.md
  - ./assura-ls-lint-performance-reassessment.md
  - ./assura-post-beta-support-release-hardening.md
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Post-Beta Capabilities Program

## Objective

Drive the next large Assura iteration after `v0.2.0` by resolving the known
post-beta gaps without reopening the completed beta release. This is the parent
goal to kick off when the next multi-goal execution cycle should proceed.

This program is still beta-track work. Completion should produce a versioned
beta increment with stronger supported capabilities; it does not promote Assura
beyond beta.

## Program Bar

Post-beta readiness means Assura moves from beta-capable local workflows to
durable product surfaces that can be installed, kept warm, benchmarked, and
extended without ambiguity. The program must cover:

- Assura dogfooding through a refined `.assura/config.yml` and resolved
  documentation/structure variance;
- a fully supported document graph for content validation, search, query,
  graph expansion, relation checks, affected-reference questions, and bounded
  agent context;
- a true long-running daemon process with IPC and stale-state safety;
- staged validation semantics where structure and coarse file-level policy run
  before deeper Markdown, content-model, reference, and language-specific
  checks;
- performance hardening that treats Rust-vs-Go CLI floor misses as defects to
  explain and resolve, not excuses;
- installed agent integration lifecycle support for Codex, OpenCode, Claude,
  and Pi;
- a markdownlint-consistent, high-performance Rust Markdown lint/fix engine;
- a supported VS Code extension path over shared daemon and CLI contracts;
- a clear extension API decision that separates first-party config extensions
  from any future public plugin API;
- final support and release hardening before any post-beta support claim.
- a versioned beta increment that honestly describes the new functionality
  without implying post-beta/GA status.

## North-Star Use Case

A maintainer adds Assura to a documentation-heavy Rust project that has product
goals, ADRs, architecture notes, release docs, generated API references, and
agent-written Markdown. The project starts clean enough to build, but the
knowledge system has the usual drift: a moved docs page, a stale code comment,
an invalid frontmatter relation, duplicated headings, a broken line anchor, and
an agent about to edit a file without knowing the affected docs.

At the end of this program, that maintainer should be able to run one supported
local workflow:

1. Initialize or refine `.assura/config.yml` so Assura first checks repository
   shape, root hygiene, directory boundaries, file placement, line limits, and
   coarse Markdown scope before inspecting deeper file internals.
2. Model the project's docs as content collections with typed frontmatter,
   stable IDs, relations, path scopes, and collection-specific validation.
3. Run `assura check` and receive ordered findings that start with high-level
   structure/config problems, then file-level Markdown policy, then Markdown
   syntax/headings/links, content-model diagnostics, repository-reference
   issues, and optional language-specific or extension findings.
4. Run `assura content` queries to answer practical questions:
   which goals reference this ADR, which docs mention this code path, which
   records have invalid or missing relations, which Markdown sections match a
   search phrase, and what context should an agent read before editing a file.
5. Start a real Assura daemon that keeps this state warm, reports config or
   file changes as stale when appropriate, and serves the same truth as a
   one-shot check through a stable local IPC contract.
6. Use Codex, OpenCode, Claude, and Pi hooks that inject compact nudges only
   when the next tool call or recent file change makes Assura feedback useful,
   without flooding context or breaking caching.
7. Use VS Code diagnostics and commands over the same daemon/CLI contracts,
   including safe-fix previews, daemon doctor guidance, and one-shot fallback
   when the daemon is unavailable.
8. Apply Markdown fixes through the fastest practical Rust markdownlint-
   compatible engine, with deterministic safe-fix behavior and no Node runtime
   requirement in the supported path. When self-dogfooding exposes consistent
   Markdown formatting drift that the selected engine cannot safely fix, Assura
   should provide small custom fix utilities with focused tests instead of
   forcing agents to patch the same prose-shape issues manually.
9. Trust CI to fail if any accepted LS-Lint-equivalent fixture is slower than
   native LS-Lint, with enough attribution to know whether process startup,
   config loading, walking, glob matching, rule evaluation, reporting, or
   benchmark harness overhead caused the regression.
10. Read public docs that clearly say what is supported, experimental,
    internal, planned, or unsupported, including extension API boundaries.

The final verification scenario for this parent goal should use a realistic
fixture or repo package that demonstrates the whole workflow with intentional
valid and invalid cases. It must prove that a user can discover, validate,
query, repair, and keep warm a repository knowledge graph without switching
between unrelated tools or relying on undocumented behavior.

## Final Verification Use Case Package

Create and keep one support-grade verification package for this program. The
package should be concrete enough that a reviewer can run it and decide whether
the major increment produced the intended user outcome, not only whether child
tasks were completed.

The fixture user is a maintainer of a Rust CLI with a documentation system
similar to Assura's own repository:

- `docs/goals/` stores active and archived product goals with typed
  frontmatter, owners, status, related ADRs, and expected evidence links.
- `docs/analysis/` stores design notes, performance reports, and research
  records that must point back to goals or roadmap entries.
- `docs/reference/` or generated API docs expose stable anchors that goals and
  analysis pages may cite.
- source files and tests include references back to docs, and docs include
  references to source paths, symbols, fixtures, and benchmark rows.
- `.assura/config.yml` defines repository shape, coarse file policy,
  collections, severities, suppressions, agent nudge policy, daemon behavior,
  Markdown lint/fix scope, and performance fixture gates.

The package must include both valid content and intentional failures:

- a misplaced root or generated file that structure validation catches before
  deeper linting;
- a stale allowlist or overly broad exception that should be rejected;
- a Markdown file with coarse file-policy drift, markdownlint-style drift,
  heading problems, broken links, and a safe-fixable formatting issue;
- a content record with invalid frontmatter, missing required fields, invalid
  relations, and collection-specific severity mapping;
- a moved document or renamed code path that leaves stale doc/code references;
- a daemon stale-state case where the daemon must refuse or mark old results
  stale after config or file changes;
- an agent workflow where Codex, OpenCode, Claude, and Pi receive concise
  Assura nudges only when the pending tool call or recent event makes them
  useful;
- a VS Code workflow where diagnostics, commands, safe-fix previews, and daemon
  doctor output match the CLI/daemon truth;
- accepted LS-Lint-equivalent performance rows that must be faster than
  LS-Lint or fail the merge gate with actionable attribution.

### Primary Verification Story

The acceptance story is a single maintainer journey, not a bundle of feature
demos. Use a fixture named as a real project, for example
`fixtures/post_beta_knowledge_workspace`, and write it so the maintainer's
intent is obvious from the files:

The maintainer has just renamed a core architecture page and moved one module.
They want to know whether the repository's knowledge system is still safe for
humans and agents to work in before merging a feature branch. They run Assura
locally, then keep it warm while an editor and several coding agents continue
to edit the project.

The story starts with a dirty but realistic repository:

- the root contains one misplaced generated artifact and one overly broad
  allowlist entry;
- `docs/goals/active/markdown-engine.md` has valid typed frontmatter but
  references a moved ADR, a renamed code path, and a stale heading anchor;
- `docs/analysis/rumdl-evaluation.md` has invalid frontmatter, duplicated
  headings, a skipped heading level, trailing spaces on blank lines, and a
  suppression without a useful reason;
- `src/cli/check/markdown.rs` references a goal that no longer owns the rule;
- a benchmark fixture row is accepted as LS-Lint-equivalent but is configured
  so Assura's measured row is slower than native LS-Lint until the performance
  slice fixes or explains the cost;
- the daemon has an old status file whose config fingerprint no longer matches
  the working tree.

The reviewer should be able to run the story in this order:

1. Run `assura init` or the documented config-refinement command and confirm
   the resulting `.assura/config.yml` models structure, coarse file policy,
   Markdown rules, content collections, references, agent nudges, daemon
   settings, VS Code support, and performance fixture gates without relying on
   private defaults.
2. Run `assura check --format json` and confirm the first diagnostics are
   structure/root-hygiene and coarse file-policy issues. Markdown internals,
   content-model failures, reference failures, and optional extension findings
   must appear later in the staged order with rule-owned severity and
   suppression metadata.
3. Run `assura fix markdown --dry-run --format json` and confirm the preview
   contains only deterministic fixes, names the source fixer, and refuses
   unsafe or semantic edits. Run `--apply`, then run the preview again and
   confirm the supported fixes are idempotent.
4. Run the supported content query commands: `assura content collections`,
   `missing-relations`, `search`, `expand`, `references`, and `context-pack`.
   The answers must explain which goals, ADRs, headings, code paths,
   benchmark rows, and release docs are affected by the renamed page and moved
   module.
5. Start `assura daemon`, request the same check/content/reference answers over
   IPC, then mutate the config and one referenced Markdown file. The daemon
   must mark or reject stale truth until refreshed, and fresh daemon answers
   must match one-shot CLI truth.
6. Trigger Codex, OpenCode, Claude, and Pi adapter events around planned reads,
   writes, and shell commands. Assura should inject short, cache-friendly
   nudges only when the current file, command, or recent edit makes the finding
   actionable.
7. Open the fixture through the VS Code extension and confirm diagnostics,
   safe-fix previews, daemon doctor output, and one-shot fallback match the
   shared daemon and CLI contracts.
8. Run the CI verification command. It must fail on the intentionally broken
   state for the expected reasons, pass after the documented repairs, and fail
   any accepted LS-Lint-equivalent performance row that is slower than native
   LS-Lint without actionable attribution.

The story passes only when the maintainer can answer: "What is structurally
wrong, what Markdown/content/reference drift exists, what can be safely fixed,
what changed context should my agents read, is my daemon fresh, do editor and
agent surfaces agree with the CLI, and is Assura at least as fast as LS-Lint on
accepted structure fixtures?"

### User-Specific Goal Criteria

The user for this increment is an Assura maintainer running a real
documentation-heavy Rust repository with AI agents in the loop. Their practical
job is not "run every Assura command." Their job is to decide whether a branch
that changed architecture docs, source layout, Markdown, frontmatter, and
agent-written references is safe to merge.

The final output of this parent goal must let that user complete this specific
workflow without undocumented steps:

1. Open the repository and immediately see whether coarse structure, root
   hygiene, config scope, and file-level policy are healthy enough to justify
   deeper checks.
2. Understand every deeper finding in terms of a stable rule, severity,
   suppression policy, affected file/heading/reference, and whether the issue
   blocks merge or is advisory.
3. Ask graph questions that match real maintenance decisions: what goals,
   ADRs, analysis notes, generated references, source files, tests, headings,
   benchmark rows, and release docs are affected by this rename or move?
4. Preview and apply only safe Markdown fixes, then prove the remaining
   Markdown issues need human or domain-specific edits rather than automatic
   rewriting.
5. Keep Assura warm through a daemon while editors and agents continue working,
   and know when daemon state is fresh, stale, crashed, or falling back to
   one-shot truth.
6. Let Codex, OpenCode, Claude, and Pi receive brief Assura guidance at useful
   event boundaries without spending large context on repeated validation
   output or invalidating cache behavior unnecessarily.
7. See VS Code diagnostics and commands that agree with CLI and daemon truth,
   including safe-fix previews and daemon doctor output.
8. Trust CI to reject the branch if supported behavior regresses, if public docs
   overclaim support, if the support matrix disagrees with implementation, or
   if any accepted LS-Lint-equivalent fixture is slower than native LS-Lint
   without actionable attribution.

Each child goal must name which of these user decisions it enables. Work that
adds commands, docs, tests, or adapters but does not improve one of these
decisions is not sufficient for this major increment unless the child goal
explicitly records why the work is prerequisite infrastructure.

The end-to-end verification path should prove this user story:

1. A maintainer clones the fixture and runs `assura init` or config refinement.
2. `assura check` reports findings in staged order: structure, coarse file
   policy, Markdown internals, content model, references, then optional
   language or extension checks.
3. `assura fix markdown --dry-run` previews only deterministic safe fixes, and
   `--apply` repairs the supported subset without changing semantic content.
4. `assura content` can validate collections and answer search/query questions
   about goals, ADRs, code paths, headings, relations, and affected context.
5. `assura daemon` serves warm state over IPC, detects stale inputs, and returns
   the same findings and content answers as the one-shot CLI when fresh.
6. Agent hooks request concise guidance before or after relevant tool events,
   and the response is small enough to preserve context and cache behavior.
7. VS Code surfaces the same diagnostics and fix previews over shared
   contracts, with one-shot fallback when the daemon is unavailable.
8. CI fails the fixture if supported behavior regresses, accepted LS-Lint rows
   are slower, docs overclaim support, or the public support matrix disagrees
   with the implemented contracts.

This package is the final acceptance lens for every child goal. A child goal
can ship only if it either advances this scenario directly or records why that
part of the scenario is explicitly deferred and how the parent still reaches a
versioned beta increment.

### Verification Artifact Contract

The final package should be runnable as one documented verification story, not
as a set of disconnected demos. It should include:

| Artifact | Required proof |
| --- | --- |
| Fixture repository | A realistic Rust CLI/doc-system fixture with committed valid and invalid states, expected findings, and repair expectations. |
| Config model | `.assura/config.yml` covers staged validation order, collections, severities, suppressions, daemon settings, agent nudges, Markdown checks, and LS-Lint fixture gates. |
| CLI transcript | Recorded commands prove `init`, `check`, `content`, `daemon`, Markdown fix preview/apply, and performance-gate behavior. |
| Daemon transcript | Fresh daemon results match one-shot truth, stale daemon state is rejected or marked stale, and warm changed-path/reference queries beat cold one-shot rows on accepted fixtures. |
| Agent transcript | Codex, OpenCode, Claude, and Pi integrations receive bounded nudges only around relevant tool/event moments, with payload size recorded. |
| Editor transcript | VS Code diagnostics, commands, safe-fix previews, and daemon doctor messages match the shared CLI/daemon contracts. |
| Support matrix | Public docs classify every exercised surface as supported, experimental, internal, planned, or unsupported without overclaiming. |
| CI gate | The package fails on intentionally broken structure, Markdown, content, reference, daemon, agent, editor, support-matrix, or LS-Lint performance regressions. |

## Major Iterations

| Order | Epic | Primary goal file | Exit bar |
| --- | --- | --- | --- |
| 1 | Self Config | [Self config and documentation variance hardening](./assura-self-config-doc-variance-hardening.md) | Assura's own config catches the right structure/doc variance without hiding drift or running deeper Markdown/content checks ahead of coarse policy. |
| 2 | Document Graph | [Supported document graph](./assura-supported-document-graph.md) | Content validation, search, query, graph expansion, relation checks, affected references, and bounded agent context have one supported contract. |
| 3 | True Daemon Mode | [True daemon mode](./assura-true-daemon-mode.md) | A real daemon process provides warm checks over IPC, survives normal editor/agent workflows, and never reports stale truth as fresh. |
| 4 | Markdown Engine | [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md) | Assura integrates or proves the fastest Rust markdownlint-compatible linter/fixer path with local parity, fix-safety, staged validation, and benchmark evidence. |
| 5 | Performance Floor | [Performance floor and fixture gate](./assura-performance-floor-and-fixture-gate.md) | CI blocks any accepted LS-Lint-equivalent fixture where Assura is slower, and CLI-floor attribution has executable remediation evidence. |
| 6 | Agent Installers | [Agent integration lifecycle](./assura-agent-integration-lifecycle.md) | Codex, OpenCode, Claude, and Pi integrations have install/update/remove/doctor paths over shared nudge and daemon contracts. |
| 7 | VS Code Extension | [VS Code supported extension](./assura-vscode-supported-extension.md) | The VS Code package has support-grade install/update/remove/doctor workflows over shared daemon and CLI contracts. |
| 8 | Extension API Decision | [Extension API clarification](./assura-extension-api-clarification.md) | Docs and checks clearly distinguish first-party `extensions.*` policies, internal Rust APIs, and any deliberately deferred public plugin API. |
| 9 | Performance Reassessment | [LS-Lint performance reassessment](./assura-ls-lint-performance-reassessment.md) | LS-Lint comparison history and fixture gates are rechecked after daemon, graph, Markdown, agent, and editor surfaces land. |
| 10 | Support Hardening | [Post-beta support and release hardening](./assura-post-beta-support-release-hardening.md) | Support policy, compatibility docs, release surfaces, target-state checks, and release evidence agree before a support claim. |

## Execution Rules

- Execute child goals in dependency order unless a narrower order avoids
  rework; record any reordering in this progress log.
- Do not claim a daemon socket/process server, public plugin API, marketplace
  integration, or full markdownlint parity until the relevant child goal proves
  it.
- Treat validation as layered: structure and coarse file-level policy first,
  then Markdown, content models, repository references, and language-specific
  checks when the file belongs in scope.
- Prefer adapting proven Rust tooling over rewriting commodity lint/fix logic.
- Keep agent integrations as adapters over shared Assura contracts; do not add
  per-agent validators.
- Performance gates must operate on accepted fixture rows, not aggregate-only
  claims.

## Definition Of Done

- All child goals are completed or explicitly deferred with independent
  review and a replacement path.
- Public docs and support policy classify post-beta supported, experimental,
  internal, roadmap, and unsupported surfaces consistently.
- Daemon, Markdown, agent, and extension surfaces share Assura finding,
  severity, suppression, and release-surface contracts.
- Document graph support is fully supported for content validation, search,
  query, graph expansion, relation diagnostics, affected-reference questions,
  and bounded agent context.
- The north-star use case is executable as a final verification package with
  valid and invalid repository examples, expected CLI/editor/agent behavior,
  daemon stale-state behavior, safe-fix preview/apply evidence, and performance
  gate evidence.
- A versioned beta increment is planned or released with docs that accurately
  describe the new supported and experimental surfaces.
- CI blocks any accepted LS-Lint-equivalent fixture that is slower than native
  LS-Lint.
- Independent completion review finds no blocker or high-risk gap.

## Validation Commands

Planning-only updates to this program should run:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
```

Child implementation goals add their own focused commands.

## Review Tasks

- R1: Confirm all requested post-beta gaps have executable child goals.
- R2: Confirm Assura self-config hardening runs before deeper lint and content
  work, and that broad exceptions do not hide drift.
- R3: Confirm document graph support is a supported workflow, not only demos or
  experimental candidate enrichment.
- R4: Confirm Markdown work chooses a measured Rust linter/fixer path before
  rewriting generic markdownlint rules.
- R5: Confirm daemon work requires a real process/IPC contract and stale-state
  safety.
- R6: Confirm agent work preserves shared contracts and avoids per-agent
  validation logic.
- R7: Confirm VS Code work remains a wrapper over shared Assura contracts.
- R8: Confirm extension API wording does not imply a public plugin marketplace
  or remote/shell plugin support without proof.
- R9: Confirm the north-star use case is executable and covers the final user
  outcome, not just isolated child-goal tasks.

## Reviewer Blocking Criteria

Block if this program omits any requested child goal, leaves "extension APIs"
ambiguous, treats aggregate performance as sufficient, allows slower accepted
LS-Lint fixture rows, claims a true daemon without a process/IPC proof, treats
document graph support as experimental after promotion, implies Markdown linting
sits above structure validation, or plans a Markdown rewrite without first
measuring `rumdl` or a better Rust markdownlint-compatible candidate. Also
block if the child goals can complete without proving the north-star use case
end to end.

## Kickoff Prompt

```text
Execute docs/goals/assura-post-beta-capabilities-program.md as the parent
post-beta goal. Start with the workflow gate, git status, live roadmap, current
release state, checked performance artifact, Assura self-check, and the child
goal files. Execute the next incomplete child goal in order unless live evidence
justifies a recorded reorder. Complete each child to its proof gates with
independent review, update this program progress log, and report the next child
goal. Use narrow checks while iterating and full gates at child-goal or PR
readiness.
```

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Created the post-beta parent program after the beta completion audit identified remaining gaps in self-config dogfooding, fully supported document graph, true daemon mode, performance floor attribution, installed agent integrations, markdownlint-compatible Rust lint/fix, VS Code support, extension API clarity, LS-Lint reassessment, and release hardening. | User request; [Assura beta code-agnostic capabilities program](./assura-beta-code-agnostic-capabilities-program.md); `.trellis/spec/assura/roadmap.md`; `.trellis/tasks/archive/2026-07/07-01-post-beta-followup-roadmap-goals/research/markdown-linter-options.md`. |
| 2026-07-01 | Started execution after PR #113 was merged into `master` as `be4e33e`. The first child goal is self-config and documentation variance hardening because it dogfoods Assura's own coarse structure rules before deeper Markdown/content/reference work. Also clarified that this parent remains beta-track work and should produce a versioned beta increment, not a post-beta/GA claim. | [Self config and documentation variance hardening](./assura-self-config-doc-variance-hardening.md); `.trellis/tasks/archive/2026-07/07-01-self-config-doc-variance-hardening/prd.md`; `gh pr view 113 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/self-config-doc-variance-hardening origin/master`; `cargo run --quiet -- check --format json .`. |
| 2026-07-01 | Completed the first child goal locally and prepared it for review/PR integration. The slice refined self-config, removed stale live config snapshots, fixed active-doc trailing-space drift, added the final verification use-case package, updated active roadmap routing, and resolved independent-review findings about stale references. | [Self config and documentation variance hardening](./assura-self-config-doc-variance-hardening.md); `.assura/config.yml`; `.trellis/spec/assura/roadmap.md`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; review agent `019f1fdd-a0b1-7ec0-bb84-8edf7333561b`. |
| 2026-07-01 | Started the second child goal after PR #114 merged into `master` as `3aa17ea`. The supported document-graph slice promotes repository-reference graph behavior into the bounded content/query/context-pack contract so the final verification package can prove affected-reference questions without a daemon, editor plugin, hosted service, semantic ranking, or code-symbol provider. | [Supported document graph](./assura-supported-document-graph.md); `.trellis/tasks/07-01-supported-document-graph/prd.md`; `.trellis/spec/assura/roadmap.md`; `src/cli/content_query/context_pack.rs`; `tests/project_intelligence_context_pack.rs`. |
| 2026-07-01 | Started the third child goal after PR #115 merged into `master` as `4e972ad`. The true-daemon slice is scoped to replacing metadata-only lifecycle behavior with a real local process and versioned IPC health/check-path contract while keeping one-shot fallback and beta-track support boundaries intact. | [True daemon mode](./assura-true-daemon-mode.md); `.trellis/tasks/archive/2026-07/07-01-07-01-true-daemon-mode/prd.md`; `.trellis/tasks/archive/2026-07/07-01-supported-document-graph/prd.md`; `.trellis/spec/assura/roadmap.md`; `gh pr view 115 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/true-daemon-mode origin/master`. |
| 2026-07-01 | Completed the first true-daemon implementation slice locally. Daemon lifecycle commands now manage and probe a real local process, `doctor` reports stopped/running/crashed remediation, and `daemon check-path --format json` uses versioned IPC when the daemon is fresh while preserving one-shot fallback and stale-config failure behavior. Remaining daemon child work should broaden warm IPC to the rest of the final verification package and prove performance/staleness gates before claiming the entire child goal complete. | [True daemon mode](./assura-true-daemon-mode.md); `.trellis/tasks/archive/2026-07/07-01-07-01-true-daemon-mode/prd.md`; `src/cli/daemon_process.rs`; `tests/daemon_cli_tests.rs`; `.trellis/spec/assura/daemon-management-cli.md`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test daemon_core_tests --quiet`; `cargo test --test editor_surface_cli --quiet`; `cargo test --test agent_surface_cli --quiet`; `cargo xtask target-state`. |
| 2026-07-01 | Processed independent review on the daemon slice and tightened the parent-program evidence. The slice now records that stale daemon config cannot be reported as fresh, stale PID metadata cannot kill an unrelated process, failed starts surface runtime errors, and daemon subprocess tests prove cleanup/replacement behavior. | Review agent `019f2030-a4ac-77f0-b5a4-df30ce68e50e`; [True daemon mode](./assura-true-daemon-mode.md); `src/cli/daemon_lifecycle.rs`; `src/cli/daemon_process.rs`; `tests/daemon_cli_tests.rs`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`. |
| 2026-07-01 | Tightened the LS-Lint no-slower gate evidence during PR hardening. The headline `assura-cli` performance row now measures quiet successful validation, matching native LS-Lint's successful no-output behavior, while JSON report support remains covered by CLI/adoption tests. Local release evidence passes the strict no-slower gate after this change. | `src/cli/performance_report/assura_cli.rs`; `src/cli/performance_report/fixture_rows.rs`; `website/src/content/docs/reference/performance.mdx`; `website/src/content/docs/reference/performance-implementation.mdx`; `target/release/assura performance-report --output target/performance/pr116-local.json --iterations 5`; `cargo xtask performance-no-slower target/performance/pr116-local.json`. |
| 2026-07-01 | Scoped CI coverage to library unit tests after the true-daemon subprocess integration tests made tarpaulin coverage too slow for merge readiness. The product behavior remains covered by normal integration, platform, installability, daemon, and performance jobs; coverage now avoids supervising managed daemon subprocess lifecycles. | `.github/workflows/ci.yml`; `tests/daemon_cli_tests.rs`; `cargo fmt --check`; `cargo xtask target-state`. |
| 2026-07-01 | Continued true daemon mode after PR #116 merged as `dc36a95` because the child goal still required reference-query parity over daemon IPC before Markdown work builds on the daemon contract. The follow-up branch adds versioned IPC for `daemon references --source`, `--target`, and moved-target queries, including stale-config errors and one-shot fallback. | [True daemon mode](./assura-true-daemon-mode.md); `.trellis/tasks/archive/2026-07/07-01-07-01-true-daemon-mode/prd.md`; `.trellis/spec/assura/daemon-management-cli.md`; `src/cli/daemon_process.rs`; `tests/daemon_reference_cli_tests.rs`; `cargo test --test daemon_reference_cli_tests -- --test-threads=1`; `cargo test --test daemon_core_tests --quiet`. |
| 2026-07-01 | Tightened the parent final-verification lens and completed the daemon child's warm-reference proof locally. The parent now requires one runnable use-case package with fixture, CLI, daemon, agent, editor, support-matrix, and CI-gate transcripts; the daemon slice proves warm changed-path/reference IPC is materially faster than cold one-shot rows on a 3000-file Markdown fixture, including the review-found stale-source refresh case. | [True daemon mode](./assura-true-daemon-mode.md); `.trellis/tasks/archive/2026-07/07-01-07-01-true-daemon-mode/prd.md`; `src/cli/daemon_process.rs`; `target/performance/daemon-reference-ipc-local.json`; `hyperfine --warmup 1 --runs 5 ...`; review agent `019f212e-4928-76a0-a351-694f3dd4c279`. |
| 2026-07-02 | Completed the True Daemon Mode child goal after PR #117 merged into `master` as `7454552`. The parent program should continue with the Markdown Engine child, starting from [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md) and a fresh Trellis task/branch from `origin/master`. | [True daemon mode](./assura-true-daemon-mode.md); PR #117; merge commit `745455215d757e49fb4614a170e48f046cb829ad`; `.trellis/spec/assura/roadmap.md`. |
| 2026-07-02 | Re-centered the active Markdown Engine task on the parent final-verification package so the linter/fixer work must prove a maintainer workflow across staged checks, safe fixes, shared diagnostics, daemon/editor/agent reuse, and benchmark attribution instead of only running a faster Markdown command. | [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md); `.trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/prd.md`; `.trellis/spec/assura/roadmap.md`. |
| 2026-07-02 | Merged the verification-story checkpoint in PR #121 and continued the Markdown Engine child with isolated candidate probes. The first live probe keeps `rumdl` as the leading Rust candidate and records `mdlint`'s unrequested fixture mutation as an adapter safety risk. | PR #121; [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md); `.trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/markdown-engine-candidate-evaluation.md`; `cargo xtask markdown-engine-probe --run-external`. |
| 2026-07-02 | Merged isolated candidate probes in PR #122 and recorded the `rumdl` adapter boundary. The Markdown Engine child should next prove an optional subprocess adapter while preserving Assura's Rust 1.70 MSRV, then revisit direct library integration only with measured evidence. | PR #122; `docs/analysis/2026-07-02-rumdl-adapter-decision.md`; [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md). |
| 2026-07-02 | Merged the optional `rumdl` subprocess adapter proof in PR #124. The adapter is opt-in, maps selected `rumdl` diagnostics to stable Assura `markdown_*` IDs, reports setup/runtime failures as `markdown_engine`, uses isolated temporary Markdown copies so `assura check` cannot mutate source files, and avoids duplicate native/candidate findings for stable rules owned by enabled native checks. | PR #124; `src/cli/check/markdown/rumdl_adapter.rs`; `tests/markdown_rumdl_adapter_tests.rs`; [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md). |
| 2026-07-02 | Started the next Markdown Engine safe-fix slice from `origin/master`: `assura fix markdown` now defaults to the full supported deterministic safe-fix subset instead of trailing-spaces only, while targeted `--rule` runs remain available. This moves the parent verification story closer to one command that can preview/apply bounded Markdown repairs for agents and maintainers. | `src/cli/check/markdown_fix.rs`; `src/cli/args.rs`; `tests/markdown_lint_fix_tests.rs`; `docs/release-notes.md`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `website/src/content/docs/reference/configuration.md`. |
| 2026-07-02 | Added the primary verification story for this major increment so every child goal can be judged against one maintainer journey: renamed architecture docs, moved code, staged diagnostics, deterministic Markdown fixes, content graph queries, stale-safe daemon IPC, compact agent nudges, VS Code parity, and an LS-Lint no-slower fixture gate. | [Primary verification story](#primary-verification-story); `.trellis/spec/assura/roadmap.md`; `docs/data/public-roadmap.json`. |
| 2026-07-02 | Continued the Markdown Engine child with measured candidate evidence. The probe now supports opt-in timing and local evidence shows Rust candidates are materially faster than `markdownlint-cli2`, while `rumdl` is still slower than current Assura checks on the small fixture. This keeps the program aligned with the requirement to treat performance misses as defects or selection blockers instead of accepting broad speed claims. | `xtask/src/main.rs`; `.trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/markdown-engine-candidate-evaluation.md`; `.trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/markdown-engine-probe-2026-07-02-measured.json`. |
| 2026-07-02 | Merged representative probe/fix-validation evidence through PR #128, then closed the Markdown Engine child decision for this beta increment. The supported default remains Assura's native Markdown validation and safe-fix path; `rumdl` stays as an opt-in markdownlint-compatible adapter because it is functionally strongest but still slower than current Assura checks; `mdlint` is rejected as a supported fixer because it loses frontmatter and fails overlapping fix cases. | PR #128; [Markdown engine selection](../analysis/2026-07-02-markdown-engine-selection.md); [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md); `.trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/markdown-engine-candidate-evaluation.md`. |
| 2026-07-02 | Closed the Supported Document Graph child goal for this beta increment. The supported graph contract is now explicitly the local deterministic content/query workflow over content models, relation diagnostics, lexical search, bounded graph expansion, repository-reference queries, object-mode context packs, and local sessions; semantic search and code-symbol outputs remain optional candidate enrichment, guarded by target-state checks. The next child goal should be the performance floor and fixture gate. | [Supported document graph](./assura-supported-document-graph.md); [Supported document graph closure](../analysis/2026-07-02-supported-document-graph-closure.md); `.trellis/tasks/07-02-supported-document-graph-closure/prd.md`; `cargo xtask target-state`. |
| 2026-07-02 | Strengthened the parent verification story with user-specific goal criteria. Future child goals must show which maintainer decision they enable in the final branch-safety workflow, so the program cannot drift into isolated task completion without proving the intended end-state outcome. | [User-specific goal criteria](#user-specific-goal-criteria); `.trellis/tasks/07-02-supported-document-graph-closure/prd.md`. |
| 2026-07-02 | Closed the Performance Floor child for this beta increment. Performance rows now classify `fixture_acceptance`, `cargo xtask performance-no-slower` enforces every accepted LS-Lint-equivalent fixture row rather than aggregate-only speed, and checked performance data was regenerated from release binaries with all current accepted `realistic-equivalent` fixtures no slower than native LS-Lint. The next child goal should be Agent Installers. | [Performance floor and fixture gate](./assura-performance-floor-and-fixture-gate.md); `.trellis/tasks/07-02-performance-floor-fixture-gate/prd.md`; `benches/history/current.json`; `website/public/data/performance/current.json`; `cargo xtask performance-no-slower`. |
| 2026-07-02 | Closed the Agent Installers child for this beta increment. Assura now owns local install/update/remove/status/doctor lifecycle commands for Codex, OpenCode, Claude, and Pi integration bundles while keeping host-agent configuration as manual opt-in and preserving the shared nudge/check/daemon contracts. The next child goal should be VS Code Support. | [Agent integration lifecycle](./assura-agent-integration-lifecycle.md); `.trellis/tasks/07-02-agent-integration-lifecycle/prd.md`; `src/cli/agent_integration.rs`; `src/cli/agent_integration_bundle.rs`; `tests/agent_surface_cli.rs`; `cargo test --test agent_surface_cli --quiet`; `cargo xtask target-state`. |
| 2026-07-02 | Started the VS Code Support child and added its user-specific verification picture: a maintainer opens a documentation-heavy branch in VS Code, sees shared Assura diagnostics for moved docs/code, gets visible daemon fallback, previews safe Markdown fixes, and verifies honest local-package support before merge. | [VS Code supported extension](./assura-vscode-supported-extension.md); `.trellis/tasks/archive/2026-07/07-02-vscode-supported-extension/prd.md`; branch `codex/vscode-supported-extension`; `.trellis/spec/assura/roadmap.md`. |
| 2026-07-02 | Closed the VS Code Support child for this beta increment. The supported surface is a local beta package, not a marketplace or full LSP claim, and the package test/build/doctor/package smoke commands gate shared-contract diagnostics, visible daemon fallback, preview-only safe fixes, and support metadata. The next child goal should be Extension API Clarification. | [VS Code supported extension](./assura-vscode-supported-extension.md); `integrations/editors/vscode/package.json`; `integrations/editors/vscode/README.md`; `integrations/editors/vscode/tests/assura-client.test.js`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `docs/data/release-surfaces.json`; `pnpm test`; `pnpm run build`; `pnpm run doctor`; `pnpm run package`; `cargo xtask docs`; `cargo xtask evidence`. |
