# Phase 2 Review: Issues and Tech Debt

## Critical Issues (Must Fix)

### 1. **Incomplete ValidationEngine Logic** (`src/validation/mod.rs`)
**Lines 59-75:** The logic for adjusting results based on context level is incomplete and doesn't make sense:

```rust
// Current (buggy)
let adjusted_result = if level == ViolationLevel::Block && result.passed {
    result
} else if !result.passed {
    ValidationResult { passed: false, ... }
} else {
    result
};
```

**Problem:** 
- If level is Block and result passed, it just returns the result (doesn't block)
- If result failed, it recreates the same result
- The context level is never actually used to influence the result

**Fix:** ViolationLevel should determine if a violation blocks or not:
```rust
let adjusted_result = if !result.passed {
    // Failed constraint - severity based on context level
    let severity_message = format!("[{}] {}", level, result.message.unwrap_or_default());
    ValidationResult {
        passed: false,
        message: Some(severity_message),
        constraint_type: result.constraint_type,
    }
} else {
    result
};
```

### 2. **should_block Logic Error** (`src/validation/mod.rs:86`)
```rust
results.iter().any(|r| {
    !r.passed && ViolationLevel::Block.should_block("default", allowed_levels)
})
```

**Problem:** Always checks ViolationLevel::Block, not the actual level of the result.

**Fix:** Should check if the result's actual level should block.

### 3. **Missing File Pairing Validation** (`src/validation/resolver.rs`)
The resolver finds patterns like `${name}.tsx` and `${name}.test.tsx`, but doesn't validate that paired files actually exist.

**Current:** Only resolves constraints for existing files.
**Missing:** Validation that for each `${name}.tsx`, a `${name}.test.tsx` exists.

**Fix:** Need separate pairing validation pass.

## Medium Issues

### 4. **No Message Resolution** (`src/validation/`)
Messages defined in rules and policy items are resolved but never attached to ValidationResults.

**Location:** `resolver.rs` collects messages into `ResolvedConstraints.messages`, but `ValidationEngine.validate_file()` doesn't use them.

**Fix:** Attach messages to validation results based on context.

### 5. **Context Inheritance Incomplete** (`src/validation/resolver.rs`)
Directory-level violation defaults (e.g., `violation: [warn, ci:block]` at directory level) are not being inherited by files in subdirectories.

**Current:** Only collects violations from matching file patterns.
**Missing:** Collection of directory-level violation defaults.

### 6. **ConstraintItem Not Imported** (`src/validation/resolver.rs`)
```rust
use crate::config::ast::{Config, Rule, Constraint, FileItem, ApplyValue, ConstraintItem};
```

The `ConstraintItem` import exists but may not be resolving properly in all match arms.

### 7. **Range Type Inconsistency**
In `constraints.rs`, `Range::RangeString` is used but never parsed - it's just compared as string.

**Example:**
```rust
Range::RangeString(s) => Self::check_range(line_count, s),
```

But `check_range` expects a parsed range, not a string.

## Tech Debt

### 8. **Missing Error Types**
All validation returns `ValidationResult` which is good, but there's no structured error type for the validation engine itself (e.g., ConfigNotFound, InvalidPattern, etc.).

### 9. **Hardcoded Conventions**
Naming convention validators are hardcoded strings. Should use regex patterns or formal grammar.

### 10. **No Incremental Validation**
Every validation re-resolves rules from scratch. Should cache resolved rules per directory.

### 11. **Version Comparison Too Simple**
`version_gte` and `version_lte` use simple numeric comparison which doesn't handle semver properly (e.g., "2.0.0-alpha" vs "2.0.0").

### 12. **Test Coverage Gaps**
- No tests for context matching with env vars
- No tests for file pairing validation
- No tests for message resolution
- No integration tests with real filesystem

## Recommended Fixes Before Phase 3

### Priority 1 (Critical)
1. Fix ValidationEngine logic to properly use ViolationLevel
2. Fix should_block to check actual violation level
3. Implement file pairing validation

### Priority 2 (Important)
4. Add message resolution to ValidationResults
5. Implement context inheritance in resolver
6. Parse Range strings properly

### Priority 3 (Tech Debt)
7. Add structured error types
8. Add caching for resolved rules
9. Improve version comparison

## Decision Needed

**Question:** Should we fix Priority 1 issues now, or proceed to Phase 3 and fix them incrementally?

**Recommendation:** Fix Priority 1 now - they affect core functionality and will be harder to change once CLI depends on them.

---

*Review completed: 2026-03-24*
*Status: 3 Critical issues found, need decision on fix timing*
