---
title: 'Assura Configuration Format - Final Design Specification'
status: historical
---

# Assura Configuration Format: Final Design Specification

**Version:** 1.0 (Unified)  
**Status:** Ready for Implementation  
**Date:** 2026-03-20  

---

## Executive Summary

Assura uses a **unified tree configuration** where:
- `rules` defines reusable rule sets (WHAT)
- `policy` defines the project structure and where rules apply (WHERE)
- `apply` connects rules to policy nodes (HOW)

**Key Principles:**
1. Unified tree mirrors directory structure
2. Every policy node is a path, pattern, or well-known directive
3. No arbitrary directive names in policy tree
4. LS-Lint compatibility maintained where possible
5. NO backwards compatibility with previous Assura versions (pre-1.0)

---

## Top-Level Structure

```yaml
# Reusable rule definitions
rules:
  rule-name:
    # Rule properties

# Project structure and policy application  
policy:
  # Tree structure

# Supporting configuration
exclude:
  - "**/node_modules/**"
```

---

## 1. Rules Section

Defines reusable rule sets that can be applied throughout the policy tree.

### 1.1 Rule Properties

```yaml
rules:
  rule-name:
    # Core properties
    extensions: [ts, tsx]           # File extensions this rule applies to
    naming: camelCase               # Naming convention (or [camelCase, PascalCase])
    
    # Size constraints
    max_lines: 400                  # Maximum lines per file
    max_size: "100KB"              # Maximum file size
    
    # Documentation
    require_docs: true              # Require documentation comments
    
    # Existence requirements (specialized)
    require_test: "{{name}}.test.tsx"  # Pattern for required test file
    
    # Messages
    message:
      violation: "{{filename}} violates {{rule}}"
      why: "Explanation of why this rule exists"
      fix: "How to fix the violation"
      override: "How to request override"
      docs: "URL to documentation"
    
    # Inheritance
    extends: parent-rule            # Inherit from another rule
```

### 1.2 Naming Conventions

Supported values for `naming`:
- `camelCase` - camelCase
- `PascalCase` - PascalCase
- `snake_case` - snake_case
- `kebab-case` - kebab-case
- `SCREAMING_SNAKE_CASE` - SCREAMING_SNAKE_CASE
- `lowercase` - lowercase
- `UPPERCASE` - UPPERCASE
- `[convention1, convention2]` - Array for OR (multiple allowed)

### 1.3 Rule Examples

```yaml
rules:
  # Basic TypeScript rule
  typescript:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 400
  
  # React components (extends typescript)
  react-component:
    extends: typescript
    naming: PascalCase              # Override naming
    max_lines: 300                  # Stricter limit
    require_test: "{{name}}.test.tsx"
    
    message:
      why: "Components should be small and focused"
      fix: "Extract sub-components or hooks"
  
  # Library rules (stricter)
  library-code:
    extends: typescript
    max_lines: 300
    require_docs: true
    
  # Internal tools (relaxed)
  internal-tool:
    extends: typescript
    max_lines: 600
    severity: warn
  
  # Styles
  stylesheet:
    extensions: [css, scss, module.css, module.scss]
    naming: kebab-case
  
  # Component styles (match component name)
  component-styles:
    extensions: [module.css, module.scss]
    naming: PascalCase
  
  # Test files
  unit-test:
    extensions: [test.ts, test.tsx, spec.ts, spec.tsx]
    naming: snake_case
    max_lines: 600
  
  # Documentation
  documentation:
    extensions: [md]
    naming: kebab-case
    max_lines: 500
    require_frontmatter: true
```

---

## 2. Policy Section (Unified Tree)

The policy tree mirrors the project directory structure. Every key is ONE of:

1. **Path** - `src/`, `packages/*/`, `.` (root)
2. **Extension** - `.ts`, `.tsx` (shorthand for all files with extension)
3. **Glob Pattern** - `**/*.test.ts`, `src/**/*.tsx`
4. **Regex Pattern** - `regex:^(README|AGENTS)`
5. **Well-known Directive** - `apply`, `require`, `exists`, `message`, `severity`

### 2.1 Path Keys

Paths use Unix-style separators and support wildcards:

```yaml
policy:
  .:                              # Root directory
    # Root-level rules
  
  src/:                          # Source directory
    # Rules for src/
  
  packages/*/:                   # Wildcard - matches any subdirectory
    # Rules for all packages
  
  packages/ui/:                  # Specific package
    # Rules for packages/ui/ specifically
```

