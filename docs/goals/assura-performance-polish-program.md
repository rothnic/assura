---
id: goal-assura-performance-polish-program
type: goal
title: Assura performance polish program
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-ls-lint-no-slower-performance-gate.md
  - ./assura-performance-floor-and-fixture-gate.md
  - ./assura-ls-lint-performance-reassessment.md
  - ./assura-supported-document-graph.md
  - ./assura-markdownlint-compatible-rust-engine.md
  - ./assura-true-daemon-mode.md
  - ../analysis/2026-07-02-ls-lint-performance-reassessment.md
  - ../analysis/2026-06-28-content-runtime-index-performance.md
  - ../analysis/2026-06-28-project-intelligence-store-spike.md
  - ../../benches/history/current.json
  - ../../website/public/data/performance/current.json
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Performance Polish Program

## Objective

Make Assura performance boringly trustworthy across both sides of the product:

- LS-Lint-equivalent structure validation remains no slower than native
  LS-Lint for every accepted fixture row.
- Assura-native capabilities that LS-Lint does not provide, including content
  collection validation, document graph querying, context packs, Markdown and
  reference workflows, daemon/session requests, editor/agent surfaces, and
  safe fixes, get their own tracked performance report, budgets, history, and
  regression gates.
- Remaining cold CLI-floor confusion is explained by evidence and reduced
  where it matters without making the normal CLI cumbersome.

This goal is the next large performance iteration to kick off after the
`v0.3.0` beta increment.

## Current Gap

Assura has several valuable performance assets, but they are not yet one
complete product-grade system.

- `assura performance-report` and CI enforce the LS-Lint comparison path.
- `cargo xtask performance-no-slower` fails accepted LS-Lint-equivalent rows
  that are slower than native LS-Lint.
- `benches/content_runtime.rs` measures content repository validation, but the
  result is local Criterion evidence rather than checked release history.
- `benches/project_intelligence.rs` measures fact-store load, search, graph
  traversal, and session-style query behavior, but it is not exposed as a
  supported native performance report.
- The public performance data under `benches/history/current.json` and
  `website/public/data/performance/current.json` is LS-Lint-focused.
- The remaining cold 2x misses are understood at a high level, but the
  highest-impact rows need a focused optimization pass with before/after proof.

## User Certainty Bar

Nick should be able to ask two questions and get machine-checkable answers:

1. "Are we slower than LS-Lint anywhere on accepted LS-Lint-equivalent
   fixtures?"
2. "Are Assura-only capabilities, especially validating and querying
   collections, document graph search, context packs, Markdown fixes, and
   daemon/session queries, fast enough and protected against regressions?"

The answer must not rely on anecdotes, aggregate-only speedups, or local-only
Criterion output. It should point to checked report JSON, CI gates, benchmark
history, and documentation that explain the accepted rows, diagnostic rows,
budgets, and optimization opportunities.

## Performance Test Plan

### Fixture Matrix

Create one shared native-performance fixture matrix with these sizes:

- `small`: 25 goals, 25 specs, 5 ADRs, representative Markdown headings and
  references.
- `medium`: 250 goals, 250 specs, 50 ADRs, matching the current content
  runtime scale.
- `large`: 2,500 goals, 2,500 specs, 500 ADRs, enough to expose nonlinear
  content, graph, search, serialization, and daemon costs.
- `reference-heavy`: many-to-many relations, duplicate IDs, missing targets,
  ambiguous references, moved docs, renamed code paths, and broken anchors.
- `adapter-mix`: Markdown frontmatter, JSON, YAML, and JSONL collections.
- `real-project`: Assura itself plus the existing realistic project fixtures
  used by project-intelligence proof.

Each fixture must define record counts, file counts, directory counts, relation
counts, heading counts, reference counts, accepted/diagnostic classification,
and expected query shapes.

### Required Rows

Add native performance rows for:

