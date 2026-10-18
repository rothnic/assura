# Assura ecosystem fit and capability economics: second pass

Date: 2026-10-17. Assura source reviewed at `66c1fe162bb837471f0c63c3aec19ceaa11bb176` on `docs/assura-new-user-review-20261017`. Research only; no product, original-project, tracker or lifecycle edits. This report does not duplicate the separate core filesystem performance audit.

## Judgment

Assura can credibly own **project-owned repository conventions, explained at the path where a human or coding agent is working, with the same deterministic policy available in CI**. Its likely wedge is a maintainer repeatedly repairing misplaced files, missing companion files, inconsistent guidance, and broken local repository references in a polyglot repository. A concise policy with useful diagnostics is the product; another quality dashboard or agent platform is not.

That is a candidate fit, not demonstrated demand. Most individual primitives already exist elsewhere. Assura must earn its additional configuration and integration cost by making a complete convention → violation → repair loop easier than the maintainer's current tools. Passing tests or supporting more command families does not establish that advantage.

The first pass proves useful configured checks and several real adoption defects. It does **not** yet show that adding Assura prevents more cleanup than a small LS-Lint policy, existing hook runner, and native linters. Nor does it measure total setup/maintenance effort, end-to-end trusted hook cost, or retained outside use. The backlog should repair correctness and first-use failures while making these economic questions answerable; adding adjacent analyzers would weaken the experiment.

## Evidence classes

| Class | What this pass can say |
| --- | --- |
| Current source/documented contract | Support policy, release-surface ledger, manifest, current strategy and CF backlog were read at the source revision above. This is capability classification, not a rerun of every surface. |
| Recorded local behavioral evidence | The October 17 TypeScript and Python trials show configured naming, local-link and closed-scope detection; preservation on repeat setup; Review and native-scope defects. Assura dogfooding and one watch fixture are recorded. This pass read those receipts/reports; it did not independently rerun them. |
| Current primary external documentation | Eight relevant tool families below were checked live through official documentation/repositories. Capability statements describe their docs, not local tests or equivalent benchmarks. |
| Unverified comparative advantage | Lower total cost, more useful repair context, better agent correction, less configuration, better cache economics and superior broad performance are hypotheses. No competitor was installed or timed here. |
| Demand | A07 is a distinct independent initialization evaluation; F02 is outside pilot/retention. Exploratory agent trials satisfy neither. |

Hindsight search and listing returned no knowledge pages. No historical Hindsight claim supplied a capability conclusion.

## Where Assura fits

Each row is a job boundary, not a claim of exhaustive feature parity. Native language linters are represented by ESLint/Biome in one family, and hook execution by pre-commit. Tool count is not a roadmap.

