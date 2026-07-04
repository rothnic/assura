---
name: assura-structure-fit
description: Use when an Assura structure check rejects a new or moved file/directory, when deciding whether to reuse an existing location or update `.assura/config.yml`, or when reviewing onboarding skill installation/routing.
---

# Assura Structure Fit

Use this skill to apply `STRUCTURE_FIT_CHECK`: fix path shape first, update
config last.

## Workflow

1. Read the Assura finding and identify the rejected path and nearest configured
   scope.
2. Inspect existing repo shape before editing config. For a new top-level
   directory, inspect at least 2-3 levels of the repo map.
3. Prefer reuse, move, or rename when the artifact fits an existing project
   role.
4. Propose a `.assura/config.yml` change only when the path has a durable,
   non-duplicative role that matches project naming and structure.
5. Rerun the relevant check, usually `assura check --format agent --warn .`
   while drafting or `assura check --format json .` before handoff.

## Read as needed

| When | Read first |
| --- | --- |
| Handling a structure mismatch, new top-level directory, or config-change proposal | `references/structure-fit-check.md` |

## Outputs

- A path-first fix, or
- A concise config-change proposal with purpose, reuse analysis, naming pattern,
  allowed children, and validation command.

## Guardrails

- Do not treat structure violations as permission to loosen config.
- Do not use broad `allow_extra: true` or catch-all patterns without an explicit
  project policy reason.
- Do not silently mutate host-agent/global skill config; install Assura skills
  project-locally under `.agents/skills/` unless the user chooses otherwise.
