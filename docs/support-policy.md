---
title: Support Policy
status: active
---

# Support Policy

This policy applies to Assura pre-1.0 releases.

The policy describes the current `v0.3.0` beta-increment surface. `v0.3.0` was
published and live-verified on 2026-07-02 with all expected archives and
checksums reachable. Assura remains pre-1.0 beta.

## Support Levels

| Surface | Level | Policy |
| --- | --- | --- |
| `assura check` structure validation | Supported | Bugs that produce incorrect pass/fail results are release blockers. |
| `assura check --format json` and `--format yaml` | Supported | Schema changes must be called out in release notes before 1.0. |
| `assura check --format advice` and `--format status` | Supported | Guided and compact output must stay deterministic enough for local hooks and agent tools. |
| `assura check --format agent` | Supported | Agent JSON shape must remain deterministic and documented. |
| `--agent codex` delivery | Supported adapter | Delivery may depend on user-approved Codex hooks, but it must not require a separate CLI. |
| `assura init` | Supported | Starter config output must be valid and self-checkable. |
| `assura status --format json` | Supported | JSON summaries must remain usable for automation. |
| `assura doctor` | Experimental local project doctor | Read-only project diagnostics report configured checks, inactive capabilities, recommended agent-project gaps, draft model files not wired into config, binary custody status, and ranked next actions. It must not mutate files or imply inactive capabilities are violations by default. |
| `assura explain` | Experimental local path explanation | Read-only path diagnostics report applied scopes, inherited rules, skipped checks, binary/read behavior, suppressions, and next actions for one path. It must reuse structure-check semantics rather than adding a separate validation engine. |
| `assura migrate` for complete LS-Lint 2.3 config semantics | Supported | Invalid LS-Lint config shapes and unsupported rule syntax must fail clearly. CLI drop-in parity is out of scope. |
| `assura hooks` for local git hooks | Supported local workflow | Hooks must be opt-in and local to a checkout. |
| `assura quality plan` | Supported local workflow | Quality-scope planning must stay config-backed and deterministic. |
| `assura performance-report` | Supported evidence command | Claims must cite checked benchmark or CI artifacts. |
| `markdown.lint_common` | Experimental Markdown validation | The common lint bundle must stay local and offline, report stable `markdown_*` rule IDs, and remain suppressible/configurable through the shared Markdown severity contract. |
| Repository reference facts and queries | Supported project-intelligence graph | Conservative Markdown, comment, docstring, string-literal, and configured frontmatter path references appear as bounded `RepositoryReference` edges in graph-oriented content surfaces. `assura content references` and object-mode context packs expose inbound, outbound, all, and unresolved repository-reference context without requiring semantic search, code-symbol providers, a daemon, or a hosted service. |
| `extensions.repository_references` checks | Experimental first-party | Opt-in checks may report locally provable missing targets, missing Markdown anchors, and invalid line anchors from supported source references and configured Markdown frontmatter fields. They do not make lower-confidence repository-reference candidates validation truth. |
| `assura fix markdown --dry-run --format json` | Experimental safe-fix preview contract | Dry-run output defaults to every supported deterministic Markdown safe-fix class, reports proposed bounded writes, and must not modify files. |
| `assura fix markdown --apply --format json` | Experimental safe-fix apply/audit contract | Apply output defaults to the same supported safe-fix subset and must report changed paths, applied fix IDs, skipped fixes, idempotent reruns, and VCS-first rollback guidance. |
| `assura agent` | Supported local agent project-intelligence surface | JSON-default commands for context, diagnostics, context packs, search/show/expand, missing relations, safe-fix previews, and local sessions must delegate to the shared content-query contracts. MCP or remote access is not required. |
| `assura agent onboard` | Experimental local agent-ready onboarding surface | Creates or preserves a broad non-domain-specific baseline, optionally activates `--content-template agent-project` or `document-project`, generates `.assura/onboarding/`, runs local verification, and tells agents which specialization questions to ask next. The onboarding report and packet include explicit nudge, warn, and gate lifecycle profiles over existing `assura agent nudge` and `assura check --format agent` commands. The generated baseline uses reusable dynamic skill-directory contracts instead of enumerating every project-local skill. It must not implement remote bootstrap behavior, silently overwrite user-authored files, silently wire host-agent config, or add domain-specific packs. |
| `assura agent nudge` | Experimental local agent nudge payload | Event-aware nudges for Codex, OpenCode, Claude, and Pi wrappers must stay bounded, avoid volatile default fields, reuse `assura check --format agent` and daemon contracts, and never introduce per-agent validation logic. |
| `assura agent integration` | Experimental local agent integration lifecycle | Install, update, remove, status, and doctor commands generate reviewable `.assura/integrations/<agent>/` bundles for Codex, OpenCode, Claude, and Pi. Bundles must call `assura agent nudge`, `assura check --format agent`, and `assura daemon` commands rather than embedding validation logic or silently mutating host-agent config. |
| `assura editor session` | Supported local editor project-intelligence surface | JSON-line request/response loop with LSP-shaped diagnostics, context, and code-action preview methods. It must reuse shared content-query contracts, reload conservatively, avoid implicit writes, and must not require MCP, remote access, or a hosted language server. |
| `assura content` collection validation and query commands | Supported first project-intelligence query surface | JSON output for collection, instance, show, relation, keyword, explicit raw text fallback, bounded graph, repository-reference, context-pack, agent-context, agent-query, and JSON-line session queries must remain deterministic enough for agent use. These commands are backed by content runtime models and the shared Project Intelligence fact store. |
| `assura content semantic-search`, `symbols`, and `symbol-refs` | Experimental candidate enrichment | Optional local semantic and code-symbol candidate outputs are useful context only. They do not decide validation correctness, are not required for modeled collection validation/querying, and must remain separable from the supported collection contract. |
| `.assura/models/**` model artifact layout | Supported project-intelligence layout policy | Model artifacts stored under `.assura/` must live under `.assura/models/**`; projects may still keep artifacts outside `.assura/` when that better fits their repository. |
| `assura content session` | Supported local project-intelligence session | JSON-line request/response loop for repeated local agent/editor queries. It reloads conservatively when project files change and does not apply fixes or require a hosted daemon. |
| `assura daemon` | Experimental local daemon process | Status, start, stop, restart, doctor, logs, health, changed-path, and reference-context commands expose local JSON contracts over daemon-ready state. Lifecycle commands now manage a local process with versioned health, check-path, and repository-reference IPC; broader editor and agent daemon workflows remain experimental follow-up work. |
| `integrations/editors/vscode` | Supported beta local VS Code package | The package shells out to shared Assura CLI, daemon, and editor-session JSON contracts for diagnostics, daemon health, daemon lifecycle commands, logs, one-shot fallback, and safe-fix previews. Package metadata, doctor, build, test, and package-smoke scripts are release-gated. It is not a marketplace release and must not implement editor-specific validation logic or apply fixes implicitly. |
| `assura info` | Experimental diagnostic | Text output can change before a documented automation contract exists. |
| Extension/API boundary policy | Supported documentation contract | [`docs/extension-api-boundaries.md`](extension-api-boundaries.md) is the canonical pre-1.0 boundary between first-party `extensions.*` config policies, supported local CLI JSON contracts, internal Rust APIs, and deferred public plugin APIs. |
| `extensions.custom_constraints` | Experimental first-party | Specialized constraint execution only. Common repository relationships should use `structure` captures, `exists:1`, `needs`, and `provides`. Breaking changes are allowed before 1.0 with release-note disclosure. |
| `extensions.release_contracts` | Experimental first-party | Release artifact, checksum, workflow, documentation, installer, and branch-reference synchronization checks. It does not publish releases or replace release automation. |
| `extensions.support_matrices` | Experimental first-party | Public command/API classification checks for repository policy. Rows must use `supported`, `experimental`, `internal`, `roadmap`, or `unsupported`. Breaking changes are allowed before 1.0 with release-note disclosure. |
| `extensions.manifest_semantics` | Experimental first-party | Configured Cargo manifest metadata checks for package policy, publish status, descriptions, keywords, and declared binaries. It does not replace Cargo, dependency hygiene, license/source policy, or semver tooling. |
| `extensions.test_relationships` | Experimental first-party | Configured source/test evidence, ignored/manual test, and fixture-family ownership checks. It does not claim coverage percentage or semantic test adequacy. |
| `extensions.module_topologies` | Experimental first-party | Configured Rust module-family ownership, root existence, and bounded public export classification checks. It does not provide a full Rust parser, public API semver guarantee, or refactoring mandate. |
| `extensions.docs_lifecycles` | Experimental first-party | Configured documentation lifecycle, historical-reference exception, and deterministic stale-claim evidence checks. It does not provide a broad natural-language classifier or automatic archival. |
| `extensions.agent_guidance` | Experimental first-party | Configured `AGENTS.md` and project-local `SKILL.md` routing checks for required sections, use-case skill routing tables, allowed skill-name patterns, progressive-disclosure references, skill index links, frontmatter fields, stable headings, SKILL doc-routing tables, supporting reference links, and actionable concise-entrypoint remediation. It does not provide a global skill registry or host-agent-specific validation engine. |
| `extensions.relationships` | Internal generated first-party | Capture relationships normalized from concise `structure` notation. Users should author captures, `exists:1`, `needs`, and `provides` instead of hand-writing this generated policy family. |
| `assura watch` | Experimental | Do not advertise as release-grade until watch-mode tests and docs exist. |
| Internal Rust APIs | Unstable | Public Rust module visibility in `src/lib.rs` is for binaries, tests, and benchmark harnesses unless a row explicitly promotes the API. No compatibility guarantee before 1.0. |
| Public plugin API or SDK | Roadmap only | Remote plugin loading, shell-executed validation plugins, TypeScript plugin APIs, plugin marketplaces, and semver-stable Rust library APIs require a future goal with sandboxing, versioning, distribution, security, and performance proof gates. |

