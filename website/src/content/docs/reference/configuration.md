---
title: Configuration Reference
description: Supported configuration fields for Assura
template: doc
sidebar:
  order: 1
---

Assura discovers configuration from `.assura/config.yml` by default. The stable
validation command is:

```bash
assura check
```

Use structured output when you need reproducible evidence:

```bash
assura check --format json .
```

## Discovery

Recommended location:

```text
.assura/config.yml
```

The CLI can also receive a config path with the global `--config` option.

## Top-Level Fields

```yaml
structure: {}
exclude: []
rules: {}
ls: null
```

| Field | Behavior |
| --- | --- |
| `structure` | Directory-shaped policy tree used by `assura check`. |
| `exclude` | Glob-like paths excluded from validation and direct-child counts. |
| `rules` | Optional reusable authoring fragments referenced from `structure` with `use:`. Rules compile into the normal structure model before validation. |
| `ls` | Compatibility input used by migration and tests, not the public `assura check` policy surface. Prefer `assura migrate` so LS-Lint rules are converted into `structure`. |
| `patterns` | Library resolver field from the older config model. It is accepted by the config type but is not the public `assura check` policy surface. Use `structure` instead. |

Assura excludes its own `.assura/**` tool-state directory automatically during
checks. Do not add it to ordinary project-shape excludes unless a future command
explicitly asks for that directory to be validated.

## Concise Structure Notation

Simple policy should stay in the tree:

```yaml
structure:
  README.md: exists:1
  AGENTS.md: exists:1
  src/: exists:1
  ./**/:
    .rs: snake_case
```

Concise keys expand to the same internal model documented below:

| Notation | Behavior |
| --- | --- |
| `README.md: exists:1` | Requires exactly one direct file named `README.md`. |
| `src/: exists:1` | Requires exactly one direct child directory named `src`. |
| `./: $rule` | Applies a node rule to the current matched directory. |
| `./*` | Targets direct child files. |
| `./*/` | Targets direct child directories. |
| `./**/` | Matches the current directory and every descendant directory. Nested selectors rebase at each match. |
| `.rs: snake_case` under `./**/` | Applies `snake_case` to direct Rust files in every matched directory. |

Use a mapping under the same path key when the directive needs more detail:

```yaml
structure:
  docs/:
    "{topic}.md":
      exists: 1
      markdown:
        outline:
          - Overview
          - ?? Prerequisites
          - Quick Start:
              - Installation
              - ?? Configuration
          - Why Assura?
          - title: "?? Debug Mode"
            optional: false
```

## Concise And Expanded Equivalents

Prefer a scalar directive when one attribute is enough. When the same group of
attributes applies to several file patterns, name it once and keep each use to
one line:

```yaml
rules:
  source-file:
    naming: kebab-case
    max_lines: 500

structure:
  ./**/:
    .{ts,tsx}: $source-file
```

The reusable shorthand above normalizes to the same configuration as expanded
attributes under each directive:

```yaml
structure:
  ./**/:
    .ts:
      naming: kebab-case
      max_lines: 500
    .tsx:
      naming: kebab-case
      max_lines: 500
```

Use the expanded form for a one-off override or when the directive needs
additional attributes. Directive-attached `naming`, `max_lines`, and `max_size`
apply only to files matching that pattern in the configured directory scope and
its inheriting descendants. Define one reusable directive and apply it to each
extension or explicit glob that should share the default.

Choose reach explicitly. Extension shorthand is direct to its current anchor;
put it under `./**/` when it should rebase at every directory:

| Notation | Reach |
| --- | --- |
| `.ts: $source-file` at root | Direct root `.ts` files. |
| `./*.ts: $source-file` | Direct files from the current rule or structure anchor. |
| `./**/*.ts: $source-file` | Root and descendant `.ts` files from the current anchor. |
| `.ts: $source-file` under `src/` | Direct `.ts` files inside `src/`. |
| `.ts: $source-file` under `./**/` | Direct `.ts` files in root and each descendant. |

The structure key controls reach. `*` matches one directory segment and `**`
crosses directory separators:

```yaml config-fragment
structure:
  packages/*/src/:
    .ts: $source-file
  packages/**/generated/:
    inherit: false
```

More specific scopes merge their file patterns by default. Set
`inherit: false` when a subtree should reset inherited policy. Run
`assura explain path/to/file.ts` to see the applied directory scopes and the
winning normalized naming, line, and size directives for that file. A file with
no matching pattern reports `matched_file_patterns=none`; JSON reports an empty
`matched_file_patterns` array. Text output marks scopes that discard inherited
policy with `(reset)`.

