---
title: 'Apply Directive - Scalability & Flexibility Analysis'
status: historical
---

# Apply Directive: Scalability & Flexibility Analysis

**Design Decision:** Use `apply` directive for rule application with YAML-native array notation.

---

## Core Syntax

```yaml
# Simple application
apply:
  - @rule-group

# With overrides
apply:
  - @rule-group:
      max_lines: 5000

# Multiple groups
apply:
  - @rule-group-1
  - @rule-group-2:
      severity: warn
```

---

## Scalability Checklist

### ✅ 1. Simple Projects (Single File Type)

```yaml
groups:
  typescript:
    extensions: [ts]
    naming: camelCase

rules:
  src/:
    apply:
      - @typescript
```

**Lines:** 8 | **Complexity:** Low | **Verdict:** ✅ Scales down

---

### ✅ 2. Multiple File Types

```yaml
groups:
  typescript:
    extensions: [ts, tsx]
    naming: camelCase
  
  styles:
    extensions: [css, scss]
    naming: kebab-case
  
  docs:
    extensions: [md]
    naming: kebab-case

rules:
  src/:
    apply:
      - @typescript
      - @styles
  
  docs/:
    apply:
      - @docs
```

**Lines:** 16 | **Complexity:** Low | **Verdict:** ✅ Clean

---

### ✅ 3. Path-Specific Overrides

```yaml
rules:
  # Global default
  .ts: camelCase
  
  # Component override
  src/components/:
    apply:
      - @typescript:
          naming: PascalCase  # Override to PascalCase
  
  # Hooks override
  src/hooks/:
    apply:
      - @typescript:
          naming: camelCase   # Explicit (same as default)
          max_lines: 200       # Stricter limit
```

**Verdict:** ✅ Overrides work at any level

---

### ✅ 4. Monorepo (Multiple Packages)

```yaml
groups:
  app-rules:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 400
  
  lib-rules:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 300
    require_docs: true
  
  internal-rules:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 600

rules:
  # Apps
  apps/web/:
    apply:
      - @app-rules
  
  apps/admin/:
    apply:
      - @app-rules
  
  # Libraries
  packages/ui/:
    apply:
      - @lib-rules
  
  packages/utils/:
    apply:
      - @lib-rules
  
  # Internal tools
  internal/:
    apply:
      - @internal-rules
```

**Lines:** 35 | **Complexity:** Medium | **Verdict:** ✅ Scales to monorepo

---

### ✅ 5. Deep Nesting (Many Levels)

```yaml
rules:
  packages/*/:
    apply:
      - @base-rules
    
    src/:
      apply:
        - @src-rules
      
      components/:
        apply:
          - @component-rules:
              naming: PascalCase
      
        Button/:
          apply:
            - @component-rules:
                naming: PascalCase
                max_lines: 200  # Even stricter for atoms
        
        Card/:
          apply:
            - @component-rules:
                max_lines: 300  # Different for molecules
```

**Depth:** 5 levels | **Verdict:** ✅ Maintains clarity at depth

---

### ✅ 6. Wildcard Patterns

```yaml
rules:
  # All packages
  packages/*:
    apply:
      - @package-rules
  
  # Specific package
  packages/core:
    apply:
      - @critical-rules:
          severity: error
  
  # All test files
  "**/*.test.ts":
    apply:
      - @test-rules
  
  # E2E tests only
  "**/e2e/*.test.ts":
    apply:
      - @test-rules:
          max_lines: 1000
```

**Verdict:** ✅ Works with wildcards and patterns

---

### ✅ 7. Required Files (exists directive)

```yaml
rules:
  packages/*/:
    # Directives alongside apply
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src, tests]
    
    apply:
      - @package-rules
  
  # Root requirements
  .:
    README.md:
      exists: true
    AGENTS.md:
      exists: true
```

**Verdict:** ✅ `apply` coexists with other directives

---

### ✅ 8. Root File Whitelist

```yaml
rules:
  .:
    # Root-level constraints
    .md:
      allow: [README.md, AGENTS.md, CHANGELOG.md]
    
    apply:
      - @root-rules
```

