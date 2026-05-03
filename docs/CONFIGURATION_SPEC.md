# Assura Configuration Specification

## Core Principle: Structure-First Representation

Configuration structure **must visually mirror** the actual project file tree. Every file key represents a real location in the project.

---

## File Keys

Files are represented as **keys** in the YAML structure, not values:

```yaml
# ✅ Correct - Files as keys
src/components/
  Button.tsx:
  Button.test.tsx:

# ❌ Wrong - Files in arrays (obscures structure)
files: [Button.tsx, Button.test.tsx]
```

### File Key Types

1. **Literal files**: Exact filename
   ```yaml
   README.md:
   Cargo.toml:
   ```

2. **Extension patterns**: All files with extension
   ```yaml
   .tsx: PascalCase
   .rs: snake_case
   ```

3. **Variable patterns**: Capture basename for pairing
   ```yaml
   ${name}.tsx:
   ${name}.test.tsx:
   ```

4. **Glob patterns**: Multiple matches
   ```yaml
   *.config.js:
   ```

---

## Conventions (Direct Values)

Naming conventions are **direct values** after the colon, matching ls-lint notation:

```yaml
# Single convention
.tsx: PascalCase
.rs: snake_case

# Multiple conventions (OR)
.ts: camelCase | PascalCase

# With exists directive
README.md: exists:1
AGENTS.md: exists:1
```

### Available Conventions

| Convention | Example | Matches |
|------------|---------|---------|
| `camelCase` | `myVariable` | `buttonGroup` |
| `PascalCase` | `MyClass` | `ButtonGroup` |
| `snake_case` | `my_variable` | `button_group` |
| `kebab-case` | `my-variable` | `button-group` |
| `SCREAMING_SNAKE_CASE` | `MY_CONSTANT` | `BUTTON_GROUP` |
| `flatcase` | `myvariable` | `buttongroup` |
| `UPPERCASE` | `MYVARIABLE` | `BUTTONGROUP` |
| `lower.case` | `my.variable` | `button.group` |
| `UPPER.CASE` | `MY.VARIABLE` | `BUTTON.GROUP` |

---

## Directives (Array Items)

When a file key needs multiple attributes, use **array notation** with directives:

```yaml
src/components/
  ${name}.tsx:
    - apply: react          # Apply rule
    - lines: ..400           # Max lines
    - violation: [warn, ci:block]  # Context-aware enforcement
```

### Core Directives

#### `apply` - Apply Rule Set
```yaml
${name}.tsx:
  - apply: react
```

#### `exists` - Require File/Directory
```yaml
# Require file exists
README.md:
  - exists: 1

# Require directory exists  
src/:
  - exists: 1

# Exists with message
AGENTS.md:
  - exists: 1
  - message: "Add AGENTS.md for project guidance"
```

#### `lines` - Line Count Constraint
```yaml
# Maximum lines
.tsx:
  - lines: ..400

# Range
.rs:
  - lines: 50..500
```

#### `violation` - Enforcement Behavior
```yaml
# Default behavior
.tsx:
  - violation: [warn]

# Context-aware
.tsx:
  - violation: [tool:warn, ci:block, feature:warn]
```

#### `message` - Custom Violation Messages
```yaml
.tsx:
  - violation: [warn]
  - message:
      warn: "Consider refactoring"
      fix: "Extract to smaller components"
```

---

## Cross-Directory Pairing

Files are paired by **shared variables** across the structure:

### Implicit Pairing (Same Variable Name)

```yaml
policy:
  src/components/
    ${name}.tsx:          # Captures "Button"
      - apply: react
      
  tests/components/
    ${name}.test.tsx:     # Same variable, pairs automatically
      - exists: 1
```

### Explicit Group Key (Multiple Patterns)

```yaml
  src/components/
    ${name}.tsx:
      - apply: react
      - group: component    # Pairing key
      
  tests/components/
    ${name}.test.tsx:
      - exists: 1
      - group: component    # Same key = paired
      
  docs/components/
    ${name}.md:
      - exists: 1
      # No group = standalone
```

---

## Test Organization Patterns

### Pattern 1: Co-located (Flat)
```yaml
src/components/
  ${name}.tsx:
    - apply: react
  ${name}.test.tsx:
    - exists: 1
```

### Pattern 2: Centralized Tests
```yaml
src/components/
  ${name}.tsx:
    - apply: react
    
tests/components/
  ${name}.test.tsx:
    - exists: 1
```

### Pattern 3: Folder-Per-Component
```yaml
src/components/
  ${name}/
    index.tsx:
      - exists: 1
    ${name}.tsx:
      - exists: 1
    ${name}.test.tsx:
      - exists: 1
```

### Pattern 4: Nested Tests
```yaml
src/components/
  ${name}/
    index.tsx:
      - exists: 1
    __tests__/
      ${name}.test.tsx:
        - exists: 1
```

---

## Rules (Reusable Definitions)

Define reusable rule sets in `rules:`, apply them in `policy:`:

```yaml
rules:
  react:
    .tsx: PascalCase
    lines: ..400
    
  typescript:
    .ts: camelCase
    lines: ..500

policy:
  src/components/
    - apply: react
    
  src/utils/
    - apply: typescript
```

---

## Context-Aware Enforcement

Define contexts, apply context-specific behavior:

```yaml
contexts:
  tool: hook: tool
  ci: hook: ci
  hotfix: hook: pre-commit, branch: "hotfix/*"

policy:
  src/components/
    ${name}.tsx:
      - apply: react
      - violation: [tool:warn, ci:block, hotfix:warn]
```

---

## Root File Representation

Special files (README.md, AGENTS.md, etc.) are represented as literal keys at the root level:

```yaml
./:
  README.md: exists:1
  AGENTS.md: exists:1
  CONSTITUTION.md:
  LICENSE:
  CHANGELOG.md:
```

Literal filenames that don't match naming conventions are implicitly allowed by their presence in the structure.

---

## Constitutional Compliance Checklist

- [ ] **Files as keys**: Every file appears as a YAML key at its actual location
- [ ] **Structure mirrors project**: Directory nesting matches actual structure
- [ ] **Variables for pairing**: `${name}` links related files across directories
- [ ] **Conventions as values**: Naming rules are direct values (`.tsx: PascalCase`)
- [ ] **Directives in arrays**: Complex behavior uses `-` notation under file keys
- [ ] **No path references**: Never use directives that point to other locations
- [ ] **Visible relationships**: Both source and target locations are visible in structure

---

## Key Files

- **CONSTITUTION.md** - Project principles and constraints
- **.assura/config.yml** - Current active configuration
- **docs/LS_LINT_NOTATION_GUIDE.md** - LS-Lint compatibility reference

---

*This specification defines the Assura configuration format for Structure-First Representation.*
