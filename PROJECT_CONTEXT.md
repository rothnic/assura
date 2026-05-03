# Assura Project - Complete Context Document

**Generated:** 2026-04-06  
**Source:** 10 OpenCode sessions (4,409 parts extracted)  
**Sessions Analyzed:** ses_2c097bd63ffeUpDp42nNp1rNsI through ses_2f51b2656ffeBpl44EdJ2xv77N  
**Session Date Range:** March 20-30, 2026

---

## ⚠️ CRITICAL CORRECTION

**The "$requires" directive discussed in sessions was REJECTED, not adopted.**

Per **CONSTITUTION.md Section 1.2** (Structure-First Representation):
- "$requires" violates the principle that **nesting represents project structure**
- It **obscures file placement** by referencing files outside their structural location
- The **adopted approach** uses structural nesting with files as keys (not arrays)

**See section: [Rejected: "$requires" Directive](#rejected--requires-directive)**

---

## ⚠️ IMPORTANT DISCLAIMER

**This document reflects the project state and design discussions from March 2026.**

The project has continued to evolve since these sessions. As of April 6, 2026:
- The actual  uses a **v2.0 format** that may differ from these discussions
- The project has **uncommitted changes** including modifications to AGENTS.md, config files, and directory reorganization
- **Verify against current codebase** and CONSTITUTION.md for authoritative guidance

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Constitutional Principles](#constitutional-principles)
3. [Configuration System](#configuration-system)
4. [Key Technical Decisions](#key-technical-decisions)
5. [Session-by-Session Analysis](#session-by-session-analysis)
6. [Implementation Status](#implementation-status)
7. [Code Examples](#code-examples)
8. [Future Work](#future-work)

---

## Executive Summary

**Assura** is a dependency-aware file system validation engine written in Rust, designed to help developers enforce project conventions through configuration that mirrors actual project structure.

### Core Philosophy
Configuration structure **must visually represent** the actual project file tree (Constitution §1.2: Structure-First Representation).

### Current State (March 2026)

- **Core Engine:** ✅ Implemented (Rust)
- **Configuration Parser:** ✅ Multi-format support
- **Naming Conventions:** ✅ 12 types supported
- **LS-Lint Compatibility:** ✅ 100% feature parity
- **Tests:** ✅ 337+ passing

---

## Constitutional Principles

From **CONSTITUTION.md Section 1.2** - Structure-First Representation:

1. **Nesting represents project structure** - Indentation mirrors directory hierarchy
2. **Visual fidelity** - Config structure resembles the actual project tree
3. **Files and folders are first-class citizens** - No abstraction layers that obscure location
4. **Trailing  indicates directories** - Consistent with  conventions

### Critical Constraint
**Files should NEVER appear in array notation** - Arrays are reserved for directives/rules only. Files must appear as keys to maintain visual structure alignment.

---

## Configuration System

### Correct Approach: Unified Tree Auto-Discovery

The **adopted approach** places BOTH patterns in the policy tree structure at their actual locations. The pairing validator automatically creates relationships when patterns share the same variable name in the same directory.

**Configuration showing both source and test locations:**
```yaml
policy:
  src/components/:
    ${name}.tsx:           # Source files at their location
      - @react-component
      
    ${name}.test.tsx:      # Test files at their location (visible!)
      - @test
```

**How it works:**
1. Scanner finds both `${name}.tsx` and `${name}.test.tsx` in the same directory
2. Pairing validator extracts variable "name" from both patterns
3. For each `.tsx` file (e.g., `Button.tsx`), system checks for matching `.test.tsx` (e.g., `Button.test.tsx`)
4. If test is missing → violation reported

**Key advantages:**
- ✅ **Structure visible**: Both source and test locations shown in tree
- ✅ **No abstraction**: Files appear where they actually exist
- ✅ **Constitutional compliance**: Nesting represents actual structure
- ✅ **Auto-discovery**: No explicit pairing directive needed

### Variable Syntax

Use `${name}` (dollar sign + curly braces):
- `${name}` - Filename without extension
- `${name}.tsx` matches `Button.tsx`, `Input.tsx`
- `${name}.test.tsx` matches `Button.test.tsx`, `Input.test.tsx`

### Required vs Optional

**Required (1:1)** - Both patterns present:
```yaml
src/components/:
  ${name}.tsx:
    - @react-component
  ${name}.test.tsx:      # Required because present in structure
    - @test
```

**Optional (0..1)** - Use `?` quantifier:
```yaml
src/components/:
  ${name}.tsx:
    - @react-component
  ${name}.test.tsx?:     # Optional (may or may not exist)
    - @test
```

Or simply omit the test pattern entirely → no requirement enforced.

### Alternative: Rule-Based (When Structure Unknown)

For cases where test location varies by project:
```yaml
rules:
  react-component:
    require_test: "${name}.test.tsx"  # Variable pattern
    
policy:
  src/components/:
    apply: [@react-component]
```

**Note:** Rule-based approach obscures test location (not preferred).

### Decision 2: Files as Keys, Directives as Arrays

**Status:** ✅ ADOPTED

**Rule:**
- Files/globs = YAML keys (maintain structure)
- Directives/rules = Array items ( notation)

Example:


### Decision 3: LS-Lint Compatibility

**Status:** ✅ MAINTAINED

Assura maintains 100% compatibility with LS-Lint configuration patterns while extending capabilities.

---

## Session-by-Session Analysis

### Major Session: ses_2e53dc1c5ffeARS3WrP987j3Zn
**1,159 messages | March 23, 2026**

**Key Discussions:**
- Configuration format design (4 approaches evaluated)
- ** proposed and REJECTED** based on Constitution
- Structural nesting approach refined
- Pattern quantifiers for requirements
- LS-Lint compatibility strategies

**Outcome:**
- Hybrid directive approach selected (shorthand + structured)
- Structure-First Representation established as constitutional
-  dismissed as architecturally unsound

### Recent Sessions (March 26-30)
**Memory extraction sessions** - Compaction and knowledge preservation

---

## Implementation Status

### ✅ Completed (March 2026)

- [x] Core validation engine
- [x] Configuration parser (v2.0 format)
- [x] 12 naming convention types
- [x] LS-Lint compatibility layer
- [x] Directory/file existence checks
- [x] 337+ unit tests
- [x] Performance benchmarks

### 🟡 In Progress

- [ ] Structural nesting parser refinement
- [ ] Pattern quantifier implementation
- [ ] File pairing via attributes (not )
- [ ] Documentation updates

### 🔴 Not Started / Rejected

- [ ] ~~ directive~~ ❌ REJECTED (constitutional violation)
- [ ] Template system (pending design)
- [ ] IDE integrations

---

## Code Examples

### Example 1: React Component Structure



### Example 2: Rust Library



### Example 3: Full-Stack Project



---

## Future Work

### Phase 1: Parser Refinement
- [ ] Pattern quantifier implementation (+, *, ?)
- [ ] Attribute-based file pairing
- [ ] Structure validation

### Phase 2: Advanced Features
- [ ] Dependency graph visualization
- [ ] File watching integration
- [ ] IDE extensions

### Phase 3: Ecosystem
- [ ] GitHub Actions
- [ ] Pre-commit hooks
- [ ] Template system (revisit post-)

---

## Appendix: Key Files

- **CONSTITUTION.md** - Project principles and constraints
- **.assura/config.yml** - Current configuration (v2.0)
- **AGENTS.md** - Development guidelines
- **.trellis/workflow.md** - Development workflow

---

*Document generated from comprehensive analysis of 10 OpenCode sessions spanning Assura project development (March 20-30, 2026).*

**CRITICAL NOTE:** The $requires directive discussed in early sessions was **REJECTED** based on constitutional principles. Current configuration uses structural nesting with attributes instead.