**Verdict:** ✅ Works at root level

---

### ✅ 9. Multiple Overrides

```yaml
rules:
  src/generated/:
    apply:
      - @typescript-rules:
          naming: [camelCase, PascalCase, snake_case]
          max_lines: 5000
          severity: warn
          require_docs: false
```

**Verdict:** ✅ Multiple properties overridable

---

### ✅ 10. Conditional Application

```yaml
rules:
  src/:
    # Default for src
    apply:
      - @standard-rules
    
    # Override for specific subdirectory
    legacy/:
      apply:
        - @standard-rules:
            severity: warn  # Don't fail on legacy
```

**Verdict:** ✅ Conditional based on tree position

---

## Complex Real-World Example

Full monorepo with all features:

```yaml
# ============================================================================
# Groups: Reusable rule sets
# ============================================================================

groups:
  # Base TypeScript
  ts-base:
    extensions: [ts, tsx]
    naming: camelCase
  
  # Application code
  ts-app:
    extends: ts-base
    max_lines: 400
  
  # Library code (stricter)
  ts-lib:
    extends: ts-base
    max_lines: 300
    require_docs: true
  
  # Internal tools (relaxed)
  ts-internal:
    extends: ts-base
    max_lines: 600
  
  # React components
  react-comp:
    extends: ts-app
    naming: PascalCase
  
  # Global styles
  styles-global:
    extensions: [css, scss]
    naming: kebab-case
  
  # Component styles (match component name)
  styles-component:
    extensions: [css, scss, module.css, module.scss]
    naming: PascalCase
  
  # Unit tests
  tests-unit:
    extensions: [test.ts, test.tsx, spec.ts, spec.tsx]
    naming: snake_case
    max_lines: 600
  
  # E2E tests (longer)
  tests-e2e:
    extensions: [test.ts, test.tsx]
    naming: snake_case
    max_lines: 1000
  
  # Documentation
  docs-standard:
    extensions: [md]
    naming: kebab-case
    max_lines: 500
    require_frontmatter: true
  
  # Templates (can be long)
  docs-templates:
    extensions: [md]
    naming: kebab-case
    max_lines: 2000
    require_frontmatter: false

# ============================================================================
# Rules: Unified tree
# ============================================================================

rules:
  # Root requirements
  README.md:
    exists: true
    severity: error
  
  AGENTS.md:
    exists: true
    severity: warn
  
  # Root markdown whitelist
  .md:
    allow: [README.md, AGENTS.md, CHANGELOG.md, LICENSE.md, CODE_OF_CONDUCT.md]
    deny_message: "Move markdown files to docs/"
  
  # Global defaults
  apply:
    - @ts-base
  
  # ============================================================================
  # Applications
  # ============================================================================
  
  apps/web/:
    apply:
      - @ts-app
      - @styles-global
    
    src/:
      apply:
        - @ts-app
      
      components/:
        apply:
          - @react-comp
          - @styles-component
        
        # Atomic components (atoms)
        atoms/:
          apply:
            - @react-comp:
                max_lines: 150  # Atoms should be tiny
      
      hooks/:
        apply:
          - @ts-app:
              max_lines: 200  # Hooks should be focused
      
      utils/:
        apply:
          - @ts-app
      
      styles/:
        apply:
          - @styles-global
      
      generated/:
        apply:
          - @ts-app:
              naming: [camelCase, PascalCase, snake_case]
              max_lines: 5000
              severity: warn
    
    tests/:
      apply:
        - @tests-e2e
  
  apps/admin/:
    apply:
      - @ts-app
    
    src/:
      apply:
        - @ts-app
      
      components/:
        apply:
          - @react-comp
  
  # ============================================================================
  # Libraries (packages)
  # ============================================================================
  
  packages/*/:
    # Package structure requirements
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src, tests]
      severity: error
    
    apply:
      - @ts-lib
  
  packages/ui/:
    src/:
      apply:
        - @ts-lib
        - @styles-component
      
      components/:
        apply:
          - @react-comp
        
        Button/:
          apply:
            - @react-comp:
                max_lines: 200
      
      # Colocated unit tests
      "**/*.test.ts":
        apply:
          - @tests-unit
  
  packages/utils/:
    src/:
      apply:
        - @ts-lib
  
  packages/types/:
    src/:
      apply:
        - @ts-lib
      
      # Type definitions can be longer
      apply:
        - @ts-lib:
            max_lines: 500
  
  # ============================================================================
  # Internal tools
  # ============================================================================
  
  internal/:
    apply:
      - @ts-internal
    
    scripts/:
      apply:
        - @ts-internal:
            severity: warn
    
    migrations/:
      apply:
        - @ts-internal
  
  # ============================================================================
  # Documentation
  # ============================================================================
  
  docs/:
    apply:
      - @docs-standard
  
  docs/guides/:
    apply:
      - @docs-standard
  
  docs/api/:
    apply:
      - @docs-templates:
          require_frontmatter: true  # API docs need structure
  
  docs/templates/:
    apply:
      - @docs-templates

# ============================================================================
# Exclusions
# ============================================================================

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/build/**"
  - "**/.next/**"
  - "**/out/**"
  - "**/coverage/**"
  - "**/.turbo/**"
  - "**/.cache/**"
```

