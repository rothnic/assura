---
title: 'Gitignore Integration Specification'
status: active
---

# Gitignore Integration Specification

## Overview

Assura should optionally respect `.gitignore` patterns when validating projects. This prevents Assura from checking files that are intentionally excluded from version control.

## Motivation

1. **Avoid Duplicate Configuration**: Users already maintain `.gitignore` - no need to duplicate exclusions in `.assura/config.yml`
2. **Consistent Behavior**: Files ignored by git should also be ignored by Assura by default
3. **Simpler Setup**: New projects get sensible defaults automatically

## Configuration Options

### Option 1: Global Setting (Recommended)

Add a top-level configuration option to control gitignore integration:

```yaml
version: "2.0"

project:
  name: "my-project"

# Gitignore integration settings
gitignore:
  # Enable reading .gitignore files (default: true)
  enabled: true

  # Additional patterns to ignore beyond .gitignore
  additional:
    - "*.tmp"
    - ".assura/execution-state.json"

  # Patterns to NOT ignore even if in .gitignore
  exceptions:
    - "*.lock"  # Still validate lock files

structure:
  src/:
    files:
      naming: snake_case
```

### Option 2: Per-Directory Override

Allow overriding gitignore behavior at the structure node level:

```yaml
structure:
  src/:
    files:
      naming: snake_case
    # Override global gitignore setting for this directory
    respect_gitignore: false  # Check even ignored files

  generated/:
    files:
      naming: kebab-case
    respect_gitignore: true  # Skip ignored files (default)
```

## Implementation Details

### Discovery Strategy

1. **Find all `.gitignore` files** starting from project root
2. **Parse patterns** using git's pattern matching rules:
   - Negation patterns (`!file`)
   - Directory markers (`dir/`)
   - Glob patterns (`*.log`, `**/*.tmp`)
   - Anchored patterns (`/file`)
3. **Merge with Assura's exclude list**
4. **Apply to file discovery** before validation

### Pattern Matching

Use the `ignore` crate (Rust implementation of gitignore matching):

```rust
use ignore::gitignore::GitignoreBuilder;

let mut builder = GitignoreBuilder::new(".");
builder.add(".gitignore");
let gitignore = builder.build()?;

// Check if path matches
if gitignore.matched(path, is_dir).is_ignore() {
    // Skip this file
}
```

### Precedence

1. Assura's `exclude` patterns (highest priority)
2. `.gitignore` patterns (if enabled)
3. Default Assura exclusions (lowest priority)

```yaml
# Example: User wants to check .env files even though gitignored
gitignore:
  enabled: true
  exceptions:
    - ".env*"  # Don't ignore .env files

exclude:
  - "target/**"  # Still excluded by Assura
```

## Default Behavior

**Proposal**: `enabled: true` by default

Rationale:
- Most users want consistent behavior between git and Assura
- Prevents validation of build artifacts, dependencies, etc.
- Can be disabled for projects that want to validate ignored files

## CLI Integration

Add command-line flags:

```bash
# Ignore gitignore patterns (default)
assura check

# Don't respect gitignore
assura check --no-gitignore

# Show what would be ignored
assura check --list-ignored
```

## Example Use Cases

### Use Case 1: Monorepo with Generated Code

```yaml
# .gitignore
*.generated.ts

# .assura/config.yml
gitignore:
  enabled: true
  exceptions:
    - "*.generated.ts"  # Still validate generated code

structure:
  packages/:
    children:
      "**/":
        files:
          naming: camelCase
```

### Use Case 2: Dotfiles Repository

```yaml
# .assura/config.yml - Don't use gitignore since everything IS dotfiles
gitignore:
  enabled: false

structure:
  ./:
    files:
      naming: "regex:^\\."
```

### Use Case 3: Mixed Source and Build

```yaml
# .assura/config.yml
gitignore:
  enabled: true  # Ignore node_modules/, dist/, etc.

structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500

  dist/:
    # Even though dist/ is gitignored, we want to check built files
    respect_gitignore: false
    files:
      naming: kebab-case
```

## Migration Path

For existing projects:

1. **Default enabled**: May change behavior (fewer files checked)
2. **Migration guide**: Add `gitignore: { enabled: false }` to preserve old behavior
3. **Audit mode**: `assura check --list-ignored` to see what would be skipped

## Performance Considerations

- Parse `.gitignore` once at startup
- Cache results for repeated path checks
- Use efficient glob matching (ignore crate)
- Minimal overhead when disabled

## Related Issues

- #XXX: Initial feature request
- #YYY: Performance concerns with large gitignore files

## References

- [gitignore documentation](https://git-scm.com/docs/gitignore)
- [ignore crate](https://docs.rs/ignore/latest/ignore/gitignore/index.html)
