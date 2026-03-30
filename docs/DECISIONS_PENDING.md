# Decisions Pending Review

This document tracks notation decisions that need final alignment.

## Status Key
- 🔴 **Undecided**: Needs discussion
- 🟡 **Draft**: Proposal ready, needs review  
- 🟢 **Decided**: Documented in SPEC.md

---

## 1. Violation Syntax 🟢

**Decision:** Array notation

```yaml
violation: [warn, ci:block, feature:warn]
```

**Rationale:**
- Cleanest conversion to JSON (already an array)
- Concise for simple cases
- Supports both flow `[a, b]` and block `- a / - b` styles
- First element is default, `context:value` pairs are overrides

---

## 2. Array Format Consistency 🟢

**Decision:** Support both flow and block styles as equivalent

**Flow style (concise):**
```yaml
apply: [typescript, tested]
violation: [warn, ci:block]
```

**Block style (readable):**
```yaml
apply:
  - typescript
  - tested

violation:
  - warn
  - ci:block
```

**Rationale:** YAML supports both natively, they are equivalent after parsing.

---

## 3. Context Inheritance Syntax 🔴

**How to set defaults at directory level:**

**Option A: Array of contexts:**
```yaml
src/components/:
  - context: ci
    violation: block
  
  - context: feature
    violation: warn
  
  ${name}.tsx:
    - apply: react
```

**Option B: Violation key at directory:**
```yaml
src/components/:
  - violation: [warn, ci:block, feature:warn]
  
  ${name}.tsx:
    - apply: react
```

**Questions:**
- Which is clearest for "applies to all rules in this directory"?
- How does override work for specific rules?

---

## 4. Message Attachment 🔴

**Where do messages live?**

**Option A: With violation:**
```yaml
- apply: react
  violation: [warn, ci:block]
  message:
    warn: "Consider refactoring"
    ci: "Must fix before merge"
```

**Option B: Separate messages key:**
```yaml
- apply: react
  violation: [warn, ci:block]

messages:
  react-warning: "Consider refactoring"
  react-error: "Must fix before merge"
```

**Option C: In rule definition:**
```yaml
rules:
  react:
    .tsx: PascalCase
    message:
      warn: "Consider refactoring"
```

**Questions:**
- Context-specific messages or generic?
- Reusable messages or inline?

---

## 5. Strict Mode Definition 🟢

**Decision:** Use `strict: true` directive

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

**Rationale:** `strict: true` is more readable and explicit than `.*: exists:0`.

---

## 6. Multiple Constraints per Rule 🔴

**How to combine constraints in a rule:**

**Option A: Multiple keys:**
```yaml
rules:
  sized:
    lines: ..400
    complexity: ..20
```

**Option B: Array of constraints:**
```yaml
rules:
  sized:
    - lines: ..400
    - complexity: ..20
```

**Option C: Single constraint key:**
```yaml
rules:
  sized:
    constraint:
      lines: ..400
      complexity: ..20
```

**Questions:**
- AND logic implicit (all must pass)?
- OR logic needed?

---

## 7. Rule/Group Naming 🟢

**Decision:** Use `rules:`

**Rationale:** 
- Clear and familiar terminology
- Differentiates from `groups:` in earlier proposal
- Used consistently in notation guide

---

## 8. Exists vs Count 🟢

**Decision:** Use `exists:` with range notation

```yaml
- exists: 1           # Exactly 1
- exists: 1..10       # Between 1 and 10
- exists: ..5         # At most 5
- exists: 5..         # At least 5
```

**Rationale:**
- Consistent with range notation for other constraints (lines, etc.)
- No separate `count` directive needed
- Clear semantic meaning

---

## 9. Env Variables in Contexts 🟡

**Draft proposal:**
```yaml
contexts:
  emergency:
    hook: pre-commit
    env:
      EMERGENCY: "true"
    violation: warn
```

**Status:** Draft, needs validation

---

## 10. Version Range Syntax 🟡

**Draft proposal:**
```yaml
contexts:
  legacy:
    version: ..1.x      # Up to 1.x
  
  modern:
    version: 2.x..      # 2.x and above
```

**Status:** Draft, needs validation

---

## Action Items

1. ✅ **Update notation guide** - Done for decided items
2. **Resolve 🔴 items:**
   - Context inheritance syntax (#3)
   - Message attachment (#4)
   - Multiple constraints per rule (#6)
3. **Validate 🟡 items:**
   - Env variables in contexts (#9)
   - Version range syntax (#10)
4. **Update SPEC.md** once all decisions finalized

---

*Created: 2026-03-24*
*Updated: 2026-03-24*
*Review with team before finalizing SPEC.md*
