---
title: Quick Start
description: Run Assura on a project in minutes
---

Use this path for a first local check.

1. **Install Assura**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
   ```

   The installer downloads a release archive and does not require a source
   checkout. Windows users can run the PowerShell installer from the
   [Installation guide](/guides/installation/) or download the zip from
   [GitHub Releases](https://github.com/rothnic/assura/releases/latest).

2. **Initialize configuration**

   ```bash
   assura init --recipe agentic-core --recipe structure-health
   ```

   This creates `.assura/config.yml` if one does not already exist.

3. **Write the first useful project shape**

   For a small Rust CLI or library, start with the files a new clone should
   always have:

   ```yaml
   structure:
     README.md: exists:1
     Cargo.toml: exists:1
     src/: exists:1
     ./*.lock: exists:0-1
     ./**/:
       .rs: snake_case
   exclude:
     - "target/**"
   ```

   This requires the core files, allows one optional root lockfile, and checks
   Rust file names at every directory depth. Assura remains open to undeclared
   paths until the project explicitly composes a closed direct-content rule.

4. **Run the supported validation command**

   ```bash
   assura check
   ```

5. **Inspect JSON output when needed**

   ```bash
   assura check --format json .
   ```

6. **Use the same command in CI**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
   assura check --format text
   ```

## Next Steps

- Read the full [Getting Started guide](/guides/getting-started/).
- Follow the [Adoption Walkthrough](/guides/adoption-walkthrough/) for the
  empty-project and LS-Lint migration paths.
- See [Configuration](/reference/configuration/) for the supported structure-first
  config shape.
- See [LS-Lint Migration](/guides/ls-lint-migration/) when adopting Assura from
  an existing `.ls-lint.yml`.
