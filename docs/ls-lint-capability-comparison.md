---
title: 'Assura vs LS-Lint - Deep Capability Comparison'
status: active
---

# Assura vs LS-Lint: Deep Capability Comparison

**Date:** 2026-03-20  
**Current product note:** Updated May 26, 2026 for the LS-Lint rule coverage
audit in `docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md`.
**Goal:** Ensure Assura can efficiently implement all LS-Lint capabilities

---

## Executive Summary

**Status:** Assura has complete LS-Lint 2.3 config semantic migration coverage
for naming, regex, extension/subextension `exists`, `.dir`, ignore,
wildcard/subextension, glob/brace directory scopes, scalar exact `exists`
keys, LS-Lint scalar naming no-op keys, multi-config migration merge, and
explicit target-path semantics as an Assura validation mode. CLI drop-in
parity, LS-Lint flags, and exact LS-Lint JSON output are out of scope.

**Critical Gaps:**
1. ⚠️ No LS-Lint CLI drop-in parity claim; Assura migrates config semantics.
2. ✅ Glob and brace directory scopes such as `packages/*`, `**`, and
   `{src,tests}` are represented as validation scopes, not required literal
   directories.
3. ⚠️ OR syntax works but is string-based, not elegant
4. ✅ LS-Lint extension/subextension, scalar exact, and `.dir` `exists`,
   `exists:0`, `exists:1`, and `exists:N-M` counts are implemented. Scalar
   naming keys are validated no-ops to match LS-Lint.
5. ✅ Multi-part extensions and 12 case conventions are supported.

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

**Assura Native:**
```yaml
structure:
  ./:
    .rs: snake_case
    src/:
      .tsx: PascalCase
```

**Can Assura Do It?** ✅ Yes
**Efficiency:** Good for structure-local extension policy
**Verdict:** Native tree notation keeps extension rules inside the project
scope they govern.

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

### 6. Extension And Directory Count Existence

**LS-Lint:**
```yaml
ls:
  packages/*:
    .md: exists:1-3
    .dir: exists:1
```

**Assura Current:** ✅ **SUPPORTED**

**Can Assura Do It?** Yes. LS-Lint pattern directory scopes are converted into
matcher-backed validation scopes, and extension/subextension plus `.dir`
`exists` checks run in each matched directory without requiring a literal
`packages/*` directory. Scalar exact `exists` keys such as
`README.md: exists:1` are converted to direct counts for default validation.
Scalar naming keys are validated and otherwise ignored.

**Native Assura Syntax:**
```yaml
structure:
  packages/*/:
    AGENTS.md: exists:1
    README.md: exists:1
    src/: exists:1
```

**Verdict:** ✅ Direct `exists` and wildcard scopes are implemented.

---

### 7. Count-Based Existence (PR #355)

**LS-Lint:**
```yaml
ls:
  packages/*:
    .md: exists:1-4    # 1-4 markdown files allowed
```

**Assura Current:** ✅ **SUPPORTED FOR DIRECT CHILDREN**

**Assessment:** Direct file and directory counts are implemented through
`files.exists` and `directories.exists`. Counts are direct-child only and work
inside explicit, wildcard, recursive, and brace directory scopes.

**Verdict:** ✅ Implemented for explicit scopes and direct child counts.

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
  src/**/c:              # ✅ Works as an LS-Lint validation scope
    files:
      naming: PascalCase
```

**Can Assura Do It?** ✅ Yes for LS-Lint directory scopes.
**Gap:** Top-level file glob shorthand remains an Assura-native ergonomics
improvement, not an LS-Lint migration blocker.

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

### 10. Root File Constraints (Assura Native)

**Assura native config:**
```yaml
structure:
  ./:
    files:
      exists:
        README.md: "1"
```

**Assura Current:** ✅ **SUPPORTED OUTSIDE LS-LINT MIGRATION PARITY**

**Need:** Only allow specific files in root  
**Use Case:** Prevent clutter, enforce documentation

**Native Assura Syntax:**
```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    AGENTS.md: exists:1
    LICENSE: exists:0-1