- content repository cold load;
- warm content repository validation;
- full `assura check --format json` with content runtime enabled;
- `assura content collections`;
- `assura content instances`;
- `assura content show`;
- `assura content expand`;
- `assura content search`;
- `assura content missing-relations`;
- `assura content references`;
- `assura content context-pack`;
- `assura content agent-query`;
- Markdown lint scan;
- Markdown safe-fix dry run;
- Markdown safe-fix apply plus idempotence check;
- repository reference validation;
- daemon cold start;
- daemon warm check request;
- daemon warm content/query request;
- session/editor repeated query request;
- JSON/text output serialization for large context packs;
- create/update write path validation for modeled collections.

Rows must separate cold subprocess cost, cold in-process load, warm repository
query, warm daemon/session query, IPC cost, serialization cost, and report
formatting cost.

### Budgets And Gates

Establish the first checked native report as baseline, then enforce these
policies:

- accepted LS-Lint-equivalent rows remain strict no-slower-than-native-LS-Lint
  merge gates;
- native Assura rows fail on material regression versus checked baseline after
  the first calibrated baseline lands;
- warm daemon/session rows get separate budgets from cold CLI rows;
- large fixtures must remain approximately linear in file and modeled-object
  count unless a written exception explains the cost;
- aggregate speedups cannot hide an accepted-row failure;
- diagnostic rows may regress only with explicit classification and reviewer
  approval.

Initial native budgets should be calibrated from measured baseline rather than
invented. After calibration, use row-specific thresholds instead of one global
number.

## Optimization Opportunities To Investigate First

Use the current evidence to rank optimization work before changing code.

1. `many_configured_scopes_regression`: current LS-Lint reassessment shows
   roughly equal time in config loading and walk/validate work. This is the
   highest structure-check row for real optimization.
2. Small cold fixtures such as `simple_library` and `web_app`: current misses
   against the stricter 2x target are floor dominated. Investigate release
   payload, binary linking, startup, config discovery, and output setup without
   requiring users to opt into a daemon for normal `assura check`.
3. Content runtime validation: prior local evidence measured about 21 ms for
   240 goals and 240 specs. Determine whether schema compilation, file
   indexing, relation validation, diagnostic construction, or serialization
   owns the cost, then optimize only with before/after proof.
4. Project-intelligence query store: current warm search and graph rows are
   fast, but incremental replacement is expensive because indexes rebuild.
   Measure daemon/session incremental update behavior before choosing any
   durable store or partial-index design.
5. Markdown engine integration: attribute time across file discovery,
   Markdown parsing, rule execution, safe-fix planning, fix application, and
   Assura diagnostic mapping. The supported Rust path should be the fastest
   practical markdownlint-compatible path that preserves deterministic output.
6. Context-pack and agent-query serialization: large bounded context payloads
   should report selection cost separately from JSON/text rendering.
7. Daemon/session IPC: prove warm requests are actually dominated by useful
   validation/query work, not process or protocol overhead.

## Scope

- Extend or add a native performance report suite that produces
  machine-readable JSON and website-ready data.
- Reuse existing benchmark infrastructure where possible; do not create a
  parallel benchmark system disconnected from `benches/`, `xtask`, CI, and
  website data.
- Promote content runtime and project-intelligence Criterion evidence into
  tracked report rows or documented source rows.
- Add CI/reporting gates for native capability regressions after the baseline
  is calibrated.
- Preserve the LS-Lint no-slower gate for accepted structure fixtures.
- Add phase attribution for native rows, not only LS-Lint rows.
- Optimize the highest-impact rows with before/after report artifacts.
- Update docs and website data so users can distinguish LS-Lint comparison,
  Assura-native performance, cold CLI, warm daemon/session, and diagnostic
  rows.

## Non-Goals

