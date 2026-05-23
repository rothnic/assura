---
title: Quick Start
description: Run Assura on a project in minutes
---

Use this path for a first local check.

1. **Install or build Assura**

   ```bash
   curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
   sudo install -m 755 assura assura-full /usr/local/bin/
   ```

   Choose the matching macOS or Windows archive from
   [GitHub Releases](https://github.com/rothnic/assura/releases/latest) when
   you are not on Linux x64. For source builds:

   ```bash
   git clone https://github.com/rothnic/assura
   cd assura
   cargo build --release
   ```

2. **Initialize configuration**

   ```bash
   assura init
   ```

   This creates `.assura/config.yml` if one does not already exist.

3. **Run the supported validation command**

   ```bash
   assura check
   ```

4. **Inspect JSON output when needed**

   ```bash
   assura check --format json .
   ```

5. **Use the same command in CI**

   ```bash
   curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
   sudo install -m 755 assura assura-full /usr/local/bin/
   assura check --format text
   ```

## Next Steps

- Read the full [Getting Started guide](/guides/getting-started/).
- See [Configuration](/docs/configuration/) for the supported structure-first
  config shape.
- See [LS-Lint Migration](/guides/ls-lint-migration/) when adopting Assura from
  an existing `.ls-lint.yml`.
