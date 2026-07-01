---
id: goal-assura-beta-code-agnostic-capabilities-program
type: goal
title: Assura beta code-agnostic capabilities program
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-public-roadmap-artifact.md
  - ./assura-incremental-release-train.md
  - ./assura-beta-structure-severity-contract.md
  - ./assura-beta-content-collections-querying.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ./assura-code-doc-reference-validation.md
  - ./assura-reference-daemon-readiness.md
  - ./assura-daemon-management-cli.md
  - ./assura-beta-agent-nudge-integrations.md
  - ./assura-agent-daemon-awareness.md
  - ./assura-vscode-daemon-integration.md
  - ./assura-ls-lint-no-slower-performance-gate.md
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Beta Code-Agnostic Capabilities Program

## Objective

Drive Assura from the current Project Intelligence and Markdown Reference
planning state to a beta release that provides code-agnostic repository quality
capabilities with daemon-aware local workflows.

This is the overarching goal to kick off when the next large chunk of work
should proceed goal by goal. A future agent should start here, pick the next
highest-leverage incomplete major iteration, execute only that iteration's
referenced goal file or files to their proof gates, update this program, then
continue to the next iteration.

## Beta Product Bar

Beta means Assura can be used locally by humans, CI, editors, and agents to
validate and understand a repository without being tied to one programming
language. The beta surface must include:

- structure validation with stable rule IDs, severity, and concise messages;
- frontmatter and Assura collection modeling, validation, querying, and bounded
  context packs;
- high-performance Markdown linting and heading validation;
- repository-internal code/doc reference validation;
- daemon mode for warm, incremental, affected-path feedback;
- daemon management commands for humans, editors, hooks, and agents;
- concise nudges for Codex, OpenCode, Claude, and Pi agents at relevant events;
- a VS Code integration over shared daemon/client contracts;
- pre-1.0 release artifacts and a public roadmap;
- LS-Lint-equivalent performance gates where Assura is never slower than native
  LS-Lint on any headline fixture.

## Ten Major Iterations

| Order | Epic | Primary goal file(s) | Exit bar |
| --- | --- | --- | --- |
| 1 | Roadmap And Releases | [Public roadmap artifact](./assura-public-roadmap-artifact.md), [Incremental release train](./assura-incremental-release-train.md) | Public roadmap and release train are repo-backed, validated, and ready to report beta progress. |
| 2 | Structure Severity | [Beta structure severity contract](./assura-beta-structure-severity-contract.md) | Structure findings have stable severity, rule IDs, messages, and agent-friendly remediation. |
| 3 | Collections Querying | [Beta content collections and querying](./assura-beta-content-collections-querying.md) | Frontmatter collections can be modeled, validated, queried, expanded, and packed for agents. |
| 4 | Markdown Quality | [Markdown lint and repository reference engine](./assura-markdown-lint-link-reference-engine.md) | Markdown linting, required headings, severity, suppressions, and safe fixes work locally and quickly. |
| 5 | Reference Graph | [Code and documentation reference validation](./assura-code-doc-reference-validation.md) | Markdown, code comment, docstring, file, heading, and line/range references are validated with inbound/outbound edges. |
| 6 | Daemon Core | [Reference daemon readiness](./assura-reference-daemon-readiness.md) | Warm daemon/session checks match one-shot truth and provide bounded affected-path feedback. |
| 7 | Daemon CLI | [Daemon management CLI](./assura-daemon-management-cli.md) | Status, start, stop, restart, doctor, logs, and fallback commands are JSON-capable and shared. |
| 8 | Agent Nudges | [Beta agent nudge integrations](./assura-beta-agent-nudge-integrations.md), [Agent daemon awareness](./assura-agent-daemon-awareness.md) | Codex, OpenCode, Claude, and Pi agents can receive concise event-aware nudges without context bloat. |
| 9 | VS Code | [VS Code daemon integration](./assura-vscode-daemon-integration.md) | VS Code diagnostics, status, and commands use the shared daemon/client contract. |
| 10 | LS-Lint Gate | [LS-Lint no-slower performance gate](./assura-ls-lint-no-slower-performance-gate.md) | No headline LS-Lint-equivalent fixture is slower than native LS-Lint; CI/review blocks regressions. |