Captures use single braces such as `{topic}`. Removed alpha capture forms such
as `${name}` and `{{name}}` are not supported in hand-authored structure
notation.

## First-Time Notation Matrix

Start with LS-Lint-equivalent policies, then add Assura-native structure when
the project needs relationships or reusable contracts.

| Use case | Notation |
| --- | --- |
| Direct file naming | `.rs: snake_case` |
| Direct file count | `README.md: exists:1` |
| Optional singleton | `"*.lock": exists:0-1` |
| Forbidden direct children | `draft-*: exists:0` |
| Closed direct-content scope | `$closed` composed from `./*: exists:0` and `./*/: exists:0` |
| Generated output ignore | `exclude: ["target/**", "node_modules/**"]` |
| Captured source/test pair | `"{component}.tsx"` and `"{component}.test.tsx": exists:1` |
| Package documentation need | `needs: doc` with `provides: doc` |
| Reusable package policy | `rules:` plus `use: $package-standard` |
| Reusable file directive | `.ts: $source-file` after defining a node rule |
| Agentic project baseline | `assura init --recipe agentic-core --recipe structure-health` |
| Markdown outline | `markdown.outline` with nested heading lists |

Use the detailed fields below when a rule needs extra attributes or when you
are reading generated migration output.

Generated LS-Lint migrations preserve dot selectors such as `.js` and
`.test.js` as exact extension combinations. In native structure shorthand,
use `*.js` when one final-extension default should also cover compound stems
such as `next.config.js`; a more-specific `*.test.js` rule can override it.
This distinction keeps migration behavior equivalent without limiting the
broader defaults available to new Assura policies.

Markdown outline notation validates ordered heading structure without separate
heading-depth fields. It is for Assura-specific document structure checks, not
a replacement for generic Markdown linting or link validation.

## Project-Owned Agentic Recipes

Materialize the standard language-agnostic guidance and structure-health layers:

```bash
assura init --recipe agentic-core --recipe structure-health
```

Assura copies ordinary commented `rules:`, `structure:`, and `exclude:` YAML
into the project. Checks use only that project-owned file; there is no hidden
recipe lookup at runtime. `agentic-core` requires root `AGENTS.md` and
`README.md`, constrains optional project-local skills, and provides repair
links. `structure-health` adds advisory line and direct-child thresholds plus
recursive coverage. It deliberately does not guess project naming. Edit or
remove any generated policy to match the project.

For a project that already has `.assura/config.yml`, preview the additive merge
before writing it:

```bash
assura config add-recipe structure-health . --dry-run
assura config add-recipe structure-health .
```

Existing values win by default. `--force` replaces only conflicting recipe
values while preserving unrelated project policy.

Project-local skill directories are intentionally strict. Once a repository
adds `.agents/skills/<skill>/`, `.agents/skills/built-in/<skill>/`, or
`.agents/skills/custom/<skill>/`, that directory is treated as a skill and must
provide a bounded `SKILL.md` entrypoint. The entrypoint should route agents to
deeper `references/`, `scripts/`, `assets/`, or process docs instead of
compressing a large skill into one file.

Global or user-level skills installed outside the repository are not validated
by this project rule. If a third-party skill is copied, vendored, or linked
under `.agents/skills/**`, treat that checked project-facing path as owned by
the project: keep a concise local `SKILL.md`, preserve upstream content in
deeper references when useful, or keep the skill global and add a small wrapper
skill that tells agents where to find it.

## Directory Nodes

Each key under `structure` follows the project hierarchy. `structure:` is the
project root. Exact literal paths are required by default, so the concise tree
below requires `apps/web/src/` without a second `required` directive:

```yaml
structure:
  apps/:
    web/:
      src/:
        .tsx: kebab-case
```

Use `exists:0-1` when an exact directory is optional. A pattern directory such
as `"{package}/"`, `"package-*/"`, or `"**/generated/"` is match-only by
default. Add an explicit direct-child count only when cardinality matters.

Use `./` only inside a reusable or nested tree when a node rule applies to the
current matched directory. `./*/` names direct child directories, while
`./**/*/` names every descendant directory. Exact and more-specific selectors
refine broader wildcard policy independently of YAML source order.

Inside one scalar composition, ` | ` applies directives left to right. A later
directive for the same attribute is the intentional local override.

## File Rules

