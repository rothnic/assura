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

The migration path targets LS-Lint 2.3 naming, regex, `.dir`, ignore, OR
syntax, wildcard/subextension rules, glob and brace directory scopes, and
`exists` behavior. Exact filename `exists` rules are documented as an Assura
compatibility extension.

Multiple LS-Lint config files can be passed to `assura migrate` in the same
order you would pass repeated LS-Lint `--config` flags.
