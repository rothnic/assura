---
title: 'Assura vs LS-Lint - Notation Guide'
status: historical
---

# Assura vs LS-Lint: Notation Guide

This document maps LS-Lint notation to Assura notation, highlighting extensions, deviations, and key use cases.

---

## 1. LS-Lint Foundation

Assura is built on LS-Lint's proven patterns with extensions for modern development workflows.

### 1.1 LS-Lint Core Concepts

**Extension-based rules:**
```yaml
# LS-Lint
ls:
  .rs: snake_case
  .tsx: PascalCase
```

**Path-specific rules:**
```yaml
# LS-Lint
ls:
  src/components/*: PascalCase
  src/utils/*: camelCase
```

**Multiple conventions (OR):**
```yaml
# LS-Lint
ls:
  .ts: camelCase | PascalCase
```

**Exists directive:**
```yaml
# LS-Lint
ls:
  .dir: exists:1
  README.md: exists:1
```

---

## 2. Assura Extensions

### 2.1 Structure-First Representation

Unlike LS-Lint's flat `ls:` key, Assura uses a tree structure matching your project:

```yaml
# Assura
policy:
  src/:
    .rs: snake_case
    
    components/:
      .tsx: PascalCase
```

**Benefits:**
- Visual fidelity with project structure
- Natural inheritance down the tree
- Easier to understand scope of rules

### 2.2 Rules (Reusable Patterns)

LS-Lint requires duplication. Assura uses `rules` for reuse:

```yaml
# LS-Lint (duplication)
ls:
  packages/ui/.tsx: PascalCase
  packages/app/.tsx: PascalCase
  packages/lib/.tsx: PascalCase

# Assura (reusable)
rules:
  react:
    .tsx:
      - PascalCase
      - lines: ..400

policy:
  packages/:
    ui/:
      - apply: react
    app/:
      - apply: react
    lib/:
      - apply: react
```

**Array Notation Equivalence:**

Both flow and block styles are equivalent:
```yaml
# Flow style (concise)
apply: [typescript, tested]
violation: [warn, ci:block]

# Block style (readable)
apply:
  - typescript
  - tested

violation:
  - warn
  - ci:block
```

### 2.3 File Pairing (The Critical Gap)

LS-Lint cannot enforce: "Each .tsx must have a matching .test.tsx"

**Assura solution - Variable capture:**
```yaml
policy:
  src/components/:
    ${name}.tsx:       # Captures "Button" from "Button.tsx"
      - apply: react
    
    ${name}.test.tsx:  # Same variable, creates pairing
      - exists: 1
```

**Centralized tests:**
```yaml
policy:
  src/components/:
    ${name}.tsx:
      - apply: react

  tests/components/:
    ${name}.test.tsx:
      - exists: 1
```

### 2.4 Content Validation

LS-Lint only validates naming. Assura validates content:

```yaml
rules:
  sized:
    ${name}.tsx:
      - PascalCase
      - lines: ..400
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: sized
```

### 2.5 Context-Aware Violations

LS-Lint has one behavior. Assura adapts to context:

```yaml
contexts:
  tool: hook: tool
  ci: hook: ci
  feature: hook: pre-commit, branch: "feature/*"

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
        violation: [warn, ci:block, feature:warn]
```

**Violation syntax:** `[default, context:value, ...]`
- `warn` - Show warning, allow
- `block` - Stop operation
- `notify` - Silent notification

### 2.6 Messaging Extensions

Provide helpful guidance when violations occur:

```yaml
policy:
  src/components/:
    ${name}.tsx:
      - apply: react
        violation: [warn, ci:block]
        message:
          warn: "Consider refactoring this component"
          ci: "Components must be under 400 lines. See docs/refactoring.md"
          fix: "Extract logic to separate files"
          docs: "https://docs.project.com/components"
```

**Message fields:**
- `warn`/`block`/`notify` - Context-specific messages
- `fix` - Suggested fix
- `docs` - Link to documentation
- `override` - How to override (e.g., "Requires @owner approval")

### 2.7 Context Inheritance

Set defaults high in the tree, override low:

```yaml
policy:
  src/:
    # Default for all of src/ and below
    - violation: [warn, ci:block]
    
    components/:
      # Inherits [warn, ci:block] from parent
      
      core/:
        # Override for core/ directory
        - violation: [block]
```

---

## 3. Key Use Cases

### 3.1 Component with Required Test

**Goal:** Each component must have a test file.

```yaml
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
    
    ${name}.test.tsx:
      - exists: 1
      - violation: [block]
```

### 3.2 Small vs Large Component Strategy

**Goal:** Small components flat, large components nested.

```yaml
policy:
  src/components/:
    # Small components (flat)
    ${name}.tsx:
      - constraints: [PascalCase, lines:..100]
      - violation: [warn]
    
    # Large components (nested directory)
    ${name}/:
      index.tsx:
        - exists: 1
      ${name}.tsx:
        - exists: 1
      ${name}.test.tsx:
        - exists: 1
```

### 3.3 Strict Directory (Whitelist)

**Goal:** Only specific files allowed.