## Execution Rules

- The table order is the default dependency order, not a rigid queue. Reorder
  epics when doing so avoids refactoring, keeps validation narrower, or
  unblocks a prerequisite more efficiently.
- Record the reason whenever work skips an earlier incomplete epic.
- Each epic should land as one or more PRs scoped to its referenced goal files.
- Complex implementation epics require independent review before PR creation.
- After each epic, update this program's progress log with PRs, validation
  commands, review artifacts, and the next epic.
- Use release-train work after any user-facing supported or experimental
  capability, not only at the end.
- Do not claim daemon, hook, editor, or performance support publicly until the
  relevant goal and release evidence proves it.
- The LS-Lint no-slower gate applies to every PR that changes
  LS-Lint-equivalent structure validation, traversal, ignore handling, rule
  planning, performance reporting, or fixture classification.

## Validation Cadence

Avoid waiting on broad checks after every small edit. Use staged validation:

- Planning/docs edits: run the workflow gate, `cargo run --quiet -- check
  --format json .`, `cargo xtask docs`, `cargo xtask evidence`, and
  `git diff --check` before commit or handoff.
- Rust implementation edits: run `cargo fmt --check` and the narrowest relevant
  test target while iterating. Run workspace tests, clippy, docs, evidence, and
  self-check at epic or PR readiness.
- Website edits: run the website build or `cargo xtask docs` when pages,
  generated content, or public roadmap data changes. Do not rebuild the website
  after unrelated Rust-only edits unless generated docs are touched.
- Performance edits: use a target artifact and short iteration count while
  diagnosing. Run the accepted 5-iteration checked report only for performance
  gate readiness or when updating tracked benchmark history.
- Daemon/editor/agent edits: prefer protocol-specific and changed-path tests
  during iteration. Run broad integration tests before support status changes
  or release claims.

No beta epic is complete until its own proof gates pass and the master program
records the evidence.

## Completion Assessment Process

Before marking this program or any major epic complete:

1. Derive concrete requirements from this file and the referenced child goals.
2. Inspect current files, command output, release state, PR state, and checked
   artifacts instead of relying on intent or prior conversation.
3. Run an independent reviewer agent over the relevant goal files and evidence.
4. Address valid major reviewer findings or record why they are intentionally
   deferred with a replacement path.
5. Update the progress log with the reviewer, validation commands, and next
   epic.

## Performance Gate Policy

The beta program rejects aggregate-only performance claims. For accepted
headline LS-Lint-equivalent fixtures:

- every fixture must be measured against native LS-Lint;
- every accepted cold CLI row must be no slower than LS-Lint;
- slower rows block merge even if the aggregate is faster;
- warm daemon/session rows are valuable but cannot pass the cold CLI gate;
- "CLI floor" is an attribution topic, not an excuse to merge slower behavior;
- any fixture removed from the headline set needs written rationale and review.

## Kickoff Prompt

```text
Execute docs/goals/assura-beta-code-agnostic-capabilities-program.md as the
master beta goal. Start with the workflow gate, git status, live roadmap,
current PRs, release state, and current performance claim summary from
benches/history/current.json. Then select the next highest-leverage incomplete
epic from the Ten Major Iterations table, read its referenced goal file(s),
execute only that epic to its proof gates with independent review for complex
implementation, update the beta program progress log, and report the next epic
and goal path. Use narrow checks while iterating and full gates at epic/PR
readiness. Do not skip the LS-Lint no-slower gate for any
structure/performance change.
```

## Definition Of Done

- All ten major iterations are completed or explicitly deferred with a
  reviewer-accepted replacement path.
- Public docs and release artifacts classify beta-supported, experimental,
  future, and unsupported surfaces consistently.
- `assura check`, daemon workflows, agent nudges, VS Code, and content/query
  commands share the same core validation and finding contracts.
- Agent nudges stay bounded and event-relevant.
- The checked performance gate fails if any headline LS-Lint-equivalent fixture
  is slower than native LS-Lint.
- A beta release tag and GitHub release artifact exist with validation evidence.

## Validation Commands

Planning-only updates to this program should run:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

Implementation epics add their own validation commands.

