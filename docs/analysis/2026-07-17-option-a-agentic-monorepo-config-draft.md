---
title: Option A agentic monorepo config draft
status: implemented
---

# Option A: Agentic Monorepo Config

This is the shareable source for Assura's explicit recursive-scope notation.
The executable website fixture is
`website/src/data/config-examples/agentic-monorepo.yml`; build validation parses
and executes that file.

## Proposed Config

```yaml
rules:
  # Progressive-disclosure entrypoints.
  agent-entrypoint:
    max_lines: 160
    severity: low
    message: See docs/agent-guidance.md.

  skill-entrypoint:
    max_lines: 500
    markdown:
      require_frontmatter: true
    message: See docs/agent-guidance.md#skills.

  # Shared non-blocking directory health.
  folder-health:
    limit_children: 10
    severity: low
    message: See docs/structure.md.

  # Closed-directory building blocks.
  closed-entry:
    exists: 0
    message: See docs/agent-guidance.md#layout.

  closed:
    ./*/: $closed-entry
    ./*: $closed-entry
  # Repeated directory contracts.
  skill:
    ./: $closed
    ./{agents,assets,references,scripts}/:
      ./: exists:0-1
      inherit: false
    SKILL.md: exists:1 | $skill-entrypoint

  workspace:
    AGENTS.md: exists:1 | $agent-entrypoint
    package.json: exists:1

structure:
  # Directories, as shown by an IDE explorer.
  .agents/:
    ./: exists:0-1 | $closed
    skills/:
      ./: exists:0-1
      ./*/: kebab-case | $skill
      ./*: $closed-entry
  apps/:
    ./: exists:0-1
    ./*/: $workspace
  packages/:
    ./*/: $workspace

  # Required root files, alphabetically.
  AGENTS.md: exists:1 | $agent-entrypoint
  package.json: exists:1
  pnpm-{lock,workspace}.yaml: exists:1
  README.md: exists:1
  turbo.json: exists:1

  # Direct root dot directories.
  ./.*/: kebab-case | exists:0-10

  # Defaults inside every directory, including root.
  ./**/:
    ./: $folder-health
    ./*/: kebab-case
    .{md,js,jsx,ts,tsx}: max_lines:500 | severity:low
    .md: kebab-case | exact:AGENTS | exact:README
    .{js,jsx,ts,tsx}: kebab-case

exclude: ["**/{node_modules,.next,.turbo,dist,coverage}/**"]
```

## Intended Result

- Root `README.md`, `AGENTS.md`, `package.json`, both pnpm YAML files, and
  `turbo.json` are required.
- `.agents/` and `apps/` are optional; `packages/` is required.
- Every direct app or package directory must contain `AGENTS.md` and
  `package.json`.
- Every direct `.agents/skills/` directory is kebab-case, requires `SKILL.md`,
  and allows only `agents/`, `assets/`, `references/`, and `scripts/` beside it.
- Agent guidance over 160 lines is advisory. Missing skill entrypoints,
  missing skill frontmatter, and skill entrypoints over 500 lines are blocking
  outer bounds with project-owned repair links.
- Markdown, JavaScript, and TypeScript files share a 500-line advisory budget.
- Markdown names are kebab-case except for `AGENTS.md` and `README.md`.
- JavaScript and TypeScript compound stems are kebab-case.
- Direct root dot directories are kebab-case and limited to ten authored
  entries after automatic and configured exclusions.
- Every authored directory warns after ten combined direct files and folders.

## Scope Contract

| Selector | Meaning |
| --- | --- |
| `./` | The current matched directory node. |
| `./*` | Direct child files. |
| `./*/` | Direct child directories. |
| `./.*/` | Direct child dot directories. |
| `./**/` | Current directory and every descendant directory. |
| `.md` under `./**/` | Direct Markdown files in each matched directory. |

The `./**/` bundle is rebased onto each matched directory exactly once. Its
nested selectors are direct relative selectors, not independently inherited
recursive rules. More-specific exact and scoped selectors refine the broad
bundle without depending on YAML source order.

Exact literal paths are required by default. An exact directory carrying
`./: exists:0-1` explicitly overrides that default and applies its child policy
only when the directory exists. Glob and capture selectors remain match-only
unless they carry explicit cardinality.

## Implementation Gates

1. Parse and normalize the complete YAML without compatibility fallback.
2. Prove `./**/` includes root and each descendant directory once.
3. Prove the nested extension rules emit no duplicate findings.
4. Prove exact-path specificity overrides `$closed` wildcard denials.
5. Prove optional `.agents/` and `apps/` do not produce missing-path findings.
6. Prove missing `packages/`, root files, workspace files, and `SKILL.md` fail.
7. Prove advisory limits do not weaken blocking naming or existence policy.
8. Compile brace sets and matchers once, outside per-entry traversal.
9. Validate direct and compiled-config execution produce identical results.
10. Render the checked source on desktop and mobile without wrapping or
    horizontal overflow.

## Related Decision Material

- [Complete notation options comparison](./2026-07-17-config-notation-options-comparison.md)
- [Config notation and site alignment goal](../goals/assura-config-notation-rule-composition-and-site-alignment.md)
