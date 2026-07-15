---
title: LS-Lint Migration
description: Convert an LS-Lint config to Assura and run assura check
---

Assura can convert an LS-Lint config into the supported structure-first
`.assura/config.yml` format.

## Starting Point

Example `.ls-lint.yml`:

```yaml ls-lint-config
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

```yaml ls-lint-config
ls:
  .dir: kebab-case
  .md: exists:1-2
  packages/*:
    .ts: camelCase
```

Assura native direct-count examples:

```yaml ls-lint-config
ls:
  README.md: exists:1
  docs/: exists:1
```

These are useful for policies such as required package `README.md` files.
During LS-Lint migration, exact scalar `exists` keys such as
`README.md: exists:1` and `docs/: exists:1` become direct counts for default
validation. Scalar naming keys such as `src: kebab-case` are validated and
otherwise ignored to match upstream LS-Lint 2.3.

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
| Exact `README.md: exists:1` style keys | Migrated as direct counts for default validation; explicit target-path behavior is covered separately. |
| Non-structure behavior such as editor hooks or auto-fix | Not migrated; configure those workflows separately. |

After conversion, use the same first-run commands as a new project:

```bash
assura status --format json
assura check --format json .
```
