# V1 to V2 Migration Guide

This guide helps you migrate your Assura configuration from Version 1 (V1) to Version 2 (V2). V2 introduces a structure-first approach that is more intuitive and maintainable.

## Why Migrate?

V2 offers several advantages over V1:

- **Clearer Structure**: Config mirrors your project directory layout
- **Reduced Duplication**: Parent rules automatically inherited by children
- **Better Organization**: Related rules are grouped together
- **Easier Maintenance**: Changes in one place affect all children
- **LS-Lint Compatibility**: Easier integration with existing LS-Lint configs

## Migration Overview

### Key Differences

| Aspect | V1 | V2 |
|--------|-----|-----|
| **Format** | Flat array of rules | Hierarchical structure |
| **Version** | Implicit | Explicit `version: "2.0"` |
| **Targeting** | `applies_to` patterns | Structure node paths |
| **Inheritance** | Manual duplication | Automatic inheritance |
| **Nesting** | Flat list | Tree with children |

### Quick Migration Steps

1. Add `version: "2.0"` at the top of your config
2. Replace `rules:` with `structure:`
3. Convert each `applies_to` to a structure node path
4. Move rule options into `files:` or `markdown:` bundles
5. Use `children:` for nested directories
6. Remove duplicate settings that can be inherited

## Detailed Migration Examples

### Example 1: Basic Naming Conventions

**V1 Configuration:**
```yaml
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
    severity: high
  
  - name: naming
    applies_to: "src/components/**/*.rs"
    convention: PascalCase
    severity: high
  
  - name: naming
    applies_to: "tests/**/*.rs"
    convention: snake_case
    severity: medium
  
  - name: naming
    applies_to: "benches/**/*.rs"
    convention: snake_case
    severity: low
```

**V2 Configuration:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      severity: high
    
    children:
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      naming: snake_case
      severity: medium
  
  benches/:
    files:
      naming: snake_case
      severity: low
```

**Changes Made:**
- Added `version: "2.0"`
- Changed `rules:` to `structure:`
- Converted `applies_to: "src/**/*.rs"` to `src/:` node
- Moved `convention` to `naming` in `files:` bundle
- Used `children:` for `components/`
- Kept `severity` in each bundle

### Example 2: Multiple Validation Rules

**V1 Configuration:**
```yaml
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
    severity: high
  
  - name: file-size
    applies_to: "src/**/*.rs"
    max_size: "500KB"
    severity: medium
  
  - name: documentation
    applies_to: "src/**/*.rs"
    require_public: true
    severity: medium
  
  - name: file-size
    applies_to: "tests/**/*.rs"
    max_size: "1MB"
    severity: low
```

**V2 Configuration:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      max_size: "500KB"
      require_docs: true
      severity: high  # Uses most restrictive
  
  tests/:
    files:
      max_size: "1MB"
      severity: low
```

**Changes Made:**
- Combined three rules targeting `src/` into one `files:` bundle
- Renamed `require_public` to `require_docs`
- In V2, only the most restrictive severity applies
- Simplified `tests/` with just size limit

### Example 3: Markdown Validation

**V1 Configuration:**
```yaml
markdown:
  validation:
    enabled: true
    rules:
      - id: frontmatter-required
        applies_to: "docs/**/*.md"
        severity: medium
      
      - id: check-links
        applies_to: "docs/**/*.md"
        severity: high
      
      - id: heading-depth
        applies_to: "docs/**/*.md"
        max_depth: 3
        severity: low
      
      - id: frontmatter-required
        applies_to: "blog/**/*.md"
        severity: low
```

**V2 Configuration:**
```yaml
version: "2.0"

structure:
  docs/:
    markdown:
      require_frontmatter: true
      check_links: true
      max_heading_depth: 3
      severity: high  # Most restrictive wins
  
  blog/:
    markdown:
      require_frontmatter: true
      severity: low
```

