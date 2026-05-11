---
title: 'Assura Unified Tree Configuration Design'
status: active
---

# Assura Unified Tree Configuration Design

**Core Principle:** Single hierarchical tree that mirrors directory structure, with directives hanging off nodes.

---

## Architecture

```
rules:              # Root of unified tree
  ├── .ts           # File type conventions (global default)
  ├── .tsx          # File type conventions (global default)
  ├── src/          # Directory path
  │   ├── .ts       # Override for src/
  │   ├── components/   # Nested directory
  │   │   ├── .tsx  # Override for components/
  │   │   └── require: [...]  # Directives at this level
  │   └── utils/
  │       └── .ts
  └── packages/*    # Wildcard path
      ├── require: [...]     # Directives apply here
      └── src/
          └── .ts   # Nested under wildcard
```

---

## Unified Tree Config Format

```yaml
# Root - the unified tree
rules:
  # Global defaults (apply everywhere unless overridden)
  .ts: camelCase
  .tsx: PascalCase
  .js: camelCase
  .jsx: PascalCase
  .dir: kebab-case
  
  # Root-level coordination docs (required files)
  README.md:
    exists: true
    message: "Add root README.md describing the project"
  AGENTS.md:
    exists: true
    message: "Add AGENTS.md for agent guidance"
  
  # Root file whitelist (only these allowed)
  .md:
    naming: kebab-case
    allow: [README.md, AGENTS.md, CHANGELOG.md]
    deny_message: "Move markdown files to docs/"
  
  # Directory-specific rules
  src/:
    # Override for all .ts in src/
    .ts: camelCase
    .tsx: PascalCase
    
    # More specific nested paths
    components/:
      # Override for components
      .tsx: PascalCase
      .ts: PascalCase
      
      # Directive: files must exist
      require:
        files: []
        message: "Components directory structure"
    
    hooks/:
      # Different convention for hooks
      .ts: camelCase
      
      # Directive: max file size
      max_lines: 400
    
    utils/:
      .ts: camelCase
      max_lines: 400
    
    generated/:
      # Relaxed rules for generated code
      .ts:
        naming: [camelCase, PascalCase, snake_case]
        max_lines: 2000
  
  # Package structure enforcement
  packages/*:
    # Wildcard matches any subdirectory
    .dir: kebab-case
    
    # Required files in every package
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src, tests]
      message: "Packages must have documentation and standard structure"
    
    # Package source code
    src/:
      .ts: camelCase
      .tsx: PascalCase
      
      components/:
        .tsx: PascalCase
  
  # Documentation
  docs/:
    .md:
      naming: kebab-case
      allow: [README.md, AGENTS.md, CHANGELOG.md]
      max_lines: 500
      require_frontmatter: true
    
    templates/:
      .md:
        max_lines: 2000
        require_frontmatter: false

# Supporting configuration (makes tree more efficient)
groups:
  ts-standard:
    naming: camelCase
    max_lines: 400
  
  ts-component:
    naming: PascalCase
    max_lines: 400
  
  ts-test:
    naming: snake_case
    max_lines: 600

# Exclusions (separate from tree)
exclude:
  - "node_modules/**"
  - "dist/**"
```

---

## How Directives Work in the Tree

### Directive: `require` (files/dirs must exist)

```yaml
rules:
  packages/*:
    # These files must exist in every package
    require:
      files: [AGENTS.md, README.md]
      dirs: [src, tests]
      severity: error
```

**Applied at:** `packages/*` level  
**Meaning:** Every package subdirectory must have these files/dirs

### Directive: `allow` (root whitelist)

```yaml
rules:
  .md:
    naming: kebab-case
    # Only these .md files allowed at root
    allow: [README.md, AGENTS.md, CHANGELOG.md]
```

**Applied at:** Root level for `.md` files  
**Meaning:** Any other .md file in root is a violation

### Directive: `max_lines`

```yaml
rules:
  src/hooks/:
    .ts: camelCase
    max_lines: 400  # Applies to all .ts in src/hooks/
```

**Applied at:** `src/hooks/` level  
**Meaning:** Files in this directory must be under 400 lines

---

