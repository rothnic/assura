---
id: goal-assura-config-notation-rule-composition-and-site-alignment
type: goal
title: Assura config notation, rule composition, and site alignment
status: completed
created: 2026-07-16
owners:
  - assura-maintainers
related:
  - .trellis/tasks/07-08-assura-dev-landing-experience/prd.md
  - .trellis/spec/assura/config-notation.md
  - .trellis/spec/assura/pattern-scoped-file-directives.md
  - .trellis/spec/assura/structure-enforcement.md
  - website/src/data/config-examples/agentic-monorepo.yml
  - website/src/content/docs/reference/configuration.md
  - website/src/content/docs/reference/rules.md
---

# Assura Config Notation, Rule Composition, and Site Alignment

## Delivery Program

Option A is the accepted target notation. Implementation is split into ordered
child goals so each semantic layer is executable and measured before a later
layer depends on it:

1. [Grammar decision and baselines](./assura-notation-01-grammar-decision-and-baselines.md)
2. [Root and selector model](./assura-notation-02-root-and-selector-model.md)
3. [Shorthand and typed rule composition](./assura-notation-03-shorthand-and-rule-composition.md)
4. [Naming and file policy](./assura-notation-04-naming-and-file-policy.md)
5. [Structure, cardinality, and repair context](./assura-notation-05-structure-cardinality-and-repair-context.md)
6. [Project-owned agentic recipes](./assura-notation-06-project-owned-agentic-recipes.md)
7. [Migration and diagnostics](./assura-notation-07-migration-and-diagnostics.md)
8. [Executable examples and release proof](./assura-notation-08-executable-examples-and-release-proof.md)

Do not promote the target homepage fixture until all preceding child goals are
complete. Each child records focused tests, repository self-check evidence, and
the unsupported notation intentionally deferred to its successor.

## Progress Log

- 2026-07-18, iteration 1: accepted Option A, selected implicit `structure:`
  root, rejected permanent inherited-extension compatibility, and created the
  ordered child-goal program. Context level: not exposed.
- 2026-07-18, iteration 2: implemented the root/selector model, typed scalar
  composition, reusable tree rebasing, naming alternatives, cardinality,
  severity isolation, repair context, and project-owned recipes.
- 2026-07-18, iteration 3: aligned the canonical homepage fixture, Starlight
  references, onboarding, explicit v1/LS-Lint migration, and build-time YAML
  validation. Added negative coverage for required root policy, misplaced
  `SKILL.md`, and direct root dot-directory limits.
- 2026-07-18, iteration 4: resolved independent notation and visual review
  findings. Formatting, Clippy, the full Rust suite, 54 executable docs
  examples, 52 Playwright checks, static site build, evidence, target-state,
  and docs gates pass with no remaining reviewer blocker.

## Objective

Make Assura's public configuration language concise enough for a landing-page
example, expressive enough for real agentic monorepos, and exact enough that
every displayed example is checked by the product that implements it.

The target experience should demonstrate one reusable top-down policy:

- the project root has agent guidance and an intentionally bounded shape;
- every package that exists under `packages/` must satisfy the same package
  contract, including its own `AGENTS.md`;
- repeated guidance and directory-health constraints are named once and reused;
- one structural glob applies common requirements to every app and package;
- exact, direct-child, and recursive selectors have visible scope;
- a failed rule can provide concise repair context or point an agent to the
  relevant project-owned document or skill.

## User Certainty Bar

A first-time reader should be able to answer these questions from the homepage
example and its linked reference documentation:

1. Which files and directories does each selector target?
2. Is the project open or closed to undeclared paths at this scope?
3. Which requirements are reused, inherited, or locally extended?
4. Does every package require its own `AGENTS.md`?
5. What happens when a rule is applied to the wrong target kind?
6. How does shorthand expand into the full notation?
7. Is the exact example accepted by the current Assura binary?

## Revalidated Current State

Option A is implemented and promoted as the current pre-1.0 notation. The
runtime, docs, and landing site share one executable `agentic-monorepo.yml`
fixture with:

- an implicit `structure:` root and explicit current/direct/recursive selectors;
- left-to-right scalar composition and specificity-based selector precedence;
- typed node and tree rules with target-kind and cycle diagnostics;
- built-in naming shorthand, finite braces, exact alternatives, and regex escape;
- literal and pattern cardinality, child limits, isolated severity, and repair messages;
- project-owned `agentic-core` and `structure-health` recipes materialized by
  explicit `assura init --recipe ...`, `config add-recipe`, and agent onboarding;
- explicit `assura-v1` and LS-Lint migration without permanent `@rule` aliases; and
- build-time parsing and execution of active website and documentation examples.

The default remains open unless a project composes a closed rule. Selected
recipes and their referenced guidance files are copied into the project and
remain editable without runtime catalog access.

## Decisions To Preserve

### Selected policy is project-owned

- Assura may ship a versioned catalog of first-party policy recipes, but the
  catalog is an authoring source for `init` and `config add-recipe`, not an invisible
  runtime dependency.
- Explicit `assura init --recipe ...` selection and agent onboarding materialize
  the broadly applicable `agentic-core` and `structure-health` layers. Language,
  framework, community-file, and monorepo rules remain explicit additions.
- Non-interactive and agent workflows accept the same explicit recipe
  selection; no interactive-only policy state exists.