## Unsupported Surfaces

Do not document these as supported:

- package feedback CLIs such as `assura-codex-feedback`;
- per-agent host-specific command entrypoints;
- per-agent `--format` values such as `--format codex-hook`;
- remote plugin loading;
- plugin marketplaces;
- shell-executed validation plugins;
- hosted telemetry or dashboards;
- required MCP or remote agent transports for local project-intelligence usage;
- full LSP server framing or editor marketplace publication as current
  supported editor behavior;
- automatic repair;
- dependency graph validation as a current release feature.
- required code-intelligence providers for normal validation.

## Issue Triage

Use these labels or equivalent GitHub issue language:

- `release-blocker`: install, crash, data-loss, or incorrect validation
  behavior on a supported surface.
- `compatibility`: LS-Lint migration, output shape, install archive, or
  documented platform mismatch.
- `docs`: stale command, unclear limitation, broken example, or missing
  release-note disclosure.
- `experimental`: custom constraint or watch-mode behavior that is not yet a
  supported release contract.
- `roadmap`: accepted idea that needs a future goal before implementation.

## Response Targets

- Release blockers: triage within two business days.
- Compatibility and docs issues: triage within one week.
- Experimental and roadmap issues: batch into roadmap planning unless they
  expose a security or install blocker.

These are maintainer targets, not a paid service-level agreement.

## Breaking Changes Before 1.0

Assura can make breaking changes before 1.0 when they are intentional and
documented. Every breaking release note must identify:

- affected command, config field, output field, or archive name;
- replacement path, if one exists;
- migration command or manual edit;
- validation command that proves the new behavior; and
- whether old behavior is removed, deprecated, or experimental-only.

## Security

Report security issues through GitHub private vulnerability reporting when
available, or by opening a minimal issue that does not include exploit details.
See [`SECURITY.md`](../SECURITY.md) for the reporting path. Security fixes can
bypass normal roadmap sequencing when necessary.

## Maintainer Completion Rules

A release PR cannot close if:

- it advertises unsupported surfaces;
- issue/support policy language conflicts with release notes;
- install docs name assets that the release workflow does not publish;
- agent feedback docs drift from `assura check --format agent`; or
- Codex delivery is described as anything other than `--agent codex` on the
  shared agent format.
