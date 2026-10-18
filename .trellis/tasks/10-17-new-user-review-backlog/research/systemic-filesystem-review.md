# Systemic filesystem-linting performance review

As of source `66c1fe162bb837471f0c63c3aec19ceaa11bb176`, 2026-10-17. Read-only source review; no builds, benchmark cohorts, test runs, runtime changes, or product edits. Hindsight knowledge-page search returned no pages.

The strongest next step is a bounded filesystem-I/O attribution spike on ordinary `assura check` with a rich but realistic repository policy. Establish the cost of needless metadata calls and repeated enumeration, then keep the smallest demonstrated fix. The code exposes specific repeated work; it does **not** establish the wall-clock fraction or a promised speedup. A shared file index, new daemon, parser swap, or engine rewrite is premature.

## What already works well

- The ordinary launcher handles common `assura check` in process through the lightweight parser; only exceptional options dispatch to the full companion (`src/main.rs:11-18`, `src/cli/check/fast_cli.rs:80-115`). Another public entrypoint is unnecessary.
- The LS-Lint-compatible path already has compiled scopes, ancestor composition, exact-suffix lookup, per-directory rule reuse, directory-entry file types, streaming enumeration, and conditional collection for count rules (`compiled_config.rs:47-89`, `ls_fast_scope_composition.rs:1-59`, `ls_fast_naming.rs:125-169`, `ls_fast.rs:100-207`). It does not read contents for naming-only policy. Static scope lookup walks ancestors through a hash index; this is **already implemented**, not an unclaimed opportunity (`check.rs:484-494`, `ls_fast_plan.rs:149-167`).
- Full traversal prunes excluded directories before descent and uses WalkDir's file type rather than statting every ordinary entry (`traversal.rs:138-160`). File content is loaded only for active line/docs/Markdown/reference rules and shared across those per-file consumers (`validators.rs:225-282`).
- The full engine precompiles glob/regex/scope rules and memoizes effective rules once per encountered directory. Rule bundles share Arc-backed fields (`rule_plan.rs:14-19,134-179`, `check.rs:441-449`). It does not scan every scope anew for every sibling file.
- Prepared sessions retain parsed configuration and recompile only when bytes change; changed-path batches already share one full-project fallback for cross-path rules (`prepared.rs:128-165,294-310`). Batch CLI paths reuse a loaded checker per project (`batch.rs:20-45`).
- Cache reuse is correctness-limited to compatible policy. It validates directory fingerprints rather than trusting an unchanged Git status (`cache.rs:131-155,194-225`). Watch mode coalesces events and falls back on unsafe policy; such conservatism is product correctness, not inefficiency to remove indiscriminately.

## Evidence and measurement limits

`docs/analysis/2026-07-05-performance-decision-matrix.md` is the governing keep/stop policy, but its table is historical. Current checked `benches/history/current.json` is September 30 evidence at `3d3c37b0`, five iterations on Intel macOS, Rust 1.94.1. The many-scope public row is 40.545 ms versus native LS-Lint 44.782 ms; separate diagnostic phases are config load 9.242 ms, checker construction 2.749 ms, walk/validation 8.847 ms, sorting 0.000038 ms. These are separate measurements and medians: **do not sum them or divide them by the public median to claim a causal fraction**. The old July decision table's 14.950 ms public row is not a live baseline.

`benches/history/native-current.json` is older, July 5 evidence: report SHA `4eb5d518`, dirty source `411cca53`, also five Intel macOS iterations. Its 437.518 ms native-large content-check and 209.818 ms content repository-validation phase describe a different workload and measurement surface, not today's core filesystem lint cost. Previously retained 95% content ingestion improvements do not show that core linting is optimized.

The timing wrapper records core walk/validate and final sorting, but custom extension work occurs between those timers without a separate extension field (`check.rs:419-436`; phase catalog `performance_report/phases.rs:11-18,32-39`). Consequently existing structure phases under-explain policy-rich checks. CPU stacks plus syscall counts, bytes read, enumeration count, allocation/peak memory, and a public-command wall-clock comparison are needed. No exact Amdahl fraction is available for any hypothesis below. Once measured fraction f is known, even eliminating that work completely is bounded by speedup 1/(1-f).

The reported current hosted many-scope failure of 1.39% is a guardrail failure requiring investigation, not proof that this source regressed. None of the full-engine hypotheses below alone explains a failure in the simpler fast-path fixture.

## Four bounded hypotheses

### P1 — Make filesystem metadata demand-driven (best first spike)

**Established source work:** `src/cli/check/file_limits.rs:19-25` constructs the tuple `(max_size, fs::metadata(path))` before pattern matching, so metadata executes even when `max_size` is None. `validators.rs:338` calls this for every file with an effective FileBundle in the full engine. A repository with Markdown, line limits, child limits, or other fast-path-ineligible policy can therefore incur one avoidable metadata call per such file even if it has no size limits. The fast naming engine does not use this function.

