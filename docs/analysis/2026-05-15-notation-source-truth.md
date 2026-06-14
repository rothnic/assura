---
id: analysis-2026-05-15-notation-source-truth
type: analysis
title: Assura notation source truth
status: active
created: 2026-05-15
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - .trellis/spec/assura/config-notation.md
  - .trellis/spec/assura/structure-enforcement.md
  - docs/ls-lint-capability-comparison.md
  - docs/unified-tree-design.md
  - docs/archive/final-config-design.md
  - docs/archive/ls-lint-notation-guide.md
---

# Assura Notation Source Truth

This document is the current notation source of truth for the next LS-Lint
parity and performance work. It separates implemented structure-first behavior
from LS-Lint parity, Assura extensions, unsupported behavior, and planned
notation. Historical `policy`/`rules`/`apply` proposals remain useful design
input, but they are not the current product config surface.

**2026-06-14 update:** `.trellis/spec/assura/config-notation.md` is the
canonical Assura-native notation spec. This analysis continues to classify
implemented behavior and LS-Lint parity boundaries; the Trellis spec owns the
syntax for new implementation work.

**2026-05-26 update:** the LS-Lint rule coverage audit added support for
regex negation, regex directory substitutions, wildcard/brace directory scopes,
exact LS-Lint extension-combination matching, advisory `--warn`, and LS-Lint
multi-config merge conversion. See
`docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md`.

## Current Supported Structure-First Notation

Assura's supported public config surface is `.assura/config.yml` with a
top-level `structure:` tree and `exclude:` patterns.

Current structure nodes support:

- `files.naming` and `files.extensions`;
- `files.naming_patterns` for glob-like filename patterns;
- `files.allowed_names`, `files.allowed_patterns`,
  `files.forbidden_patterns`, and `files.allow_extra`;
- `files.exists` for direct child file count patterns;
- `directories.naming`;
- `directories.required`;
- `directories.allowed_names`, `directories.allowed_patterns`,
  `directories.forbidden_patterns`, and `directories.allow_extra`;
- `directories.exists` for direct child directory count patterns;
- `children` for explicit nested structure nodes.
- native path keys such as `README.md`, `src/`, `.md`, `.test.ts`, `.dir`,
  and wildcard directory scopes;
- `rules:` fragments referenced through quoted `"@rule"` values and `use`;
- `exists:1`, `exists:0-1`, `exists:0`, and `exists:N-M` shorthand for direct
  child cardinality.

Direct-content rules are hierarchical but not recursively strict by default:
`allow_extra`, allowed names/patterns, forbidden patterns, and direct count
checks apply to the configured directory's direct children. Inherited naming and
file rules may still apply to descendants.

## LS-Lint-Compatible Subset

The LS-Lint compatibility layer currently supports these LS-Lint 2.3 concepts:

- extension naming rules such as `.ts: camelCase`;
- wildcard extension rules such as `.*` and `.*.js`;
- subextension rules such as `.d.ts`, `.test.ts`, `.spec.ts`, and
  `.module.css`;
- `.dir` directory naming for the indexed directory itself and descendant
  directories governed by the same LS-Lint scope;
- explicit nested directory scopes such as `src:` and `packages/core:`;
- glob directory scopes such as `packages/*` and recursive scopes such as
  `src/**/c`;
- brace directory scopes such as `src/{a,b}/*`;
- OR syntax such as `kebab-case | snake_case`;
- regex naming, including LS-Lint anchoring, negation, and directory
  substitutions such as `${0}` and `${1}`;
- `ignore` mapped to Assura `exclude`;
- extension `exists`, `exists:0`, `exists:1`, and `exists:N-M` direct child
  file counts;
- `.dir` `exists` self-directory presence checks;
- direct-child-only count semantics.

Native LS-Lint parity claims must be limited to behavior supported by LS-Lint
2.3 and covered by equivalent fixtures.

## Assura Compatibility Extensions

Assura intentionally supports some behavior beyond native LS-Lint 2.3:

- exact direct filename `exists`, such as `README.md: exists:1`, maps to a
  direct file count for `README.md`;
- trailing-slash direct directory `exists`, such as `docs/: exists:1`, maps to
  a direct directory count for `docs`;
- closed-world direct-content checks can reject unexpected direct files and
  directories through `allow_extra: false`;