## Review Tasks

- R1: Confirm the ten iterations reflect the actual missing beta capabilities.
- R2: Confirm every iteration points to an executable goal file.
- R3: Confirm agent integrations do not revive per-agent validation logic.
- R4: Confirm daemon and editor support are not claimed before proof exists.
- R5: Confirm the LS-Lint no-slower gate blocks per-fixture regressions.

## Reviewer Blocking Criteria

Block if the program is only a roadmap without executable goal files, omits
daemon mode, omits Codex/OpenCode/Claude/Pi agent nudges, omits frontmatter and
collection querying, omits Markdown/reference validation, lets VS Code bypass
the shared daemon contract, or allows any headline LS-Lint fixture to be slower
than native LS-Lint without blocking the merge.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Created the beta master program with ten major iterations and a hard no-slower LS-Lint performance gate after review clarified the desired beta destination. | User request; [.trellis/spec/assura/roadmap.md](../../.trellis/spec/assura/roadmap.md); `jq '.claim_summary,.warm_claim_summary' benches/history/current.json`. |
| 2026-06-30 | Started execution under the persistent beta goal. Current workflow gate is ready on branch `codex/markdown-reference-master-goal`; repo is clean; current checked performance summary shows cold `assura-cli` faster on 7 of 8 realistic-equivalent fixtures and warm session faster on 8 of 8. Added dependency-aware ordering, staged validation cadence, and explicit independent-review completion assessment. | `python3 ./.trellis/scripts/workflow_gate.py --platform codex`; `git status --short --branch`; `jq '{timestamp, claim_summary, warm_claim_summary, ls_lint_status}' benches/history/current.json`. |
| 2026-06-30 | Pulled the LS-Lint gate forward because it protects later structure/performance work without requiring repeated full benchmark runs. Added a cheap `cargo xtask performance-no-slower` command over existing report JSON; current checked data is expected to fail until the `simple_library` cold Assura row is no slower than native LS-Lint. | [assura-ls-lint-no-slower-performance-gate.md](./assura-ls-lint-no-slower-performance-gate.md); `xtask/src/main.rs`; `cargo test -p xtask performance_no_slower`; `cargo xtask performance-no-slower`. |
| 2026-06-30 | Advanced the LS-Lint gate from known-failing to green. The `simple_library` miss was a stale release-build recipe measuring a default-feature launcher; the remaining `rule_heavy_repo` miss was fixed with exact extension-segment lookup for LS-Lint-style naming patterns. Checked data now shows cold `assura-cli` faster on 8 of 8 headline fixtures; warm session remains faster on 8 of 8. | [assura-ls-lint-no-slower-performance-gate.md](./assura-ls-lint-no-slower-performance-gate.md); `cargo xtask performance-no-slower`; `jq '.claim_summary,.warm_claim_summary' benches/history/current.json`. |
| 2026-06-30 | Completed the no-slower gate enforcement slice by adding CI Performance Report enforcement and target-state drift checks. This closes the beta program's immediate performance blocker while keeping the stricter 2x claim as separate future performance work. | `.github/workflows/ci.yml`; `xtask/src/main.rs`; `cargo xtask target-state`; `cargo xtask performance-no-slower`. |
| 2026-06-30 | Completed the public roadmap artifact slice of Epic 1 with a repo-owned JSON artifact, website page, sidebar route, and target-state drift checks. Release-train work remains the next Roadmap And Releases slice. | [assura-public-roadmap-artifact.md](./assura-public-roadmap-artifact.md); `docs/data/public-roadmap.json`; `website/src/content/docs/roadmap.mdx`; `cargo xtask target-state`; independent review. |
| 2026-06-30 | Completed the release-train readiness slice of Epic 1 with a structured release-surface manifest and `cargo xtask release-readiness --format json`. The command intentionally fails while latest GitHub release `v0.1.0` still lacks current branch supported/experimental surfaces. Next default epic is Structure Severity. | [assura-incremental-release-train.md](./assura-incremental-release-train.md); `docs/data/release-surfaces.json`; `docs/release-train.md`; `cargo xtask release-readiness --format json`; independent review. |
| 2026-06-30 | Completed Epic 2, Structure Severity. The shared finding contract now exposes advisory/blocking severity fields across check reports and agent feedback, `--fail-fast` continues past advisory findings to later blocking validators, and docs describe the beta contract. Next default epic is Collections Querying. | [assura-beta-structure-severity-contract.md](./assura-beta-structure-severity-contract.md); `src/cli/check/report.rs`; `tests/cli_check_warn_tests.rs`; `cargo fmt --check`; `cargo test --test cli_check_warn_tests --quiet`; `cargo test --test cli_command_surface_tests --quiet`; `cargo test --test real_project_agentic_feedback_tests --quiet`; `cargo xtask docs`; `cargo xtask evidence`; `cargo xtask target-state`; independent review Bohr. |
| 2026-06-30 | Completed Epic 3, Collections Querying, as a beta closure/hardening slice over already-completed child goals. The release manifest now has a distinct content collections/querying surface, docs separate supported model-backed collection validation/querying from semantic/code-symbol candidate enrichment, and the persistent-session startup proof passes after reviewer-requested hardening. Next default epic is Markdown Quality. | [assura-beta-content-collections-querying.md](./assura-beta-content-collections-querying.md); [assura-content-model-source-of-truth.md](./assura-content-model-source-of-truth.md); [assura-content-query-and-search-cli.md](./assura-content-query-and-search-cli.md); [assura-project-intelligence-context-pack.md](./assura-project-intelligence-context-pack.md); [assura-project-intelligence-persistent-session.md](./assura-project-intelligence-persistent-session.md); `docs/data/release-surfaces.json`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `cargo test --test project_intelligence_session --quiet`; `cargo xtask docs`; `cargo xtask evidence`; independent review Hypatia. |
| 2026-06-30 | Started Epic 4, Markdown Quality, with the first `markdown.check_links` validation slice. Markdown Quality is now in the public roadmap active lane and the new local-link behavior is tracked as an experimental unreleased surface; the remaining Markdown epic work still includes broader linting, suppressions, missing-heading safe fixes, Markdown link fact ingestion, and performance evidence. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `docs/data/public-roadmap.json`; `docs/data/release-surfaces.json`; `src/cli/check/markdown/links.rs`; `tests/markdown_link_reference_tests.rs`; reviewer Pasteur. |
| 2026-06-30 | Continued Epic 4 with Markdown rule severity and reasoned suppression support. This advances the Markdown Quality exit bar for warning/error behavior and intentional exceptions while keeping broad linting, safe heading insertion, link facts, and performance evidence as the next incomplete Markdown slices. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `src/cli/check/markdown/suppression.rs`; `tests/markdown_suppression_severity_tests.rs`; `crates/assura-check-cli/tests/compiled_markdown_cli.rs`; `docs/data/release-surfaces.json`; `cargo test --test markdown_suppression_severity_tests --quiet`; `cargo test -p assura-check-cli --test compiled_markdown_cli --quiet`; independent review Jason. |
| 2026-07-01 | Continued Epic 4 with outbound Markdown link facts. This removes the reference-graph prerequisite that Markdown-authored links be available as stable Project Intelligence facts while keeping broad Markdown linting, malformed non-link reference detection, missing-heading safe fixes, and Markdown performance evidence as the next incomplete Markdown slices. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `src/markdown/links.rs`; `src/intelligence/facts/markdown_link_ingest.rs`; `tests/markdown_link_fact_tests.rs`; `docs/project-intelligence-facts.md`; `cargo test --test markdown_link_fact_tests --quiet`; `cargo test --test markdown_link_reference_tests --quiet`; `cargo test --lib docs_lifecycle --quiet`; independent review Goodall. |
| 2026-07-01 | Continued Epic 4 with malformed non-link local reference detection and corrected nested Markdown rule configuration. `markdown.check_links` now reports existing local file references in prose or inline code that should be rendered as Markdown links, and severity overrides now use `markdown.rules.<rule_id>.severity` so later rule options can attach to the rule object. Broad markdownlint compatibility, missing-heading safe fixes, and Markdown performance evidence remain open. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `src/markdown/links.rs`; `src/cli/check/markdown/links.rs`; `src/config/config/bundles/markdown.rs`; `tests/markdown_link_reference_tests.rs`; `tests/markdown_suppression_severity_tests.rs`; `website/src/content/docs/product/markdown-validation.md`; `website/src/content/docs/reference/configuration.md`. |
| 2026-07-01 | Continued Epic 4 with deterministic missing-heading safe fixes for configured `markdown.required_sections`. `assura fix markdown --rule required-sections` now supports dry-run/apply audit output for bounded heading appends, leaving broad markdownlint compatibility and Markdown performance evidence as the remaining Markdown Quality gaps. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `src/cli/check/markdown_fix.rs`; `src/cli/check/markdown_required_sections_fix.rs`; `tests/markdown_required_section_fix_tests.rs`; `docs/release-notes.md`; `website/src/content/docs/product/markdown-validation.md`; `website/src/content/docs/reference/configuration.md`. |
| 2026-07-01 | Continued Epic 4 with the opt-in `markdown.lint_common` bundle for common Rust-native Markdown linting. Independent review Laplace found and confirmed fixes for frontmatter-aware heading scanning, compiled artifact schema/version coverage, and config-surface support metadata. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `src/cli/check/markdown/common_lint.rs`; `tests/markdown_common_lint_tests.rs`; `crates/assura-check-cli/tests/compiled_markdown_cli.rs`; `docs/data/release-surfaces.json`; `website/src/content/docs/product/markdown-validation.md`; `cargo test --test markdown_common_lint_tests --quiet`; `cargo test --test markdown_suppression_severity_tests --quiet`; `cargo test -p assura-check-cli --test compiled_markdown_cli --quiet`; `cargo test -p assura --lib docs_claim_surfaces_reads_config_surface_rows --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask docs`; `cargo xtask evidence`; `cargo xtask target-state`; independent review Laplace. |
| 2026-07-01 | Added Markdown common-lint release-mode timing evidence. The copied docs corpus check remains fast with diagnostics rendered: 23.2 ms mean for `markdown.lint_common: true` over 175 Markdown files and 25 findings, versus 14.0 ms with common lint disabled. Next default epic is Reference Graph unless a closure reviewer requires more Markdown Quality work first. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); [2026-06-18-markdown-tooling-evaluation.md](../analysis/2026-06-18-markdown-tooling-evaluation.md); `cargo build --release --quiet`; `hyperfine --warmup 5 --runs 30 --ignore-failure "target/release/assura check --format json $bench_root/off" "target/release/assura check --format json $bench_root/on"`. |
| 2026-07-01 | Completed Epic 4, Markdown Quality. Independent closure review Anscombe found no blocker-level gaps and recommended proceeding to Reference Graph; full markdownlint-compatible third-party coverage remains a planned future dependency or external-binary decision rather than a beta blocker. Next default epic is Reference Graph. | [assura-markdown-lint-link-reference-engine.md](./assura-markdown-lint-link-reference-engine.md); `cargo test --test markdown_link_reference_tests --quiet`; `cargo test --test markdown_suppression_severity_tests --quiet`; `cargo test --test markdown_required_section_fix_tests --quiet`; `cargo test --test markdown_common_lint_tests --quiet`; `cargo run --quiet -- check --format json .`; `git diff --check`; independent review Anscombe. |
| 2026-07-01 | Started Epic 5, Reference Graph. Archived the completed Markdown Trellis task, created `.trellis/tasks/06-30-07-01-reference-graph-validation`, revalidated the child goal against current MarkdownLink facts, and moved the public roadmap Now lane from Markdown Quality to Reference Graph. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); `.trellis/tasks/06-30-07-01-reference-graph-validation/prd.md`; [.trellis/spec/assura/roadmap.md](../../.trellis/spec/assura/roadmap.md); `docs/data/public-roadmap.json`; `python3 ./.trellis/scripts/workflow_gate.py --platform codex`; `git status --short --branch`. |
| 2026-07-01 | Continued Epic 5 with the first repository-reference edge slice. Existing MarkdownLink facts now emit `RepositoryReference` edges, the in-memory fact store can answer inbound references for resolved target resources, and `content agent-context` loads Markdown links so the public query surface reports repository-reference counts; code/comment/docstring discovery remains the next Reference Graph slice. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); `src/intelligence/facts/markdown_link_ingest.rs`; `src/intelligence/store.rs`; `src/cli/content_query/context.rs`; `tests/repository_reference_graph_tests.rs`; `tests/content_query_cli.rs`; `docs/project-intelligence-facts.md`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test content_query_cli content_query_reports_generic_agent_context --quiet`. |
| 2026-07-01 | Continued Epic 5 with conservative source/comment/string reference discovery. Graph-oriented content commands now feed lower-confidence `RepositoryReference` edges into the shared fact graph with bounded source/config scans, unresolved local targets remain graph context instead of public diagnostics, and `content agent-context` summary counts include both Markdown and source-derived references. The epic still needs changed-source/changed-target query proofs and broader check-report integration before closure. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); `src/intelligence/facts/repository_reference_ingest.rs`; `src/cli/content_query/context.rs`; `tests/repository_reference_graph_tests.rs`; `tests/content_query_cli.rs`; `docs/project-intelligence-facts.md`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test content_query_cli content_query_reports_generic_agent_context --quiet`. |
| 2026-07-01 | Continued Epic 5 with affected-set query proof. `assura content references` now exposes bounded changed-source outbound targets and changed-target inbound sources over the shared `RepositoryReference` graph; independent review Euclid's default-output context blocker was fixed before commit. Broader check-report integration and closure review remain before moving to Daemon Core. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); `src/cli/content_query/mod.rs`; `src/intelligence/store.rs`; `tests/repository_reference_graph_tests.rs`; `tests/content_query_cli.rs`; `cargo fmt --check`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test content_query_cli content_query_reports_repository_reference_context --quiet`; `cargo test --test content_query_cli content_query_references_requires_exactly_one_direction --quiet`; `cargo xtask target-state`; `cargo run --quiet -- check --format json .`; `git diff --check`; independent review Euclid. |
| 2026-07-01 | Continued Epic 5 with opt-in check-report integration. `extensions.repository_references` reports locally provable source/comment/docstring missing targets, Markdown anchors, and line anchors while keeping the default structure fast path unchanged; independent review Rawls's compiled fast-artifact blocker was fixed with fast-plan disqualification coverage. Epic 5 now needs closure review against the full child goal before moving to Daemon Core. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); `src/cli/check/repository_references.rs`; `tests/repository_reference_check_tests.rs`; `cargo test --test repository_reference_check_tests --quiet`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test -p assura --lib fast_plan_rejects_repository_reference_diagnostics --quiet`; `cargo xtask target-state`; `cargo run --quiet -- check --format json .`; `cargo xtask docs`; `cargo xtask evidence`; independent review Rawls. |
| 2026-07-01 | Completed Epic 5, Reference Graph. Independent closure review Kant found no blockers and recommended completion; Reference Graph now has Markdown and source-derived repository-reference facts, inbound/outbound affected-set queries, GitHub-renderable Markdown link enforcement, and opt-in check-report diagnostics without LSP or remote services. Next default epic is Daemon Core. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); `cargo fmt --check`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test repository_reference_check_tests --quiet`; `cargo test --test markdown_link_reference_tests --quiet`; `cargo test -p assura --lib fast_plan_rejects_repository_reference_diagnostics --quiet`; `cargo test -p assura --lib docs_claim_surfaces_reads_config_surface_rows --quiet`; `cargo xtask target-state`; `cargo run --quiet -- check --format json .`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`; independent closure review Kant. |
| 2026-07-01 | Started Epic 6, Daemon Core, with a shared `LocalDaemonCore` state contract. This proves the non-process daemon-ready layer can answer prepared changed-path structure checks, bounded changed-source and changed-target reference questions, config-stale failures, prior-graph target-delete feedback, and one-shot fallback metadata for future CLI/editor/hook/agent clients. Full daemon management commands remain Epic 7. | [assura-reference-daemon-readiness.md](./assura-reference-daemon-readiness.md); `src/daemon/mod.rs`; `src/daemon/fingerprint.rs`; `tests/daemon_core_tests.rs`; `cargo test --test daemon_core_tests --quiet`; `cargo test --lib daemon::tests::health_states_are_serialized_for_clients --quiet`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test repository_reference_check_tests --quiet`; `cargo fmt --check`. |
| 2026-07-01 | Hardened the first Daemon Core slice after independent review. The shared daemon state now preserves explicit config fallback semantics, handles configs outside `.assura/`, returns structured stale health for missing config, proves mutated-source parity against `assura content references`, and exposes target-move feedback with old/new path context. Reviewer Bacon found no remaining blocker or high-risk findings. | [assura-reference-daemon-readiness.md](./assura-reference-daemon-readiness.md); `src/cli/content_query/context.rs`; `src/daemon/types.rs`; `tests/daemon_core_tests.rs`; `cargo test --test daemon_core_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; independent review Bacon. |
| 2026-07-01 | Continued Epic 6 with an experimental `assura daemon` probe surface for local clients. `daemon health`, `daemon check-path`, and `daemon references` now serialize daemon-ready health, changed-path structure checks, and bounded reference context over the same `LocalDaemonCore`; reviewer-requested hardening added pre-load reference-flag validation and structured JSON unavailable health output. The full start/stop/status/logs management surface remains the next Epic 7 goal. | [assura-reference-daemon-readiness.md](./assura-reference-daemon-readiness.md); `src/cli/daemon.rs`; `tests/daemon_cli_tests.rs`; `.assura/command-surface.yml`; `docs/data/release-surfaces.json`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test daemon_core_tests --quiet`; `cargo run --quiet -- check --format json .`; independent review Galileo. |
| 2026-07-01 | Completed Epic 6, Daemon Core. Independent closure review Locke found no blockers and recommended moving to Epic 7; the remaining low-risk target-side CLI parity assertion should be added in the daemon management CLI work. Next default epic is Daemon CLI via [assura-daemon-management-cli.md](./assura-daemon-management-cli.md). | [assura-reference-daemon-readiness.md](./assura-reference-daemon-readiness.md); `cargo fmt --check`; `cargo test --test daemon_core_tests --quiet`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test repository_reference_check_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `git diff --check`; independent closure review Locke. |
| 2026-07-01 | Started Epic 7, Daemon CLI, with JSON-first `daemon status` and `daemon doctor` management-preview commands plus target-side daemon/content reference parity coverage. Independent review Pauli found no blocker or high-risk findings. This is not Epic 7 closure: idempotent start/stop/restart/logs lifecycle commands remain the next incomplete slice before daemon CLI can be marked complete. | [assura-daemon-management-cli.md](./assura-daemon-management-cli.md); `src/cli/daemon.rs`; `src/cli/daemon_management.rs`; `src/cli/daemon_text.rs`; `tests/daemon_cli_tests.rs`; `.assura/command-surface.yml`; `.assura/config.yml`; `docs/compatibility-and-surface.md`; `docs/support-policy.md`; `docs/release-notes.md`; `xtask/src/main.rs`; `cargo fmt --check`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent review Pauli. |
| 2026-07-01 | Continued Epic 7 with idempotent runtime-metadata lifecycle commands and bounded logs. Independent review Averroes found no blocker or high-risk findings; suggested hardening added unavailable stop/logs, repeated restart, and log truncation coverage. This closes the CLI shape for `start`, `stop`, `restart`, and `logs` without claiming a long-running socket/process server; closure review is still needed before Epic 7 can complete. | [assura-daemon-management-cli.md](./assura-daemon-management-cli.md); `src/cli/daemon.rs`; `src/cli/daemon_lifecycle.rs`; `src/cli/daemon_management.rs`; `tests/daemon_cli_tests.rs`; `.assura/command-surface.yml`; `.assura/config.yml`; `.trellis/spec/assura/daemon-management-cli.md`; `docs/compatibility-and-surface.md`; `docs/support-policy.md`; `docs/release-notes.md`; `xtask/src/main.rs`; `cargo fmt --check`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent review Averroes. |
| 2026-07-01 | Filled the remaining Epic 7 status-contract gap before closure review. `daemon status --format json` now includes project metadata with a stable config fingerprint and git dirty paths, covering the explicit machine-readable status fields named by the Daemon CLI goal. | [assura-daemon-management-cli.md](./assura-daemon-management-cli.md); `src/cli/daemon_management.rs`; `tests/daemon_cli_tests.rs`; `.trellis/spec/assura/daemon-management-cli.md`; `cargo test --test daemon_cli_tests --quiet`. |
| 2026-07-01 | Completed Epic 7, Daemon CLI. Independent closure review Gibbs found no blockers and accepted the experimental runtime-metadata preview as complete for this epic because docs do not claim a long-running socket/process server. Next default epic is Agent Nudges via [assura-beta-agent-nudge-integrations.md](./assura-beta-agent-nudge-integrations.md) and [assura-agent-daemon-awareness.md](./assura-agent-daemon-awareness.md). | [assura-daemon-management-cli.md](./assura-daemon-management-cli.md); `cargo fmt --check`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent closure review Gibbs. |
| 2026-07-01 | Started Epic 8, Agent Nudges, with a shared `assura agent nudge` payload. The new experimental command covers session-start, before-tool, and after-tool event policy, labels Codex/OpenCode/Claude/Pi host integrations without adding private validation paths, and summarizes daemon health, changed-path findings, reference context, and performance-gate reminders while pointing to existing `check --format agent` and daemon commands for detail. Hook/plugin recipes and closure review remain before Epic 8 is complete. | [assura-beta-agent-nudge-integrations.md](./assura-beta-agent-nudge-integrations.md); [assura-agent-daemon-awareness.md](./assura-agent-daemon-awareness.md); `src/cli/agent_nudge.rs`; `src/cli/agent_nudge_helpers.rs`; `tests/agent_surface_cli.rs`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `integrations/agents/README.md`; `cargo fmt --check`; `cargo test --test agent_surface_cli --quiet`; `cargo test --test cli_command_surface_tests --quiet`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test real_project_agentic_feedback_tests --quiet`; `cargo run --quiet -- check --format agent --agent codex .`. |
| 2026-07-01 | Hardened the Epic 8 shared nudge payload after independent review. The daemon-unavailable fallback now points wrappers at `assura check --format agent`, `--max-issues` bounds the combined injected payload, omitted findings are counted, per-finding detail commands preserve the target path and agent label, and stale OpenCode prototype docs no longer claim a completed beta plugin. Epic 8 remains active for agent-specific hook/plugin recipes before closure review. | [assura-beta-agent-nudge-integrations.md](./assura-beta-agent-nudge-integrations.md); `src/cli/agent_nudge.rs`; `src/cli/agent_nudge_helpers.rs`; `tests/agent_surface_cli.rs`; `integrations/agents/README.md`; `integrations/agents/opencode/README.md`; independent reviews Russell, Volta, Euler; `cargo test --test agent_surface_cli --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `git diff --check`. |
| 2026-07-01 | Continued Epic 8 with concrete Codex, OpenCode, Claude, and Pi hook/plugin recipes over the shared `assura.agent-nudge.v1` payload. Recipes now cover session-start, before-tool, after-tool, deep diagnostics, and daemon recovery without introducing per-agent validation logic; closure review remains the next step before moving to VS Code. | [assura-beta-agent-nudge-integrations.md](./assura-beta-agent-nudge-integrations.md); [assura-agent-daemon-awareness.md](./assura-agent-daemon-awareness.md); `integrations/agents/README.md`; `integrations/agents/codex/README.md`; `integrations/agents/opencode/README.md`; `website/src/content/docs/reference/agent-feedback.md`; `tests/agent_surface_cli.rs`; `cargo test --test agent_surface_cli --quiet`. |
| 2026-07-01 | Completed Epic 8, Agent Nudges. Independent closure review Faraday found no blocker or high-risk gaps; follow-up hardening tightened path-read event wording and expanded the shared nudge smoke across all four target agents and all three event types. Next default epic is VS Code via [assura-vscode-daemon-integration.md](./assura-vscode-daemon-integration.md). | [assura-beta-agent-nudge-integrations.md](./assura-beta-agent-nudge-integrations.md); [assura-agent-daemon-awareness.md](./assura-agent-daemon-awareness.md); independent closure review Faraday; `cargo fmt --check`; `cargo test --test agent_surface_cli --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `git diff --check`. |