**Opportunity:** Resolve whether size is required before fetching it; if other validators also need metadata, pass one demand-loaded per-file fact rather than statting repeatedly. Start with the local guard, not a new abstraction. Work reduction is up to O(N) metadata calls per full check. End-to-end improvement and Amdahl bound are unknown.

**Smallest discriminator:** One source-pinned release bundle, one policy-rich repository and a controlled wide-tree fixture. Record stat-family calls attributed to this function, total wall time, and eligible file count with and without size policy. Profile ordinary public check, not only an internal row. A temporary trace/counter may prove the call exists; use uninstrumented alternating timings for the performance decision.

**Correctness controls:** missing/unreadable file, specific/default size limits, max-size pattern override, Unicode/hidden names, symlink behavior, zero size, content/read failures, and byte-identical findings/severity/exit status. A no-size policy is the negative control: zero file-size metadata requests. Naming-only compatible runs must keep the fast path. No new caching or invalidation is needed for the first fix.

**Keep/stop:** First prove eliminated calls. Keep the implementation as a performance change only if the target public row clears the existing decision matrix (>=5% and >=0.75 ms; exact quiet tie-breaker >=3%; no accepted fixture spillover >5% or >0.25 ms; strict no-slower gate passes; two same-host order-reversed runs agree). Otherwise report the small source inefficiency and defer it rather than expanding the design.

### P2 — Reuse per-check enumeration where rich policies rescan the same tree

**Established source work:** After core traversal, independent extension walkers enumerate again. Test relationships enumerate checked scope and project root separately, including twice over the same tree for a root check (`test_relationship.rs:18-19,173-199`). Docs lifecycle separately discovers active and status files (`docs_lifecycle.rs:31-33,173-211`), then reads overlapping files separately (`43,69-70`). Command-surface constraints are already grouped into **one** walker (`custom_constraints.rs:136-188`); do not propose fixing a per-constraint walk there. Agent-guidance skill selection has its own enumeration (`agent_guidance.rs:72-110`). Direct child aggregates reread a directory before ordinary traversal reads its entries (`direct_contents.rs:44-73,304-327`).

Assura's current dogfood config enables test relationships, both nonempty docs-lifecycle selectors, and grouped command-surface checks (`.assura/config.yml:293,452-478,783-817`). With ordinary completed root checking, those three extension families plus core traversal imply at least six full-scope enumeration passes from source shape, before other policy-specific filesystem work. That is a count of walks, **not six times the runtime**.

**Opportunity:** Start with exact identical-scope reuse in test relationships or a combined active/status selection pass. Only after a profile demonstrates broader enumeration/reads dominate, consider sharing a lazy, check-lifetime path/type inventory among these consumers and reusing requested content. Current cost scales approximately O(K*N) enumeration plus repeated bytes read; shared discovery could approach O(N) discovery plus rule evaluation. Counts can reuse entries per directory without retaining the whole repository. Do not turn the naming fast path into an eager O(N)-memory inventory.

**Discriminator:** Same rich dogfood/pinned app fixture with counters for walks, directory opens, file-content opens and bytes by consumer. Compare equivalent policy outcomes, public elapsed time and peak memory; count-only, naming-only, ignored-generated-heavy, high-fanout, deep-tree, missing/unreadable, symlink and violation-heavy controls.

**Dependencies/risk:** First establish traversal semantics across validators; they differ in root inclusion, file-target inclusion, exclusions, symlink treatment and error propagation. Share only facts with equal semantics. Preserve deterministic report order and fail-fast behavior. Keep data within one check initially, so there is no persistent invalidation problem; do not conflate eliminating duplicate reads with a new filesystem snapshot guarantee.

**Keep/stop:** Apply the same performance keep bar. Stop at exact-scope reuse if wider sharing fails it or increases peak memory materially. Historical rejected cold fast-count integration does not prove this rich-policy opportunity works; it is a different lane and still needs evidence.

### P3 — Bound scope candidate search on rich/overlapping configurations

**Established source work:** Full rule resolution scans all S scopes for each uncached directory D (`rule_plan.rs:58-82,123-131`). Cached siblings avoid repeat work, but first resolutions cost O(D*S) predicate checks; wildcard ancestor matching may add depth work. The fast engine's index is all-or-nothing: **one** magical scope disables the static index (`check.rs:484-487`), leaving `ls_fast_plan.rs:82-93` linear matching.

**Opportunity:** If stacks identify scope matching/merging as dominant, index candidate scopes by literal prefix/ancestor while retaining wildcard candidates and original precedence; feed the existing resolver, rather than substituting nearest-ancestor semantics. Pure static compatible configurations already have the cheap lookup, so they are a negative control. Expected benefit applies to many-scope, overlapping/glob-rich monorepo policies; it is not a universal per-file optimization. Time fraction/Amdahl bound unknown.

**Discriminator:** Vary S independently of file count, D and scope depth; record matcher calls per uncached directory, candidates admitted, configuration setup cost, total public runtime and memory. Distinguish config normalization/compilation from execution: `ConfigLoader` already has a canonical direct parse path plus normalized notation fallback (`loader.rs:31-49`), and compilation already emits effective scopes.

