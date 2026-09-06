---
title: Local Agent Patterns
status: active
---

# Local Agent Patterns

Assura local patterns are ordinary, project-owned YAML policy fragments. They
can contain the existing `rules`, `structure`, `exclude`, and `quality` keys;
they do not execute code, fetch remote content, or introduce a template
language.

## Start with a small pattern

The repository includes three conservative examples in
`tests/fixtures/agent_init/patterns/`:

- `rust-library.yml` for Cargo libraries with `src/lib.rs` and `tests/`.
- `typescript-bun-utility.yml` for utilities with `src/` and `test/`; scoped
  component paths permit PascalCase `.tsx` files.
- `python-pytest.yml` for Python packages with `pyproject.toml`, `src/`, and
  pytest-style `tests/`. It leaves `__init__.py` conventional rather than
  forbidding it.

Copy an example into your repository and edit it to match evidence already in
the project. Service and framework layouts are variants: select them only when
the manifest, repository instructions, or existing paths establish that need.

## Apply explicit local intent

Use one local pattern explicitly:

```sh
assura init . --recipe-file .assura/patterns/rust-library.yml
assura agent onboard . --recipe-file .assura/patterns/rust-library.yml --format json
```

`init` applies the pattern to a fresh starter policy. `agent onboard` merges it
with an existing project policy. Existing scalar and sequence values are
preserved on collision; the command reports the policy path plus existing and
incoming values, and does not partially modify config for that conflict.

Assura validates the prospective complete policy before writing it atomically.
The selected source path and SHA-256 are recorded in
`.assura/onboarding/profile-selection.json`, separate from policy semantics.
Rerunning the same explicit `init --force` input produces byte-identical policy
and provenance.

## Boundaries

Patterns are local YAML only. They cannot execute scripts, infer exclusions to
hide violations, replace project instructions, or select a remote marketplace
artifact. Use `assura check --format agent .` after application and make each
exception visible in the project-owned policy.