---

## Scalability Metrics

| Metric | Result |
|--------|--------|
| **Lines for simple project** | 8-15 ✅ |
| **Lines for monorepo** | 150-250 ✅ (manageable) |
| **Max nesting depth** | Unlimited ✅ (tree structure) |
| **Override flexibility** | Full ✅ (any property) |
| **Learning curve** | Low ✅ (YAML-native) |
| **Error clarity** | High ✅ (explicit structure) |

---

## Flexibility Analysis

### ✅ Works With:

1. **Extension lists** - `extensions: [ts, tsx, js]`
2. **Pattern matching** - `"**/*.test.ts"`
3. **Wildcards** - `packages/*`
4. **Deep nesting** - 5+ levels
5. **Multiple groups** - `apply: [@g1, @g2, @g3]`
6. **Property overrides** - `max_lines: 5000`
7. **Nested overrides** - `components/atoms/` different from `components/`
8. **Directives coexistence** - `apply` + `require` + `allow`
9. **Group inheritance** - `extends: ts-base`
10. **Severity control** - per-override severity

### ✅ Future-Proof:

1. **New directives** - Add alongside `apply`
2. **New properties** - Extend groups
3. **Plugin rules** - Groups can include plugin-defined rules
4. **Conditional logic** - Can add `when:` later
5. **Import/export** - Groups can be imported from files

---

## Comparison: Apply vs Alternatives

| Feature | `apply` | `use: [@g]` | `@g: true` |
|---------|---------|-------------|------------|
| Simple case | ✅ `- @group` | ✅ `- @group` | ✅ `@group: true` |
| Override | ✅ `- @g: {prop}` | ⚠️ Verbose | ⚠️ Separate syntax |
| Multiple groups | ✅ Array | ✅ Array | ⚠️ Repeats key |
| Consistency | ✅ Always same | ✅ Always same | ❌ Mixed types |
| YAML-native | ✅ Yes | ✅ Yes | ⚠️ Boolean vs object |
| Scales to complex | ✅ Yes | ✅ Yes | ⚠️ Confusing |

---

## Verdict

**`apply` directive with YAML array notation scales to all requirements:**

✅ Simple projects (8 lines)  
✅ Monorepos (250 lines, manageable)  
✅ Deep nesting (5+ levels)  
✅ Path-specific overrides  
✅ Wildcard patterns  
✅ Required files  
✅ Root constraints  
✅ Progressive complexity  
✅ Future extensibility  

**The syntax is:**
- **Intuitive** - matches YAML patterns developers know
- **Consistent** - same pattern from simple to complex
- **Flexible** - handles all requirements
- **Scalable** - grows with project complexity

---

*Apply directive is production-ready for all use cases*