**Correctness/dependencies:** Preserve equal-specificity tie ordering, overlap composition, inherit:false resets, exact versus descendant direct rules, wildcard/braces, dynamic naming, platform paths, empty scopes and fail-fast output. Differentially compare full findings and exits against the current resolver. Configuration changes invalidate the candidate index; prepared sessions must rebuild it.

**Keep/stop:** Require profile dominance and current public-command keep criteria before implementation retention. Do not reopen generic suffix maps or automatic compiled-plan caches; those experiments were already rejected. Prepared-checker reuse was also previously rejected in `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md:86`; sharing more state by itself is no benefit.

### P4 — Expand incremental eligibility only by proven dependency classes

**Established source behavior:** `prepared.rs:340-382` disables incremental paths for all listed extensions, Markdown patterns, Markdown bundles and require_docs. Unsafe policy is intentionally one full fallback per batch (`128-148`). Watch mode is narrower still: it chooses incremental only for one path, a previously clean project, and safe policy (`watch.rs:411-438`); multi-file bursts are not equivalent to current prepared-batch behavior.

**Opportunity:** Classify genuinely local rules separately from directory aggregates and cross-file dependencies, beginning with a narrowly local docs/Markdown rule if profiling shows this fallback dominates realistic edit latency. Retain full fallback for links/references/providers or unknown dependencies. Aggregate dirty parents once per burst only when coverage is proven. This changes repeat-use cost from full N to changed m plus explicit dependents; real fraction/Amdahl bound unknown.

**Discriminator:** Existing real-project feedback fixture plus actual installed adapter: idle, one edit, five edits in one parent, five across scopes, config edit, deletion/rename, watcher loss, project already failing. Record actual validation counts, fallback count/reason, coverage, p50/p95, CPU and bytes; a fast path that skips a required check fails regardless of latency.

**Hard prerequisite:** Resolve targeted CLI versus prepared-API relationship coverage before broadening optimization. Root's release probe saw a missing counterpart rejected by full check but accepted by file-target check. Source explains this: direct public check never invokes prepared safety fallback (`check.rs:181-189`); extension traversal uses the requested path (`419-424`); `relationship_entries` skips an entry equal to checked_path, including a file target (`custom_constraints.rs:330-334`). Prepared changed-path checks deliberately fall back (`prepared.rs:109-112`) with an existing regression assertion (`prepared_tests.rs:168-206`). A target-only report may reasonably be partial, but silently excluding the selected producer and equivalently labeling coverage is a distinct capability question. This review did not independently rerun that probe.

**Keep/stop:** Retain expanded eligibility only with identical required findings/coverage, observed validation-count reduction, and actual adapter p95 improvement on the stated workload. CF03's existing adapter targets remain unchanged (idle p95 <=25 ms, <=50 ms with slow-collector injection, structural-edit target <=100 ms); misses stay held and named. Stop if watcher/dependency invalidation is incomplete or the actual adapter benefit is absent. Preserve cold no-slower and native/warm regression gates separately.

**Ownership:** This is CF03's existing warm/adapter outcome with CF04 evidence, plus R01 watch correctness where applicable. Extend their evidence/acceptance; do not create a duplicate generic “fast hooks” card. Resolve local capability coverage and established correctness issues first. Do not reopen terminal CF03/04 historical status merely to copy this proposal.

## Backlog disposition and sequence

1. Preserve NU11/R02/R03's current-head strict no-slower diagnosis obligation separately. P1/P2 are not claimed as explanations for the current hosted miss.
2. Add/attach **one bounded research spike**, “Attribute filesystem work in ordinary policy-rich checks,” with P1 as first discriminator and P2 as next observation. It produces a source/binary/policy-bound profile and an explicit keep/defer decision, not an architecture deliverable.
3. Keep P3 conditional on profile dominance, and P4 routed through CF03/04/R01 after coverage semantics are resolved. No four-item implementation campaign is justified by this read-only review.
4. Measure user work: supported files/rules covered, reliable full/scoped result, one edit/burst latency, repeated filesystem work, CPU/memory, and setup cost. LS-Lint equivalence is a useful naming guardrail; it does not measure Assura's rich-policy capability or real installed-hook efficiency.
5. Defer parser substitution, binary splitting, process-floor chasing, new public check entrypoints, unconditional parallel traversal, a persistent shared filesystem database, and broader daemon adoption. Existing evidence rejects or fails to justify these directions. `traversal.rs:164-179` already shows why raw parallel enumeration need not improve validation: it collects, sorts/deduplicates, then validates serially. Preserve the production choice until end-to-end evidence differs.

Skipped runtime profiling, benchmark execution, tests and implementation as requested; performance magnitudes remain hypotheses. The concrete eager-stat and duplicate-walk findings are source-verified, while their priority depends on measured public-command cost.
