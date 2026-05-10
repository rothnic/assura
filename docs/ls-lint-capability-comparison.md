---
title: 'Assura vs LS-Lint - Deep Capability Comparison'
status: active
---

# Assura vs LS-Lint: Deep Capability Comparison

**Date:** 2026-03-20  
**Goal:** Ensure Assura can efficiently implement all LS-Lint capabilities

---

## Executive Summary

**Status:** Assura has ~80% parity with LS-Lint core features, but with verbosity issues in ~30% of cases.

**Critical Gaps:**
1. ❌ No required file/directory existence checks (`exists` capability)
2. ❌ No top-level glob patterns (must nest everything under `structure`)
3. ⚠️ OR syntax works but is string-based, not elegant
4. ✅ Multi-part extensions fully supported
5. ✅ 12 case conventions (full parity)

**Efficiency Issues:**
- LS-Lint: `**/*.rs: snake_case` (1 line)
- Assura: Must wrap in structure nodes (3-5 lines)

---

## Feature-by-Feature Comparison

### 1. Extension-Based Rules

**LS-Lint:**
```yaml
ls:
  .rs: snake_case
  .tsx: PascalCase
```

**Assura Current (Verbose):**
```yaml
structure:
  ./:
    files:
      extensions: ["rs"]
      naming: snake_case
  src/:
    files:
      extensions: ["tsx"]
      naming: PascalCase
```

**Can Assura Do It?** ✅ Yes, but verbose  
**Efficiency:** Poor - requires structure wrapping  
**Gap:** No top-level extension shorthand

**Proposed Efficient Syntax:**
```yaml
patterns:
  "**/*.rs": snake_case
  "**/*.tsx": PascalCase
```

**Verdict:** Needs `patterns` key for efficiency

---

### 2. Path-Specific Rules

**LS-Lint:**
```yaml
ls:
  src/components/*:
    .tsx: PascalCase
  src/utils/*:
    .ts: camelCase
```

**Assura Current:**
```yaml
structure:
  src/components/:
    files:
      naming: PascalCase
  src/utils/:
    files:
      naming: camelCase
```

**Can Assura Do It?** ✅ Yes  
**Efficiency:** Good - this is Assura's strength  
**Gap:** None

**Verdict:** ✅ Already efficient

---

### 3. OR Syntax (Multiple Conventions)

**LS-Lint:**
```yaml
ls:
  .ts: camelCase | PascalCase
```

**Assura Current:**
```yaml
structure:
  src/:
    files:
      naming: "camelCase|PascalCase"
```

**Can Assura Do It?** ✅ Yes  
**Efficiency:** Acceptable - string-based OR  
**Gap:** No array-based alternative

**Proposed Alternative:**
```yaml
structure:
  src/:
    files:
      naming: [camelCase, PascalCase]  # More explicit
```

**Verdict:** ✅ Works, array syntax would be nicer

---

### 4. Multi-Part Extensions

**LS-Lint:**
```yaml
ls:
  .d.ts: PascalCase
  .test.js: kebab-case
  .min.css: kebab-case
```

**Assura Current:**
```yaml
structure:
  src/:
    files:
      naming: snake_case  # Applied to all extensions
```

**Can Assura Do It?** ✅ Yes, with ls_compat module  
**Efficiency:** Good - supports `.d.ts`, `.test.js`  
**Implementation:** See `constraints/ls_lint/extension.rs`

**Verdict:** ✅ Full parity

---

### 5. Directory Naming Rules

**LS-Lint:**
```yaml
ls:
  .dir: kebab-case
```

**Assura Current:**
```yaml
structure:
  ./:
    files:  # Applies to files only?
      naming: snake_case
```

**Can Assura Do It?** ✅ Yes  
**Implementation:** See `constraints/ls_lint/directory.rs`  
**Efficiency:** Good - DirectoryConstraint exists

**Verdict:** ✅ Full parity

---

### 6. Required Files Existence (PR #355)

**LS-Lint:**
```yaml
ls:
  packages/*:
    AGENTS.md: exists:1
    README.md: exists:1
    src: exists:1
```

**Assura Current:** ❌ **NOT SUPPORTED**

**Can Assura Do It?** ❌ No  
**Gap:** No `exists` or `require` directive

**Proposed Efficient Syntax:**
```yaml
structure:
  packages/*:
    require: [AGENTS.md, README.md, src/]
```

**Verdict:** ❌ **Critical Gap - Must Implement**

---

### 7. Count-Based Existence (PR #355)

**LS-Lint:**
```yaml
ls:
  packages/*:
    .md: exists:1-4    # 1-4 markdown files allowed
```

**Assura Current:** ❌ **NOT SUPPORTED**

**Assessment:** YAGNI - Do we need count ranges?  
**Decision:** Skip for now. Focus on boolean exists.

**Verdict:** ❌ Out of scope (not efficient anyway)

---

### 8. Glob Patterns in Paths

**LS-Lint:**
```yaml
ls:
  packages/*:           # Matches any subdirectory
    .dir: kebab-case
  "**/*.test.ts":      # Matches any test file
    kebab-case
```

**Assura Current:**
```yaml
structure:
  packages/*:           # ✅ Works
    files:
      naming: kebab-case
  # "**/*.test.ts" - NOT supported
```

**Can Assura Do It?** ⚠️ Partial  
**Gap:** No double-glob `**` support in structure keys

**Proposed Solution:**
```yaml
patterns:
  "**/*.test.ts":
    naming: kebab-case
```