```yaml
rules:
  source-file:
    naming: kebab-case
    max_lines: 500
    max_size: 100KB

structure:
  README.md: exists:1
  ./**/:
    .{ts,tsx}: $source-file
    ./*.tmp: exists:0
```

| Field | Behavior |
| --- | --- |
| `naming` | Built-in case name or `regex:<pattern>`. |
| `max_lines` | Language-agnostic maximum line count. |
| `max_size` | Maximum file size such as `100KB` or `2MB`. |
| `require_docs` | Requires Rust documentation text for matching Rust files. |
| `exists` | Direct-child count: `0`, `1`, `0-1`, or a bounded range. |
| `markdown` | Markdown checks attached to matching `.md` files. |

`exists` does not count recursively. Use `"./*.ts": exists:1` at the root or
`"*.ts": exists:1` inside the relevant directory scope. Assura rejects
cross-directory forms such as `"./**/*.ts": exists:1` so a recursive-looking
rule cannot silently count only direct children.

## Directory Rules

```yaml
structure:
  packages/:
    ./package-*/: kebab-case
  ./tmp-*/: exists:0
```

For reusable directory contracts, define a tree fragment and apply it with a
scalar rule reference:

```yaml
rules:
  package-standard:
    package.json: exists:1
    README.md: exists:0-1
    src/: exists:1

structure:
  packages/:
    ./*/: kebab-case | $package-standard
```

## Markdown Rules

```yaml
markdown:
  require_frontmatter: true
  lint_trailing_spaces: true
  lint_common: true
  max_heading_depth: 3
  required_sections:
    - Summary
  outline:
    - Overview
    - ?? Prerequisites
    - Quick Start:
        - Installation
        - ?? Configuration
  rules:
    markdown_link_target:
      severity: low
```

| Field | Behavior |
| --- | --- |
| `require_frontmatter` | Requires YAML frontmatter in direct child Markdown files as a generic document-style rule. |
| `lint_trailing_spaces` | Reports blank Markdown lines that contain spaces or tabs. `assura fix markdown --dry-run --format json` includes this whitespace class by default; use `--rule trailing-spaces` to target only this bounded fix class. |
| `lint_common` | Reports common Rust-native Markdown lint findings for skipped heading levels, malformed heading marker spacing, duplicate headings, and multiple consecutive blank lines. |
| `max_heading_depth` | Fails when a Markdown heading is deeper than the configured level. |
| `required_sections` | Requires headings with the configured text. `assura fix markdown --dry-run --format json` includes deterministic missing-heading insertions by default; use `--rule required-sections` to target only this fix class. |
| `outline` | Validates ordered nested headings without requiring users to maintain heading depth numbers. Use `?? ` for optional headings and object form such as `title: "?? Debug Mode"` when a required heading starts with literal question marks. |
| `check_links` | Validates local relative Markdown links to files, Markdown heading anchors, and GitHub-style line or line-range anchors such as `#L12` and `#L12-L34`. It also reports existing local file references in prose or inline code that should be rendered as Markdown links. Remote URLs and same-file `#heading` links are ignored by this local check. |
| `rules` | Maps supported `markdown_*` rule IDs to per-rule options. `severity` accepts `low`, `medium`, `high`, or `critical`; `low` findings are advisory and the other severities are blocking. |

Use `models`, `collections`, and `relations` for typed Markdown frontmatter
fields. `markdown.required_fields` is rejected in Assura-authored config so
frontmatter field ownership stays in one content model path.

Use a reasoned suppression comment for intentional exceptions:

```markdown
<!-- assura-ignore markdown_link_target: generated fixture points at future docs -->
```

Suppressions must name a supported Markdown rule ID and include a non-empty
reason. Invalid suppressions are reported as `markdown_suppression`.

## Repository References

Opt into experimental source/comment/docstring reference diagnostics with
`extensions.repository_references`:

```yaml config-fragment
extensions:
  repository_references:
    - id: source_refs
      paths:
        - "src/**"
      frontmatter_fields:
        - source_documents
        - related
      severity: high
```

Assura scans supported source and config file types under matching paths for
local file references. When `frontmatter_fields` is set, matching Markdown files
also treat string or list values in those frontmatter fields as repository
references. This is useful for fields such as `source_documents`, `related`,
`evidence`, or `requirements`.

The check reports locally provable missing targets, missing Markdown heading
anchors, and invalid line anchors as `repository_reference_*` rules. Ambiguous
lower-confidence references remain available as graph context through
`assura content references`.

## Agent Guidance

Opt into experimental agent guidance diagnostics with
`extensions.agent_guidance`:

```yaml config-fragment
extensions:
  agent_guidance:
    - id: agent_project_guidance
      severity: low
      agents_path: AGENTS.md
      skill_paths:
        - ".agents/skills/*/SKILL.md"
      required_agents_sections:
        - Operating Rules
        - Process Docs vs Skills
        - Skills
        - Anchors
      required_skill_frontmatter:
        - name
        - description
        - applies_when
      required_skill_sections:
        - Workflow
        - Read as needed
        - Outputs
        - Guardrails
      skill_index_section: Skills
      best_practices_reference: "Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as concise indexes to deeper references."
      skill_routing_section: Skills
      allowed_skill_name_patterns:
        - "assura-*"
        - "assura_*"
      skill_reference_sections:
        - Read as needed
      skill_doc_routing_section: Read as needed
      skill_reference_prefixes:
        - references/
        - scripts/
        - assets/
        - docs/process/
      max_agents_lines: 160
      max_skill_lines: 120
```

This policy checks local guidance shape only. It reports stale or missing
`AGENTS.md` sections, duplicate heading anchors, missing project-local skill
links, missing `SKILL.md` frontmatter fields, missing required skill sections,
oversized guidance entrypoints, missing progressive-disclosure references,
unknown skill names in configured use-case routing tables, and SKILL sections
that fail to point to deeper references. When `skill_doc_routing_section` is
configured, that SKILL section may be empty, but non-empty content must be a
use-case table that routes agents to approved local docs, scripts, assets, or
process references. It does not install a global skill registry or create
host-agent-specific validation logic.

## Requirements Traceability

Opt into experimental requirements, claims, evidence, source-document, and
finding traceability diagnostics with `extensions.requirements_traceability`:

```yaml config-fragment
extensions:
  requirements_traceability:
    - id: document_project_traceability
      severity: high
      requirements_collection: requirements
      priority_field: priority
      high_priority_values:
        - high
        - critical
      coverage_collections:
        - evidence
        - claims
        - docs
      claim_collections:
        - claims
      evidence_collections:
        - evidence
      source_document_collections:
        - source_documents
      finding_collections:
        - findings
      owner_fields:
        - owner
      status_fields:
        - status
```

This policy is backed by the content runtime in the full CLI. It checks that
configured collections exist, high-priority requirements have coverage from
configured collections, claims link to evidence through modeled relations,
evidence links to source documents, and findings carry owner and status
metadata. It does not infer domain-specific scoring, replace repository
reference checks, or add a public plugin API.

## Computed Checks

Opt into experimental project-local computed findings with
`extensions.computed_checks`:

```yaml config-fragment
extensions:
  computed_checks:
    - id: rollup_score
      severity: high
      script: scripts/assura-rollup-score.sh
      windows_script: scripts/assura-rollup-score.cmd
      args:
        - --threshold
        - "80"
      timeout_ms: 5000
```

Assura executes only configured project-relative scripts, selects
`windows_script` on Windows when present, passes a versioned JSON request on
stdin, and accepts only versioned JSON findings on stdout.
Each accepted finding becomes a normal diagnostic with a
`computed_check:<policy-id>:<finding-code>` rule ID. Missing scripts, unsafe
paths, invalid output, nonzero exits, and timeouts are reported as ordinary
Assura findings so they flow through reports, doctor, agent-query gaps, hooks,
and merge gates. Computed checks are an advanced first-party extension policy,
not a public plugin API, remote execution surface, marketplace, or
domain-specific scoring preset.

## First-Party Extension Policies

`extensions.*` entries are first-party config policies executed by
`assura check`; they are not a public plugin API. Use them when a deterministic
cross-file policy does not fit ordinary `structure` notation.