**LS-Lint approach:**
```yaml
ls:
  src/core/:
    .*: exists:0          # No other files allowed
    lib.rs: exists:1      # Exactly 1 lib.rs
    mod.rs: exists:1      # Exactly 1 mod.rs
```

**Assura approach:**
```yaml
policy:
  src/core/:
    - strict: true        # Only listed files allowed
    
    lib.rs:
      - exists: 1
    
    mod.rs:
      - exists: 1
```

### 3.4 Monorepo with Different Rules

**Goal:** Different packages use different conventions.

```yaml
rules:
  frontend:
    .tsx: PascalCase
    lines: ..400
  
  backend:
    .rs: snake_case
    lines: ..500

policy:
  packages/:
    frontend/:
      - apply: frontend
    
    backend/:
      - apply: backend
```

### 3.5 Progressive Enforcement

**Goal:** Warn in development, block in CI.

```yaml
contexts:
  tool: hook: tool
  ci: hook: ci

policy:
  src/:
    # Set violations for tool and CI contexts at this level
    - violation: [tool:warn, ci:block]
    
    ${name}.tsx:
      - apply: react
```

### 3.6 Custom Messages

**Goal:** Provide helpful guidance on violations.

```yaml
policy:
  src/components/:
    ${name}.tsx:
      - apply: react
        violation: [warn, ci:block]
        message:
          warn: "Consider refactoring large components"
          block: "Components must be under 400 lines. See docs/refactoring.md"
```

### 3.7 Emergency Override

**Goal:** Allow emergency hotfixes to bypass some rules.

```yaml
contexts:
  hotfix:
    hook: pre-commit
    branch: "hotfix/*"

policy:
  src/components/:
    - context: hotfix
      violation: warn    # Downgrade to warnings
    
    ${name}.tsx:
      - apply: react
        violation: [block, hotfix:warn]  # But tests still block
```

---

## 4. Notation Comparison Table

| Concept | LS-Lint | Assura |
|---------|---------|--------|
| Extension rule | `.rs: snake_case` | `.rs: snake_case` |
| Path rule | `src/*: PascalCase` | `src/: .tsx: PascalCase` |
| OR conventions | `camelCase \| PascalCase` | `camelCase \| PascalCase` |
| Reusable rules | Not supported | `rules:` + `apply:` |
| File pairing | Not supported | `${name}.tsx` + `${name}.test.tsx` |
| Line limits | Not supported | `lines: ..400` |
| Context awareness | Not supported | `violation: [warn, ci:block]` |
| Messaging | Not supported | `message: {warn: "...", fix: "..."}` |
| Exists directive | `.dir: exists:1` | `exists: 1` |
| Strict mode | `.*: exists:0` | `strict: true` |
| Strict mode | Not supported | `strict: true` |

---

## 5. Top-Level Structure

All Assura configs have these top-level keys:

```yaml
rules:       # Reusable pattern definitions
contexts:    # When/where contexts run  
messages:    # Optional: Reusable message templates
policy:      # Tree structure matching project (required)
```

---

## 6. Migration from LS-Lint

### Step 1: Convert flat structure to tree

```yaml
# LS-Lint
ls:
  .rs: snake_case
  src/components/*: PascalCase

# Assura
policy:
  .rs: snake_case
  src/components/: .tsx: PascalCase
```

### Step 2: Extract reusable rules

```yaml
# LS-Lint (repeated)
ls:
  packages/ui/.tsx: PascalCase
  packages/app/.tsx: PascalCase

# Assura
rules:
  react: .tsx: PascalCase

policy:
  packages/:
    ui/: apply: react
    app/: apply: react
```

### Step 3: Add missing features

```yaml
# Add file pairing
${name}.tsx:
  - apply: react

${name}.test.tsx:
  - exists: 1

# Add messages
${name}.tsx:
  - apply: react
    message:
      warn: "Consider refactoring"
```

---

## 7. Deviations from LS-Lint

### 7.1 No `ls:` Key
Assura uses `policy:` as the root to indicate we're doing more than file naming.

### 7.2 No `.dir` Pseudo-Extension
Assura uses trailing `/` for directories: `src/` not `.dir: exists:1`

### 7.3 Array Notation for Directives
LS-Lint uses flat keys. Assura uses `-` array items for non-structural elements:
```yaml
# LS-Lint
ls:
  .tsx: PascalCase

# Assura
policy:
  ${name}.tsx:
    - apply: typescript
```

### 7.4 Variable Substitution
LS-Lint uses `${0}`, `${1}` for directories. Assura uses `${name}` for filenames (more intuitive for pairing).

### 7.5 Violation Array
LS-Lint has one severity. Assura uses arrays: `[warn, ci:block, feature:warn]`

### 7.6 Messaging
LS-Lint has no messaging. Assura adds `message:` with context-specific guidance.

---

## 8. Performance

**Target:** 2x faster than LS-Lint for equivalent workloads.

**Strategy:**
- Glob patterns for fast discovery
- Parallel directory traversal
- Compiled pattern matching
- Two-pass evaluation (index then validate)

---

*Last Updated: 2026-03-24*
*For detailed notation specifications, see SPEC.md*
*For core principles, see CONSTITUTION.md*