**Specificity Rules:**
- Exact paths beat wildcards
- `packages/ui/` beats `packages/*/`
- Deeper paths beat shallower paths
- `packages/ui/src/` beats `packages/*/src/`

### 2.2 Extension Keys

Extension keys are shorthand for "all files with this extension":

```yaml
policy:
  .ts: camelCase                  # All .ts files use camelCase
  .tsx: PascalCase                # All .tsx files use PascalCase
```

**Equivalent to:**
```yaml
policy:
  "**/*.ts": camelCase
  "**/*.tsx": PascalCase
```

### 2.3 Well-Known Directives

Directives can appear at any node in the tree:

#### 2.3.1 `apply` Directive

Applies rule groups to the current node.

**Shorthand (array of strings):**
```yaml
policy:
  src/utils/:
    apply: [@typescript, @testing-rules]
```

**Object form (with overrides):**
```yaml
policy:
  src/components/:
    apply:
      - @react-component:          # Override specific properties
          naming: PascalCase
          max_lines: 250
      - @typescript-standard       # No overrides
```

**Mixed form:**
```yaml
policy:
  src/pages/:
    apply:
      - @typescript
      - @react-component:
          max_lines: 400
      - @styling-rules
```

#### 2.3.2 `require` Directive

Specifies required files or directories at the current level.

**Simple list:**
```yaml
policy:
  packages/*/:
    require: [AGENTS.md, README.md, package.json, src/]
```

**With messages:**
```yaml
policy:
  packages/*/:
    require:
      files:
        - AGENTS.md
        - README.md
        - package.json
      dirs:
        - src/
      message: "Packages must have documentation and source directory"
```

**With severity:**
```yaml
policy:
  root:
    require:
      files: [README.md]
      severity: error
      message: "Root README.md is required"
```

#### 2.3.3 `exists` Directive

Alias for `require`. Use whichever reads better.

```yaml
policy:
  packages/*/:
    exists: [AGENTS.md, README.md]    # Same as require
```

#### 2.3.4 `message` Directive

Custom messages for violations at this level.

```yaml
policy:
  src/generated/:
    apply: [@typescript]
    message:
      violation: "Generated code issue"
      why: "This is auto-generated code"
      fix: "Regenerate from source, do not edit manually"
```

#### 2.3.5 `severity` Directive

Override default severity.

```yaml
policy:
  src/legacy/:
    apply: [@typescript]
    severity: warn              # Don't fail on legacy code
```

### 2.4 Policy Tree Example

```yaml
policy:
  # Root level
  .:
    require: [README.md, AGENTS.md, LICENSE*]
    
    .md:
      allow: [README.md, AGENTS.md, CHANGELOG.md, LICENSE.md]
      deny_message: "Move markdown files to docs/"
  
  # Global defaults by extension
  .ts: camelCase
  .tsx: PascalCase
  .test.ts: snake_case
  .css: kebab-case
  
  # Source code
  src/:
    apply: [@typescript-standard]
    
    components/:
      apply:
        - @react-component:
            max_lines: 300
        - @component-styles
      
      # Atomic components (stricter)
      atoms/:
        apply:
          - @react-component:
              max_lines: 150
    
    hooks/:
      apply:
        - @typescript-standard:
            max_lines: 200
    
    utils/:
      apply: [@typescript-standard]
    
    generated/:
      apply:
        - @typescript-standard:
            naming: [camelCase, PascalCase, snake_case]
            max_lines: 5000
            severity: warn
  
  # Packages
  packages/*/:
    require: [AGENTS.md, README.md, package.json, src/]
    
    src/:
      apply: [@library-code]
      
      components/:
        apply: [@react-component]
  
  # Apps
  apps/web/:
    apply: [@app-code]
    
    src/:
      apply: [@app-code]
      
      components/:
        apply: [@react-component]
  
  # Documentation
  docs/:
    apply: [@documentation]
    
    templates/:
      apply:
        - @documentation:
            max_lines: 2000
            require_frontmatter: false
```

---

## 3. LS-Lint Compatibility

### 3.1 Config Conversion

LS-Lint config can be automatically converted:

**LS-Lint:**
```yaml
ls:
  .ts: camelCase
  src/components:
    .tsx: PascalCase
```

**Assura Equivalent:**
```yaml
policy:
  .ts: camelCase
  src/components/:
    .tsx: PascalCase
```

### 3.2 Compatibility Layer