**Changes Made:**
- Converted `markdown:` rules to `markdown:` bundle in structure nodes
- Combined multiple rules into single bundles
- Used `max_heading_depth` instead of `max_depth`
- Simplified structure with direct options

### Example 4: Complex Project Structure

**V1 Configuration:**
```yaml
rules:
  # Source code rules
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
    severity: high
  
  - name: file-size
    applies_to: "src/**/*.rs"
    max_size: "500KB"
    severity: medium
  
  - name: naming
    applies_to: "src/api/**/*.rs"
    convention: snake_case
    severity: critical
  
  - name: file-size
    applies_to: "src/api/**/*.rs"
    max_size: "300KB"
    severity: critical
  
  - name: documentation
    applies_to: "src/api/**/*.rs"
    require_public: true
    severity: critical
  
  # Component rules
  - name: naming
    applies_to: "src/components/**/*.rs"
    convention: PascalCase
    severity: high
  
  # Internal rules
  - name: naming
    applies_to: "src/internal/**/*.rs"
    convention: snake_case
    severity: low
  
  - name: documentation
    applies_to: "src/internal/**/*.rs"
    require_public: false
    severity: low
  
  # Test rules
  - name: naming
    applies_to: "tests/**/*.rs"
    convention: snake_case
    severity: medium
  
  - name: file-size
    applies_to: "tests/**/*.rs"
    max_size: "2MB"
    severity: low
```

**V2 Configuration:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      max_size: "500KB"
      severity: high
    
    children:
      api/:
        files:
          naming: snake_case
          max_size: "300KB"
          require_docs: true
          severity: critical
      
      components/:
        files:
          naming: PascalCase
          severity: high
      
      internal/:
        files:
          naming: snake_case
          require_docs: false
          severity: low
  
  tests/:
    files:
      naming: snake_case
      max_size: "2MB"
      severity: medium
```

**Changes Made:**
- Organized all rules under `src/` hierarchy
- Used `children:` for nested directories
- `api/` overrides parent's `max_size` and adds `require_docs`
- `components/` only overrides `naming`
- `internal/` relaxes `severity` and disables docs
- Eliminated 8 rule entries with inheritance

### Example 5: LS-Lint Configuration

**V1 LS-Lint Style:**
```yaml
ls:
  .rs: snake_case
  .ts: camelCase
  .tsx: PascalCase
  src/:
    .rs: snake_case
  src/components/:
    .tsx: PascalCase
  tests/:
    .rs: snake_case
```

**V2 Native Structure:**
```yaml
version: "2.0"

structure:
  "":
    files:
      naming: snake_case  # Default for .rs
  
  # TypeScript files at root
  "*.ts":
    files:
      naming: camelCase
  
  "*.tsx":
    files:
      naming: PascalCase
  
  src/:
    files:
      naming: snake_case
    
    children:
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      naming: snake_case
```

**Changes Made:**
- Converted LS-Lint extension-based rules to structure nodes
- Used `""` for root-level files
- Added explicit patterns for `.ts` and `.tsx`
- Organized path-specific rules under `structure:`

Or keep LS-Lint compatibility:

```yaml
version: "2.0"

ls:
  .rs: snake_case
  .ts: camelCase
  .tsx: PascalCase
  src/:
    .rs: snake_case
  src/components/:
    .tsx: PascalCase
  tests/:
    .rs: snake_case
```

## Common Patterns and Their V2 Equivalents

### Pattern 1: Different Rules for Different File Types

**V1:**
```yaml
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
  - name: naming
    applies_to: "src/**/*.md"
    convention: kebab-case
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      extensions:
        - "rs"
  
  # Markdown files handled separately
  docs/:
    markdown:
      require_frontmatter: true
```

### Pattern 2: Progressive Relaxation

**V1:**
```yaml
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
    severity: high
  - name: naming
    applies_to: "src/experimental/**/*.rs"
    convention: snake_case
    severity: low
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
      severity: high
    
    children:
      experimental/:
        files:
          naming: snake_case
          severity: low