- Selecting a recipe copies its `rules:`, `structure:`, relevant
  `extensions:`, comments, and repair-document references into
  `.assura/config.yml`. The resulting project checks do not consult a hidden
  catalog at runtime.
- A later Assura release may show a catalog diff or propose an update, but it
  must never silently rewrite a project's owned policy.
- Replace the current hidden `$agentic-project`, `$agents-dir`, and
  `$agent-skill-*` onboarding dependency with materialized project rules once
  the new recipe path has equivalent tests and migration proof.
- When a selected default requires a missing `README.md` or `AGENTS.md`, init
  previews a minimal project-owned starter and creates it only with the user's
  selection or an explicit non-interactive flag. Existing files are preserved.

The default `agentic-core` recipe is intentionally opinionated but
language-agnostic. It requires root `README.md` and `AGENTS.md`, allows an
optional `.agents/skills/` tree, treats every direct child there as a skill,
requires that skill's `SKILL.md`, and closes each skill root to `SKILL.md` plus
the `scripts/`, `references/`, and `assets/` resource directories. This is
stricter than the Agent Skills specification, which permits arbitrary extra
entries, so the generated config and `init` prompt must state that the strict
shape is Assura policy and can be edited or deselected.

The same recipe materializes an `extensions.agent_guidance` policy for the
content checks that structure notation does not express: required `name` and
`description` frontmatter, skill-directory/name agreement, skill discovery
from `AGENTS.md`, concise entrypoints, and progressive disclosure into deeper
resources. Structure rules must not be described as enforcing those content
semantics on their own.

### One primary authored form

- Define rules with plain names under `rules:`.
- Reference rules with `$name`.
- Treat every rule mapping as exactly one kind: either a node constraint or a
  child tree. A node-constraint mapping contains directives; a child-tree
  mapping contains only path selectors.
- Attach a path's own cardinality and its child-tree contract on the path
  value, as in `packages/: exists:1 | $package-tree`. Do not place `exists`,
  `severity`, `message`, or another node directive beside child selectors in
  the same mapping.
- Use `./: $node-rule` inside a tree only when policy applies to the current
  directory node itself. Child selectors remain siblings beneath that tree.
- Order authored trees like a conventional IDE explorer: current-scope policy
  first, concrete directories alphabetically with each subtree expanded in
  place, then concrete files alphabetically, then wildcard and inherited
  policies. Within policy groups, keep direct selectors before recursive ones.
  Apply the same directories-before-files order inside reusable tree rules.
- Make precedence independent of YAML source order. Selector specificity and
  explicit composition determine behavior; reordering equivalent entries for
  readability must not change validation results.
- Prefer scalar shorthand for one simple naming or parameterized directive.
- Use `|` composition only when it removes repeated nested attributes or
  composes reusable rules.
- Preserve expanded YAML mappings as the escape hatch for nested values,
  custom failure guidance, and other complex directives.
- Do not add JSON-like inline maps to public examples.

### Defaults and exceptions are separate from existence

- Put broad naming defaults at the highest applicable scope and inherit them
  downward.
- Make built-in file naming directives segment-aware. After the selector's
  terminal extension is removed, apply `kebab-case`, `snake_case`,
  `camelCase`, or `PascalCase` independently to every dot-separated stem
  segment. `vite.config.ts`, `button.test.tsx`, and `next-env.d.ts` therefore
  need no regex, while `vite.BadConfig.ts`, `button..test.tsx`, and
  `user_menu.ts` still fail under `kebab-case`.
- Keep directory naming single-segment. When a selector contains a fixed
  literal prefix or suffix, validate only the wildcard capture. For
  `./.*/: kebab-case`, validate `github` for `.github/` rather than treating the
  leading selector-owned dot as part of the directory name.
- Let a more-specific compound-extension selector override the generic final
  extension rule when a project needs different policy for `.config.ts`,
  `.test.ts`, or another explicit suffix.
- Support `exact:NAME` as a built-in naming alternative for conventional
  literal stems. `.md: kebab-case | exact:README | exact:AGENTS` is the concise
  regex-free form for allowing only those uppercase Markdown names alongside
  ordinary kebab-case documents.
- Use exact path entries to require files; a naming rule must never imply that
  a matching file exists.
- Treat an exact literal filename selector as an explicit naming exception to
  broader extension defaults. Do not require authors to repeat the same name as
  `naming: regex:^NAME$`. Other compatible inherited constraints still apply.
- Apply the same rule to descendant selectors with a literal basename, such as
  `./**/CLAUDE.md`; wildcard basenames such as `./**/*.md` do not create a
  naming exception.
- Allow SCREAMING_SNAKE_CASE Markdown only through finite project-owned naming
  rules materialized by the selected naming recipe. Do not expose generic
  `SCREAMING_SNAKE_CASE` as an alternative to `kebab-case` for every Markdown
  file.
- `$markdown-name` is naming-only and permits `kebab-case`, `README`, and
  `AGENTS` at any depth.
- `$community-markdown-name` composes `$markdown-name` and adds GitHub community
  names: `CONTRIBUTING`, `CODE_OF_CONDUCT`, `GOVERNANCE`, `SECURITY`, and
  `SUPPORT`. Permit those names only as direct files at root, `docs/`, or
  `.github/`.
- `$root-markdown-name` composes `$community-markdown-name` and adds only
  root-specific project names: `CHANGELOG` and `LICENSE`.
