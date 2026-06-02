---
title: Adoption Walkthrough
description: Install Assura, initialize a new project, migrate LS-Lint, and recover from first-run failures
---

This walkthrough uses only supported release commands. It assumes `assura` was
installed from a release archive or the install script described in
[Installation](/guides/installation/).

## Empty Project

Create a project and install a starter config:

```bash
mkdir empty-project
cd empty-project
printf '# Empty Project\n' > README.md
assura init --no-git-hooks
```

Inspect the generated config:

```bash
assura status --format json
```

Run the first passing check:

```bash
assura check --format json .
```

Create an intentional failure:

```bash
printf 'fn main() {}\n' > BadName.rs
assura check --format json .
```

The failing command exits with status `1` and reports a file naming violation.
Rename the file to kebab case or update `.assura/config.yml`, then rerun the
same check.

## Existing LS-Lint Project

Start from an LS-Lint config:

```bash
mkdir ls-lint-project
cd ls-lint-project
cat > .ls-lint.yml <<'YAML'
ls:
  .dir: kebab-case
  .rs: snake_case
ignore:
  - target
YAML
printf 'fn main() {}\n' > good_name.rs
```

Convert the config and run Assura:

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
assura status --format json
assura check --format json .
```

Assura writes `.assura/config.yml` and excludes `.assura/**` so the generated
config directory is not validated by the converted policy.

## Recovery Cases

Missing config:

```bash
assura status --format json
assura init --no-git-hooks
```

`status` exits nonzero until `.assura/config.yml` exists.

Existing config:

```bash
assura init
```

`init` refuses to overwrite `.assura/config.yml`. Use `assura init --force` only
when replacing the existing file is intentional.

Invalid config:

```bash
assura check --format json .
```

Fix the YAML or unsupported fields reported in stderr, then rerun the same
command.

Unsupported migration rule:

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
```

Fix the reported LS-Lint rule or remove unsupported behavior, then rerun
`migrate`. The supported and unsupported migration surface is documented in
[LS-Lint Migration](/guides/ls-lint-migration/).

Hook prerequisites:

```bash
assura hooks install
assura hooks status
```

Git hooks are optional and local to your checkout. Codex prompt hooks are not
installed automatically; use `assura check --format agent --agent codex` only
after Codex hooks are enabled and the project hook command has been approved.

## CI Smoke Contract

The release-style smoke test used by this repository installs from a local
release archive and runs this same journey:

```bash
ASSURA_BIN=/path/to/assura ./scripts/smoke-install-adoption.sh
```

CI runs the equivalent script on `ubuntu-latest` x86_64, `macos-14` arm64,
`macos-13` x86_64, and `windows-latest` x86_64.
