# Assura Config Notation

This is the canonical contract for hand-authored structure notation. Assura is
pre-1.0: one primary authored form is preferred, and removed alpha forms do not
become permanent parser aliases.

## Model Roles

`config::config::Config` plus `ConfigLoader` is the sole current runtime
interpretation: `check`, `status`, `explain`, `doctor`, and `info` load it from
the selected config path. The loader parses, normalizes, and semantically
validates this `structure` notation before a command consumes it.

`ast::LegacyNotationConfig` with `LegacyConfigParser` is retained only for the
older policy/context notation used by legacy validation and the named LS-Lint
compatibility adapter. `types::LegacyPolicyConfig` similarly serves the legacy
policy engine. Neither is a fallback for current runtime commands. Migration
normalizes supported LS-Lint input through its named adapter and validates the
result with `ConfigLoader`.

## Design Criteria

Notation decisions are judged by:

1. **Hierarchy fit:** configuration reads in the same order as an IDE tree.
2. **Scope certainty:** current, direct-child, and recursive reach are explicit.
3. **Brevity:** simple naming and parameterized directives stay scalar.
4. **Composition:** repeated constraints and repeated child trees are reusable.
5. **Type safety:** impossible file, directory, node, and tree combinations fail
   during config loading.
6. **Agent utility:** a failure preserves threshold, severity, and bounded
   project-owned repair context.
7. **Performance:** selectors and rule expansions compile once, outside
   per-entry traversal.

## Extension API Boundaries

Structure notation is the deterministic project-policy language. It does not
embed executable plugins, remote rules, or an extension SDK. Language-specific
and custom validation remains behind the documented extension/API boundary so
structure selectors stay local, serializable, and compilation-friendly.

## Root And Tree Model

`structure:` is the project root. Authors do not add another `./:` wrapper.
Inside a tree, `./` means the current matched directory node.

Order entries like an IDE explorer:

1. current-node policy;
2. concrete directories, alphabetically, with subtrees in place;
3. concrete files, alphabetically;
4. direct wildcard policies;
5. recursive policies.

Comments label related chunks. YAML order is presentation only; selector
specificity and explicit composition determine behavior. Within one ` | `
composition, directives apply left to right and a later value for the same
attribute is the explicit local override.

## Selector Matrix

All wildcard selectors are relative to their current `structure:` or tree-rule
anchor.

| Selector | Target |
| --- | --- |
| `./` | Current matched directory node. |
| `./*` | Direct child files. |
| `./*/` | Direct child directories. |
| `./.*/` | Direct child dot directories; naming validates the captured name without the leading dot. |
| `./**/` | Current directory and every descendant directory. |
| `./**/*` | Files at any depth below the anchor. |
| `./**/*/` | Directories at any depth below the anchor. |
| `./**/*.{ts,tsx}` | TypeScript files at any depth below the anchor. |
| `.ts` | Direct `.ts` files at the current anchor. |
| `packages/*/` | Direct package directories below `packages/`. |
| `AGENTS.md` | One exact direct file. |

The `./` prefix on wildcard selectors is an explicit relative-scope marker and
allows unquoted YAML keys such as `./*/`. It is removed during normalization.
Exact hierarchy keys such as `apps/` and `web/` do not need the prefix.

`./**/` includes its anchor exactly once. Nested selectors are rebased onto
each matched directory; they do not become a second independently inherited
recursive policy.

## Cardinality And Defaults

The engine is open by default. Unmentioned paths are unchecked, not passing.

- Exact literal files and directories default to `exists:1`.
- Exact directories can set their own optionality with nested
  `./: exists:0-1`.
- Glob and capture selectors are match-only unless they declare cardinality.
- `exists:0` forbids a direct match.
- `exists:0-1` permits an optional singleton.
- `exists:N-M` applies a bounded direct-child count.
- Recursive file counts are invalid; move `exists` to a matching directory
  scope so it counts direct children.

Finite comma-brace selectors expand independently. For example,
`pnpm-{lock,workspace}.yaml: exists:1` requires both files, while
`./{assets,references,scripts}/: exists:0-1` permits each directory separately.
Expansion is bounded during config loading.

## Scalar Shorthand

Prefer scalar directives when a nested mapping adds no information:

```yaml
structure:
  ./**/:
    ./*/: kebab-case
    .{ts,tsx}: kebab-case | max_lines:500 | severity:low
```

The file value above is equivalent to:

```yaml config-fragment
naming: kebab-case
max_lines: 500
severity: low
```

Supported scalar forms are:

- bare naming: `kebab-case`, `snake_case`, `camelCase`, `PascalCase`;
- naming alternatives: `kebab-case | exact:README | exact:AGENTS`;
- parameterized directives: `exists:1`, `max_lines:500`,
  `max_size:100KB`, `limit_children:10`, `severity:low`, and
  `message:See docs/structure.md.`;
- rule references: `$name`;
- compatible top-level composition separated by ` | `.

Composition splits only on a top-level pipe surrounded by spaces. Pipes inside
regex, braces, brackets, or parentheses remain part of that directive.

Use expanded mappings when values are nested or a multi-attribute rule is more
readable. Do not introduce JSON-like inline objects.

