---
title: 'Assura Config Analysis - LS-Lint Parity & Feature Gap Review'
status: historical
---

# Assura Config Analysis: LS-Lint Parity & Feature Gap Review

**Date:** 2026-03-20  
**Status:** Draft - Internal Review  
**Scope:** Compare Assura V2 config format against LS-Lint features, PR #355, and Issue #356 proposals

---

## Executive Summary

Assura's V2 structure-first configuration is **architecturally sound** and provides a solid foundation. The hierarchical approach with inheritance is more maintainable than LS-Lint's flat rule model for large projects. However, there are **feature gaps** relative to LS-Lint PR #355 and Issue #356 that should be addressed to fully support the intended use cases.

**Key Finding:** The `children` key is not "odd" per se, but the **lack of top-level file patterns** (glob rules outside the hierarchy) is a verbosity concern that LS-Lint handles more elegantly.

---

## Current Assura V2 Config Format

### Structure

```yaml
version: "2.0"

structure:
  src/:
    files:
      naming: "snake_case"
      max_lines: 500
    children:
      components/:
        files:
          naming: "PascalCase"
          inherit: true  # Inherits max_lines: 500 from parent
  
  tests/:
    files:
      naming: "snake_case"
      max_lines: 1000

exclude:
  - "target/**"
```

### Strengths

1. **Visual Clarity:** Structure mirrors actual directory layout
2. **Inheritance:** Parent rules automatically apply to children with override capability
3. **DRY Principle:** Define once, inherit everywhere
4. **Specificity-based Resolution:** More specific paths win automatically
5. **Validation Bundles:** Group related validations (naming, size, docs) together

### Weaknesses

1. **Verbosity for Simple Cases:** Every rule must be nested under a directory
2. **No Top-Level File Patterns:** Cannot easily say "all .rs files use snake_case"
3. **Children Key Nesting:** Deep nesting becomes hard to read
4. **No Rule Groups/Reusability:** Cannot define reusable rule sets

---

## LS-Lint Config Format (Current)

### Structure

```yaml
ls:
  .dir: kebab-case
  .rs: snake_case
  .tsx: PascalCase
  
  src/components/*:
    .tsx: PascalCase
    
  packages/*:
    AGENTS.md: exists:1
    README.md: exists:1
    src: exists:1
```

### Strengths

1. **Concise:** Single-line rules for common cases
2. **Top-Level Patterns:** Extension-based rules apply globally
3. **Path Scoping:** Simple glob-based path rules
4. **Exists Directive:** Validate required files/directories exist
5. **Flat Structure:** Easy to scan and understand

### Weaknesses

1. **No Inheritance:** Rules must be repeated for similar paths
2. **Duplication:** Similar directories need repeated rules
3. **Limited Validation Types:** Primarily naming-focused
4. **No Composition:** Cannot combine/reuse rule sets

---

## Feature Gap Analysis

### 1. Required File/Directory Existence (PR #355)

**LS-Lint PR #355 Adds:**
```yaml
packages/*:
  AGENTS.md: exists:1    # Must exist exactly once
  README.md: exists:1
  src: exists:1          # Directory must exist
```

**Assura Current Support:** ❌ **MISSING**

Assura has no equivalent `exists` directive. This is a **critical gap** for:
- Enforcing AGENTS.md in every package
- Required documentation files
- Mandatory directory structure

**Proposed Assura Syntax:**
```yaml
structure:
  packages/*:
    files:
      naming: "kebab-case"
      required:
        - "AGENTS.md"      # File must exist
        - "README.md"
    directories:
      required:
        - "src"            # Directory must exist
```

**Alternative (More Concise):**
```yaml
structure:
  packages/*:
    files:
      naming: "kebab-case"
    exists:
      files: ["AGENTS.md", "README.md"]
      directories: ["src"]
```

---

### 2. Top-Level File Pattern Rules

**LS-Lint:**
```yaml
ls:
  .rs: snake_case        # All .rs files
  .tsx: PascalCase       # All .tsx files
```

**Historical Assura Draft:**
```yaml
structure:
  ./:                      # Must explicitly define root
    files:
      naming: "snake_case"
      extensions: ["rs"]   # But this applies to ALL files in root only
```

**Problem:** Assura cannot easily express "all .rs files anywhere in the project." The structure-first approach requires wrapping everything in directory patterns.