```

### Pattern 3: Generated Code Exceptions

**V1:**
```yaml
rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
  - name: naming
    applies_to: "src/generated/**/*.rs"
    convention: regex:.*
    severity: low
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: snake_case
    
    children:
      generated/:
        inherit: false  # Don't inherit parent rules
        files:
          naming: "regex:.*"
          severity: low
```

### Pattern 4: Documentation Requirements

**V1:**
```yaml
rules:
  - name: documentation
    applies_to: "src/public/**/*.rs"
    require_public: true
    severity: high
  - name: documentation
    applies_to: "src/internal/**/*.rs"
    require_public: false
    severity: low
```

**V2:**
```yaml
version: "2.0"

structure:
  src/:
    children:
      public/:
        files:
          require_docs: true
          severity: high
      
      internal/:
        files:
          require_docs: false
          severity: low
```

## CLI Migration Tool

Assura provides a migration helper to automate the conversion:

### Installation

```bash
cargo install assura
```

### Dry Run

Preview changes without modifying files:

```bash
assura migrate --from v1 --to v2 --dry-run
```

### Migration Commands

```bash
# Migrate and output to new file
assura migrate --from v1 --to v2 --output .assura/config-v2.yml

# Migrate in place (backup automatically created)
assura migrate --from v1 --to v2 --in-place

# Migrate with specific config path
assura migrate --from v1 --to v2 --config ./my-config.yml --output ./my-config-v2.yml
```

### Validate Migrated Config

```bash
# Check if migrated config is valid
assura config validate --config .assura/config-v2.yml

# Test with verbose output
assura validate --config .assura/config-v2.yml --verbose
```

## Troubleshooting Migration Issues

### Issue: "Config not recognized as V2"

**Problem:** Assura treats V2 config as V1

**Solution:**
- Ensure `version: "2.0"` is the first key
- Check for YAML syntax errors before the version field
- Verify no tabs are used (use spaces)

```yaml
# Correct
version: "2.0"
structure:
  ...

# Incorrect
project:
  name: My Project
version: "2.0"  # Too late!
```

### Issue: "Rules not being applied"

**Problem:** Files aren't being validated

**Solution:**
- Check that directory paths end with `/`
- Verify file extensions match
- Ensure `inherit: false` isn't blocking inheritance

```yaml
# Correct
structure:
  src/:
    files:
      naming: snake_case

# Incorrect - missing trailing slash
structure:
  src:
    files:
      naming: snake_case
```

### Issue: "Inheritance not working"

**Problem:** Child rules don't inherit from parent

**Solution:**
- Verify `inherit: true` (default) is set
- Check child is under parent's `children:`
- Ensure parent path is correct

```yaml
# Correct - explicit inheritance
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
    
    children:
      components/:
        inherit: true
        files:
          naming: PascalCase
          # max_lines: 500 inherited

# Incorrect - missing children key
structure:
  src/:
    files:
      naming: snake_case
  
  src/components/:  # Not a child of src/!
    files:
      naming: PascalCase
```

### Issue: "Markdown validation not working"

**Problem:** Markdown files aren't being validated

**Solution:**
- Use `markdown:` bundle, not `files:`
- Ensure path matches markdown files location
- Check that `require_frontmatter` is set

```yaml
# Correct
structure:
  docs/:
    markdown:
      require_frontmatter: true
      check_links: true

# Incorrect - using files for markdown
structure:
  docs/:
    files:  # Wrong bundle type
      require_frontmatter: true