- exact allowed names and forbidden patterns can express root/project hygiene
  that LS-Lint does not model directly.

Exact filename `exists` must remain labeled as an Assura compatibility
extension. Live LS-Lint 2.3 does not treat `README.md: exists:1` as an exact
filename count.

## Unsupported Behavior

No known LS-Lint 2.3 rule behavior is currently classified as unsupported by
the LS-Lint compatibility claim. Exact filename `exists` remains an Assura
extension. Assura-native `rules:` fragments and `use` composition are separate
from LS-Lint compatibility.

## Naming Decisions

Historical docs use several names for similar concepts. Current status:

| Name | Current status |
| --- | --- |
| `structure` | Current supported product tree. Keep this for v0.1 docs and examples. |
| `exclude` | Current supported ignore/pruning surface. |
| `files.exists` / `directories.exists` | Current direct count checks. Keep for LS-Lint parity and range counts. |
| `directories.required` | Current exact required direct directory list. |
| `policy` | Historical proposed replacement for `structure`; not implemented. |
| top-level `rules` | Current reusable fragment registry for quoted `"@rule"` references and `use`; not a standalone policy tree. |
| `apply` | Historical reusable-rule attachment proposal; not implemented. |
| `require` | Historical shorthand concept; current product uses path keys with `exists:1`. |
| `allow` / `strict` | Historical shorthand concepts; current product uses path keys, `exists`, and `extra: false`. |

Future docs should use `structure` for current behavior and reserve
`policy`, `apply`, `require`, and `allow` for explicitly planned notation
sections.

## Native Notation Status

Native notation should refine the structure-first model instead of switching to
a parallel config language.

Use `.trellis/spec/assura/config-notation.md` as the canonical design for
implemented syntax and planned extension points.

Implemented:

1. Path-key shorthand under `structure:` for exact files, exact
   directories, extension rules, subextension rules, and directory scopes.
2. `exists` cardinality as the common presence model so required,
   optional, forbidden, and bounded direct contents do not require duplicate
   `required` and `allowed` declarations.
3. Reusable `rules:` fragments and `"@rule"` references, with `use` for
   tree fragments and deterministic merge order.

Planned:

4. Add nested Markdown `outline:` validation with `?? ` optional heading
   markers and object-form escape hatches for custom cases.
5. Add deterministic relation checks for code-to-doc use cases only after the
   path-key, reusable-rule, and outline surfaces are stable.
6. Add array-based OR naming as a readability improvement while preserving the
   existing string OR syntax.

## Pattern Scope Model

LS-Lint migration pattern scopes compile into a scope matcher plus rule
payload. Assura-native notation exposes that model for current wildcard
directory scopes:

- exact path scopes match one known directory path;
- single-segment wildcard scopes match existing direct children;
- recursive scopes match existing descendants;
- brace scopes expand into a small set of scope matchers;
- existence requirements remain explicit through `required`, `exists`, or a
  future `require` shorthand.

This distinction prevents lint scopes from becoming required directories. For
example, `packages/*` validates package directories that exist. It does not
report `required_directory` for `packages/*` as a literal child, and it does
not require a package named by a pattern unless a separate requirement declares
it.

## Performance Model

Notation should be designed so the checker can scale without expanding broad
patterns into huge concrete trees.

- Compile glob, regex, and naming patterns once per config load.
- Index file rules by extension or suffix when a rule key is extension-like.
- Use fallback glob scans only for broad patterns that cannot be indexed.
- Preserve direct-content checks as directory-local operations.
- Build or reuse a direct-child directory index instead of repeating
  `read_dir` for each count check.
- Keep exclusions available before expensive traversal and validation work.
- Treat pattern scopes as matchers over existing traversal data, not as
  generated required nodes.

## Historical Inputs

- `docs/unified-tree-design.md` is historical design input for a possible
  future tree syntax. It is not the current product source of truth.
- `docs/archive/final-config-design.md` is historical and describes the
  unimplemented `policy`/`rules`/`apply` proposal.
- `docs/archive/ls-lint-notation-guide.md` is historical and contains useful
  examples, but it overstates implemented content, pairing, context, and
  messaging behavior.
- `docs/ls-lint-capability-comparison.md` remains an active capability
  comparison, but this document owns current notation classification.