```

**Verdict:** ✅ Native path keys and `extra: false` support this behavior.

---

## Efficiency Scorecard

| Feature | LS-Lint Lines | Assura Lines | Efficiency | Status |
|---------|--------------|--------------|------------|---------|
| Extension rules | 2 | 4 | ✅ Good | Implemented through native extension keys |
| Path rules | 4 | 4 | ✅ Good | Explicit scopes implemented |
| OR syntax | 1 | 1 | ✅ Good | Implemented, string-based |
| Multi-part ext | 3 | 3 | ✅ Good | Implemented |
| Directory rules | 1 | 3 | ✅ Good | Implemented |
| Required direct children | 3 | 3-5 | ✅ Good | Implemented through native path keys and `exists` |
| Root constraints | Not direct | 4-8 | ✅ Good | Implemented through `extra: false` and native path keys |

**Overall Efficiency:** Good for explicit structure and direct-content
contracts. Broad recursive file patterns remain a deliberate non-goal for the
structure tree unless a future relation or pattern surface proves necessary.

---

## Capability Matrix

| Capability | LS-Lint | Assura | Can Do? | Efficient? |
|-----------|---------|--------|---------|------------|
| Extension rules | ✅ | ✅ | Yes | ✅ Yes |
| Path-specific rules | ✅ | ✅ | Yes | ✅ Yes |
| OR syntax | ✅ | ✅ | Yes | ✅ Yes |
| Multi-part extensions | ✅ | ✅ | Yes | ✅ Yes |
| Directory rules | ✅ | ✅ | Yes | ✅ Yes |
| Required files/directories | ✅ | ✅ | Yes for direct scopes, including wildcard/brace scopes | ✅ Yes |
| Root whitelist / closed world | ❌ | ✅ | Yes as Assura extension | ✅ Yes |
| Glob patterns `**` | ✅ | ✅ | Yes for LS-Lint directory scopes | ⚠️ Native shorthand pending |
| Exclude patterns | ✅ | ✅ | Yes | ✅ Yes |
| Count-based exists | ✅ | ✅ | Yes for direct children | ✅ Yes |
| File relocation | ❌ | ❌ | **No** | N/A |

**Summary:**
- ✅ Core LS-Lint 2.3 naming, regex, ignore, `.dir`, OR, wildcard extension,
  pattern-scope, and direct `exists` behavior is implemented.
- ✅ Pattern scopes such as `packages/*`, `**`, and `{src,tests}` are validation
  scopes instead of required child nodes.
- ✅ Assura implements closed-world direct-content checks beyond native
  LS-Lint.

---

## Remaining Assura Ergonomics

These are native Assura syntax improvements, not LS-Lint migration blockers.

### Resolved: Direct-Content Shorthand

**Priority:** Implemented
**Use Case:** Express required files, direct counts, and root closed-world
policies without low-level `files.*` / `directories.*` bundles.
**Current Assura Syntax:** Native path keys with `exists` shorthand and
`extra: false`.

**Implementation:** Keep fixtures for native path keys, wildcard scopes, count
shorthand, and `extra: false`; extend the same native tree notation only when a
use case cannot be represented by path keys plus nested attributes.

### Gap 2: Top-Level Glob Patterns

**Priority:** HIGH
**Use Case:** Apply rules to `**/*.rs` without repeating structure nodes.
**LS-Lint Syntax:** `.rs: snake_case` or broad glob scopes.
**Future Assura Syntax:** Extend native notation with explicit pattern
attributes or pattern scopes only if repeated structure nodes become a proven
burden.

**Implementation:** Preserve the distinction between file matchers and
directory requirements.

---

## What We DON'T Need from LS-Lint

**Skip These (YAGNI):**

1. **Implicit required directories from pattern scopes:** unsafe and misleading
2. **Context-aware configs:** Handle at CLI layer
3. **Content validation:** Out of scope (markdown frontmatter is enough)
4. **Rule-level messages:** Can add later if needed
5. **LS-Lint YAML format:** Design our own efficient syntax

**Reason:** These add complexity without proportional value

---

## Recommended Implementation Order

### Phase 1: Native Ergonomics

1. ⏳ **Direct-content shorthand** - ergonomic sugar for existing explicit fields
2. ⏳ **Top-level patterns** - compiled/indexed matchers

### Phase 2: Efficiency Improvements

3. Array-based OR syntax - `naming: [kebab-case, snake_case]`
4. Double-glob support - `**/*.test.ts` in structure keys

### Phase 3: Nice to Have

5. Rule groups - `use: @group-name`
6. Detailed `exists` diagnostics for count failures

---

## Verdict

**Can Assura efficiently implement LS-Lint capabilities?**

**Answer:** Yes for LS-Lint 2.3 compatibility. Current Assura covers the direct
count, regex, wildcard/subextension, glob/brace scope, and closed-world pieces
that earlier docs listed as missing. The remaining work is native Assura
notation ergonomics:

1. first-class documented pattern-scope notation,
2. shorthand for existing direct-content fields,
3. reusable `rules:`/directive groups for policy reuse,
4. indexed pattern matching for scalable broad rules.

See `docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md` for the current
compatibility evidence.

---

*Ready for implementation planning*
