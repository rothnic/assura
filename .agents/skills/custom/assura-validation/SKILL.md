---
name: assura-validation
description: "Validate project structure with assura check and .assura/config.yml."
---

# Assura Validation

Use this skill to validate project structure with Assura's supported
structure-first CLI surface.

## Quick Start

```bash
# Check the entire project
assura check

# Check specific files or directories
assura check src/
assura check docs/

# Machine-readable output
assura check --format json .
```

## Common Use Cases

### 1. Validate Project Structure

```bash
# Enforce naming, file extension, direct contents, and markdown rules from config
assura check
```

### 2. Produce Agent Feedback

```bash
# Generic structured feedback
assura check --format agent . --warn

# Codex delivery adapter for approved hooks
assura check --format agent --agent codex . --warn
```

### 3. Show Project Status

```bash
assura status --format json
```

### 4. Migrate LS-Lint Rules

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
```

## Configuration

Assura looks for `.assura/config.yml` in the project root. Example configuration:

```yaml
version: "2.0"

structure:
  ./:
    files:
      allowed_names:
        - "README.md"
        - "Cargo.toml"
      allow_extra: false
    directories:
      allowed_names:
        - "src"
        - "tests"
      allow_extra: false
    children:
      src/:
        files:
          naming: "snake_case"
          extensions:
            - "rs"
```

## Best Practices

1. **Run checks before committing**: Use advisory checks while drafting and
   blocking checks before push or CI.
   ```bash
   assura check --format agent --warn .
   assura check --format json .
   ```

2. **Start with warnings**: Set severity to low/medium initially, then increase

3. **Use onboarding packets for agents**: Run `assura agent onboard . --format
   json` when a repo needs project-local AGENTS/skills guidance.

4. **Exclude generated files**: Use `exclude` for build artifacts

5. **Document exceptions**: Keep policy exceptions in `.assura/config.yml`

## Troubleshooting

### "No config found"
- Create `.assura/config.yml` or run `assura init`

### "Permission denied"
- Ensure you have read access to all files being validated

### Slow performance
- Prefer narrow path checks such as `assura check src/` while iterating
- Exclude `node_modules`, `target`, `.git` directories

## Examples

### Validate a Rust project
```bash
# Check Rust source structure and naming policy from .assura/config.yml
assura check src/
```

### Validate documentation
```bash
# Check docs rules configured for markdown/frontmatter
assura check docs/
```

### CI/CD Integration
```yaml
# .github/workflows/assura.yml
name: Assura Validation
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
      - run: assura check --format json .
```

## See Also

- Full documentation: https://assura.dev/docs
- Constraint reference: https://assura.dev/docs/rules
- Configuration guide: https://assura.dev/docs/configuration
