---
title: Config notation options comparison
status: active
---

# Config Notation Options Comparison

This comparison holds one agentic-monorepo policy constant across two proposed
Assura forms and the closest valid LS-Lint 2.3 approximation. Option A is the
implemented notation and executable website fixture; Option B remains a
comparison-only alternative.

LS-Lint global extension rules are concise, but a directory-specific
configuration replaces the inherited rule set for that scope. LS-Lint does not
express Assura's language-agnostic line budgets, aggregate direct-child
warning, reusable repair guidance, or reliable exact root/workspace file
requirements without narrowing an entire extension count.

## Option A: One Explicit Recursive Directory Scope

`./**/` matches the current directory and every descendant directory. Its
nested selectors are direct relative selectors evaluated once per matched
directory. Recursive reach is visible once without repeating the full glob on
every file policy. The focused shareable version is maintained in
[Option A: Agentic Monorepo Config](./2026-07-17-option-a-agentic-monorepo-config-draft.md).

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

## Option B: Inherited Extension Shorthand

Extension selectors authored at a scope inherit through its descendants.
Recursive directory policy remains explicit because it targets directory
nodes rather than file extensions.

```yaml
rules:
  # Progressive-disclosure entrypoints.
  agent-entrypoint:
    max_lines: 160
    severity: low
    message: See docs/agent-guidance.md.

  skill-entrypoint:
    max_lines: 500
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
    ./{assets,references,scripts}/: exists:0-1
    SKILL.md: exists:1 | $skill-entrypoint

  workspace:
    AGENTS.md: exists:1 | $agent-entrypoint
    package.json: exists:1

structure:
  # Current project directory.
  ./: $folder-health

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

  # Inherited file defaults.
  .{md,js,jsx,ts,tsx}: max_lines:500 | severity:low
  .md: kebab-case | exact:AGENTS | exact:README
  .{js,jsx,ts,tsx}: kebab-case

  # Direct root and recursive directory defaults.
  ./.*/: kebab-case | exists:0-10
  ./**/*/: kebab-case | $folder-health

exclude:
  - "**/{node_modules,.next,.turbo,dist,coverage}/**"
```

## Option C: Closest Valid LS-Lint 2.3 Policy

LS-Lint uses global extension defaults and replaces them inside more-specific
directory scopes. Source naming therefore repeats inside the combined
app/package scope. Brace scopes reduce some duplication but do not provide
named policy composition or agent-facing repair context.

```yaml
ls:
  # Directories, as shown by an IDE explorer.
  .agents:
    skills:
      .dir: kebab-case
      "*":
        .dir: kebab-case
        "{assets,references,scripts}":
          .dir: kebab-case
        "*":
          .dir: exists:0
        .*: exists:0
        .md: regex:SKILL | exists:1

  "{apps,packages}":
    .dir: kebab-case
    "*":
      .dir: kebab-case
      .js: kebab-case
      .jsx: kebab-case
      .md: kebab-case | regex:(AGENTS|README)
      .ts: kebab-case
      .tsx: kebab-case

  # Global directory and file naming.
  .dir: kebab-case
  .js: kebab-case
  .jsx: kebab-case
  .md: kebab-case | regex:(AGENTS|README)
  .ts: kebab-case
  .tsx: kebab-case
  .yaml: kebab-case

ignore:
  - .assura
  - .git
  - "**/{node_modules,.next,.turbo,dist,coverage}/**"
```

LS-Lint does not express these remaining lines from either Assura option:

- required root `README.md`, `AGENTS.md`, package manager, and Turborepo files;
- required per-workspace `AGENTS.md` and `package.json` without constraining all
  direct Markdown or JSON counts;
- advisory 160/500-line progressive-disclosure limits with repair links;
- one advisory 500-line policy across selected authored file types;
- one advisory aggregate ten-child budget for every authored directory; or
- reusable `$closed`, `$skill`, and `$workspace` contracts with rule-specific
  agent feedback.

## Practical Comparison

| Criterion | Recursive scope | Inherited shorthand | LS-Lint 2.3 |
| --- | --- | --- | --- |
| Recursive reach | Explicit once with `./**/` | Implicit on extension keys | Implicit globally until a scope replaces it |
| Common-case density | Medium | Best | Best for naming-only policy |
| Scope visibility | Best | Requires knowing inheritance | Requires knowing replacement semantics |
| Reusable policy | Node and tree rules | Node and tree rules | Brace scopes and repeated rules |
| Agent repair guidance | Yes | Yes | No |
| File/content health | Yes | Yes | No |
| Runtime model | Scope matcher plus direct indexes | Inherited suffix indexes | Global map plus replacing scopes |

Option A is the preferred homepage direction if executable proof confirms that
the matched-scope bundle normalizes once and does not duplicate findings.
Option B remains a useful compact compatibility shorthand but should not be the
only way documentation explains reach.

## Sources

- [LS-Lint 2.3 configuration basics](https://ls-lint.org/2.3/configuration/the-basics.html)
- [LS-Lint 2.3 rules](https://ls-lint.org/2.3/configuration/the-rules.html)