## Rule Types And Rebasing

Rule definitions use plain names. References use `$name` and do not require
quotes.

A rule is exactly one kind:

- **node constraint:** contains directives such as `exists`, `max_lines`,
  `severity`, or `message`;
- **child tree:** contains only relative path selectors.

Mixing node directives and child selectors in one rule is invalid. Applying a
tree rule to a file, a file-only directive to a directory, or an incompatible
pipeline is also invalid before traversal.

```yaml
rules:
  entrypoint:
    max_lines: 160
    severity: low
    message: See docs/agent-guidance.md.

  workspace:
    AGENTS.md: $entrypoint
    package.json: exists:1

structure:
  packages/:
    ./*/: $workspace
  AGENTS.md: $entrypoint
```

Applying `$workspace` to `./*/` is equivalent to copying its two relative
selectors under every matching direct child directory. Node rules keep the
selector at the use site.

Create a leaf rule when it is referenced at least twice or carries meaningful
reusable repair guidance. A tree rule may have one authored selector when that
selector expands to repeated runtime matches, as `$workspace` does above.
One-use aliases around one primitive are advisory config smells.

## Naming

Built-in naming checks avoid regex compilation. For files, built-in naming is
segment-aware after removing the selector-owned extension. Under `kebab-case`,
`vite.config.ts`, `button.test.tsx`, and `next-env.d.ts` pass; empty, uppercase,
or underscore-separated segments fail.

`exact:NAME` is a naming alternative, not an existence requirement:

```yaml
structure:
  ./**/:
    .md: kebab-case | exact:AGENTS | exact:README
```

Exact literal path declarations are naming exceptions to broader extension
defaults while still inheriting compatible limits. Anchored `regex:` remains
the advanced escape hatch. More-specific compound-extension selectors override
broader final-extension selectors.

Generated LS-Lint compatibility bundles preserve dot-prefixed selectors as
exact extension combinations: `.js` does not select `button.test.js`, while
`.test.js` does. Native Option A patterns use star-dot selectors when broad
final-extension behavior is intended: `*.js` selects both files. This
distinction is migration metadata in normalized policy, not a second
recommended authoring style.

## Closed Direct Contents

Closed scope uses existing `exists:0`; there is no `forbid`, `contents:`, or
`structure!` alternative.

```yaml
rules:
  closed-entry:
    exists: 0
    message: See docs/agent-guidance.md#layout.

  closed:
    ./*/: $closed-entry
    ./*: $closed-entry

structure:
  .agents/:
    ./: exists:0-1 | $closed
    skills/: exists:0-1
```

More-specific exact declarations refine broad wildcard denial independently of
source order. Closed policy applies only where composed; the rest of the project
remains open for incremental adoption and LS-Lint migration.

## Thresholds And Repair Context

`limit_children` counts combined direct files and directories after automatic
and configured exclusions. A concise advisory policy is:

```yaml
rules:
  folder-health:
    limit_children: 10
    severity: low
    message: See docs/structure.md.

structure:
  ./**/:
    ./: $folder-health
```

Severity belongs to the composed constraint; an advisory threshold must not
weaken a blocking existence or naming rule. Text, JSON, agent output, hooks, and
explain preserve the concrete threshold and append bounded repair context.

## Project-Owned Recipes

First-party recipes are authoring sources, not hidden runtime dependencies:

```bash
assura init --recipe agentic-core --recipe structure-health
```

The command copies ordinary commented YAML into `.assura/config.yml`. The
project can edit it, checks work with the catalog unavailable, and Assura never
silently rewrites it. `agentic-core` requires root guidance and constrains
optional project-local skills. `structure-health` provides advisory file and
directory health defaults without guessing a language or framework.

## Performance And Portability

- Parse and expand finite selectors once.
- Type-check and rebase rules during config loading.
- Compile file and directory matchers before traversal.
- Keep selector specificity deterministic and independent of YAML order.
- Round-trip every checking field through the portable compiled-config model.
- Bump the compiled schema whenever its payload changes so stale artifacts fail
  as incompatible instead of silently dropping policy.

Notation changes affecting normalization, selector matching, or compiled
artifacts must run parser, direct-check, compiled-check, cold performance, and
warm-session regression gates.

## Migration Boundary

Legacy Assura and LS-Lint are explicit migration inputs, not permanent native
aliases. Migration output must use this grammar and parse successfully before it
can be written. Public docs and the website must not teach compatibility-only
forms such as a second root `./:` wrapper, implicit cascading extension reach,
mixed node/tree rules, or `.dir` as the primary directory selector.

## Required Proof

- Every selector has positive and negative coverage.
- The canonical website YAML is loaded by Rust tests and executed against valid
  and intentionally drifting project trees.
- Direct and compiled checks return equivalent findings.
- Exact declarations refine `$closed` wildcard denial.
- Recursive defaults produce one finding per violated constraint.
- Generated recipe YAML validates with the recipe catalog unavailable.
- Documentation YAML fences and website data fixtures load during the build.
- Parser/normalizer regressions stay below 2%, accepted cold LS-Lint rows do not
  regress, and the warm loop retains its target.
