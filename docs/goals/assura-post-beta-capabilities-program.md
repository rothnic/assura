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
   requirement in the supported path.
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
| 2026-07-01 | Created the post-beta parent program after the beta completion audit identified remaining gaps in self-config dogfooding, fully supported document graph, true daemon mode, performance floor attribution, installed agent integrations, markdownlint-compatible Rust lint/fix, VS Code support, extension API clarity, LS-Lint reassessment, and release hardening. | User request; [Assura beta code-agnostic capabilities program](./assura-beta-code-agnostic-capabilities-program.md); `.trellis/spec/assura/roadmap.md`; `.trellis/tasks/07-01-post-beta-followup-roadmap-goals/research/markdown-linter-options.md`. |