| Family | Status | Purpose |
| --- | --- | --- |
| `extensions.custom_constraints` | Experimental first-party | Specialized built-in constraints. Prefer `structure` captures and `needs`/`provides` for common relationships. |
| `extensions.release_contracts` | Experimental first-party | Release artifact, checksum, workflow, docs, installer, and branch-reference synchronization. |
| `extensions.support_matrices` | Experimental first-party | Explicit support classification for commands, Rust export families, docs tables, packages, and binaries. |
| `extensions.manifest_semantics` | Experimental first-party | Cargo manifest metadata, publish policy, description, keyword, and binary checks. |
| `extensions.test_relationships` | Experimental first-party | Source/test evidence, manual-test exceptions, and fixture-family ownership. |
| `extensions.module_topologies` | Experimental first-party | Rust module-family ownership, roots, export classification, and internal visibility. |
| `extensions.docs_lifecycles` | Experimental first-party | Documentation lifecycle, frontmatter status, historical exceptions, and deterministic claim evidence. |
| `extensions.repository_references` | Experimental first-party | Locally provable repository-reference diagnostics. |
| `extensions.agent_guidance` | Experimental first-party | `AGENTS.md` and project-local `SKILL.md` routing contracts. |
| `extensions.requirements_traceability` | Experimental first-party | Content-runtime-backed requirement, claim, evidence, source-document, and finding traceability checks. |
| `extensions.computed_checks` | Experimental first-party | Project-local script-backed computed findings with versioned JSON contracts. |
| `extensions.relationships` | Internal generated first-party | Relationships normalized from `structure` captures, `exists:1`, `needs`, and `provides`. |

Assura does not currently support remote plugin loading, shell-executed
validation plugins, plugin marketplaces, TypeScript plugin APIs, or
semver-stable Rust library APIs. See
[Extension API Boundaries](/reference/extension-api-boundaries/).

## Relationships

Captured paths can express relationships without leaving the project tree. A
captured path without `exists` is optional; a captured path with `exists:1`
becomes required for each matching source with the same capture names.
Required captured children inside a captured directory stay ordinary structure
requirements for that directory.

```yaml
structure:
  src/components/:
    "{component}.tsx": {}
    "{component}.test.tsx": exists:1
```

If `src/components/Button.tsx` exists, Assura requires
`src/components/Button.test.tsx`. If no component exists, no test file is
required.

Use `needs:` and `provides:` when a relationship can be satisfied by more than
one artifact:

```yaml
structure:
  packages/:
    "{package}/":
      needs: doc
  docs/packages/:
    ./: exists:0-1
    "{package}.md":
      provides: doc
  docs/:
    ./: exists:0-1
    packages.md:
      sections:
        "{package}":
          provides: doc
```

For each package directory, either `docs/packages/<package>.md` or a heading
named `<package>` in `docs/packages.md` satisfies the `doc` need.

Entries that only declare `provides:` are providers, not producers. Missing
relationship reports name the producer, source pattern, declaring structure
entry, provider kind, and expanded counterpart or provider path. Duplicate
provider alternatives for the same need and capture set are rejected as
ambiguous during config loading.

## Closed-World Example

This policy rejects stray files and directories at the project root while
allowing generated output to stay outside the source contract.

```yaml
rules:
  closed-entry:
    exists: 0
  closed:
    ./*/: $closed-entry
    ./*: $closed-entry

structure:
  ./: $closed
  README.md: exists:1
  Cargo.toml: exists:1
  ./*.lock: exists:0-1
  src/: exists:1
  docs/: exists:1
  ./package-*/: exists:0-20
  ./draft-*: exists:0
  ./tmp-*/: exists:0
exclude:
  - "target/**"
  - "generated/**"
```

Given `draft-plan.md`, `scratch.txt`, and `tmp-cache/`, JSON output includes
stable path, rule, severity, blocking, message, and corrective context fields:

```json
{
  "path": "draft-plan.md",
  "rule": "forbidden_file",
  "message": "File 'draft-plan.md' is forbidden by policy",
  "severity": "high",
  "severity_label": "High",
  "blocking": true,
  "corrective_context": "Remove or rename the file, or narrow files.forbidden_patterns if this file should be allowed."
}
```

## Direct Counts And LS-Lint Boundary

Direct count rules apply only to direct children of the configured directory.

```yaml
structure:
  README.md: exists:1
  ./*.tmp: exists:0
  ./package-*/: exists:1-5
```

LS-Lint extension rules such as `.md: exists:1-2` map to direct file counts and
are treated as LS-Lint parity. Exact direct filename rules such as
`README.md: exists:1` are an Assura compatibility extension when produced by
`assura migrate`; upstream LS-Lint 2.3 does not treat exact filenames as count
targets.

## Report Formats

```bash
assura check --format text
assura check --format json .
assura check --format yaml .
assura check --format agent .
assura check --format agent --agent codex . --warn
```

The JSON report contains `success`, `project_root`, `config_path`,
`checked_path`, `files_checked`, `dirs_checked`, and `violations`.
Each violation contains `path`, `rule`, `message`, `severity`,
`severity_label`, `blocking`, and `corrective_context`. The `success` field is
false only when at least one violation has `"blocking": true`; low-severity
findings remain visible while allowing an exit code of 0.