- Support `.ls-lint.yml` format via automatic conversion
- `ls:` key maps to `policy:` root
- `ignore:` maps to `exclude:`
- `exists:1` maps to `require: [file]`

### 3.3 No Assura Version Compatibility

**IMPORTANT:** No backwards compatibility with previous Assura config versions.
- This is pre-1.0
- Only the unified format described here is supported
- No `version` field in config
- Migration tool can convert old configs

---

## 4. Variable Replacement

Variables available in patterns and messages:

| Variable | Example | Description |
|----------|---------|-------------|
| `{{name}}` | `Button` from `Button.tsx` | Filename without extension |
| `{{ext}}` | `tsx` from `Button.tsx` | Extension only |
| `{{filename}}` | `Button.tsx` | Full filename |
| `{{base}}` | `Button.test` from `Button.test.tsx` | Filename without final ext |
| `{{dir}}` | `components` from `src/components/Button.tsx` | Immediate parent directory |
| `{{path}}` | `components/Button` from `src/components/Button.tsx` | Relative path without ext |

### 4.1 Usage in Rules

```yaml
rules:
  component:
    extensions: [tsx]
    require_test: "{{name}}.test.tsx"
    message:
      violation: "{{filename}} missing test file"
      fix: "Create {{name}}.test.tsx"
```

### 4.2 Usage in Policy

```yaml
policy:
  src/components/:
    require: ["{{name}}.test.tsx"]    # Each .tsx requires matching test
```

---

## 5. Complete Real-World Example

```yaml
# ============================================================================
# RULES: Reusable rule definitions
# ============================================================================

rules:
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
    severity: warn
  
  # React components
  react-comp:
    extends: ts-app
    naming: PascalCase
    require_test: "{{name}}.test.tsx"
    message:
      fix: "Split into smaller components or extract hooks"
  
  # Global styles
  styles-global:
    extensions: [css, scss]
    naming: kebab-case
  
  # Component styles
  styles-component:
    extensions: [module.css, module.scss]
    naming: PascalCase
  
  # Unit tests
  test-unit:
    extensions: [test.ts, test.tsx, spec.ts, spec.tsx]
    naming: snake_case
    max_lines: 600
  
  # E2E tests
  test-e2e:
    extensions: [e2e.ts]
    naming: snake_case
    max_lines: 1000
  
  # Documentation
  docs-standard:
    extensions: [md]
    naming: kebab-case
    max_lines: 500
    require_frontmatter: true
  
  # Templates (can be long)
  docs-template:
    extends: docs-standard
    max_lines: 2000
    require_frontmatter: false

# ============================================================================
# POLICY: Project structure and rule application
# ============================================================================

policy:
  # Root requirements
  README.md:
    exists: true
    message: "Add root README.md with project description"
  
  AGENTS.md:
    exists: true
    severity: warn
    message: "Recommended: Add AGENTS.md for agent guidance"
  
  # Root markdown whitelist
  .md:
    allow: [README.md, AGENTS.md, CHANGELOG.md, LICENSE.md, CODE_OF_CONDUCT.md]
    deny_message: "Move markdown files to docs/ directory"
  
  # Global defaults
  .ts: camelCase
  .tsx: PascalCase
  .test.ts: snake_case
  .css: kebab-case
  
  # Source code
  src/:
    apply: [@ts-app]
    
    components/:
      apply:
        - @react-comp:
            max_lines: 300
        - @styles-component
      
      atoms/:
        apply:
          - @react-comp:
              max_lines: 150
    
    hooks/:
      apply:
        - @ts-app:
            max_lines: 200
    
    utils/:
      apply: [@ts-app]
    
    generated/:
      apply:
        - @ts-app:
            naming: [camelCase, PascalCase, snake_case]
            max_lines: 5000
            severity: warn
      message:
        fix: "Do not manually edit. Regenerate from source."
  
  # Libraries
  packages/*/:
    require: [AGENTS.md, README.md, package.json, src/]
    
    src/:
      apply: [@ts-lib]
      
      components/:
        apply: [@react-comp]
  
  # Apps
  apps/*/:
    require: [README.md, src/]
    
    src/:
      apply: [@ts-app]
      
      components/:
        apply: [@react-comp]
  
  # Documentation
  docs/:
    apply: [@docs-standard]
    
    templates/:
      apply: [@docs-template]
  
  # Internal tools
  internal/:
    apply: [@ts-internal]

# ============================================================================
# EXCLUSIONS
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
  - "**/*.min.js"
