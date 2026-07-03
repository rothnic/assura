# STRUCTURE_FIT_CHECK

Use this reference when a new or moved path does not fit `.assura/config.yml`.
The short violation nudge can be:

```text
Structure mismatch: `{path}`. Apply STRUCTURE_FIT_CHECK before editing config.
```

For top-level directories:

```text
New top-level path: `{path}`. Apply STRUCTURE_FIT_CHECK with a 2-3 level repo map before editing config.
```

## Decision Frame

Choose one:

| Choice | Use when | Result |
| --- | --- | --- |
| Reuse existing home | The artifact fits an existing project role | Move or rename the path |
| Keep path, no config change | The path is temporary or should not be committed | Remove, ignore locally, or document as scratch |
| Update config | The path has a durable, distinct role | Add a narrow rule and validation proof |

## Repo Map

Before adding a top-level directory, inspect the shape that already exists:

```bash
find . -maxdepth 3 -type d \
  -not -path './.git*' \
  -not -path './target*' \
  -not -path './node_modules*' \
  | sort
```

Ask:

- Does an existing directory already own this responsibility?
- Would the new path split one concept across two homes?
- Does the name match nearby naming style?
- What direct children should be allowed?
- What source/test/docs relationships should exist?

## Config Change Bar

Only update `.assura/config.yml` after answering:

```text
Purpose:
Why existing paths do not fit:
Expected direct children:
Naming/style rule:
Related tests/docs/content records:
Validation command:
```

Prefer exact names, narrow patterns, `exists`, captures, `needs`, and
`provides` over broad extras. Keep config changes local to the smallest
directory scope that owns the policy.

## Onboarding Install

Assura onboarding should install this skill project-locally:

```text
.agents/skills/assura-structure-fit/SKILL.md
.agents/skills/assura-structure-fit/references/structure-fit-check.md
```

Generated `AGENTS.md` should route structure mismatches to that local skill.
Generated `.assura/onboarding/agent-next.md` may mention `STRUCTURE_FIT_CHECK`
as the compact anchor. Assura should preserve user-authored skill edits and
must not silently mutate host-agent or global skill configuration.