- Path-owned names remain path-specific rather than entering either broad
  allowlist. In particular, permit `SKILL.md` only in configured skill roots,
  and permit `PULL_REQUEST_TEMPLATE.md` only in its GitHub-owned path.
- Harness-specific names such as `CLAUDE.md` are not built-ins or presets. A
  project that uses one must add an explicit user-authored path rule at the
  intended root or descendant scope. The literal selector supplies the naming
  exception, so the rule needs only existence or content constraints.
- Invalid uppercase Markdown diagnostics must tell agents to rename the file to
  kebab-case or move content into an existing appropriate document. They must
  not suggest adding a new uppercase exception or weakening the naming policy.
- Explicit negative examples include `IMPLEMENTATION_NOTES.md`, `TODO.md`,
  `FINAL_SUMMARY.md`, and similarly improvised agent handoff or scratch files.
- Permit conventional dot-directory names only as direct root children through
  a project-owned exact regex, and bound that root pattern to at most ten
  authored matches. Ordinary directories continue to inherit `kebab-case`.
- Remove automatic tool state such as `.git/` and `.assura/`, plus configured
  exclusions, before evaluating the root dot-directory count.

### Reuse should carry meaning

- Create a user-defined leaf rule only when it is referenced at least twice or
  when it carries a meaningful reusable diagnostic contract.
- A tree rule may have one authored selector only when that selector expands to
  repeated runtime matches or marks a materialized recipe boundary. `$skill`
  and `$workspace` are valid dynamic examples; a root-only `$monorepo` wrapper
  is not.
- Keep a complex one-use fragment inline when its name would only hide one
  nearby primitive and it has no independent diagnostic meaning.
- Keep one-off constraints inline at the point where they apply.
- Do not create a rule solely to move a one-off section out of `structure:`.
- Treat a one-use leaf alias around one primitive as an advisory config smell;
  suggest inlining it. Exempt entry policies, dynamic tree contracts,
  diagnostic bundles with custom guidance, materialized recipe boundaries,
  and generated migration internals.
- Keep selectors at the use site when the rule is a reusable node constraint.
- Put relative selectors inside a tree rule when the hierarchy itself is the
  reusable contract.

### Open by default

- Keep the engine default open for incremental adoption and LS-Lint migration.
- Do not add `forbid`, `contents: declared`, `structure!`, or another parallel
  enforcement mechanism.
- Make checked and unchecked coverage visible through `assura explain`
  and agent-formatted diagnostics.
- Keep the engine open by default, but let the project-owned agentic recipe
  define a reusable `$closed` tree rule from the existing `exists:0`
  directive. Apply it only to `.agents/` and skill roots, where an exact
  direct-content contract is intentional. More-specific declared children
  override the broad direct-child deny selectors.

### Invalid composition is an error

Configuration loading must reject, with source-aware diagnostics:

- a mapping that mixes node directives and child path selectors;
- a tree rule applied to a file selector;
- a file-only directive such as `max_lines` applied to a directory;
- a directory-self rule applied to a file; and
- pipelines that combine incompatible target kinds.

Warnings remain appropriate for unmatched, unused, shadowed, or intentionally
open coverage, not for impossible rule applications.

## Proposed Selector Model

Use one path-shaped grammar, relative to the current structure or rule anchor:

| Selector | Meaning |
| --- | --- |
| `./` | The current directory node. |
| `./*` | Direct child files. |
| `./*/` | Direct child directories. |
| `./.*/` | Direct child dot directories. |
| `./**/*` | Files at any descendant depth. |
| `./**/*/` | Directories at any descendant depth. |
| `./**/*.{ts,tsx}` | Matching descendant files. |
| `packages/*/` | Direct package directories below this anchor. |
| `AGENTS.md` | One exact child file. |

Keep `./` on relative wildcard selectors. It is not another root node: it
means "from the current structure or rule anchor," distinguishes a rebased
selector from an absolute-looking project path, and keeps wildcard keys valid
as unquoted YAML. For example, `./*/` is the readable unquoted form of the YAML
key `"*/"`.

Selector specificity, not YAML order, determines policy layering. Exact paths
refine matching globs, and narrower globs refine broader globs without forcing
the shared policy to be restated. Composition merges compatible directives;
explicit conflicts must fail or be resolved by a documented local override
rather than accidental map order.

A finite brace-only selector is compile-time shorthand for independent exact
selectors. `pnpm-{lock,workspace}.yaml: exists:1` requires each file, while
`./{assets,references,scripts}/: exists:0-1` makes each directory independently
optional. Wildcard and capture selectors retain aggregate cardinality over
their runtime matches; `./.*/: exists:0-10` limits the combined direct dot
directories. This distinction must be visible in `assura explain`.

## What The LS-Lint Prototype Got Right

The reviewed `rothnic/ls-lint` prototype uses a short `groups:` namespace,
compact `@name` references, group composition, and ordinary path entries at the
use site. Its strongest idea is not the exact punctuation. It separates:

- **what repeats**, which belongs in a small named group; from
- **where it applies**, which remains visible in the project-shaped tree.

The Assura homepage example should preserve that distinction. Do not compress
the whole tree into one brace-heavy glob just to remove lines. Combine a finite
set such as the two required pnpm YAML files when the selector remains obvious,
but keep `apps/` and `packages/` as visible nested scopes. Their repeated child
policy is where reusable rules demonstrate value.

