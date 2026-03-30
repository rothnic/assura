---
name: assura-validation
description: "Validate project files using Assura constraints. Use when you need to check file naming conventions, validate markdown files, detect circular dependencies, or enforce project structure rules. Works with any project that has an .assura/config.yml file."
triggers: ["validate", "assura check", "file naming", "project structure"]
---

# Assura Validation

Use this skill to validate project files using Assura's constraint engine.

## Quick Start

```bash
# Check the entire project
assura check

# Check specific files or directories
assura check src/
assura check docs/*.md

# Check with specific maturity level
assura check --maturity stable
```

## Common Use Cases

### 1. Validate File Naming Conventions

```bash
# Check if all files follow naming conventions defined in .assura/config.yml
assura check --constraint naming
```

### 2. Validate Markdown Files

```bash
# Check markdown files for frontmatter, heading structure, etc.
assura check --constraint markdown
```

### 3. Check for Circular Dependencies

```bash
# Analyze import/require statements for circular dependencies
assura check --constraint dependencies
```

### 4. Validate Project Structure

```bash
# Ensure files are in the correct directories
assura check --constraint organization
```

## Configuration

Assura looks for `.assura/config.yml` in the project root. Example configuration:

```yaml
version: "1.0"
maturity: stable

naming:
  conventions:
    - name: "rust_source"
      pattern: "^[a-z_][a-z0-9_]*\.rs$"
      applies_to: "src/**/*.rs"
      severity: high

markdown:
  validation:
    enabled: true
    rules:
      - id: "frontmatter-required"
        applies_to: "**/*.md"
        severity: medium

dependencies:
  constraints:
    - id: "no-circular-deps"
      severity: critical
```

## Best Practices

1. **Run checks before committing**: Use git hooks to validate automatically
   ```bash
   assura hooks install
   ```

2. **Start with warnings**: Set severity to low/medium initially, then increase

3. **Exclude generated files**: Use `exclude.paths` for build artifacts

4. **Document exceptions**: Add comments in code when suppressing rules

## Troubleshooting

### "No config found"
- Create `.assura/config.yml` or run `assura init`

### "Permission denied"
- Ensure you have read access to all files being validated

### Slow performance
- Use `--parallel` flag for large projects
- Exclude `node_modules`, `target`, `.git` directories

## Examples

### Validate a Rust project
```bash
# Check all Rust files follow snake_case
assura check src/ --pattern "*.rs" --convention snake_case
```

### Validate documentation
```bash
# Ensure all markdown files have frontmatter
assura check docs/ --require-frontmatter
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
      - uses: actions/checkout@v3
      - uses: assura/assura-action@v1
      - run: assura check --strict
```

## See Also

- Full documentation: https://assura.dev/docs
- Constraint reference: https://assura.dev/docs/rules
- Configuration guide: https://assura.dev/docs/configuration