```

## Migration Checklist

- [ ] Backup your existing V1 config
- [ ] Add `version: "2.0"` at the top
- [ ] Replace `rules:` with `structure:`
- [ ] Convert `applies_to` patterns to structure paths
- [ ] Move rule options to `files:` or `markdown:` bundles
- [ ] Use `children:` for nested directories
- [ ] Remove duplicate settings (use inheritance)
- [ ] Add `inherit: false` where needed
- [ ] Validate the new config
- [ ] Test with `assura validate --verbose`
- [ ] Update CI/CD to use new config
- [ ] Document the changes for your team

## Backwards Compatibility

### V1 Configs Still Work

You can continue using V1 configs without any changes:

```bash
# This still works exactly as before
assura validate --config .assura/config.yml
```

### Gradual Migration

You can migrate gradually:

```bash
# Keep V1 config as default
assura validate  # Uses .assura/config.yml (V1)

# Test V2 config explicitly
assura validate --config .assura/config-v2.yml
```

### Mixed Usage

You can use both formats in different projects:

```
project-a/
  .assura/
    config.yml  # V1 config

project-b/
  .assura/
    config.yml  # V2 config
```

Assura automatically detects the version and handles each appropriately.

## Best Practices

### 1. Start with Parent Rules

Define common settings at the highest level:

```yaml
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
      severity: high
    
    children:
      # Only override what's different
      components/:
        files:
          naming: PascalCase
```

### 2. Document Exceptions

Add comments for unusual configurations:

```yaml
structure:
  src/:
    children:
      # Third-party code - minimal validation
      vendor/:
        inherit: false
        files:
          naming: regex:.*
          severity: low
```

### 3. Use Severity Appropriately

- `critical`: Blocks CI/CD, must fix
- `high`: Serious issues, fix soon
- `medium`: Should review
- `low`: Optional suggestions

### 4. Validate Incrementally

Test your config after each major change:

```bash
assura config validate
assura validate --verbose | head -20
```

### 5. Keep Exclusions Minimal

Only exclude truly necessary paths:

```yaml
exclude:
  - "target/**"      # Build artifacts
  - ".git/**"        # Version control
  - "node_modules/**"  # Dependencies
  # Not recommended: excluding source directories
  # - "src/generated/**"  # Instead use inherit: false
```

## Next Steps

After migration:

1. **Review the new config** - Ensure all rules are captured
2. **Run full validation** - Check that everything works
3. **Update documentation** - Notify your team of changes
4. **Consider optimization** - Look for further simplification opportunities
5. **Explore advanced features** - Try new V2 capabilities

## Support

Need help with migration?

- **Documentation**: https://assura.dev/docs/config-v2
- **Issues**: https://github.com/assura/assura/issues
- **Discussions**: https://github.com/assura/assura/discussions
- **Migration Tool**: `assura migrate --help`

## Appendix: Complete Before/After

### V1 Complete Example

```yaml
name: My Project
version: "1.0"

settings:
  parallel: true
  max_workers: 8

rules:
  - name: naming
    applies_to: "src/**/*.rs"
    convention: snake_case
    severity: high
  
  - name: file-size
    applies_to: "src/**/*.rs"
    max_size: "500KB"
    severity: medium
  
  - name: naming
    applies_to: "src/components/**/*.rs"
    convention: PascalCase
    severity: high
  
  - name: documentation
    applies_to: "src/**/*.rs"
    require_public: true
    severity: medium
  
  - name: file-size
    applies_to: "tests/**/*.rs"
    max_size: "1MB"
    severity: low

markdown:
  validation:
    enabled: true
    rules:
      - id: frontmatter-required
        applies_to: "docs/**/*.md"
        severity: medium

excludes:
  - "target/**"
  - ".git/**"
```

### V2 Complete Example

```yaml
version: "2.0"

project:
  name: My Project

structure:
  src/:
    files:
      naming: snake_case
      max_size: "500KB"
      require_docs: true
      severity: high
    
    children:
      components/:
        files:
          naming: PascalCase
  
  tests/:
    files:
      max_size: "1MB"
      severity: low
  
  docs/:
    markdown:
      require_frontmatter: true

exclude:
  - "target/**"
  - ".git/**"
```

**Lines of config reduced from 48 to 30 (37% reduction)**

---

Happy migrating! 🚀
