---
title: LS-Lint Migration
description: Convert an LS-Lint config to Assura and run assura check
---

Assura can convert an LS-Lint config into the supported structure-first
`.assura/config.yml` format.

## Starting Point

Example `.ls-lint.yml`:

```yaml
ls:
  .dir: kebab-case
  .ts: kebab-case
ignore:
  - node_modules
  - dist
```

## Convert

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
```

The generated config excludes `.assura/**` so Assura does not validate the
configuration directory it just created.

## Check

```bash
assura check
```

For automation:

```bash
assura check --format json .
```

## Compatibility Notes

The migration path targets LS-Lint 2.3 naming, `.dir`, ignore, OR syntax, and
`exists` behavior where Assura has parity coverage. Exact filename `exists`
rules are documented as an Assura compatibility extension.

Advanced directory-scope syntax is intentionally not converted yet. Scopes such
as `packages/*`, `**`, and `{src,tests}` fail with a clear migration error
instead of being silently treated as literal directories.
