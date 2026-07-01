---
id: analysis-2026-06-18-markdown-tooling-evaluation
type: analysis
title: Markdown tooling evaluation
status: active
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-goal-11-markdown-outline-validation.md
  - .trellis/spec/assura/config-notation.md
---

# Markdown Tooling Evaluation

Assura should not build a full Markdown linting stack before evaluating
existing Rust-native tooling. Markdown validation is likely part of the normal
Assura project contract, so bringing in Markdown linting, frontmatter parsing,
and link validation is acceptable when the tool is maintained, performant, and
configurable enough for project-structure checks.

## Current Repo Baseline

Assura already has in-tree Markdown checks for frontmatter presence, heading
depth, required sections, and model-aware outline validation. Typed
frontmatter fields belong to content runtime models and collections, not
generic Markdown rules. The implementation is purpose-built and lightweight,
but it does not replace a Markdown linter or general link checker.

`Cargo.toml` currently lists optional Markdown-related dependencies:

- `pulldown-cmark`
- `frontmatter`

## Tooling Candidates

- `rumdl`: Rust Markdown linter and formatter with markdownlint-compatible rule
  coverage, extra rules, config support, and performance claims. It includes
  relevant rule areas for headings, links, frontmatter spacing/key ordering,
  table of contents validation, and relative-link validation.
  Source: https://rumdl.dev/
- `lychee`: Rust async link checker for Markdown, HTML, reStructuredText, and
  websites with JSON output and CI-friendly behavior.
  Source: https://lychee.cli.rs/overview/
- `comrak`: Rust CommonMark and GitHub Flavored Markdown parser/renderer with
  an AST API. It is a stronger parser candidate than ad hoc heading scanning
  when Assura needs precise CommonMark/GFM behavior.
  Source: https://github.com/kivikakk/comrak
- `markdown` / `markdown-rs`: Rust Markdown parser with mdast output and support
  for common extensions including MDX and frontmatter. It may be a better fit
  if Assura wants a unified-style AST model.
  Source: https://github.com/wooorm/markdown-rs
- `gray_matter` or `markdown-frontmatter`: focused frontmatter parsers that can
  replace hand-rolled frontmatter splitting if the selected Markdown parser does
  not cover Assura's needs.
  Sources: https://docs.rs/gray_matter and
  https://docs.rs/markdown-frontmatter

## Initial Direction

Goal 11 should start with a tooling fit decision before custom implementation.
The expected default is:

- prefer `rumdl` or another mature linter for general Markdown lint rules;
- prefer `lychee` or an equivalent Rust link checker for link validation;
- prefer a real parser/AST for outline semantics if heading behavior needs to
  match CommonMark or GFM precisely;
- keep Assura-owned code for config-specific outline semantics, diagnostics,
  project-structure scoping, and relationship composition that generic tools do
  not understand.

The implementation decision must compare library versus CLI integration,
configuration model, output normalization, offline behavior, internal-anchor
support, performance cost, and support surface before adding dependencies.

## Goal 11 Decision

Assura will not add a new generic Markdown lint, link-checking, or frontmatter
dependency for the Goal 11 outline slice. The runtime change is deliberately
limited to config-specific `markdown.outline` semantics:

- keep existing lightweight frontmatter, heading-depth, and required-section
  checks in place for the current `assura check` surface;
- continue treating `rumdl` as the preferred future candidate for broad
  markdownlint-compatible rules, but do not wrap it until Assura exposes a
  supported generic Markdown lint contract;
- continue treating `lychee` as the preferred future candidate for link
  validation, because link checking needs offline/network policy and output
  normalization that this goal does not define;
- keep `pulldown-cmark` available for the full internal Markdown module, but
  avoid making the check-only CLI path depend on that optional full-CLI stack
  solely for outline matching;
- use Assura-owned code for nested outline semantics, relationship-provider
  composition, deterministic ambiguous-root errors, and diagnostics that name
  the configured outline entry and observed heading location.

This keeps the implementation cost bounded: outline matching scans extracted
ATX headings once and then walks configured outline entries in document order.
The scan ignores fenced code blocks and indented code in the same way as the
existing heading-depth and required-section checks. If future support requires
CommonMark/GFM edge cases beyond this scanner, the next decision should compare
moving the check-only path to a parser/AST dependency against the measured
binary-size and runtime cost.

## 2026-06-28 Project Intelligence Decision

The Rust Markdown validation/fixing successor goal refreshed current upstream
state for `rumdl`, `mkdlint`, and `comrak` before implementation. Assura should
not add the current releases of these crates as direct dependencies while
`Cargo.toml` declares `rust-version = "1.70.0"`.