## Proposed Rule Expansion

A node rule carries constraints but no path:

```yaml
rules:
  source-file: max_lines:500

structure:
  ./:
    ./src/**/*.{ts,tsx}: $source-file
    ./tests/**/*.{ts,tsx}: $source-file
```

A reusable tree rule carries relative child policy and is rebased wherever a
matching directory references it. This is equivalent to copying those child
entries under each matched directory, but keeps repeated workspace policy in
one place:

```yaml
rules:
  workspace:
    AGENTS.md: exists:1
    package.json: exists:1

structure:
  apps/:
    ./*/: kebab-case | $workspace
  packages/:
    ./*/: kebab-case | $workspace
```

The config-quality analyzer counts literal `$name` use sites in the authored
config before expansion; runtime glob matches do not inflate reuse. A
user-defined rule referenced zero or one time produces a
low-severity suggestion to inline it or add the intended second use. Built-ins
and generated migration internals are excluded from that authoring advisory. A
single reference from another rule still counts as one authored use even when
the outer rule later matches many paths.

## Research Basis For The Example

The homepage example uses a pnpm/Turborepo-style monorepo because its shape is
familiar and independently documented:

- [Turborepo recommends `apps/*` and `packages/*`](https://turborepo.dev/docs/crafting-your-repository/structuring-a-repository),
  requires a root manifest and lockfile, and requires `package.json` in every
  workspace package.
- [The maintained Turborepo basic starter](https://github.com/vercel/turborepo/tree/main/examples/basic)
  includes applications plus source and tooling packages, so not every package
  should be forced to contain `src/`.
- [Nx recommends grouping projects by scope](https://nx.dev/docs/concepts/decisions/folder-structure),
  which supports an advisory flat-directory signal rather than an automatic
  blocking refactor.
- [GitHub documents nearest-path `AGENTS.md` precedence](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions-in-your-ide/add-repository-instructions-in-your-ide),
  which supports package-local guidance for agents working in a monorepo.

The example therefore requires only stable workspace invariants and leaves
framework-specific files open. Its selected JavaScript/TypeScript policy
applies one naming and advisory line limit at every depth, while generated
dependency/build/coverage trees are excluded. The structure-health layer warns
when the root or an authored descendant directory exceeds 10 direct children.
The value 10 is an explicit early-warning product choice, not a claim sourced
from Nx; automatic tool state and configured exclusions must be removed before
Assura counts children.

## Complete Notation Comparison

The complete IDE-ordered comparison is maintained in
[Config Notation Options Comparison](../analysis/2026-07-17-config-notation-options-comparison.md).
It holds one policy constant across the explicit `./**/` scope bundle,
inherited extension shorthand, and the closest valid LS-Lint 2.3 policy.
Option A is the preferred homepage direction pending executable normalization
and duplicate-finding proof.

## Homepage Config Proposal

This is the target structure excerpt to implement and then promote into the
canonical website fixture. It demonstrates the three default project-owned
layers: agentic structure, naming hygiene, and advisory structure health. The
complete fixture also contains the explicit `extensions.agent_guidance` fields
materialized by `agentic-core`; the homepage should label this block as the
structure excerpt and link to the checked full config rather than implying that
the excerpt alone validates document contents.

The 160-line AGENTS.md threshold keeps the root guidance file useful as a
router. The 500-line SKILL.md threshold follows the Agent Skills progressive
disclosure recommendation. Both are advisory with project-owned repair
guidance. The stricter skill-directory allowlist is an Assura policy choice,
not an Agent Skills specification requirement.

This particular use-case fixture is a JavaScript/TypeScript monorepo, so it
also shows selected source naming/health policy and a workspace add-on. Those
lines are not part of the universal default initialization profile.

Use Option A as the canonical checked homepage source. Its **71 total lines**
must not appear as one mobile wall. Render four segmented views from the same
source: `Agent files`, `Skill layout`, `Naming and health`, and
`Workspace reuse`. Each view stays at or below 24 visible lines, and the full
checked file remains available from one secondary link. Do not use a draggable
comparison handle. Policy lines should stay at or below 44 characters where
practical.

### Why this example earns its space

- Root-only agentic and monorepo requirements stay directly under
  `structure:` instead of being hidden behind one-use `$agentic-project` and
  `$monorepo` wrappers.
- `$closed-entry` wraps the existing `exists:0` directive with project-owned
  repair guidance. `$closed` applies it to direct files and directories at
  `.agents/` and every skill root instead of introducing a parallel `forbid`,
  `strict`, or `contents` mechanism.
- `.agents/`, `skills/`, and `apps/` explicitly override exact-path required
  defaults with `./: exists:0-1`. `packages/` remains required by its exact
  literal mapping. Every matched direct skill directory receives the reusable
  `$skill` contract, so one selector fans out across all project skills.
- `$agent-entrypoint` and `$skill-entrypoint` carry bounded, rule-specific
  repair context for agent output. `$agent-entrypoint` is authored at root and
  workspace scope; `$skill-entrypoint` fans out to every matched skill. Their
  low severity does not weaken the blocking `exists:1` requirement composed
  beside them.
- `$workspace` reuses the agent guide for every existing direct app or package,
  so local agent context follows the code an agent is editing without copying
  the package contract beneath both containers.
- The root pnpm/Turborepo invariants remain inline. This use case requires
  `packages/`, allows `apps/`, and applies `$workspace` inside both IDE-shaped
  directory branches instead of hiding their hierarchy in one brace selector.
- One `./**/` directory-scope bundle applies naming and advisory file-health
  defaults once per directory. Naming stays on separate selectors, so a
  low-severity health constraint cannot weaken blocking naming policy.
- `exact:README` and `exact:AGENTS` make the two recognized uppercase Markdown
  stems explicit without a custom regex or a repeated health rule.
- Built-in `kebab-case` validates every dot-separated source stem segment, so
  `vite.config.ts`, `button.test.tsx`, `widget.stories.tsx`, and declaration
  files work without a regex or growing suffix allowlist. Uppercase,
  underscores, empty segments, and repeated separators still fail.
- `$folder-health` is a distinct advisory diagnostic applied to descendant
  directories. It warns at 11 direct authored children without changing
  existence or naming results. Its scalar `limit_children: 10` is the concise
  form of `limit_children: { max: 10 }`.
- `./.*/` visibly means direct root dot directories and applies `kebab-case` to
  the captured name after the fixed dot. The explicit anchor avoids confusing
  it with `./*/`, which means every direct directory.
- The full checked fixture owns the progressive-disclosure extension settings,
  while this excerpt remains readable enough to explain the structural model.

## LS-Lint Comparison Boundary

The capability comparison should show the closest valid LS-Lint 2.3 policy for
the same naming and scope concerns, including repeated package declarations
where a more-specific package scope replaces parent policy. It must state that
LS-Lint does not express Assura-only capabilities such as:

- reusable composed tree contracts;
- a language-agnostic file-line threshold;
- per-package required `AGENTS.md` with reusable repair guidance; or
- rule-specific feedback suitable for an editing agent.

Do not include those Assura-only checks in timed LS-Lint parity claims. Keep the
capability example and equivalent-work performance suite visually and
semantically separate.

LS-Lint's `exists:N-M` counts entries matched by one extension or directory
rule. Assura preserves that model for selector-scoped cardinality such as
`.md: exists:1-10` or `./*/: exists:0-10`. The homepage's
`limit_children: 10` is intentionally different: it counts all direct files
and directories as one aggregate budget. Expressing the closest LS-Lint policy
requires separate file and directory rules, which can permit more than ten
combined children. Do not present selector-scoped `exists` and aggregate
`limit_children` as equivalent constraints.

## Implementation Workstreams

### 1. Finalize the notation contract

- Amend the Assura notation specs with the selector table, precedence,
  composition, rebasing, and target-kind matrix.
- Confirm bare `kebab-case` remains the preferred naming shorthand; `$` remains
  reserved for reusable rules.
- Specify cascading extension sets so `.{ts,tsx}` expands into the same indexed
  suffix rules as separate `.ts` and `.tsx` entries, with no broad-glob scan.
- Specify `exact:NAME` as a naming alternative over the filename stem selected
  by the surrounding extension rule. Reject empty values and path separators;
  keep path placement and file existence separate.
- Preserve the existing naming-alternative meaning of `|`. Lex exact ` | `
  separators only at the top level, outside escaped text and balanced regex
  groups. If every top-level token is a naming case or `regex:` value, the
  scalar remains naming alternatives; otherwise every token must be a known
  directive or `$rule` composition. Ambiguous ungrouped regex uses expanded
  mapping instead of guessing.
- Specify scalar, pipeline, and expanded mapping equivalence.
- Specify composition of a directory naming directive and a rebased tree rule,
  as in `./*/: kebab-case | $workspace`.
- Reuse the existing `limit_children`, severity, and message primitives rather
  than adding a second directory-pressure directive.
- Specify how custom failure feedback attaches in expanded form and how agent
  output includes its bounded repair link or instruction.
- Specify `$closed` as an ordinary project-authored tree fragment using
  `./*: exists:0` and `./*/: exists:0`. Exact declared children override those
  defaults by selector specificity; the engine does not gain a second strict
  mode.
- Specify recipe materialization independently from runtime rule resolution.
  A generated config must be self-contained after `init` exits.

### 2. Implement normalization and type checking

- Parse pipe composition without splitting regex bodies.
- Normalize scalar directives, node rules, and tree rules into one compiled
  internal policy representation.
- Normalize the matched filename into dot-separated stem segments after
  removing the selector's terminal extension. Run built-in naming checks over
  those segments without compiling or executing a regex.
- Preserve selector capture boundaries for directory naming. Fixed selector
  punctuation such as the leading dot in `.*/` is not part of the captured
  directory name presented to `kebab-case`.
- Rebase relative selectors in tree rules onto each matched directory and
  support composing the matched directory's naming with that tree rule.
- Expand finite brace-only selector sets into independent exact selectors
  during normalization. `pnpm-{lock,workspace}.yaml: exists:1` requires both
  named files, and unrelated `pnpm-*.yaml` files cannot satisfy either member.
  `./{assets,references,scripts}/: exists:0-1` makes each named directory
  optional, allows all three to coexist, and does not allow another skill-root
  directory. Preserve aggregate cardinality for wildcard and capture matches.
- Support bounded cardinality on direct pattern scopes. For
  `./.*/: kebab-case | exists:0-10`, count only matching direct root
  directories after automatic and configured exclusions, and reject an
  eleventh authored dot directory without scanning descendants.
- Normalize `limit_children: 10` to the existing `{ max: 10 }` model.
- Reject incompatible target kinds before traversal.
- Preserve rule provenance for `assura explain` and agent feedback.
- Count explicit user-rule references from the authored config AST and emit a
  low-severity config-quality finding for zero- or one-use rules.
- Keep expansion and matcher compilation outside the per-entry hot path.
- Round-trip the feature through compiled config artifacts and bump the portable
  config schema when the serialized normalized shape changes.
- Replace runtime-only agentic built-ins in generated onboarding configs with
  materialized project-owned rules after parity is proven.
- Add interactive and non-interactive recipe selection to `assura init`, plus
  an additive command for copying selected recipes into an existing config.
  Both paths must preview conflicts and preserve existing user-authored rules.
- Materialize the selected recipe's `extensions.agent_guidance` block together
  with its structure rules; do not hide progressive-disclosure checks behind a
  tree-rule name.

### 3. Prove behavior with focused fixtures

- Add shorthand-to-expanded equivalence tests.
- Prove `exact:README` and `exact:AGENTS` compose as alternatives to
  `kebab-case`, remain scoped to Markdown stems, and do not make either file
  exist or permit another uppercase Markdown name.
- Add extension-set equivalence, inheritance, specificity, and performance-path
  tests for `.{ts,tsx}` versus separate `.ts` and `.tsx` entries.
- Add segment-aware built-in naming tests for plain, compound, declaration,
  test, story, and config filenames. Verify valid lowercase segments pass and
  uppercase, underscore, empty, leading-dot, trailing-dot, and repeated-dot
  segments fail without invoking the regex engine.
- Prove an explicit `.config.ts` or `.test.ts` selector can override the
  generic `.ts` policy without changing unrelated compound filenames.
- Prove nested Markdown accepts kebab-case, `README`, and `AGENTS`; direct files
  at root, `docs/`, and `.github/` accept the enumerated GitHub community names;
  only root accepts `CHANGELOG` and `LICENSE`; deeper misplaced community names
  and arbitrary SCREAMING_SNAKE_CASE names fail; and no naming rule makes any
  special file required by itself.
- Assert agent-formatted feedback for improvised uppercase notes recommends a
  concrete kebab-case rename or consolidation target and never recommends a
  config exception.
- Prove `SKILL.md` and `PULL_REQUEST_TEMPLATE.md` are accepted only in
  configured owning paths, and harness instruction files remain invalid until
  the user authors an explicit path-scoped exception.
- Prove exact `AGENTS.md: exists:1` entries require root and workspace guidance
  while preserving the inherited Markdown naming policy.
- Add exact, direct-child, and recursive selector tests for files and folders.
- Prove exact literal filenames and recursive selectors with literal basenames
  override broad extension naming without a duplicate regex, while wildcard
  basenames continue to inherit the extension naming rule.
- Add finite brace-set expansion tests where both pnpm files pass, either
  missing file fails, all three optional skill-resource directories coexist,
  and an unrelated path cannot satisfy a named member.
- Add root dot-directory tests for zero, ten, and eleven authored matches;
  confirm `.git/`, `.assura/`, and configured exclusions do not consume the
  limit; and prove nested dot directories do not match the root-only exception.
- Add tree-rule rebasing tests at root and nested package scopes.
- Prove `packages/: exists:1 | $package-tree` applies cardinality to the
  matched directory while rebasing the tree rule onto that directory's
  children.
- Prove the IDE-shaped nested equivalent under `packages/:`, with
  `./: exists:1` before `./*/: $workspace`, requires `packages/` itself and
  rebases `$workspace` onto every existing direct child without treating `./`
  as another child.
- Reject mixed mappings such as `exists: 0-1` beside `skills/: ...` with a
  source-aware diagnostic that points to the scalar-composition form.
- Prove a directory can compose one tree rule and one directory-node rule, as
  in `packages/: exists:1 | $package-tree`, without leaking severity or
  changing child selectors.
- Add composition, precedence, conflict, regex-pipe, and invalid-target tests.
- Prove configs with the same entries in different YAML source orders compile
  to equivalent precedence and produce identical findings.
- Preserve `regex:^(foo|bar)$ | kebab-case` as two naming alternatives, keep
  grouped regex alternation inside one token, and parse `exists:1 | $agent-doc`
  as directive composition.
- Prove homepage `kebab-case` accepts `vite.config.ts`, `next-env.d.ts`,
  `button.test.tsx`, and `widget.stories.tsx`, while rejecting uppercase,
  underscores, empty segments, and repeated separators.
- Add a passing monorepo fixture with multiple packages.
- Add a failing package fixture where one existing package lacks `AGENTS.md`.
- Add custom repair-message proof that points an agent to project-owned docs.
- Verify the 160-line AGENTS.md limit and its progressive-disclosure guidance
  remain aligned with the generated agent-guidance preset.
- Verify a missing package `AGENTS.md` blocks while an oversized one reports a
  low-severity warning and exits successfully when no other gate fails.
- Add an 11-direct-child fixture that emits one low-severity, non-blocking
  finding with the configured project-structure repair link.
- Add a combined fixture where an 11-child package missing `AGENTS.md` emits one
  advisory child-count finding and one blocking required-file finding.
- Filter configured exclusions before direct-child counting; ten authored
  children plus excluded `node_modules/` must remain below the threshold.
- Add config-quality fixtures for unused, single-use, and genuinely reused
  user rules, including semantic entry-policy, dynamic-contract,
  diagnostic-bundle, and generated-rule exemptions.
- Prove `agentic-core` requires root `README.md` and `AGENTS.md`, allows the
  `.agents/` tree to be absent, and validates every existing direct skill.
- Prove a skill without `SKILL.md`, a non-kebab skill name, or an undeclared
  direct file/directory fails; `scripts/`, `references/`, and `assets/` pass.
- Prove the materialized agent-guidance policy validates required `name` and
  `description` frontmatter, name/directory agreement, skill discovery from
  AGENTS.md, and the configured progressive-disclosure thresholds.
- Prove `init` output contains all selected rule and extension definitions and
  produces the same check result with the recipe catalog unavailable.
- Prove deselected recipes add no policy, existing configs are not silently
  overwritten, and later catalog changes do not alter owned project policy.
- Prove root files, config-only packages, and excluded generated trees do not
  trigger false positive child-count or source-line findings.
- Verify a 501-line authored source file is advisory, while unrelated blocking
  structure findings retain their own severity.
- Prove the shared extension-set health selector composes with the narrower
  Markdown and source naming selectors exactly once per file, with no duplicate
  finding and no severity leakage into naming.
- Run the full real-project check to catch inheritance bugs missed by focused
  parser tests.

### 4. Align docs and site

- Make `website/src/data/config-examples/agentic-monorepo.yml` the executable
  source for the homepage config and rendered tree.
- Add a dedicated path-targeting reference with the complete selector table.
- Document rule kinds, rebasing, composition, precedence, and invalid targets.
- Document segment-aware built-in file naming, selector-owned literal prefixes,
  and compound-extension override precedence before presenting regex as the
  advanced escape hatch.
- Show scalar shorthand beside its expanded mapping equivalent.
- Add regex naming examples, including alternation and anchoring.
- Document `exact:NAME` before regex as the preferred concise alternative for
  a finite set of literal stems.
- Document the exact expansion and composition of `$markdown-name`,
  `$community-markdown-name`, and `$root-markdown-name`, plus path-owned
  exceptions for agent skills and GitHub templates. Show user-authored examples
  for harness files without adding them to Assura defaults or presets.
- Explain GitHub's root, `.github/`, and `docs/` community-file locations and
  show how the direct-scope selector prevents those uppercase names from
  becoming valid throughout arbitrary documentation subtrees.
- Explain the open default, checked versus unchecked coverage, captured
  workspace scopes, and the single-use-rule advisory.
- Document custom failure feedback and how hooks deliver it to agents.
- Add `docs/structure.md` to the canonical example project with concise grouping
  and exception guidance; fixture validation must reject unresolved repair-doc
  references.
- Replace stale syntax across docs, blog, examples, tests, and site fixtures in
  the same change; do not carry pre-1.0 notation aliases.
- Render the capability comparison separately from performance evidence.
- Add a first-party recipe reference that shows exactly what each selectable
  recipe copies, which layers are recommended by default, and how to edit or
  remove them after initialization.
- Document that the strict skill-root allowlist is Assura's opinionated policy;
  the Agent Skills specification itself permits additional entries.
- Replace docs that describe `$agentic-project` as hidden built-in behavior
  with the project-owned materialized form and migration guidance.

### 5. Verify visual communication

- Keep each mobile config view at or below 24 visible lines and derive all
  views from one checked full fixture.
- Use a segmented control or explicit tabs for the four views; do not rely on
  a drag handle or side-by-side code at mobile widths.
- Use comments only to mark conceptual sections, not narrate every directive.
- Give selectors, rule references, primitives, and comments consistent syntax
  roles; color must not imply different directory semantics.
- Keep the config and applied project tree aligned from one fixture model.
- Verify mobile widths at 320, 375, and 430 CSS pixels plus desktop widths at
  768 and 1440 pixels.
- Check code wrapping, horizontal scrolling, touch interactions, sticky header
  overlap, and light/dark contrast.

### 6. Protect performance

- Capture parser, normalization, cold check, and persistent-session baselines
  before implementation.
- Compile rule expansion, selector specificity, and matcher metadata once per
  configuration hash.
- Keep the existing accepted LS-Lint-equivalent cold rows no slower.
- Reject a repeatable parser or normalization regression above 2% on the same
  fixture and host unless the goal records and approves the measured tradeoff.
- Re-run the warm multi-agent-loop suite to prove composition does not add
  repeated filesystem or regex compilation work.
- Benchmark segment-aware built-in naming separately and require it to avoid
  regex compilation, allocation per segment, and extra filesystem traversal.

## Measurable Outcomes

- 100% of homepage and reference-page config examples are parsed by the release
  binary during `cargo xtask docs` or an equivalent required build gate.
- 100% of project-owned repair-document references in canonical examples resolve
  inside their fixture project.
- A freshly initialized default project contains editable YAML for every
  selected recipe and can validate with no runtime access to the recipe catalog.
- Default interactive selection clearly recommends `agentic-core` and
  `structure-health`; detected naming policy is separately confirmed. Opting
  out of each offered layer is covered by a CLI test.
- Root `README.md` and `AGENTS.md` are blocking requirements. AGENTS.md over
  160 lines and SKILL.md over 500 lines produce advisory repair context without
  weakening those existence gates.
- Every skill directory has `SKILL.md`, valid required frontmatter, and no
  undeclared direct child outside `scripts/`, `references/`, and `assets/`.
- The canonical homepage fixture passes; its paired negative fixture fails for
  the expected missing package `AGENTS.md` finding.
- Every selector in the proposed selector table has at least one positive and
  one negative scope test.
- Every public shorthand form has an expanded-form equivalence test.
- Trivial one-use leaf aliases emit one low-severity config-quality finding;
  named entry policies, dynamic tree contracts, and diagnostic bundles do not.
- An 11-child authored directory emits one advisory finding, while excluded
  generated children do not contribute to the count.
- Invalid node/tree/file/directory compositions fail at config load with the
  rule name, selector, expected target kind, and remediation.
- `assura explain` shows the source rule, rebased effective selector, winning
  precedence, and checked/unchecked status for the homepage fixture.
- The canonical excerpt stays at or below 72 lines; each mobile view exposes no
  more than 24 lines at once and has no horizontal page overflow at the
  required breakpoints.
- Accepted equivalent-work cold benchmark rows remain no slower than LS-Lint;
  the warm session continues to meet the existing 2x target.

## Non-Goals

- Changing the engine default from open to closed.
- Adding a second structural enforcement path.
- Copying prototype LS-Lint notation directly.
- Treating Assura-only checks as equivalent LS-Lint benchmark work.
- Imposing a universal 200-line or 500-line limit on every file type.
- Preserving superseded pre-1.0 syntax through compatibility aliases.

## Definition of Done

- [x] Governing notation specs describe the final grammar and semantics.
- [x] The parser and normalizer support the complete homepage proposal.
- [x] Rule expansion and target-kind errors have focused regression coverage.
- [x] Every existing package matched by `packages/*/` requires `AGENTS.md`.
- [x] `agentic-core`, naming, health, and extension policy are materialized as
      editable project YAML by explicit initialization and agent onboarding.
- [x] Trivial one-use aliases are reported without flagging semantic entry or
      dynamic-contract rules.
- [x] Every existing skill has valid `SKILL.md`, allowed direct resources, and
      project-owned progressive-disclosure checks.
- [x] Child-limit severity remains isolated from blocking existence policy.
- [x] Custom failure guidance reaches bounded agent-formatted output.
- [x] Homepage, docs, examples, tests, and fixtures use one canonical notation.
- [x] The canonical fixture and rendered project tree come from checked data.
- [x] Mobile and desktop screenshots pass the visual verification checklist.
- [x] Cold and warm benchmark gates pass without capability inflation.
- [x] An independent notation reviewer finds no blocking ambiguity or invalid
      example.
- [x] An independent visual reviewer finds no blocking scanability, overflow,
      or light/dark inconsistency.

## Validation Commands

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test structure_config_notation_tests --quiet
cargo test --test ls_lint_rule_coverage_tests --quiet
cargo test -p assura-check-cli --test compiled_config_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask check
cargo xtask test
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
mkdir -p target/performance
target/release/assura performance-report --output target/performance/ls-lint.json --iterations 16
cargo xtask performance-no-slower target/performance/ls-lint.json
target/release/assura performance-report --suite native --output target/performance/native.json --history target/performance/native.jsonl --iterations 5
cargo xtask native-performance-no-regression target/performance/native.json
cargo xtask warm-loop-benchmark --binary target/release/assura-full --iterations 20 --output target/performance/warm.json
cargo xtask warm-loop-no-regression target/performance/warm.json
cargo xtask evidence
cargo xtask target-state
cargo xtask docs
pnpm --dir website test:marketing
pnpm --dir website build
git diff --check
```

Long Cargo integration filters should run sequentially to avoid lock contention.
Use the repository's current exact test target names if they change while this
goal is being implemented.

## Reviewer Blocking Criteria

A reviewer should block completion when:

- any displayed config is not accepted by the current release binary;
- the homepage implies that package `AGENTS.md` is required but a matching
  package can omit it;
- rule expansion changes with YAML key order;
- regex alternation is misparsed as a composition pipe;
- a tree or directory rule can silently apply to a file;
- a trivial one-use leaf alias is retained without a config-quality finding,
  or a semantic entry/dynamic rule is incorrectly flagged only for one use;
- selected policy exists only in Rust or a remote catalog after initialization;
- the homepage claims the tree rule alone checks AGENTS.md/SKILL.md contents;
- a low-severity child limit makes a required-file violation advisory;
- excluded/generated children contribute to the direct-child threshold;
- docs omit a selector's current/direct/recursive or file/directory scope;
- the capability comparison is presented as a timed LS-Lint equivalent;
- the implementation performs rule expansion or regex compilation per file;
- the site fixture, rendered tree, and docs example can drift independently;
  or
- mobile/light-mode output is clipped, ambiguous, or horizontally overflows.

## Delivery Order

1. Approve the grammar, type matrix, and homepage target example.
2. Capture current parser and validation performance baselines.
3. Implement normalization, composition, scope targeting, and diagnostics.
4. Add positive, negative, equivalence, and real-project tests.
5. Migrate all docs and executable site fixtures atomically.
6. Verify performance, responsive screenshots, light/dark mode, and docs routes.
7. Obtain independent notation and visual reviews, then resolve valid findings.
