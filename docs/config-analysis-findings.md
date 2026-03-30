
---

## Revised Philosophy: Config Efficiency First

### Guiding Principle

**Assura should support extended features but NOT require verbose representations.**

We don't need to mimic every LS-Lint directive exactly. Instead:
- ✅ Support the same USE CASES (require files exist, validate structure)
- ✅ Design our own efficient syntax
- ✅ Concise by default, verbose when necessary
- ✅ Simple things simple, complex things possible

### What This Means

**Don't Copy LS-Lint Syntax Blindly:**

LS-Lint PR #355:
```yaml
packages/*:
  AGENTS.md: exists:1
  README.md: exists:1
```

Assura Efficient Syntax (proposed):
```yaml
structure:
  packages/*:
    require: [AGENTS.md, README.md, src/]
```

Or even more concise:
```yaml
structure:
  packages/*:
    must_have: [AGENTS.md, README.md, src/]
```

**Key Difference:**
- LS-Lint uses `exists:1` (count-based, verbose)
- Assura can use `require: [...]` (list-based, concise)
- Same capability, more efficient representation

### Efficient vs Verbose

**Efficient (Default):**
```yaml
structure:
  src/:
    files: { naming: snake_case, max_lines: 500 }
    require: [lib.rs, mod.rs]
```

**Verbose (When Needed):**
```yaml
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
      severity: high
    require:
      files:
        - name: lib.rs
          severity: critical
          message: "Library entry point required"
        - name: mod.rs
          severity: high
```

### Which LS-Lint Features to Adopt (Capability, Not Syntax)

| LS-Lint Capability | Assura Syntax (Efficient) | Priority |
|-------------------|---------------------------|----------|
| Required files | `require: [file1, file2]` | High |
| Required dirs | `require: [dir/, dir2/]` | High |
| Extension rules | `patterns: {"**/*.rs": snake_case}` | High |
| OR syntax | `naming: "kebab-case\|snake_case"` | Already have |
| Rule groups | `use: @group-name` | Medium |
| Max lines | `max_lines: 500` | Already have |
| Content validation | `content: { patterns: [...] }` | Low |
| Context-aware | Handle at CLI layer | Low |

### What We DON'T Need

❌ **Don't implement:**
- `exists:0-1` (bounded ranges) - YAGNI
- `exists:1-4` (count ranges) - Overly complex
- Rule-level context definitions - Handle at CLI/hooks
- Complex content validation - Out of scope for file naming tool
- LS-Lint's exact YAML structure - Design our own

✅ **Do implement:**
- Simple existence checks (`require: [...]`)
- Top-level patterns for common cases
- Rule reusability (groups)
- Root file constraints (dogfooding)

### Config Efficiency Examples

**Example 1: Simple Package Requirements**
```yaml
# Most concise - list form
structure:
  packages/*:
    require: [AGENTS.md, README.md, src/]

# Verbose when you need details
structure:
  packages/*:
    require:
      files: [AGENTS.md, README.md]
      dirs: [src/]
      severity: error
      message: "Packages must have documentation"
```

**Example 2: Extension Rules**
```yaml
# Concise - single line per extension
patterns:
  "**/*.rs": snake_case
  "**/*.tsx": PascalCase
  "**/*.md": kebab-case

# Verbose when you need constraints
patterns:
  "**/*.rs":
    naming: snake_case
    max_lines: 500
    max_size: 100KB
```

**Example 3: Root Constraints (Dogfooding)**
```yaml
# Concise - whitelist only
structure:
  ./:
    allow: [README.md, AGENTS.md, CHANGELOG.md, LICENSE*]

# Verbose with violations
structure:
  ./:
    files:
      allow:
        - README.md
        - AGENTS.md
        - CHANGELOG.md
      deny_message: "Move markdown files to docs/"
```

### Updated Implementation Priority

**High (Efficiency Critical):**
1. Root file whitelist (`allow: [...]`)
2. Simple existence check (`require: [...]`)
3. Top-level patterns (`patterns: {glob: rules}`)

**Medium (Nice to Have):**
4. Rule groups (`use: @group`)
5. Verbose mode for exists/require

**Low (Out of Scope):**
6. Count-based exists (`exists:1-4`)
7. Content validation beyond frontmatter
8. Context-aware configs

### Principle Applied to Root Files

**Current Violation:** CONFIG_ANALYSIS_FINDINGS.md in root

**Efficient Fix:**
```yaml
structure:
  ./:
    allow: [README.md, AGENTS.md, CHANGELOG.md, CONTRIBUTING.md, 
            LICENSE*, PROJECT_MEMORIES.md, RELEASE_NOTES.md]
```

Not:
```yaml
# Don't do this - too verbose
structure:
  ./:
    files:
      AGENTS.md: exists:1
      CHANGELOG.md: exists:1
      CONFIG_ANALYSIS_FINDINGS.md: exists:0
      # ... etc
```

---

*Philosophy update: Config efficiency over feature parity*