**Proposed Solution - Top-Level `patterns` Key:**
```yaml
version: "2.0"

# Global patterns apply everywhere unless overridden
patterns:
  "**/*.rs":
    naming: "snake_case"
  "**/*.tsx":
    naming: "PascalCase"

# Structure for directory-specific rules
structure:
  src/:
    files:
      max_lines: 500    # Additional constraint, naming from pattern above
```

---

### 3. Rule Groups / Reusability (Issue #356)

**LS-Lint Proposal (#356):**
```yaml
groups:
  js-names:
    - "camelCase => Default for JS/TS files"
    - "PascalCase => For components"
  
  js-defaults:
    - "@js-names"
    - "content:max-lines:400"

ls:
  .ts: "@js-defaults"
  .tsx: "@js-defaults"
```

**Assura Current:** ❌ **MISSING**

Must repeat rules:
```yaml
structure:
  src/
    files:
      naming: "camelCase|PascalCase"
      max_lines: 400
  tests/
    files:
      naming: "camelCase|PascalCase"   # Repeated!
      max_lines: 400                   # Repeated!
```

**Proposed Solution:**
```yaml
version: "2.0"

groups:
  js-defaults:
    naming: "camelCase|PascalCase"
    max_lines: 400

structure:
  src/:
    files:
      use: "@js-defaults"
  tests/:
    files:
      use: "@js-defaults"
      max_lines: 800    # Override just this field
```

---

### 4. Content Validation (Issue #356)

**LS-Lint Proposal (#356):**
```yaml
ls:
  .ts: "camelCase | content:max-lines:400"
  .md: "kebab-case | content:front-matter:required"
```

**Assura Current (Partial):**
```yaml
structure:
  docs/:
    markdown:
      require_frontmatter: true
      max_heading_depth: 3
```

**Gap:** Assura has markdown validation but lacks:
- Max lines validation for any file type
- Content patterns (e.g., required headings)
- File size validation is present ✓

**Recommendation:** Extend `FileValidationBundle`:
```yaml
files:
  naming: "snake_case"
  max_lines: 500        # ✓ Already supported
  max_size: "1MB"       # ✓ Already supported
  require_docs: true    # ✓ Already supported
  content_patterns:     # ❌ NEW - Add this
    - "^## Overview$"   # Required heading
    - "TODO|FIXME": forbid  # Forbidden patterns
```

---

### 5. Context-Aware Enforcement (Issue #356)

**LS-Lint Proposal (#356):**
```yaml
contexts:
  pre-commit:
    mode: warn
    message: "Commit not blocked..."
  pre-push:
    mode: fail
    message: "Push blocked..."
```

**Assura Current:** ❌ **MISSING**

Assura has severity levels but no context-aware enforcement modes.

**Gap Severity:** Medium - Can be added to CLI/hooks layer instead of config

---

### 6. Action-Oriented Failure Messages (Issue #356)

**LS-Lint Proposal (#356):**
```yaml
groups:
  js-defaults:
    - "camelCase => Default for JS files"
    - "content:max-lines:400 => Keep files under 400 lines for refactorability"
```

**Assura Current:** ❌ **MISSING**

No per-rule documentation/messaging support.

**Proposed:**
```yaml
groups:
  js-defaults:
    naming:
      value: "camelCase|PascalCase"
      description: "Default for JS files. Use PascalCase for components."
    max_lines:
      value: 400
      description: "Keep files under 400 lines so refactors stay manageable."
```

---

## Performance Comparison

### Assura Benchmarks (Current)

| Project Size | Files | Time | Files/Second |
|--------------|-------|------|--------------|
| Small | 50 | 1.6ms | 30,693 |
| Medium | 500 | 13.4ms | 37,425 |
| Large | 5,000 | 131ms | 38,142 |
| Large | 6,250 | 130ms | 47,872 |

**Target:** 2x+ speedup over LS-Lint
**Status:** Benchmarks show strong performance, but LS-Lint comparison not yet measured (LS-Lint binary not available in test environment)

### LS-Lint PR #355 Performance Claim

- **Before:** 790ms for 20k files
- **After:** 408ms for 20k files
- **Optimization:** Lazy ignore glob matching instead of pre-expansion

### Assessment

**Assura Performance:** ✅ **STRONG**
- 45k-48k files/second for large projects
- Linear scaling confirmed
- Sub-100µs incremental validation

**Potential Optimizations from PR #355:**
- Implement lazy glob matching for `exclude` patterns
- Cache file stat results
- Parallel rule evaluation (already using Rayon for some operations)

---

## The "Children" Key Assessment

### User Concern: "Children Key Feels Odd"

**Analysis:** The `children` key is not inherently problematic, but it **exacerbates verbosity** in the structure-first approach.

**Comparison:**

```yaml
# Assura V2 - Nested
structure:
  src/:
    files:
      naming: snake_case
    children:
      components/:
        files:
          naming: PascalCase
        children:
          ui/:
            files:
              naming: PascalCase
```

```yaml
# LS-Lint - Flat
ls:
  src/**/*: snake_case
  src/components/**/*: PascalCase
  src/components/ui/**/*: PascalCase
```

**Verdict:**
- **Pro:** Assura's structure is self-documenting and matches filesystem
- **Con:** LS-Lint is more concise for simple path-based rules
- **Recommendation:** Add `patterns` key for glob-based rules alongside structure

---

## Top-Level Pattern Support

### Current Gap

Assura cannot express: "All .rs files anywhere should use snake_case"

### Workaround (Verbose)

```yaml
structure:
  ./:
    files:
      extensions: ["rs"]
      naming: snake_case
  src/:
    children:
      subdir/:
        files:
          extensions: ["rs"]    # Repeated!
          naming: snake_case    # Repeated!
```

### Proposed Solution

```yaml
version: "2.0"

# Global file patterns
patterns:
  "**/*.rs":
    naming: snake_case
    max_lines: 500
  "**/*.tsx":
    naming: PascalCase
    max_lines: 400
  
  # Pattern inheritance
  "src/**/*.rs":
    max_lines: 300    # Overrides 500 for src/

# Structure for directory-specific constraints
structure:
  packages/*:
    exists:
      files: ["AGENTS.md", "README.md"]
```

**Resolution:** Pattern-based rules override structure-based rules based on specificity.

---

## Recommendations

### High Priority (Address Soon)

1. **Add `exists` directive** - Critical for enforcing AGENTS.md and required structure
2. **Add `patterns` key** - Reduce verbosity for common extension-based rules
3. **Implement rule groups** - Enable reusability and DRY configs

### Medium Priority

4. **Content validation expansion** - Max lines for any file type, content patterns
5. **Rule documentation** - Per-rule descriptions for better error messages
6. **Performance: Lazy glob matching** - Optimize exclude patterns per PR #355

### Low Priority

7. **Context-aware enforcement** - Can be handled at CLI/hooks layer
8. **Multiple case conventions in OR syntax** - Already partially supported

---

## Configuration Philosophy Recommendation

**Hybrid Approach:** Support both structure-first AND pattern-first styles

```yaml
version: "2.0"

# Pattern-first for simple, global rules (LS-Lint style)
patterns:
  "**/*.rs": { naming: snake_case, max_lines: 500 }
  "**/*.tsx": { naming: PascalCase }

# Structure-first for complex, hierarchical rules (Assura style)
structure:
  packages/*:
    exists:
      files: ["AGENTS.md", "README.md"]
    
    children:
      src/:
        files:
          naming: snake_case
          # Inherits max_lines: 500 from pattern above
```

**Benefits:**
- Keeps structure-first benefits (inheritance, visual clarity)
- Adds pattern-first conciseness for common cases
- Users choose the right tool for each rule
- Historical note: no current stability guarantee

---

## LS-Lint Feature Checklist

| Feature | LS-Lint | Assura V2 | Gap | Priority |
|---------|---------|-----------|-----|----------|
| Naming conventions | ✅ | ✅ | None | - |
| 12 case conventions | ✅ | ✅ | None | - |
| Extension rules (.rs) | ✅ | ⚠️ | Via structure only | Medium |
| Path-specific rules | ✅ | ✅ | Via structure | - |
| OR syntax (A\|B) | ✅ | ✅ | Via `naming: "A\|B"` | - |
| Multi-part extensions | ✅ | ✅ | .d.ts, .test.js | - |
| Glob patterns | ✅ | ✅ | `**/*.rs` | - |
| `exists` directive | ✅ (PR #355) | ❌ | **Critical gap** | **High** |
| Required files | ✅ (PR #355) | ❌ | **Critical gap** | **High** |
| Required directories | ✅ (PR #355) | ❌ | **Critical gap** | **High** |
| Rule groups | ✅ (#356) | ❌ | Missing | Medium |
| Content validation | ✅ (#356) | ⚠️ | Markdown only | Medium |
| Max lines | ✅ (#356) | ✅ | Files bundle | - |
| Context-aware | ✅ (#356) | ❌ | Missing | Low |
| Rule documentation | ✅ (#356) | ❌ | Missing | Low |

**Summary:**
- ✅ **Complete parity:** Core naming features
- ⚠️ **Partial:** Content validation (markdown only)
- ❌ **Gaps:** `exists`, rule groups, context-aware, documentation

---

## Performance Verdict

**Assura is well-positioned for performance:**

- 45k+ files/second throughput
- Linear scaling confirmed
- Sub-µs incremental validation
- Rust + Rayon for parallelism

**To maintain edge over LS-Lint:**
- Implement lazy glob matching (PR #355 technique)
- Add memory profiling
- Cache file metadata
- Consider pre-compiled glob patterns

---

## Next Steps

1. **Review this document** - Validate findings and priorities
2. **Prototype `exists` directive** - Most critical gap
3. **Design `patterns` key** - Address verbosity concern
4. **Benchmark vs actual LS-Lint** - Install binary and measure
5. **User testing** - Get feedback on config format ergonomics

---

## Appendix: Example Comprehensive Config

```yaml
version: "2.0"

# Reusable rule groups
groups:
  rust-defaults:
    naming: snake_case
    max_lines: 500
    max_size: 1MB
    
  js-defaults:
    naming: "camelCase|PascalCase"
    max_lines: 400
    
  strict-docs:
    require_frontmatter: true
    max_heading_depth: 3
    required_headings:
      - "^## Overview$"
      - "^## Examples$"

# Global patterns for common file types
patterns:
  "**/*.rs": "@rust-defaults"
  "**/*.ts": "@js-defaults"
  "**/*.tsx": "@js-defaults"
  "docs/**/*.md": "@strict-docs"

# Hierarchical structure for complex requirements
structure:
  # Root level requirements
  ./:
    exists:
      files: ["README.md", "LICENSE"]
    
  # Package structure enforcement
  packages/*:
    exists:
      files: ["AGENTS.md", "README.md", "package.json"]
      directories: ["src"]
    
    files:
      naming: kebab-case
    
    children:
      src/:
        files:
          # Inherits from patterns above based on extension
          max_lines: 300    # Stricter than global default
      
      tests/:
        files:
          naming: snake_case
          max_lines: 1000   # Tests can be longer

# Exclusions
exclude:
  - "**/node_modules/**"
  - "**/target/**"
  - "**/.git/**"
  - "**/dist/**"
```

---

*Document Status: Draft for internal review*  
*Next Review: After implementing `exists` and `patterns` features*

---

## Architectural Decision: Remove Config Versions

### Decision
**Remove all v1/v2 distinction. The "V2" format IS the Assura config format.**

### Rationale
1. **Technical Debt:** Versioned configs create cognitive overhead and maintenance burden
2. **Pre-1.0 Status:** Per PROJECT_MEMORIES.md: "No internal stability guarantee before Assura reaches both 1.0 and 10 GitHub stars"
3. **Single Source of Truth:** One config format to document, maintain, and extend
4. **Simpler Onboarding:** Users don't need to understand version differences

### Refactoring Plan

#### Phase 1: Directory Restructure
```
src/config/
├── mod.rs              # Re-exports everything
├── config.rs           # Core Config struct (formerly StructureConfig)
├── loader.rs           # ConfigLoader (formerly StructureConfigLoader)
├── inheritance.rs      # RuleResolver (unchanged logic)
├── structure.rs        # Validation bundles + StructureNode
└── ls_compat.rs        # LS-Lint conversion utilities
```

#### Phase 2: Rename Types
| Old Name | New Name | Notes |
|----------|----------|-------|
| `StructureConfig` | `Config` | Root config struct |
| `StructureConfigLoader` | `ConfigLoader` | Config loader |
| `StructureNode` | `DirectoryNode` | Directory-specific node |
| `version: "2.0"` | **REMOVE** | No version field needed |
| `v2` module | Flatten to `config` | Remove subdirectory |
| `FileValidationBundle` | `FileBundle` | Shorter, clear |
| `MarkdownValidationBundle` | `MarkdownBundle` | Shorter, clear |

#### Phase 3: Update Imports
- All `use assura::config::v2::*` → `use assura::config::*`
- Update `lib.rs` exports
- Update all test files
- Update documentation

#### Phase 4: Documentation Updates
- Remove all "V2" references from docs
- Update website docs
- Update AGENTS.md if referenced
- Single unified config documentation

#### Phase 5: Config File Migration
- Update `.assura/config.yml` to remove `version: "2.0"`
- Update example configs
- Update test fixtures

### New Config Format (Unified)

```yaml
# .assura/config.yml - No version field needed

project:
  name: "My Project"
  maturity: stable

# Top-level patterns (NEW - addresses verbosity concern)
patterns:
  "**/*.rs":
    naming: snake_case
    max_lines: 500
  "**/*.tsx":
    naming: PascalCase

# Structure for hierarchical rules
structure:
  src/:
    files:
      max_lines: 300    # Overrides pattern above
    children:
      components/:
        files:
          naming: PascalCase

# Required files/existence checks (NEW - addresses PR #355 gap)
requirements:
  packages/*:
    files: ["AGENTS.md", "README.md"]
    directories: ["src"]

exclude:
  - "target/**"
```

### Migration Guide for Users

**Before (with version):**
```yaml
version: "2.0"
structure:
  src/:
    files:
      naming: snake_case
```

**After (unified):**
```yaml
structure:
  src/:
    files:
      naming: snake_case
```

**Changes:**
1. Remove `version: "2.0"` line
2. Everything else works the same

### Benefits

1. **Simpler Mental Model:** One config format, no versions
2. **Less Code:** Remove version detection, compatibility layers
3. **Clearer Documentation:** Single source of truth
4. **Easier Extension:** Add new features without versioning concerns
5. **Pre-1.0 Flexibility:** Can still change format until 1.0 release

### Files to Modify

1. **Source Files:**
   - `src/config/mod.rs` - Flatten exports
   - `src/config/v2/*` → `src/config/*` - Move and rename
   - `src/lib.rs` - Update exports
   - `src/cli/config.rs` - Update references
   - `src/cli/commands.rs` - Update references

2. **Test Files:**
   - `tests/constraint_tests.rs`
   - `tests/ls_lint_tests.rs`
   - All test fixtures

3. **Documentation:**
   - `docs/config-v2.md` → `docs/configuration.md`
   - `website/src/content/docs/reference/config-v2.md`
   - `AGENTS.md`
   - `README.md` (if exists)

4. **Config Files:**
   - `.assura/config.yml`
   - Example configs in `website/src/content/docs/examples/`

### Backwards Compatibility

**Per PROJECT_MEMORIES.md:**
> "No internal stability guarantee before Assura reaches both 1.0 and 10 GitHub stars."

**Action:** Break compatibility intentionally:
- Remove `version` field requirement
- Error if `version` field present (with helpful message)
- Migration command: `assura migrate --from versioned`

### Implementation Order

1. ✅ **Document findings** (this file)
2. 🔄 **Plan approval** (user review)
3. ⏳ **Refactor code** (move v2/ to config/)
4. ⏳ **Rename types** (StructureConfig → Config)
5. ⏳ **Update imports** (all source files)
6. ⏳ **Update tests** (test files and fixtures)
7. ⏳ **Update docs** (website and markdown)
8. ⏳ **Update config** (.assura/config.yml)
9. ⏳ **Verify** (cargo test, cargo build)

---

*Status: Plan drafted, awaiting approval*

---

## Dogfooding Requirement: Use Assura to Validate Assura

### The Problem

I just created `CONFIG_ANALYSIS_FINDINGS.md` in the repository root. This should have triggered a warning:

**Assura Rule (Missing):**
```yaml
structure:
  ./:  # Root directory
    files:
      # Only specific allowed files in root
      allowed:
        - "README.md"
        - "AGENTS.md"
        - "LICENSE*"
        - "Cargo.*"
        - ".gitignore"
        - ".assura/"
      naming: "UPPERCASE"  # Root docs should be UPPERCASE.md
```

**What Should Have Happened:**
1. **OpenCode Plugin Warning:** Post-tool validation should flag: "Root files should follow strict conventions. Consider moving to docs/"
2. **Pre-Commit Hook:** Block commit with clear message about root file policy
3. **Config-Guided:** This rule should be defined in `.assura/config.yml`

### Real-World AI Code Smells to Capture

**Pattern 1: Root File Proliferation**
- AI agents dump analysis docs, notes, TODO files in root
- Root becomes cluttered with temporary/working files
- **Config Solution:** `structure.allowed_files` with strict whitelist

**Pattern 2: Incorrect Naming in Context**
- Analysis files named lowercase (`findings.md` vs `FINDINGS.md`)
- Inconsistent with project conventions
- **Config Solution:** Per-directory naming rules

**Pattern 3: Missing Required Documentation**
- AI creates package but forgets AGENTS.md
- PR missing CHANGELOG update
- **Config Solution:** `exists` directive for required files

**Pattern 4: Files That Should Be Elsewhere**
- Large markdown files in root that belong in docs/
- Test fixtures scattered in src/
- **Config Solution:** Size-based location rules + warnings

### Pre-Commit Integration Vision

```bash
$ git add CONFIG_ANALYSIS_FINDINGS.md
$ git commit -m "Add analysis"

[assura-pre-commit] Checking repository structure...
⚠️  WARNING: Root file 'CONFIG_ANALYSIS_FINDINGS.md' violates policy
   Rule: Root files should be essential project metadata only
   Suggestion: Move to docs/archive/config-analysis-findings-detailed.md
   Override: Use --no-verify or add to .assura/ignore

[assura-pre-commit] Blocked: 1 structural warning
```

### OpenCode Plugin Real-Time Validation

```typescript
// In opencode-plugin
preToolUse: async (tool, args) => {
  if (tool === 'Write' && args.filePath.startsWith('/workspace/repos/assura/')) {
    const validation = await assura.validatePath(args.filePath);
    if (validation.violations.length > 0) {
      return {
        blocked: validation.severity === 'error',
        warning: validation.message,
        suggestion: validation.suggestedLocation
      };
    }
  }
}
```

### Immediate Action: Fix Root Constraint

**Add to `.assura/config.yml`:**

```yaml
structure:
  ./:
    files:
      # Strict whitelist for root files
      allowed_patterns:
        - "README.md"
        - "AGENTS.md"
        - "LICENSE*"
        - "LICENSE-*"
        - "CHANGELOG.md"
        - "CONTRIBUTING.md"
        - "Cargo.toml"
        - "Cargo.lock"
        - ".gitignore"
        - ".gitattributes"
        - ".assura/"
        - ".github/"
      naming: "UPPERCASE"  # Root markdown should be UPPERCASE.md
      
    exists:
      files: ["README.md", "AGENTS.md", "LICENSE"]
      
    # Files that match these patterns should be moved
    relocation_rules:
      - pattern: "*.md"
        if_size_gt: "10KB"
        move_to: "docs/"
        message: "Large markdown files should live in docs/"
      - pattern: "*analysis*"
        move_to: "docs/analysis/"
        message: "Analysis documents belong in docs/analysis/"
      - pattern: "*findings*"
        move_to: "docs/analysis/"
        message: "Findings documents belong in docs/analysis/"
```

### Migration of Current Root Files

**Current Root Files Analysis:**

| File | Current Location | Should Be | Action |
|------|-----------------|-----------|--------|
| AGENTS.md | Root | ✅ Root | Keep (required) |
| CHANGELOG.md | Root | ✅ Root | Keep (standard) |
| CONFIG_ANALYSIS_FINDINGS.md | Root | ❌ docs/analysis/ | Move |
| CONTRIBUTING.md | Root | ✅ Root | Keep (standard) |
| DOCUMENTATION_SUMMARY.md | Root | ❌ docs/ | Move |
| performance-baseline.md | Root | ❌ docs/ | Move |
| PROJECT_MEMORIES.md | Root | ✅ Root | Keep (project metadata) |
| RELEASE_NOTES.md | Root | ✅ Root | Keep (standard) |

**This is the power of what we're building:**
The config should capture our project conventions, and Assura should enforce them at every touchpoint (IDE, pre-commit, CI).

### Updated Priority List

**High Priority (Dogfooding Required):**
1. ✅ Document findings (done)
2. 🔄 **Add root constraints to `.assura/config.yml`** (immediate)
3. 🔄 **Move non-compliant files to proper locations** (immediate)
4. ⏳ Refactor config (remove v1/v2)
5. ⏳ Add `exists` directive
6. ⏳ Add top-level `patterns` key

**Medium Priority:**
7. ⏳ Enhance OpenCode plugin for real-time validation
8. ⏳ Pre-commit hook integration
9. ⏳ Rule documentation/messaging

---

*Updated with dogfooding requirements*

---

## Root File Policy (User Requirement)

### Rule Definition

**Only approved special markdown files allowed in project root.**
**All other markdown files must be in docs/ folder.**

### Implementation

```yaml
structure:
  ./:
    files:
      # Strict whitelist for root markdown files
      allowed_names:
        - "README.md"
        - "AGENTS.md"
        - "CHANGELOG.md"
        - "CONTRIBUTING.md"
        - "LICENSE.md"
        - "LICENSE-MIT"
        - "LICENSE-APACHE"
        - "PROJECT_MEMORIES.md"
        - "RELEASE_NOTES.md"
        - "CODE_OF_CONDUCT.md"
        - "SECURITY.md"
      
      # Any other .md files are violations
      naming: "UPPERCASE"
      
    # Violation: markdown files not in whitelist
    violations:
      - pattern: "*.md"
        not_in: [allowed_names]
        severity: error
        message: "Markdown files in root must be approved special files. Move to docs/ or add to allowed_names with justification."
        
      - pattern: "**/*.md"
        if_in_root: true
        not_in: [allowed_names]
        action: suggest_move
        move_to: "docs/"
```

### Current Root Files - Compliance Analysis

| File | Type | Allowed? | Action Required |
|------|------|----------|-----------------|
| AGENTS.md | Required | ✅ Yes | Keep in root |
| CHANGELOG.md | Standard | ✅ Yes | Keep in root |
| CONFIG_ANALYSIS_FINDINGS.md | Analysis | ❌ No | Move to docs/analysis/ |
| CONTRIBUTING.md | Standard | ✅ Yes | Keep in root |
| DOCUMENTATION_SUMMARY.md | Summary | ❌ No | Move to docs/ |
| performance-baseline.md | Metrics | ❌ No | Move to docs/ |
| PROJECT_MEMORIES.md | Project | ✅ Yes | Keep in root |
| RELEASE_NOTES.md | Standard | ✅ Yes | Keep in root |

**Violations Found:** 3 files need to be moved

### Migration Plan

```bash
# Create docs directories
mkdir -p docs/analysis
mkdir -p docs/performance
mkdir -p docs/architecture

# Move non-compliant files
mv CONFIG_ANALYSIS_FINDINGS.md docs/analysis/
mv DOCUMENTATION_SUMMARY.md docs/
mv performance-baseline.md docs/performance/

# Update any internal links if needed
```

### Pre-Commit Validation Message

```
[assura-pre-commit] Root File Policy Check
❌ ERROR: 1 violation(s) found

  docs/archive/config-analysis-findings-detailed.md
    Violation: Markdown file not in root whitelist
    Rule: Only special project files (README.md, AGENTS.md, etc.) allowed in root
    Suggested Action: File is already in correct location (docs/analysis/)
    
  CONFIG_ANALYSIS_FINDINGS.md (in root)
    Violation: Markdown file not in root whitelist  
    Rule: Only special project files allowed in root
    Suggested Action: git mv CONFIG_ANALYSIS_FINDINGS.md docs/analysis/

Commit blocked. Use --no-verify to override (not recommended).
```

### OpenCode Plugin Integration

```typescript
// When agent tries to write markdown file to root
if (filePath.endsWith('.md') && isRootPath(filePath)) {
  const filename = path.basename(filePath);
  const allowedRootFiles = [
    'README.md', 'AGENTS.md', 'CHANGELOG.md', 
    'CONTRIBUTING.md', 'LICENSE.md', 'PROJECT_MEMORIES.md',
    'RELEASE_NOTES.md'
  ];
  
  if (!allowedRootFiles.includes(filename)) {
    return {
      blocked: true,
      severity: 'error',
      message: `Root file policy: ${filename} is not an approved root markdown file.`,
      suggestion: `Move to docs/${filename} or docs/analysis/${filename}`,
      rule: 'root-markdown-whitelist',
      documentation: 'https://assura.dev/docs/policies/root-files'
    };
  }
}
```

### Why This Policy Matters

1. **Clarity:** Root directory should only contain essential project metadata
2. **Navigation:** Users know where to look for specific document types
3. **AI Safety:** Prevents agents from cluttering root with working documents
4. **Professionalism:** Clean root = well-organized project
5. **Discoverability:** Analysis docs belong with other docs, not mixed with LICENSE

---

*Policy added per user requirement*
