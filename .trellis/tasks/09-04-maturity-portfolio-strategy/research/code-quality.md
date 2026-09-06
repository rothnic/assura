# Code quality, Rust skill and contribution plan

Target: GitHub master `ed093668918bc271fc98b9112acaf7c1bf3eb314`, inspected 2026-09-04.

## Assessment

There is real engineering here: scoped CI, deterministic reports, schema/version fingerprints, shared hash/watch crates, transactional integration handling, extensive fixtures, and explicit cold/warm performance paths. The primary risk is accumulated overlapping mechanisms and process state. Code style cannot reliably identify AI authorship; the findings below concern maintainability and behavior regardless of who wrote them.

A source inventory found 94,902 physical Rust lines under `src` and `crates`, including inline tests; this is scale context, not a quality score. `xtask/src/main.rs` separately has 7,481 lines. Do not optimize line-count reduction as a substitute for clearer ownership.

## Prioritized findings

| Priority | Evidence | Consequence and action |
| --- | --- | --- |
| Release blocker | [Latest Rust CI](https://github.com/rothnic/assura/actions/runs/32668254739): macOS watch-scope failure, no-slower performance gate failure, missing native/warm artifacts | Diagnose independently. Preserve failed evidence. Do not label CI green or simply relax tests to release. Determine environment noise versus behavioral defect before changing thresholds. |
| Release blocker | `Cargo.toml:14` advertises Rust 1.70; locked default dependency path `assura → jsonschema/validator → idna → idna_adapter → icu_normalizer` includes crates declaring Rust 1.86 | Raise the declared minimum to an actually tested version or constrain dependencies deliberately. Add an MSRV CI lane. Metadata/dependency-path evidence establishes the contradiction; no Rust 1.70 compilation was run in this review. Platform-only WASI requirements must not be used to infer the host minimum. |
| High | Three independently defined `Config` structures: `src/config/ast.rs:14`, `src/config/types.rs:120`, `src/config/config.rs:87`; check uses `ConfigLoader` and `config::config::Config`, older info path uses `ConfigParser` | Trace call sites and supported formats, then consolidate around one authoritative parse/normalize/validate model. Distinct representations can be legitimate when named and bounded; unexplained parallel roots confuse contributors and agents. `info` remains experimental, so avoid claiming all public commands disagree without a behavior test. |
| High | Core validator modules and compiled plans live in `src/cli/check.rs` and `src/cli/check/*` | Domain logic depends conceptually on CLI organization; future adapters are harder to reason about. Incrementally introduce a policy/checking boundary independent of argument parsing/rendering. Retain proven fast paths with parity tests. Do not perform a repository-wide rename first. |
| High | `src/config/config/structure_notation.rs:672` uses four textual `include!` fragments; `src/cli/check/compiled_artifact.rs:185` onward includes numerous fragments | Physical file splitting leaves shared implicit scope and no module interface. Prefer normal modules with narrow visibility where it clarifies ownership. Document any generated-code or macro need. The proximity of many files to 500 lines is a warning about possible threshold-driven splitting, not proof of intent. |
| High | `src/maturity/environment.rs:17` onward scores CI file presence, bonuses GitHub Actions, counts package managers, treats `pyproject.toml` as a Black signal; `src/maturity/engine.rs:7` defines overall levels | Weak proxies can sound like objective maturity. Keep this experimental; freeze expansion and audit consumers before retiring or replacing scores with observed capability states. Actual CI execution and tool config semantics matter. |
| High | `CONTRIBUTING.md:39` tells contributors to branch from `main`; actual default target is `master`. It advertises Rust 1.70 and a v0.1 release example. `AGENTS.md` still describes v0.1 and implemented capabilities as roadmap work | Correct the public contributor path first. Generate/check version and command references from a small source of truth; keep AGENTS an index. Avoid presenting Trellis internals as prerequisites for an outside bug fix. |
| Medium | `src/lib.rs` exposes modules and many of their items again at crate root; internal APIs are documented as unstable | Choose canonical internal paths and reduce visibility where practical. Public-for-integration-tests is a reason to inspect test placement, not to promise a stable SDK. No pre-1.0 internal compatibility scaffolding merely to avoid updating imports. |
| Medium | `src/cli/performance_report/*` contains many `too_many_arguments` allowances; `xtask/src/main.rs` combines thousands of lines of orchestration/checks | Group cohesive inputs in named types and separate subcommands by responsibility. Benchmark-only plumbing should remain cheap to maintain and should not dominate product runtime architecture. Keep one command surface, not many new helper abstractions. |
| Medium | `src/maturity/engine/engine_tests.rs:7` restates fixed thresholds; substantial tests exist elsewhere | Review tests for independent behavioral value. A documented stable enum/string contract may deserve an exact assertion; experimental magic-score constants need meaningful boundary or scenario tests instead. Never indiscriminately remove snapshots or exact-value tests. |
| Medium | `src/main.rs:43` converts companion spawn failure to `None`; serialization paths in `agent_integration_bundle.rs` use `unwrap_or_default`; caches deliberately swallow some I/O errors | Audit each boundary for lost diagnostics. Missing companion, incompatible companion, and permission failure need distinguishable behavior. Cache miss fallback can be correct; do not globally replace `.ok()` or `let _ =` without considering the contract. No failure reproduction is claimed for these review candidates. |

Stable source base for all paths above: [master snapshot](https://github.com/rothnic/assura/tree/ed093668918bc271fc98b9112acaf7c1bf3eb314).

## Refactor sequence and proof

This is a dependency order for candidate improvements, not a commitment to execute every refactor. First examine consumer count, churn, defect history and contributor confusion; select one bounded change with a named benefit. Defer the rest when the maintenance cost is not demonstrated.

1. Correct toolchain/release/contributor truth and diagnose CI. No cosmetic rewrite needed.
2. Map config and execution entry points; add a small differential corpus across one-shot, compiled, cached and warm paths. Include invalid config, exclusions, renamed/deleted files, explicit config roots and stale artifacts. Reject drift in diagnostics, checked scope and exit codes.
3. Extract one cohesive validation boundary, update consumers, and run that corpus. Move helpers only when ownership becomes clearer.
4. Convert the highest-churn `include!` family into modules; keep a justified line-budget exception if that avoids artificial fragmentation.
5. Audit experimental maturity/legacy config consumers and remove only confirmed unused or deliberately retired paths. Preserve supported consumers through an explicit release decision.
6. Split xtask into command modules while keeping invocation contracts. Improve PR-scoped gate selection using the existing tiers rather than adding a second test runner.

No generic “remove AI code” sweep. Each patch should identify the maintenance cost or failure it removes and a behavioral proof that survives the refactor.

## Recommended custom skill: assura-rust-quality

Create a project-owned skill in `.agents/skills/assura-rust-quality/` in the first implementation increment. Register it in AGENTS and the Assura structure allowlist. This document specifies it; no new skill was installed during the review.

Use [Microsoft's Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/) and [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) as primary references. Microsoft's [AI chapter](https://microsoft.github.io/rust-guidelines/guidelines/ai/index.html) specifically addresses duplicated item paths, process narratives and tautological tests. Its full agent bundle is about 33k tokens; route to relevant sections rather than injecting it wholesale.

Suggested contents:

```text
assura-rust-quality/
  SKILL.md                     # short task router and required evidence
  references/
    architecture.md            # config -> plan -> evaluation -> diagnostics
    errors-and-effects.md      # errors, paths, subprocesses, cancellation
    performance.md             # allocations, traversal, cache equivalence
    tests-and-review.md        # behavioral corpus, mutation probes, gates
```

SKILL.md should direct an implementer to identify the supported contract, inspect one canonical example, name invariants, choose one ownership boundary, implement the smallest cohesive change, and produce evidence. Target a concise index (roughly 100–150 lines), with reference files loaded only for the affected surface.

Local rules worth enforcing:

- One canonical model per semantic role; explicit conversions where representations differ. Domain names distinguish parsed, normalized and compiled policy.
- Default private visibility. Use enums/newtypes for meaningful states and identifiers; avoid a newtype for every string without a semantic benefit.
- Typed errors in the core, contextual errors at application boundaries. No user-input panic, silent successful empty report, or misreported partial scan. Legitimate best-effort cleanup/cache fallback is documented.
- Preserve OS paths, deterministic ordering, exit semantics, cancellation and resource cleanup. Test Unicode, non-UTF8 paths where supported, symlinks, worktrees and platform differences in touched code.
- Reuse current traversal/compiled policy. Allocation, clone, concurrency or cache complexity needs profiling evidence when performance is the rationale.
- No artificial module splitting solely to meet a line limit; no one-implementation trait solely for hypothetical reuse; no broad abstraction before a second real use.
- Test behavior using an independent expected fixture or invariant. Exact diagnostic/schema snapshots remain valuable public contracts. New validation rules need positive, negative and valid-exception cases.
- Do not weaken tests/config/CI to finish an implementation without an explicit rationale and separate maintainer review.

[actionbook/rust-skills](https://github.com/actionbook/rust-skills) is an optional reference catalog for ownership, errors, CLI and performance. I inspected its anti-pattern skill. Some advice is too blanket for Assura: large enums do not automatically require traits, and replacing unwrap with expect does not remove a panic. Curate selected references and pin/review external revisions; do not install the entire router/hook system as the project's standard.

Keep existing Assura skills for validation, structure-fit, harness hooks and performance reporting. A new Rust skill should route to them where needed, rather than duplicate their workflows. Evaluate the skill on comparable small refactoring tasks: independent reviewer findings, behavior preserved, repair loops and cost—not whether the agent says it followed the skill.

## Contributions when patches are cheap

Make `CONTRIBUTING.md` the human entry point and AGENTS a compact router into the same standards. Outside contributors should need Git, the supported Rust toolchain and the relevant test commands; Trellis can remain the maintainer workflow.

Require every substantial PR to supply: concrete user problem; scope and non-goals; relevant reproducer; smallest cohesive change; behavior/contract proof; exact commands/results; known gaps; dependency and policy changes; a person accountable for understanding the contribution. Welcome AI assistance, require truthful disclosure of material generated work, and reject unverifiable claims regardless of authorship.

Proposal-first for new commands, dependencies, architecture changes, policy syntax and expanded support commitments. Small fixes can go straight to a PR. Avoid a giant template for trivial documentation edits. Do not require private prompts or confidential transcripts; a short provenance/verification note is enough.

Use the existing `cargo xtask fast`, `pr`, docs/evidence and release tiers after validating their current definitions. Ensure docs-only changes do not run unrelated heavy suites. Add host/MSRV matrix and differential checking at appropriate boundaries. The authoritative merge checks belong in protected CI; an agent-created local commit does not authorize merge or release.

Maintainer controls to plan: CODEOWNERS or equivalent review routing for configuration, CI/release and install scripts; explicit review of exclusions/thresholds/test deletions; no privileged execution of untrusted PR code; limited workflow permissions; a reviewer other than the generating agent for complex changes, followed by maintainer judgment. AI review supplements human accountability.

Set work-in-progress by review capacity. Prefer one coherent patch per agent and rebase after integration. Decline unsolicited sweeping refactors, speculative frameworks, bulk generated docs, duplicated tests and mass dependency churn. Measure time to trustworthy review and escaped defects rather than merged PR volume.
