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
`exists` behavior.

Native LS-Lint parity examples:

```yaml
ls:
  .dir: kebab-case
  .md: exists:1-2
  packages/*:
    .ts: camelCase
```

Assura compatibility extension examples:

```yaml
ls:
  README.md: exists:1
  docs/: exists:1
```

The extension examples become direct child count checks in `.assura/config.yml`.
They are useful for policies such as required package `README.md` files, but
they are not native LS-Lint 2.3 behavior. Upstream LS-Lint reports exact scalar
filename `exists` keys differently, so Assura docs and tests keep this behavior
separate from the native parity surface.

Multiple LS-Lint config files can be passed to `assura migrate` in the same
order you would pass repeated LS-Lint `--config` flags.
