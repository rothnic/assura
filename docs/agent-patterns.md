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

Assura bundles three conservative layout recommendations:

- `rust-library.yml` for Cargo libraries with `src/lib.rs` and `tests/`.
- `typescript-bun-utility.yml` for utilities with `src/` and `test/`; scoped
  component paths permit PascalCase `.tsx` files.
- `python-pytest.yml` for Python packages with `pyproject.toml`, `src/`, and
  pytest-style `tests/`. It leaves `__init__.py` conventional rather than
  forbidding it.

Materialize one with `assura init --recipe <name>`, then edit the resulting
project-owned policy to match evidence already in the project. The checked
fixtures exercise these bundled recommendations; they are not the public
catalog. Service and framework layouts are variants: select them only when the
manifest, repository instructions, or existing paths establish that need.

## Apply explicit local intent

Use one local pattern explicitly:

```sh
assura init . --recipe-file .assura/patterns/rust-library.yml
assura agent onboard . --recipe-file .assura/patterns/rust-library.yml --format json
```

For a bundled recommendation, use one language layout recipe by itself:

```sh
assura init . --recipe rust-library
assura init . --recipe typescript-bun-utility
assura init . --recipe python-pytest
```

Language layout recipes are deliberately mutually exclusive with each other
and with the generic policy recipes. Compose further local intent through one
explicit `--recipe-file`.

`init` applies the pattern to a fresh starter policy. `agent onboard` merges it
with an existing project policy. Existing scalar and sequence values are
preserved on collision, except explicit pattern `exclude` entries are unioned
in stable de-duplicated order. The command reports the policy path plus
existing and incoming values, and does not partially modify config for a
conflict.

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