| Existing job/tool family | Verified documented capability | Assura relationship and adoption decision |
| --- | --- | --- |
| Filesystem naming and cardinality: LS-Lint | Built-in naming styles, regular expressions, directory substitutions, and `exists` counts/ranges for files **and directories**. | Closest substitute for the core naming/layout slice. Do not describe it as naming-only or claim required-file counts are unique to Assura. Keep LS-Lint when its policy is sufficient. Switch only for a demonstrated additional convention/diagnostic/workflow need with equivalent-policy results. [Rules](https://ls-lint.org/2.3/configuration/the-rules.html) |
| Language/source lint: ESLint or Biome | ESLint provides configurable rules/plugins, fixes and editor suggestions. Biome statically analyzes source and emits diagnostics, with its own language/rule scope and safe/unsafe fixes. | Compose. Assura path/layout/content conventions do not prove language correctness, type soundness, security, or semantic architecture. Keep existing native tools and expose the native check that an edit requires. Do not compare a naming scan with their richer source analysis as if the work were equal. [ESLint](https://eslint.org/docs/latest/use/core-concepts/), [Biome](https://biomejs.dev/linter/) |
| Import/dependency architecture: dependency-cruiser | Validates JavaScript/TypeScript dependency rules and produces graphs; documented rules include forbidden and required relationships, including reachability. | Compose when import direction, cycles or actual dependency structure are the need. File-pair/path constraints are not dependency validation. Assura explicitly does not support dependency graph validation as a current release feature. [Overview](https://github.com/sverweij/dependency-cruiser), [Rules](https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-reference.md) |
| Unused files/exports/dependencies: Knip | Reports unused files, dependencies and exports; also lists unresolved imports and configurable cycle findings. | Delegate this job. Assura file existence/reference facts cannot establish that executable code is unused. Do not grow semantic/code-symbol analysis to compete while the core adoption path is failing. [Issue types](https://knip.dev/reference/issue-types) |
| Markdown style: markdownlint | Documents heading, whitespace, list, link-syntax and other Markdown rules with configuration and per-file suppression. | Partial overlap. Retain an existing configured tool when it already serves the docs team. Assura's common Markdown subset can reduce tools only after an explicit rule-by-rule mapping and diagnostic/ignore comparison; no replacement-completeness claim is supported. [Rules and configuration](https://github.com/DavidAnson/markdownlint) |
| Local/remote link verification: lychee | Offers an offline mode that checks local files without network requests, and optional fragment checking. | Offline local-link checking is not unique to Assura. Assura can add a convention-aware local policy and common repair context; compose with a dedicated link checker for remote/site workflows. Avoid duplicating findings or inventing network checking in the core. [Offline and fragment flags](https://lychee.cli.rs/guides/cli/) |
| Multi-language hooks/execution: pre-commit | Manages installation/execution of hooks in multiple languages, normally scopes hooks to changed files, and can run all files. | Assura should be a check invoked by an existing runner; its managed agent adapters solve a separate delivery problem. Do not ask users to replace a working runner or let two installers compete for hook ownership. A generated quality plan is not proof that native lint/tests executed. [Official usage](https://pre-commit.com/) |
| Arbitrary structured-data policy: Conftest/OPA | Conftest tests structured configuration data with Rego and provides a policy verification path. | Delegate complex infrastructure/configuration policy to a policy specialist already used by the team. Assura should own concise repository conventions, not require a second general policy language or absorb infrastructure governance. A shell-backed computed check remains an explicitly bounded experimental bridge, not a remote plugin system. [Conftest](https://www.conftest.dev/) |

This comparison intentionally excludes speculative integrations and exhaustive tool catalogs. A useful composition can simply be separate existing commands under the maintainer's runner; a universal aggregation layer is not required.

## Capability strength and limits

The strongest recorded product demonstration is deliberately configured naming/local-link/closed-scope policy: clean passes, controlled mutation fails at the configured severity, repair restores the baseline. Preservation of owner policy and unmanaged host entries also matters. These are more defensible than claiming broad AI trust or repository quality.

An additional current recorded probe sharpens the boundary: a full-project check reports the missing captured `Button.test.tsx` counterpart for `Button.tsx`, while an explicit check targeting only `Button.tsx` passes and reports that file as `checked_path`. A comment-only counterpart satisfies the presence rule. Full cached checks do detect counterpart deletion and policy restoration. These receipts live at `../capability-pass/transcript.json` (`missing-test-full`, `missing-test-file-target`, `presence-only-full`, `cached-test-deletion`, `cached-restored-policy`). This is evidence of different checked scope and existence semantics, not by itself a proved defect in an explicit target contract. Do not advertise targeted-file latency as full cross-file enforcement, or a required test filename as test execution/quality. Watch/agent fast paths must prove full fallback or disclose omitted cross-file coverage; the watch support contract already promises a full-scope fallback for repository-wide policy.

The weakest part of the current story is that **the configuration author has to already know the intended conventions**. Generic onboarding leaves naming/root/link policies inactive and specialization unverified. That honesty is correct, but it limits immediate value: installation alone does not prevent the structural failures used in the marketing example. If specialization takes substantial explanation and trial/error, Assura can become a second convention language maintained beside AGENTS, native config and hook config.

The recorded TypeScript closed-scope trial illustrates this cost. `.tsx: kebab-case` did not exempt matching files from a closed wildcard count; adding an explicit `exists` range made the policy work. This is not a demonstrated validator defect, and changing its semantics is not recommended here. It is evidence that policy explanations and examples need to make independent naming/cardinality effects obvious. NU07 should address actionable offenders and actual limits; its acceptance can also exercise this user misconception without adding a new DSL.

The source has 31 registered release surfaces (30 classified supported, one experimental) while the support table describes many additional contracts and experimental extensions. `.assura/config.yml` is 1,565 lines. These are measured scope/complexity indicators for **Assura's own repository**, not proof of typical user configuration size or poor implementation. They show why a compact core story matters: every command family, output schema, platform and host adds compatibility, documentation, lifecycle and test obligations. Four managed hosts cannot be waved away because a local Codex demonstration works.

The release records themselves still disagree: support policy and scope ledger say latest published 0.4.1; the JSON manifest has `checked_source_version: v0.4.1` but `current_published_release: v0.3.0`. The prior review records a live GitHub 0.4.1 query, but this pass did not requery GitHub. Treat the JSON field as a source inconsistency for P01/W02 to resolve, not evidence that the release was absent. A machine-readable inventory that disagrees with user instructions adds avoidable support cost.

## Efficiency means more than a fast subprocess

Respect the existing strict performance gates. NU11's retained 11.812 ms versus 11.650 ms many-scope result fails that contract; a small absolute gap does not permit lowering it. It also does not show the size of a user's whole workflow or establish a general speed ranking. The dedicated core-performance lane should identify dominant filesystem/planning/read work and validate equivalent coverage, rather than optimize a headline ratio alone.

Measure three different costs:

1. **Execution cost:** cold full tree, repeated unchanged check, one-file edit, a burst, rename/delete/config/ignore change, and actual hook end-to-end latency. Include process launches, CPU, RSS, filesystem reads, background work and fallback counts; cache hits must preserve correctness. Report rich-policy and naming-equivalent workloads separately.
2. **Attention cost:** automatic messages/bytes, duplicate findings, intervention count, active time to locate the offending path and correct a violation, time lost to false positives or incomplete coverage. Silence under quota is not validation success.
3. **Policy ownership cost:** active setup minutes, edits/retries required to encode five chosen conventions, files/config changed, ongoing maintenance after a new source root or tool upgrade, drift between native and Assura include/exclude scopes, disable/removal time, and any owner policy overwritten or reinterpreted.

Useful accounting is **total active maintainer time spent setting up, maintaining, diagnosing and repairing intended convention failures**, alongside the number and consequence of failures prevented before review. Record raw components and denominators; do not collapse these into an invented quality/productivity score. CPU-time savings cannot be inferred from foreground latency if a collector moves the same work into the background.

An apparent performance win fails the adoption test if repeated setup assistance, missed source roots, or duplicated hook work erase it. Conversely, a richer check could be worth modest overhead when it prevents a recurring expensive convention error; that does not change the accepted equivalent-naming gate.

## Proposed decision criteria

These are evaluation proposals to review with the existing acceptance owners, **not new product thresholds or changes to accepted gates**.

| Decision | Minimal evidence to record | Continue / switch / compose criterion |
| --- | --- | --- |
| Add Assura to an established toolchain | Pick five real costly conventions before setup; record which existing tools already enforce them. Use clean, mutation, repair and unrelated-negative controls. | Adopt only if at least one material unmet convention or materially cheaper repair workflow is demonstrated, while preserving existing tools/policy. Naming alone favors LS-Lint unless a measured need warrants migration. |
| Replace a filesystem linter | Run the same pinned tree, explicitly mapped policy, exclusions, symlinks/path behavior and violating cases on both tools; preserve every mismatch. Measure configuration/repair time and relevant latency distributions. | Switch only with agreed pass/fail semantics, no lost intended coverage, current performance gates passed, and a concrete maintenance/workflow benefit. A one-number speed chart is insufficient. |
| Compose with native checks | Mutate each actual application/test root plus config/lockfiles; inspect and execute the applicable native plan in a disposable project with dependencies installed. | Assura must not make a real code edit look completely checked while only structure ran. Preserve native errors and runner ownership. NU03's plan repair is prerequisite to claiming correct generated coverage; execution proof remains separate. |
| Keep an agent integration enabled | Observe trusted host events and exact candidate identity, policy violation → useful bounded feedback → repair, plus idle/burst/config behavior. Count all processes/notifications. | Keep it if feedback changes an intended repair decision and the existing host budget/correctness contract holds. Generated bundles and schema tests are insufficient. |
| Retain the product | F02's existing two-week pilot, all users/dropouts counted, with the current-tool alternative recorded. | Keep F02's existing target of at least three retained maintainers with useful catches/workflow improvement. Report whether benefits remain after setup/maintenance cost; narrow or pause growth when existing tools suffice. |

Avoid designing a huge randomized benchmark before the known first-use failures are fixed. A small paired workflow trial with an explicit current-tool baseline is enough to reject an unhelpful wedge; it cannot prove a market or universal superiority.

## Backlog implications and genuine dependencies

Keep the twelve known findings, but group the next decisions by user outcome rather than creating a competitor-feature parity backlog.

| Outcome | Existing route / recommended scope | Priority and dependency reasoning |
| --- | --- | --- |
| A user reaches a meaningful first pass without repairing the tool's setup | NU02 preservation, NU04 quickstart, NU05 executable identity, NU06 recovery; NU08 explicit source coverage. | Correctness/setup defects lead. These are independently repairable; they do not depend on adding a resident service, content model or new ecosystem adapter. |
| A user sees the conventions they chose and can repair the exact offender | NU01 active advisories/history; NU07 offenders/actual limits; bounded deliberate specialization. | High: advisory suppression is a capability/attention failure, not merely cosmetic output. Require meaningful negative controls and no optional-capability upsell dominating actual configured debt. |
| Native tools still cover the actual application | NU03 under A05; add a small composition acceptance matrix to the existing route. | High: main-source-root omission weakens Assura's complementary-tool promise. Keep plan selection and executed-native proof distinct. Do not build a new runner. |
| Core work is fast and scales with equivalent checked scope | NU11, R02/R03 and the dedicated systemic performance review. | Retain the gate and historical evidence; add a new implementation card only for a demonstrated mechanism absent from completed CF work. Architecture rewrites need attribution and equivalent-coverage proof. |
| The convention language costs less than it saves | Extend A07 evaluation measurements where allowed and F02 baseline/retention collection; use public nonsealed cases for current research. | Mostly an acceptance gap, not a product feature. Never treat today's unblinded projects as A07 credit or modify private evaluator protocol under this recommendation. |
| Public users can choose correctly | W02/P01: fit/non-fit/composition docs, truthful release JSON and absolute performance scope. | Add acceptance to existing story work. No website deployment or publication authority is implied. |
| Own policy demonstrates intentional coverage | NU09 self-debt disposition; NU10 maintained local docs; NU12 ownership closure. | Useful dogfooding, but not an adoption proxy. Configure only intended scopes and preserve unfavorable findings; topology bookkeeping does not prove product benefit. |

**Potentially new bounded work only if no current card owns it:** “Prove Assura composes with one existing hook runner and native linter.” Acceptance: an existing configured project retains runner bytes and execution order; install/add/run/remove Assura; exact source changes run native checks; structural and native failures both remain visible; no duplicated hook/scan loop; original runner still works after removal. This can be attached to A05/R06 and NU03 rather than a second product stream. A documentation recipe suffices if existing behavior already works. No adapter/plugin package is presumed necessary.

There is no genuine blocking edge from ecosystem research to fixing NU01/NU02/NU03/NU04/NU05. Relevant performance correctness repairs can also progress independently. External retention still depends on the existing releasable/installable candidate, independent evaluation and participant authorization routes; fixing a small report defect does not itself authorize or satisfy those outcomes.

## Existing work to reuse, not reopen

- CF01–CF04 are **recorded terminal** in `.agents/skills/custom/assura-feedback-execution/references/backlog.md`, with merge SHAs. They already own cheap existing feedback, cached facts, one-line scheduling/async delivery, and performance/project evidence. This pass read that state; it did not revalidate every merge or live host. Recheck the specific residual when evidence disagrees; do not create a generic “add caching/background collection/bounded feedback” epic from scratch.
- R01 owns watch coverage, R02/R03 equivalent-comparison/many-scope performance, A06 bounded-feedback baseline. Their historical completion does not waive a fresh current failure; NU11 is a new concrete failed-gate observation.
- A07 is active with explicit no-credit cautions and A04/A05/A06 prerequisites. Its role is correctness/evaluation independence. F02 owns external retention and has R06/A07/F01 dependencies. Keep these outcomes distinct; append cost/composition observations to the appropriate owner without inventing a new evaluation queue.
- Existing W02 already asks why not LS-Lint, where Assura fits, native tools remaining necessary and claim provenance. This report sharpens the evidence and economics behind that existing decision; it does not authorize a redesign/launch.

## When a user should not adopt Assura

- Their problem is fully served by a small existing naming/cardinality policy; they have no recurring additional convention failure and do not want another configuration language.
- They primarily need language correctness, dependency architecture, dead-code discovery, vulnerability analysis, or remote website availability. Use the relevant specialist; a passing Assura check cannot establish these outcomes.
- They cannot identify intended conventions or an owner for maintaining them. Generic green defaults alone are not a benefit.
- They require a stable public Rust/plugin API, hosted orchestration, remote marketplace execution, semantic quality scoring or broad automatic repair. Those are outside the present growth contract.
- They require complete trusted delivery proof for a specific platform/host that the available candidate evidence does not establish, or their migration would sacrifice existing policy. Treat that as an evidence/support decision, not permission to weaken the advertised four-host contract.

The strongest reason to try Assura is a recurring, concrete repository-convention failure that existing tools leave expensive to configure or repair. Its architecture and backlog should serve that one reason before adding adjacent capabilities.

## Sources and limits

Local anchors: `docs/support-policy.md`; `docs/analysis/assura-scope-decisions.md`; `docs/data/release-surfaces.json`; `.trellis/tasks/09-04-maturity-portfolio-strategy/research/positioning-launch.md`, `adoption-packet.md`, `backlog.json`; `.agents/skills/custom/assura-feedback-execution/references/backlog.md` and `feedback-contract.md`; `docs/analysis/2026-10-17-new-user-review.md`; `.trellis/tasks/10-17-new-user-review-backlog/research/{backlog.json,typescript-trial.md,python-trial.md}`; and the parent's current `capability-pass/transcript.json`. External citations are canonical primary URLs, fetched on the review date; `sources.json` records purpose and evidence class.

Skipped competitor installs/benchmarks, product implementation, external interviews, trusted-host reruns and private evaluator changes. Comparative advantage and retained user value remain unproven; the report proposes how to test them.
