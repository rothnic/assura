# Assura maturity and professional portfolio strategy

Reviewed 2026-09-04. Proposed roadmap; implementation and publication are separate work.

## Recommendation

Specialize Assura in executable repository conventions for agent-assisted development. A maintainer defines acceptable structure and workflow; agents receive precise, inexpensive feedback as they work; the same policy can gate a merge. Make initialization, policy explanation, and ongoing enforcement work exceptionally well before expanding the product surface.

The professional objective is **technical product / AI systems leadership**, explicitly selected by Nick. The strongest portfolio story is identifying a real coordination problem, constraining scope, designing an evaluation, improving outcomes, and operating a dependable tool. Code volume, number of agents, and a long completed backlog are weak substitutes.

## Evidence and limits

- GitHub `master` was verified at `ed093668918bc271fc98b9112acaf7c1bf3eb314`; Cargo declares `0.4.0`. The earlier local branch is not the evaluation target.
- GitHub Releases still returns `v0.3.0` as the latest published release. This is a distribution finding, not a proposal to benchmark old code. PR #142, installer hardening, is open and should be reviewed/reused before creating overlapping work.
- [Latest master Rust CI](https://github.com/rothnic/assura/actions/runs/32668254739) failed: macOS `watch_honors_the_requested_directory_scope` reported an extra event; the performance job failed its no-slower gate and lacked two upload artifacts. Root causes remain to be diagnosed. Documentation and security workflows succeeded; that is not a green release.
- A fresh current-source self-check returned `success: true` with 16 violations. These are advisory findings, so successful exit does not mean an advisory-clean repository.
- Earlier trials in this conversation used the exact master binary with independent Rust, TypeScript/Bun, and Python fixtures. All achieved passing checks/native tests; Git hooks were installed for Rust/Python and omitted for TypeScript. Deliberate naming probes passed in all three. This demonstrates incomplete specialization under the expected policies, not that PascalCase is universally invalid in TypeScript.
- Three trials are exploratory evidence, not a success-rate estimate. Direct harness hook execution was checked separately; this is not proof of a full session in every supported host.
- Both live websites were inspected with a browser. Assura has a polished landing page, genuine configuration/output examples, and creator links. NickRoth.com's visible work index has no Assura case study. The Assura About page's two Start links target `#onboard`, but that page has no such element.
- This is a targeted architecture/code review, not an exhaustive correctness or security audit. Broad Rust suites were not rerun for a planning-only task; existing hosted failures are retained as unresolved evidence.

Details: [code review and skill proposal](research/code-quality.md), [website and launch plan](research/positioning-launch.md).

Execution handoff: [stable end-to-end goal prompt](research/e2e-goal-prompt.md), [ordered backlog and coverage map](research/execution-backlog.md), [machine-readable queue](research/backlog.json), [standalone agent prompt](research/executor-prompt.md). The packets specify solutions, owned files, dependencies and acceptance evidence for every concern. Start with B00; implementation is separate from preparing this backlog.

## Product boundary

Primary user: a maintainer or small engineering team already using coding agents, with conventions spread across prose, scripts, and review comments. Start with existing repositories and a few common stacks; new-project initialization remains an important entry point.

Job: “Let my agent choose a sensible project structure, express it as maintainable local policy, and catch departures early without interrupting every edit.”

| Invest | Integrate with | Defer or retire from the growth roadmap |
| --- | --- | --- |
| Structure, naming, required/forbidden paths, generated boundaries, small guidance indexes | Clippy, Ruff, ESLint/Biome, compilers, test runners | General source-code linting or security scanning engines |
| Explainable local policy, reusable stack patterns and project overrides | Existing pre-commit/Husky/Lefthook setups and CI | A replacement hook/task orchestration platform |
| Agent-assisted init, reliable hook lifecycle, concise repair feedback | Codex first; other hosts via shared adapters and real support proof | One-off validation engines per host |
| Stable rule identity, explicit checked scope, honest inactive state | Existing docs and skills through links/indexes | General knowledge graphs, semantic search, wiki/content platform expansion |
| Bounded incremental feedback and deterministic final gates | Git branch protection configured by maintainers | Autonomous PM, agent scheduling, broad automatic repair |

Keep local links and concise guidance checks when they directly prevent broken agent instructions. Existing broader features require a consumer audit before removal: separate supported behavior from experiments, stop adding to experiments, and offer explicit deprecation/release notes for used surfaces. Do not delete functionality merely because it is outside the new headline.

Avoid a universal “project maturity score.” A directory with a workflow YAML is not proven CI; file count and tool count do not establish quality. Report observable states and named policy results instead.

Alternatives considered: a minimal naming linter is simpler but loses the strongest agent-workflow distinction; a broad project-intelligence platform offers breadth but creates competing maintenance and positioning obligations. The repository-policy specialization offers the best fit with current strengths and Nick's professional objective. It is a hypothesis to validate with outside users.

## Updated roadmap

Windows below are planning ranges assuming focused part-time effort, not delivery promises. Exit gates determine progression.

| Stage | Concrete work | Exit evidence |
| --- | --- | --- |
| 0. Establish trust; first 1–2 weeks | Begin 3–5 maintainer problem interviews now; diagnose latest CI failures; repair install/docs mismatch; resolve or record self-check advisories; correct contributor branch/toolchain guidance; repair broken CTAs; declare supported and experimental surfaces | Same candidate passes relevant hosted OS/feature gates; clean-machine install can run every advertised quickstart; published version and site agree; explicit residual-issue list; interview evidence informs stage-1 scope |
| 1. Improve first use; next 2–3 weeks | Repeatable init benchmark; Rust, TS/Bun, Python profile examples; project-owned pattern composition; explicit hook/CI state; preserve existing config and conventions | Candidate acceptance batch: 10 runs per stack, at least 9/10 on each stack meet the full contract without user repair; 0 destructive overwrites; every critical seeded violation detected; narrow the supported matrix if a stack misses its gate |
| 2. Reduce maintenance cost; next 2–4 weeks, narrow lane alongside stage 1 | Introduce local Rust quality skill and contribution checks; assess consumer/churn cost, then select one bounded refactor from config consolidation, policy boundary, textual includes or xtask organization | Named maintenance/correctness benefit; no behavior/output regressions on changed surfaces; reviewer can trace a rule from config to diagnostic; one outside contributor completes a small change with documented gates; defer further refactors until justified |
| 3. Validate usefulness; 3–5 external maintainers over 2–4 weeks | Observe setup on their repos; collect false positives, useful catches, setup repairs, and continued usage | At least 3 maintainers keep it enabled for 2 weeks and can identify a useful catch or workflow improvement; observed evidence, not install/star counts |
| 4. Public beta narrative | Publish verified release, focused demo, Assura case study, method/results, limitations; small LinkedIn and eligible community rollout | Public instructions reproduced independently; feedback channel staffed; install failures triaged; message contains no unsupported outcome claims |
| 5. v1 decision | Stabilize config/report/exit semantics; release/support policy; real host/OS soak; dependency and upgrade discipline | Existing 30-day/50-session/3-repository/4-host gate satisfied if four hosts remain supported; or explicitly revise support scope before the gate. Add external adoption evidence and no unresolved high-severity release issues |

Problem discovery begins immediately; hands-on external trials wait for a usable installation path. Stages 1–3 can overlap after stage 0. Do not expand the harness/profile matrix beyond maintainable proof. If the broader four-host claim is retained, one-host tests cannot satisfy it. A narrower support declaration is a product decision and must update docs, manifests, tests, and roadmap together.

Change the current “make every marketed capability mature” program to: classify each claim, retain the focused support contract, remove premature promotion, then mature the retained contract. Marketing copy should not force indefinite support for speculative functionality.

After this strategy is accepted, update `.trellis/spec/assura/roadmap.md`, `docs/data/public-roadmap.json`, the release goal, release-surface manifest and site support pages in one consistency pass. The roadmap still names the merged landing branch and prerequisites already implemented on master; revalidate each child before scheduling it. Preserve historical evidence with explicit supersession links.

## Initialization as an optimization problem

Treat agent instructions, CLI help/defaults, feedback, profile examples and discovery paths as versioned inputs. Optimize successful setup and correct later behavior subject to cost and preservation constraints. A passing `assura check` alone is insufficient: a permissive config can always pass.

1. Freeze a fixture and acceptance specification independently of the initializing agent. Include new and existing repos, workspaces, framework naming exceptions, existing hook managers, custom local profiles, conflicting guidance and intentionally invalid paths.
2. Give a fresh agent only “Initialize this project with Assura,” normal repository context and the candidate tool. Record model, harness, tool version, instruction hash, CLI output, commands, elapsed time and token/tool cost. No evaluator hints or cross-run memory.
3. Evaluate preservation, layout fit, policy coverage, concise discoverable guidance, actual hook activation, real hook execution, CI gate presence, native tooling reuse and rerun idempotence. Distinguish unavailable host permission from failure; do not claim activation merely because files exist.
4. Inject expected violations after setup: wrong placement, missing required path, generated files in source, agreed naming violation, excessive module size, broken guidance reference. Also inject valid exceptions to measure false positives. Check enforcement through the actual execution path.
5. Give a second agent a small feature change. Measure whether the setup prevents/reveals drift and enables a correct small repair without disabling policy or sprawling refactors.
6. Compare baseline versus one candidate change at a time; keep model and fixtures fixed. Use 3 stacks × 5 repetitions × 2 conditions = 30 runs for a screening round. Add holdout repositories after tuning; report counts and variability, not a universal success claim. A 30-run candidate acceptance batch is separate from that screening round.

Acceptance scorecard: structural suitability; enforceable policy; guidance discoverability; hook lifecycle; native gate reuse; preservation/idempotence; successful subsequent change; setup cost. Missing critical hooks where supported, silent bypass, destructive overwrite, or critical false negatives are hard failures and cannot be averaged away. Unsupported capabilities are explicit, not silently scored as passing.

Efficiency measures: median and p95 time/token cost, duplicate tool executions, idle-hook output, configuration repair count, change-to-useful-feedback latency. Set absolute latency/output budgets from representative hardware and projects before optimizing; compare equivalent cold and warm paths separately. Do not demand a warm daemon beat another tool's cold start as evidence of universal speed superiority.

## Reusable patterns without taking design away from the agent

A pattern packages a purpose, applicability evidence, small policy fragments, conventional exceptions, optional file templates, native gate mappings, and passing/failing fixtures. It is an editable starting point, not an architecture mandate or opaque plugin.

Selection order: preserve explicit local intent; examine manifests and existing layout; select the smallest applicable built-in pattern; apply local overrides; record meaningful decisions and uncertainty. Ask only about genuine conflicts or consequential ambiguity. A new Python library and FastAPI service should not silently receive the same layout. TS component PascalCase and kebab-case utility conventions must be modeled by scope.

Start with project-local patterns and a small maintained bundled catalog. Record origin/version and show an update diff; rerunning init must not erase local changes. Remote registries, marketplace ranking and executable downloaded templates can wait. Do not introduce a new DSL until current composition mechanisms have been evaluated.

## Gate policy

During edits: bounded changed-area advice, debounced, with little or no output when unchanged. On commit: cheap deterministic checks appropriate to local workflow. Before merge: authoritative configured full-project checks and relevant native tests in CI. Git hooks are convenience, not the sole enforcement boundary; local hooks can be bypassed.

Teach the agent to improve the implementation before changing its policy. Any weakened threshold, new exclusion, disabled rule or changed CI scope must appear explicitly in the change explanation and receive maintainer review. Preserve room for justified exceptions; do not force arbitrary splits merely to satisfy a file-length rule.

## Professional evidence and success measures

The case study should show: observed failure mode → scope decision → instrumented experiment → unfavorable result → iteration → released behavior → outside feedback. Include personal ownership of decisions and independent verification; describe AI assistance truthfully.

Product metrics: successful no-repair setup, useful findings, false-positive rate, time-to-repair, retained use. Operational metrics: reproducible installs, supported-matrix health, feedback overhead, support response time. Professional metrics: case-study engagement, qualified conversations, interviews and thoughtful practitioner feedback. Treat cross-site clicks and stars as interest signals only.

Stop/reconsider expansion if fewer than 3 of 5 target maintainers find a repeated unmet need, if setup costs exceed perceived value, or if most feedback is adequately solved by existing tools. Narrow further before adding features. Budget experiments by reviewer capacity; stop agent work when unreviewed patches accumulate.

## Deliverable status

This task produces the updated roadmap, evidence-backed review and launch plan. It does not implement the refactors, install skills, publish posts, alter public support contracts, or merge PR #142. These are sequenced work items with acceptance gates, not completed claims.

Independent read-only review confirmed the representative source findings and recommended earlier demand discovery, per-stack acceptance, and cost-justified refactoring. Those refinements are incorporated. Planning files are the only owned changes in the original checkout; the inspected master worktree remains clean. The turn-start workflow gate was ready on the clean baseline; its later dirty-state warning corresponds only to these newly created planning files, not unknown work.
