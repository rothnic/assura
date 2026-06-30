---
title: Structure Validation
description: The shipped base layer for repository shape checks.
---

Structure validation is Assura's base layer. It checks ordinary repository
files and directories against `.assura/config.yml`, with no daemon, database, or
hosted service in the validation path.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Structure-first config | Shipped | `structure`, `files`, `directories`, `children`, `exists`, and `exclude` are the current foundation. |
| LS-Lint migration | Shipped | `assura migrate` converts supported LS-Lint 2.3 shape rules into Assura config. |
| JSON/YAML/text reports | Shipped | `assura check` emits local and CI-friendly reports. |
| Guided feedback formats | Experimental | Advice, status, and agent JSON reuse the same check pipeline. |

## When To Use It

Use structure validation when the repository shape is part of the contract:
allowed file names, directory names, required files, forbidden outputs, and
scope-specific naming rules.

```bash
assura check --format json .
```

Start with [Configuration](/docs/configuration/) and the
[LS-Lint Migration](/guides/ls-lint-migration/) guide.
