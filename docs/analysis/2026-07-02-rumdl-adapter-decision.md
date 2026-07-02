---
title: Rumdl Adapter Decision
status: active
date: 2026-07-02
---

# Rumdl Adapter Decision

## Decision

Prototype `rumdl` as an optional subprocess adapter before considering a direct
library dependency.

Assura currently declares `rust-version = "1.70.0"` in `Cargo.toml`. The live
candidate evidence for `rumdl 0.2.27` reports `rust-version: 1.94.0`. A direct
library dependency would therefore turn the Markdown engine goal into an MSRV
policy change before Assura has proved the adapter behavior, config mapping,
safe-fix boundary, or performance value. That is the wrong order.

The next implementation slice should keep Assura's current MSRV and prove the
runtime boundary first:

1. discover an installed `rumdl` binary only when the user enables the
   markdownlint-compatible engine for a Markdown scope;
2. run `rumdl check --output-format json --no-cache` against explicit Markdown
   paths or isolated copies;
3. map accepted `rumdl` diagnostics into stable Assura `markdown_*` rule IDs;
4. preserve Assura-owned severity, suppression, staged ordering, link/reference
   checks, and safe-fix reporting;
5. refuse or skip the adapter with clear diagnostics when `rumdl` is missing,
   returns an unsupported schema, or proposes unsafe fixes;
6. measure subprocess overhead separately from engine lint/fix time before any
   direct dependency or MSRV decision.

## Rationale

`rumdl` is still the leading Rust candidate because it produced JSON
diagnostics, line/column data, rule IDs, severities, and fix metadata for the
candidate fixture. It also overlapped the target markdownlint-compatible rule
set better than `mado`, and it behaved more safely than `mdlint`, which rewrote
probe fixtures in check mode.

The adapter must not replace Assura's current Markdown contract wholesale.
Assura already has frontmatter-aware heading parsing, reasoned suppressions,
rule-owned severity overrides, repository link/reference checks, staged
structure-first ordering, and bounded safe-fix reports. `rumdl` can provide
commodity markdownlint-compatible diagnostics and candidate fix metadata, but
Assura remains responsible for:

- deciding which Markdown scopes are eligible;
- deciding which candidate rules are accepted;
- translating candidate rule names to stable Assura IDs;
- applying severity and suppression policy;
- deciding whether a candidate fix is safe enough to preview or apply;
- preserving daemon/editor/agent report contracts.

## Adapter Proof Gate

The first adapter slice is done only when a reviewer can prove all of these:

| Requirement | Proof |
| --- | --- |
| No MSRV change | `Cargo.toml` keeps the current Assura MSRV and no `rumdl` dependency is added. |
| Explicit opt-in | Config or command behavior cannot run external `rumdl` unexpectedly. |
| No source mutation during check | Tests prove check-mode adapter execution does not modify committed or temp source files. |
| Stable rule mapping | At least `MD001`, `MD009`, `MD012`, `MD018`, and `MD024` map to stable Assura rule IDs. |
| Assura-owned reference checks | `markdown_link_target`, heading-anchor, line-anchor, and root-link behavior remain covered by existing tests. |
| Severity and suppression | Candidate findings use Assura rule-owned severity and reasoned `assura-ignore` semantics after mapping. |
| Safe-fix boundary | Candidate fix metadata is previewed as unsafe or adapter-owned until Assura explicitly allows an operation. |
| Performance attribution | Timings separate process startup, file preparation, `rumdl` execution, JSON parsing, mapping, and report generation. |

## Open Questions

- Whether the adapter should use source files directly for check-only mode or
  always use isolated copies until `rumdl` fix behavior is independently
  constrained.
- Whether the first public config should be `markdown.engine: assura|rumdl` or
  a narrower `markdown.markdownlint_candidate.enabled` field while the support
  surface remains beta.
- Which `rumdl` config options are needed to prevent false positives on
  Assura-valid frontmatter and title conventions.
- Whether `rumdl` fix metadata should ever feed `assura fix markdown --apply`
  directly, or only through Assura-owned operation allowlists.
