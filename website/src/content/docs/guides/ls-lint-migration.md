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

## Unsupported Or Explicitly Bounded Behavior

`assura migrate` is intentionally a structure-policy migration, not a full
LS-Lint runtime emulator.

| Input shape | Result |
| --- | --- |
| Invalid YAML | `migrate` exits nonzero and prints the parse error. |
| Empty `ls:` section | Assura writes a config with exclusions but no structure rules. |
| Empty `exists:` value | `migrate` exits nonzero with an invalid exists-rule error. |
| Empty `regex:` value | `migrate` exits nonzero with an unsupported regex-rule error. |
| Exact `README.md: exists:1` style keys | Migrated as Assura compatibility extensions, not native LS-Lint parity. |
| Non-structure behavior such as editor hooks or auto-fix | Not migrated; configure those workflows separately. |

After conversion, use the same first-run commands as a new project:

```bash
assura status --format json
assura check --format json .
```