## Inheritance and Override Rules

**1. Most Specific Path Wins**

```yaml
rules:
  .ts: camelCase           # Global default
  src/: 
    .ts: snake_case        # Overrides global for src/
    components/:
      .ts: PascalCase      # Overrides for src/components/
```

- `file.ts` → camelCase (global)
- `src/file.ts` → snake_case (src/ override)
- `src/components/file.ts` → PascalCase (components/ override)

**2. Directives Inherit Downward**

```yaml
rules:
  packages/*:
    max_lines: 400         # Applies to all files in packages/*
    
    src/:
      # Inherits max_lines: 400 from parent
      .ts: camelCase
```

**3. Wildcards Match Any Subdirectory**

```yaml
rules:
  packages/*:               # Matches packages/ui, packages/core, etc.
    require: [README.md]    # Every package needs README
```

---

## Comparison: LS-Lint vs Assura Unified Tree

### LS-Lint
```yaml
ls:
  .ts: camelCase
  src/components:
    .tsx: PascalCase
  packages/*:
    AGENTS.md: exists:1
```

### Assura (Unified Tree)
```yaml
rules:
  .ts: camelCase
  src/components:
    .tsx: PascalCase
  packages/*:
    require:
      files: [AGENTS.md]
```

**Key Differences:**
- Same tree structure
- Assura uses `require:` instead of `exists:1`
- Assura supports directives like `allow`, `max_lines`
- Assura has `groups` for reusability

---

## Directives Reference

Directives can be attached to any node in the tree:

| Directive | Purpose | Example |
|-----------|---------|---------|
| `require` | Files/dirs must exist | `require: [AGENTS.md, src/]` |
| `allow` | Only these files allowed | `allow: [README.md, LICENSE*]` |
| `max_lines` | File size limit | `max_lines: 400` |
| `max_size` | File size in bytes | `max_size: "100KB"` |
| `severity` | Error level | `severity: error` |
| `message` | Custom error message | `message: "Move to docs/"` |

---

## Groups for Efficiency

Groups make the tree more concise by extracting reusable rule sets:

```yaml
# Define reusable groups
groups:
  ts-defaults:
    naming: camelCase
    max_lines: 400
  
  ts-components:
    naming: PascalCase
    max_lines: 400

# Use in tree
rules:
  src/utils/:
    use: ts-defaults        # Applies all rules from group
  
  src/components/:
    use: ts-components
```

**Note:** Groups are expanded at load time - the tree remains unified.

---

## Full Real-World Example

```yaml
# TypeScript monorepo with unified tree

rules:
  # Global defaults
  .dir: kebab-case
  .ts: camelCase
  .tsx: PascalCase
  .test.ts: snake_case
  .test.tsx: snake_case
  
  # Root requirements
  README.md:
    exists: true
  AGENTS.md:
    exists: true
  
  # Root markdown whitelist
  .md:
    naming: kebab-case
    allow: [README.md, AGENTS.md, CHANGELOG.md, LICENSE.md]
  
  # Source code
  src/:
    components/:
      .tsx: PascalCase
      max_lines: 400
      
    hooks/:
      .ts: camelCase
      max_lines: 400
      
    utils/:
      .ts: camelCase
      max_lines: 400
  
  # Package structure
  packages/*:
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src]
    
    src/:
      .ts: camelCase
      
      components/:
        .tsx: PascalCase
  
  # Documentation
  docs/:
    .md:
      naming: kebab-case
      max_lines: 500
      require_frontmatter: true
    
    templates/:
      .md:
        max_lines: 2000

groups:
  ts-standard:
    naming: camelCase
    max_lines: 400

exclude:
  - "node_modules/**"
  - "dist/**"
```

---

## Summary

**Key Principles:**

1. **Unified tree** mirrors directory structure
2. **Directives hang off nodes** in the tree
3. **Most specific path wins** for conflicts
4. **Groups support efficiency** but tree remains primary
5. **Intuitive override visibility** - look at the tree, see the rules

This preserves LS-Lint's best idea (the tree) while adding Assura's powerful directives.

---

*Design follows principle: Unified tree for intuitive structure, directives for powerful constraints*