- No replacing product behavior with benchmark-only fast paths.
- No using warm daemon/session results to prove cold CLI LS-Lint parity.
- No relaxing accepted LS-Lint no-slower gates.
- No adding a persistent database, cache, or search engine dependency without
  proving it beats the in-memory baseline on the same fixtures and packaging
  constraints.
- No broad rewrite before phase attribution identifies the cost.
- No performance claim based on one local ad hoc run.

## Definition Of Done

- A native Assura performance report exists with checked JSON history and
  website data.
- CI publishes the native performance report and blocks material regressions
  once the baseline is calibrated.
- Every accepted LS-Lint-equivalent row remains no slower than native LS-Lint.
- Content collection validation and querying have explicit cold and warm
  performance rows.
- Document graph search, expansion, references, missing relations, and context
  packs have explicit performance rows.
- Markdown lint/fix rows attribute parser, rule, fix, and mapping cost.
- Daemon/session rows prove warm check and query behavior separately from cold
  process startup.
- The highest-impact current optimization opportunities are either improved
  with before/after evidence or recorded with bounded follow-up rationale.
- Docs explain how to interpret LS-Lint comparison versus Assura-native
  performance.
- Independent review confirms that gates cannot pass by hiding slow accepted
  rows, warm-only substitutions, or diagnostic reclassification.

## Validation Commands

Start with focused evidence:

```bash
cargo bench --bench content_runtime -- --noplot
cargo bench --bench project_intelligence -- --noplot
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output target/performance/ls-lint-current.json \
  --history target/performance/ls-lint-current.jsonl \
  --website-dir target/performance/ls-lint-website \
  --iterations 5
cargo xtask performance-no-slower target/performance/ls-lint-current.json
```

The implementation should add a native report command or suite. When it
exists, the final gate should include:

```bash
target/release/assura performance-report \
  --suite native \
  --output benches/history/native-current.json \
  --history benches/history/native-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
cargo xtask native-performance-no-regression benches/history/native-current.json
cargo xtask performance-no-slower benches/history/current.json
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

If the final command names differ, update this goal before execution closure so
the checked proof is exact.

## Review Tasks

- R1: Confirm native Assura rows are separate from LS-Lint comparison rows.
- R2: Confirm accepted LS-Lint-equivalent rows still fail merge when slower
  than native LS-Lint.
- R3: Confirm content validation and querying performance is covered by public
  CLI surfaces, not only internal library benches.
- R4: Confirm daemon/session performance is measured as warm state and not
  used as cold CLI proof.
- R5: Confirm Markdown lint/fix performance attributes parser, rule, fix, and
  diagnostic-mapping costs.
- R6: Confirm any optimization has before/after evidence from comparable
  builds and comparable machines.
- R7: Confirm website/docs do not overclaim native performance before checked
  artifacts exist.

## Reviewer Blocking Criteria

Block completion if any accepted LS-Lint-equivalent fixture is slower than
native LS-Lint, if native Assura performance remains local-only Criterion
evidence, if aggregate speedups hide slower accepted rows, if warm daemon
results are used as cold CLI proof, if fixture classifications can be changed
without review, or if normal CLI usage becomes cumbersome to satisfy a
benchmark.

## Kickoff Text

Use this prompt to start the large goal-driven work:

```text
Execute docs/goals/assura-performance-polish-program.md.

Revalidate the goal against the current roadmap, checked LS-Lint performance
data, content runtime benchmarks, project-intelligence benchmarks, CI gates,
and website performance artifacts before coding. Preserve the strict
no-slower-than-native-LS-Lint gate for every accepted LS-Lint-equivalent
fixture. Then add a tracked native Assura performance report for content
collection validation/querying, document graph operations, context packs,
Markdown lint/fix, repository references, daemon/session warm queries, and
agent/editor-facing query paths. Calibrate baseline budgets from measured
evidence, wire merge gates after calibration, optimize the highest-impact rows
with before/after proof, update docs/website data, and get independent review
focused on whether slow rows can be hidden or misclassified.
```
