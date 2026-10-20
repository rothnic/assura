# NU18: Explain selected Markdown policy

## Goal

Make CLI explain show which existing Markdown policy applies to a file,
including check_links and exact winning matcher or effective directory fallback.
Bead ae-whr.4 owns live status; related NU10 ae-bku is closed.

## Authority

Nick authorized dependency-ready Beads implementation with independent GPT-6
Luna review, corrections and root-owned gated merges. Root owns provider,
Beads, native delivery, push, PR, merge and closure. Implementation owner is
nu18-markdown-explain. No release or policy expansion is authorized.

## Contract

- Share the existing canonical best_file_pattern_match selection between
  validator, Markdown fix and explain. A matched Markdown bundle wins over
  the effective directory fallback; preserve precedence and checked behavior.
- Expose optional selected Markdown policy in CLI JSON/YAML/text/agent output,
  including selection kind, winning pattern when present, and check_links.
  Keep existing top-level output/schema and unrelated fields compatible.
- Preserve public Rust PathExplainReport, PathExplainRules and exported
  explain_structure_path signature and serialization. Use an internal resolved
  builder plus private CLI flatten adapter; this is an existing consumer
  compatibility boundary, not a speculative shim.
- Report effective matcher/directory selection only. Do not invent authored
  config/scope provenance lost by rule merge.
- check_links-only selected policy must not be described as unconfigured.
  Preserve extension filtering, exclusions, errors and deterministic output.
- Existing files only: check/rules.rs, validators.rs, markdown_fix.rs,
  explain.rs, explain_rules.rs, doctor.rs, private agent renderer and existing
  explain-pattern integration tests as needed. No new public module, API,
  dependency, matcher, config, thresholds or origin-tracking subsystem.

## Acceptance

Actual CLI explain and check controls agree for specific override versus broad
matcher, directory fallback, nonmatching patterns and link-only configuration.
Prove missing local target behavior discriminates true/false check_links;
selected settings must not leak to another file. Verify compatible public Rust
construction/return types and all CLI renderers. Independent Luna source and
behavior review, corrections and applicable current PR/hosted gates precede
root merge; verify merged outcome/native closure and clean owned refs/worktree.

## Evidence

Base a1d2f0aaf405a8d7e6359eb4c42aaaa4251a599e. Reviewed scope and public-contract
decision: external nu18/implementation-scope.md and
nu18/public-contract-scope-review.md. Candidate source implementation began at
6baf31f2. Root admits source editing and cheap format/diff/context/structure
checks only. NU15 owns the local Cargo validation lane; do not build/test/install,
publish, write Beads/native state or dispatch agents. Root will admit focused
runtime proof after resource and source review. Preserve failed gates for
three-way read-only diagnosis; do not weaken assertions or gate thresholds.

## Scoped correction

The independent Rust 1.94.1 visibility canary at
`nu18/visibility-canary.json` (stderr alongside it) produced E0364/E0603 when
the private CLI adapter and its selected-policy return type were declared
`pub(super)` in `cli::check::explain`: the existing `cli::check` re-export was
already scoped to `cli`, but the items were not visible to sibling
`cli::doctor`. The minimal correction scopes only those two declarations to
`pub(in crate::cli)`. Individual fields remain at their prior visibility;
doctor consumes only the existing compact renderer and serde output. This
does not add or change any externally public Rust API. The Rust visibility
canary is static evidence only.

The first admitted focused Cargo run on `83a8c667` exited 101 before either
integration test ran. Its seven E0425 diagnostics identified a removed
`rules` argument still used by the pre-existing ordinary-file
`file_structure_rules` predicate in `explain_skipped_checks`. The correction
restores that parameter and passes the already computed `&effective_rules`
alongside `selected_markdown`; the existing seven-field predicate and Markdown
selection condition remain unchanged. Raw receipt/stdout/stderr and the three
read-only triages are preserved under
`nu18/compile-failure-83a8c667/` and
`nu18/focused-validation-83a8c667/`. The two-target focused command remains
unrun on the corrected source and is admitted to the environment agent.