**Verdict:** ⚠️ Needs `patterns` key for `**` globs

---

### 9. Ignore/Exclude Patterns

**LS-Lint:**
```yaml
ignore:
  - node_modules
  - .git
  - "**/dist/**"
```

**Assura Current:**
```yaml
exclude:
  - "target/**"
  - ".git/**"
```

**Can Assura Do It?** ✅ Yes  
**Efficiency:** Good  
**Gap:** None

**Verdict:** ✅ Full parity

---

### 10. Root File Constraints (Dogfooding)

**LS-Lint:**
```yaml
ls:
  README.md: exists:1  # Required in root
  # Implicit: other files not restricted
```

**Assura Current:** ❌ **NOT SUPPORTED**

**Need:** Only allow specific files in root  
**Use Case:** Prevent clutter, enforce documentation

**Proposed Efficient Syntax:**
```yaml
structure:
  ./:
    allow: [README.md, AGENTS.md, LICENSE*]
    # Implicitly denies everything else
```

**Verdict:** ❌ **Critical Gap - Must Implement**

---

## Efficiency Scorecard

| Feature | LS-Lint Lines | Assura Lines | Efficiency | Status |
|---------|--------------|--------------|------------|---------|
| Extension rules | 2 | 6-10 | ⚠️ Poor | Needs patterns |
| Path rules | 4 | 4 | ✅ Good | Native strength |
| OR syntax | 1 | 1 | ✅ Good | String-based |
| Multi-part ext | 3 | 3 | ✅ Good | Full parity |
| Directory rules | 1 | 3 | ✅ Good | Works well |
| Required files | 3 | ❌ N/A | ❌ Missing | Critical gap |
| Root constraints | Implicit | ❌ N/A | ❌ Missing | Critical gap |

**Overall Efficiency:** 70% (good, but needs patterns for 100%)

---

## Capability Matrix

| Capability | LS-Lint | Assura | Can Do? | Efficient? |
|-----------|---------|--------|---------|------------|
| Extension rules | ✅ | ✅ | Yes | ⚠️ No |
| Path-specific rules | ✅ | ✅ | Yes | ✅ Yes |
| OR syntax | ✅ | ✅ | Yes | ✅ Yes |
| Multi-part extensions | ✅ | ✅ | Yes | ✅ Yes |
| Directory rules | ✅ | ✅ | Yes | ✅ Yes |
| Required files | ✅ (PR #355) | ❌ | **No** | N/A |
| Root whitelist | ❌ | ❌ | **No** | N/A |
| Glob patterns `**` | ✅ | ⚠️ | Partial | ⚠️ No |
| Exclude patterns | ✅ | ✅ | Yes | ✅ Yes |
| Count-based exists | ✅ (PR #355) | ❌ | **No** | N/A |
| File relocation | ❌ | ❌ | **No** | N/A |

**Summary:**
- ✅ **7/11** capabilities fully supported
- ⚠️ **2/11** partially supported (verbosity issues)
- ❌ **2/11** not supported (critical gaps)

---

## Critical Gaps to Address

### Gap 1: Required File Existence (`exists`)

**Priority:** CRITICAL  
**Use Case:** Enforce AGENTS.md in every package  
**LS-Lint Syntax:** `AGENTS.md: exists:1`  
**Proposed Assura Syntax:** `require: [AGENTS.md]`

**Implementation:** Add `require` field to DirectoryNode

### Gap 2: Top-Level Glob Patterns

**Priority:** HIGH  
**Use Case:** Apply rules to `**/*.rs` without nesting  
**LS-Lint Syntax:** `.rs: snake_case`  
**Proposed Assura Syntax:** `patterns: {"**/*.rs": snake_case}`

**Implementation:** Add `patterns` HashMap to Config

### Gap 3: Root File Whitelist

**Priority:** HIGH (Dogfooding)  
**Use Case:** Keep root clean, docs in docs/  
**LS-Lint Syntax:** Not directly supported  
**Proposed Assura Syntax:** `allow: [README.md, AGENTS.md]`

**Implementation:** Add `allow` field to DirectoryNode for root

---

## What We DON'T Need from LS-Lint

**Skip These (YAGNI):**

1. **Count-based exists:** `exists:1-4` - Overly complex
2. **Context-aware configs:** Handle at CLI layer
3. **Content validation:** Out of scope (markdown frontmatter is enough)
4. **Rule-level messages:** Can add later if needed
5. **LS-Lint YAML format:** Design our own efficient syntax

**Reason:** These add complexity without proportional value

---

## Recommended Implementation Order

### Phase 1: Critical Gaps (Must Have)

1. ✅ **Root whitelist** - `allow: [...]` for dogfooding
2. ✅ **Required files** - `require: [...]` for packages
3. ✅ **Top-level patterns** - `patterns: {glob: rule}`

### Phase 2: Efficiency Improvements

4. Array-based OR syntax - `naming: [kebab-case, snake_case]`
5. Double-glob support - `**/*.test.ts` in structure keys

### Phase 3: Nice to Have

6. Rule groups - `use: @group-name`
7. Verbose mode for exists - detailed error messages

---

## Verdict

**Can Assura efficiently implement LS-Lint capabilities?**

**Answer:** Almost. With 3 additions:
1. `require: [...]` directive
2. `patterns: {...}` top-level key  
3. `allow: [...]` root whitelist

Assura will have **full capability parity** with **equal or better efficiency**.

**Current State:** 70% parity, needs 3 features for 100%  
**Target State:** 100% parity with superior config efficiency

---

*Ready for implementation planning*
