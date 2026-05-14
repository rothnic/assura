---
title: Git Hooks Setup
description: Run Assura before commits or pushes
template: doc
sidebar:
  order: 4
---

import { Aside, Steps } from '@astrojs/starlight/components';

Assura can run from Git hooks when you want local commits or pushes to respect
the same structure rules used in CI.

<Aside type="note">
  `assura check` validates the configured project path. It does not currently
  offer staged-file-only validation.
</Aside>

## Pre-Commit Hook

<Steps>

1. **Create `.git/hooks/pre-commit`**

   ```bash
   #!/usr/bin/env bash
   set -euo pipefail

   if ! command -v assura >/dev/null 2>&1; then
     echo "assura is not installed"
     echo "Install from source with: cargo install --path ."
     exit 1
   fi

   assura check --format text .
   ```

2. **Make it executable**

   ```bash
   chmod +x .git/hooks/pre-commit
   ```

3. **Test it**

   ```bash
   git commit --allow-empty -m "test: verify assura hook"
   ```

</Steps>

If the check fails, fix the reported files and commit again.

## Pre-Push Hook

Create `.git/hooks/pre-push`:

```bash
#!/usr/bin/env bash
set -euo pipefail

assura check --format text .
```

Then run:

```bash
chmod +x .git/hooks/pre-push
```

## Pre-Commit Framework

If your project uses [pre-commit](https://pre-commit.com/), add a local hook:

```yaml
repos:
  - repo: local
    hooks:
      - id: assura-check
        name: Assura check
        entry: assura check --format text .
        language: system
        pass_filenames: false
        always_run: true
```

Then install it:

```bash
pre-commit install
```

## CI Still Matters

Local hooks are easy to bypass with `--no-verify`, so keep `assura check` in CI
as the source of truth for pull requests.
