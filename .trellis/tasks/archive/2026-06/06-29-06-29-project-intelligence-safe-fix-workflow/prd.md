# Project Intelligence Safe Fix Workflow

## Goal

Complete `docs/goals/assura-project-intelligence-safe-fix-workflow.md` by
turning the existing Markdown safe-fix command from a summary-style fix runner
into a bounded preview/apply/audit workflow that humans, agents, MCP wrappers,
and future editor transports can correlate safely.

## What I Already Know

- The active usability program requires safe fixes to support preview, bounded
  apply, machine-readable audit output, and a clear no-automatic-repair policy.
- `assura fix markdown` already supports `--dry-run`, JSON output, and applying
  the trailing-spaces rule, but the report only exposes aggregate counts.
- The next product gap is not adding broad Markdown formatting; it is making
  accepted safe repairs auditable, idempotent, and reusable by transport
  surfaces.
- Current docs classify `assura fix markdown --dry-run --format json` as
  experimental and already describe the current apply command.

## Requirements

- Keep explicit opt-in for all writes. Preview is the default no-write mode;
  `--dry-run` remains explicit preview, and `--apply` is required to write.
- Preserve the supported safe-fix class: configured Markdown blank-line
  trailing spaces only.
- Add a common plan/audit schema family for Markdown safe-fix dry-run and
  apply reports.
- Include per-file and per-fix detail: stable fix IDs, paths, line numbers,
  operation names, status, before/after counts, changed paths, applied fix IDs,
  skipped fixes, and failure reasons.
- Keep apply idempotent: rerunning after an apply should report no planned or
  applied fixes and leave files unchanged.
- Report skipped files/fixes for non-target Markdown or Markdown files outside
  configured lint scopes instead of making agents infer why counts stayed zero.
- Preserve predictable failure behavior for invalid paths and unreadable/write
  failures by returning normal CLI errors without broad or partial semantic
  rewrites.
- Document recovery expectations as VCS-first rollback guidance.
- Update docs, support policy, release notes, and visual demo examples so users
  can see preview -> apply -> audit -> rerun.

## Acceptance Criteria

- [x] `assura fix markdown --dry-run --format json <path>` returns the shared
  schema with fix IDs and detailed planned/skipped records without writing.
- [x] `assura fix markdown --apply --format json <path>` returns the same schema
  family with applied records, changed paths, and applied fix IDs.
- [x] Re-running apply on an already fixed project reports no planned/applied
  fixes.
- [x] Tests cover no-op, dry-run, apply, partial skip, invalid path, and dirty
  non-target file behavior.
- [x] Context-pack and agent/session safe-fix previews retain enough metadata
  to correlate with CLI plan/audit records.
- [x] Documentation site examples visually demonstrate preview, apply, audit,
  and recovery expectations.
- [x] Support policy and release notes classify safe-fix apply behavior
  without claiming automatic repair.

## Technical Approach

Extend `MarkdownFixReport` rather than creating a separate command. The report
becomes the shared plan/audit contract, with detail vectors for files and
fixes. Keep the current `assura.safe-fix.markdown.v1` schema unless tests show
that consumers need an explicit v2; compatibility can be handled by adding
fields because the product is pre-1.0 and current tests already bind v1.

The implementation should keep all write logic in
`src/cli/check/markdown_fix.rs`, reuse existing Markdown lint scope resolution,
and update content-query safe-fix output only if the current fact surface lacks
correlation data.

## Decision (ADR-lite)

Context: Future MCP/LSP transports need stable identifiers and audit records,
but this goal should not introduce a daemon, editor server, or broad formatter.

Decision: Promote the existing CLI command into a detailed safe-fix plan/audit
contract for the single proven trailing-spaces fix class. Treat dry-run and
apply as two modes of the same report shape.

Consequences: This keeps the surface small and testable while giving future
transports stable fields to wrap. Additional Markdown fixes remain out of
scope until separately proven safe.

## Out Of Scope

- Automatic repair without explicit approval.
- Semantic rewrites, prose generation, or cross-file relation repair.
- New Markdown formatter dependencies.
- MCP or LSP transport implementation; those are separate successor goals.

## Technical Notes

- Goal: `docs/goals/assura-project-intelligence-safe-fix-workflow.md`
- Program: `docs/goals/assura-project-intelligence-usability-program.md`
- Existing implementation: `src/cli/check/markdown_fix.rs`
- Existing tests: `tests/markdown_lint_fix_tests.rs`
- Existing docs: `website/src/content/docs/examples/project-intelligence-demo.md`,
  `website/src/content/docs/product/markdown-validation.md`,
  `docs/support-policy.md`, `docs/release-notes.md`
- Relevant specs: `.trellis/spec/assura/index.md`,
  `.trellis/spec/guides/code-reuse-thinking-guide.md`,
  `.trellis/spec/guides/cross-layer-thinking-guide.md`