| Candidate | Current evidence | Fit decision |
| --- | --- | --- |
| `rumdl 0.2.25` | `cargo info rumdl` reports `rust-version: 1.94.0`; local CLI probe on Rust 1.94.1 produced JSON diagnostics with line, column, severity, fixable flag, and byte-range fixes; `rumdl check --fix --fixable MD009` preserved YAML frontmatter and only removed trailing spaces. Upstream docs describe 76 lint rules, `rumdl check --fix`, JSON-capable CLI output, multiple Markdown flavors, and single-binary installation. | Best future broad markdownlint-compatible linter/formatter candidate, but not a direct dependency under the current MSRV. Safe only as a separately installed/bundled binary after Assura defines packaging and version policy. |
| `mkdlint 0.11.9` | `cargo info mkdlint` reports `rust-version: 1.93`; docs.rs exposes sync/async library APIs, `LintError`, `FixInfo`, custom rules, and `apply_fixes`; CLI JSON probe reported markdownlint-style diagnostics and fix metadata. Local `--fix` with all non-MD009 rules disabled preserved YAML frontmatter and removed trailing spaces. | Best embeddable library shape among the candidates, but not MSRV-compatible. Revisit if Assura raises MSRV or if a compatible release exists. |
| `comrak 0.52.0` | `cargo info comrak@0.52.0` reports `rust-version: 1.85`; docs.rs exposes CommonMark/GFM parsing, AST traversal, and CommonMark formatting. Local CLI probe with `--front-matter-delimiter '---' --to commonmark` preserved YAML frontmatter but formatted malformed Markdown rather than reporting actionable lint diagnostics. `comrak 0.49.0` is compatible with Rust 1.70, but it is a parser/formatter, not a linter. | Good parser/AST candidate when Assura needs CommonMark-precise analysis, but not a generic lint/fix provider. Do not use it alone to satisfy lint diagnostics. |

Local probe fixture:

```text
---
title: Probe
---

#Title


Body with trailing spaces.

## Child

```

Evidence commands that use Assura or Cargo's stable public surface:

```bash
cargo search rumdl --limit 5
cargo search mkdlint --limit 5
cargo search comrak --limit 5
cargo info rumdl
cargo info mkdlint
cargo info comrak@0.52.0
rustc --version
```

Local candidate probes also ran each downloaded crate's CLI against the fixture
with its native JSON or CommonMark options; the exact third-party flags are
intentionally not recorded as Assura command-surface examples.

Implementation direction for this successor goal:

- Add a narrow Assura-owned generic Markdown lint/fix slice for blank-line
  trailing spaces. This is deliberately one deterministic rule, not a broad
  linter reimplementation.
- Report the diagnostic through `assura check` with a Markdown lint-specific rule
  code and source line/column text in the message.
- Add a safe fix command that applies only this deterministic blank-line
  whitespace class initially; do not rewrite content-line hard breaks.
- Keep `markdown.require_frontmatter`, heading depth, required sections, and
  nested outline as Assura-owned structure behavior.
- Keep typed frontmatter fields in content runtime models and collections.
- Revisit `rumdl`/`mkdlint` integration when Assura either raises MSRV or
  defines an external-binary packaging contract that does not depend on the
  user's repo installing JavaScript or arbitrary commands.

### First-Slice Overhead Evidence

Release-mode timing used a temporary copy of this repository's `docs/` corpus
with 139 Markdown files. Before timing, `assura fix markdown` cleaned the copied
corpus so JSON diagnostic rendering would not dominate the lint-on run; the
cleaning step changed 14 files and applied 503 blank-line whitespace fixes in
the temporary copy only.

The timing compared two otherwise identical configs:

- `markdown.lint_trailing_spaces: false`
- `markdown.lint_trailing_spaces: true`

Both used `target/release/assura check --format json <fixture>`, measured with
`hyperfine 1.20.0`, 5 warmups, and 30 runs.

| Scenario | Mean | Median | Range | Interpretation |
| --- | ---: | ---: | ---: | --- |
| Markdown scope configured, trailing-space lint off | 10.1 ms ± 1.2 ms | 10.06 ms | 7.9-13.2 ms | Baseline configured Markdown traversal over copied docs. |
| Markdown scope configured, trailing-space lint on | 12.8 ms ± 2.2 ms | 11.97 ms | 10.6-19.2 ms | Adds about 2.7 ms mean over 139 Markdown files. |

Evidence command shape:

```bash
cargo build --release --quiet
target/release/assura fix markdown "$bench_root/on"
hyperfine --warmup 5 --runs 30 \
  "target/release/assura check --format json $bench_root/off" \
  "target/release/assura check --format json $bench_root/on"
```

## 2026-07-01 Common Lint Slice

Assura added an opt-in `markdown.lint_common` bundle as a bounded Rust-native
step toward the Markdown Quality goal. The bundle intentionally covers only
low-risk structural lint checks that fit the existing lightweight scanner:

- `markdown_heading_increment`
- `markdown_heading_marker_spacing`
- `markdown_duplicate_heading`
- `markdown_multiple_blank_lines`

These rules share the existing Markdown severity and reasoned suppression
contract. They do not claim full markdownlint compatibility and do not replace
the future `rumdl` or `mkdlint` decision. Before this Markdown epic can close,
Assura still needs release-mode timing evidence for the current Markdown corpus
with common lints enabled.
