# Implementation Gaps Report

This document tracks the current implementation status of features defined in the [Configuration Specification](./CONFIGURATION_SPEC.md).

## Legend

- **Implemented** - Feature is fully functional and tested
- **Partially Implemented** - Feature exists but has gaps or limitations
- **Not Implemented** - Feature is not yet implemented
- **Unknown** - Status could not be determined

---

## Implemented

### Variable Capture (`${name}`)

**Status**: Implemented

**Evidence**:
- Parsing/Preprocessing: `src/config/preprocessor.rs:150` quotes `${name}.tsx` patterns to prevent YAML interpretation
- Pattern Matching: `src/validation/resolver.rs` - `RuleResolver::pattern_matches()` handles `${name}` patterns with regex-based matching
- Grouping: `src/validation/pairing.rs` - `PairingValidator` replaces variable base for grouping

**Notes**:
- Variable capture works for file naming constraints and basic pairing
- Cross-directory pairing with same variable name is NOT implemented (see Cross-Directory Pairing below)

### `lines` Directive

**Status**: Implemented

**Evidence**:
- AST: `src/config/ast.rs:40` - `Constraint::Lines { lines: Range }`
- Validation: `src/validation/constraints.rs:40` - `ConstraintValidator::validate()` checks line count
- Legacy: `src/config/validator.rs:164` - `validate_max_lines()` supports `..400`, `50..500`, exact numbers

**Notes**:
- Supports exact numbers, open-ended ranges (`..400`), and bounded ranges (`50..500`)
- Works in both AST and legacy validation engines

### `apply` Directive for Rules

**Status**: Implemented

**Evidence**:
- AST: `src/config/ast.rs:30` - `FileItem::Apply { apply: ApplyValue }`
- Resolution: `src/validation/resolver.rs` - `RuleResolver::resolve_file_items()` resolves rule names
- Merge: `src/validation/resolver.rs:15` - `ResolvedConstraints` merges rule constraints

**Notes**:
- Supports single rule (`apply: react-component`) or array of rules (`apply: [react-component, test-pattern]`)
- Rule resolution follows specificity ordering (exact > wildcard > directory)

### Context Matching (`when`)

**Status**: Implemented

**Evidence**:
- AST: `src/config/ast.rs:60` - `ViolationEntry::ContextSpecific { context, level }`
- Matching: `src/validation/context.rs` - `ContextMatcher` matches by hook, branch, version, env vars
- Engine: `src/validation/mod.rs` - `ValidationEngine::validate_file()` uses `context_matcher.resolve_level()`

**Notes**:
- Supports `hook`, `branch`, `version`, and environment variable conditions
- Context-specific violation levels are resolved correctly

---

## Partially Implemented

### `exists` Directive

**Status**: Partially Implemented

**Evidence**:
- Parsing: `src/config/ast.rs:425` - `FileItem::Exists { exists: Range }` is parsed
- Legacy: `src/config/types.rs:226` - `Directive::Exists(Vec<String>)` exists
- Validation Gap: `src/validation/constraints.rs:59` - Returns `ValidationResult::pass("exists")` with comment "Exists is handled at directory level, not file level"
- Legacy Gap: `src/config/validator.rs` - `validate_file()` does NOT check `exists` at all

**Notes**:
- The directive is parsed but validation is stubbed out
- Legacy `ExistsValidation` in `src/config/config.rs:141` is for a different feature (required files/directories at root level)
- The spec's `exists: 1` (require exactly one file matching pattern) is not enforced

### `violation` Directive with Contexts

**Status**: Partially Implemented

**Evidence**:
- AST: `src/config/ast.rs:279` - `ViolationEntry` enum with `Level(String)` and `ContextSpecific { context, level }`
- Resolution: `src/validation/context.rs` - `ContextMatcher` resolves context-specific levels
- Engine: `src/validation/mod.rs` - `ValidationEngine::validate_file()` attaches violation level

**Gaps**:
- `should_block()` logic is awkward: checks if result message contains level string rather than using resolved level directly
- The violation level is prefixed to the message but not used for actual blocking decisions

### `message` Directive

**Status**: Partially Implemented

**Evidence**:
- AST: `src/config/ast.rs:324` - `Message` struct with `contexts: HashMap<String, String>`
- Resolution: `src/validation/resolver.rs:15` - `ResolvedConstraints.messages` stores merged messages

**Gaps**:
- `ValidationEngine::validate_file()` in `src/validation/mod.rs` does NOT use `resolved.messages`
- Only prefixes constraint's own message with violation level
- Custom context-specific messages from config are completely ignored

### `require_test` / Test File Pairing

**Status**: Partially Implemented

**Evidence**:
- Types: `src/config/types.rs:41,207` - `require_test: Option<String>` on `Rule` and `InlineRule`
- Resolution: `src/config/engine.rs:325` - `PolicyEngine` resolves `require_test` patterns

**Gaps**:
- `config::validator::ValidationEngine::validate_file()` does NOT check `require_test` at all
- Only `validate_naming`, `validate_max_lines`, `validate_max_size`, `validate_require_docs` are called
- The pairing requirement exists in data structures but is never validated

---

## Not Implemented

### Cross-Directory Pairing

**Status**: Not Implemented

**Evidence**:
- `src/validation/pairing.rs` - `PairingValidator::group_patterns()` uses `full_path.replace(pattern, var_base)`
- This means `src/components/${name}.tsx` and `tests/components/${name}.test.tsx` get different group keys

**Notes**:
- The spec implies implicit pairing: files with same `${name}` across directories should pair automatically
- Current implementation only pairs within same directory structure
- The `PairingValidator` is exported but NOT integrated into `ValidationEngine`

### `group` Attribute for Explicit Pairing

**Status**: Not Implemented

**Evidence**:
- No `group` field exists in `FileItem`, `ConstraintItem`, or `PairingRequirement`
- No parsing logic for `group` attribute in `src/config/ast.rs`

**Notes**:
- The spec mentions explicit pairing via `group: "api-routes"`
- This would allow pairing files that do not share variable names or directory structure

### `children_limit` / `max_children` Constraint

**Status**: Not Implemented

**Evidence**:
- `src/constraints/children_limit.rs` - `ChildrenLimitConstraint` struct exists
- NOT integrated into either validation engine
- NOT referenced in `ValidationEngine` or CLI

**Notes**:
- The constraint type exists but is orphaned
- No validation logic connects it to directory scanning

### CLI `check` Command

**Status**: Not Implemented

**Evidence**:
- `src/cli/commands.rs:69` - TODO comment: "Walk directory tree and validate files"
- File scanning and validation loop is not implemented

**Notes**:
- The command exists in CLI but only shows help/usage
- No actual file system traversal or validation execution

---

## Unknown

### `severity` Directive Integration

**Status**: Unknown

**Evidence**:
- `src/config/types.rs:226` - `Directive::Severity(SeverityLevel)` exists in legacy types
- `src/config/ast.rs` - No equivalent in AST types

**Notes**:
- Legacy directive exists but unclear if it is wired to validation output
- AST system uses `ViolationEntry` for levels instead

### `limit_children` Directive

**Status**: Unknown

**Evidence**:
- `src/config/types.rs:226` - `Directive::LimitChildren(usize)` exists in legacy types
- `src/constraints/children_limit.rs` - Constraint exists but not integrated

**Notes**:
- Legacy directive exists but not connected to actual validation
- May be superseded by unimplemented `children_limit` constraint

---

## Summary Table

| Feature | Status | Priority |
|---------|--------|----------|
| Variable Capture (`${name}`) | Implemented | - |
| `lines` Directive | Implemented | - |
| `apply` Directive | Implemented | - |
| Context Matching (`when`) | Implemented | - |
| `exists` Directive | Partially Implemented | High |
| `violation` Directive | Partially Implemented | Medium |
| `message` Directive | Partially Implemented | Medium |
| `require_test` / Pairing | Partially Implemented | High |
| Cross-Directory Pairing | Not Implemented | High |
| `group` Attribute | Not Implemented | Medium |
| `children_limit` Constraint | Not Implemented | Low |
| CLI `check` Command | Not Implemented | High |
| `severity` Directive | Unknown | Low |
| `limit_children` Directive | Unknown | Low |

---

## Recommendations

1. **High Priority**: Implement `exists` validation in `ConstraintValidator` and `ValidationEngine`
2. **High Priority**: Integrate `PairingValidator` into `ValidationEngine` and fix cross-directory grouping
3. **High Priority**: Implement CLI `check` command with file system traversal
4. **Medium Priority**: Wire `message` directive into validation output
5. **Medium Priority**: Implement `group` attribute for explicit pairing
6. **Low Priority**: Integrate `ChildrenLimitConstraint` or remove orphaned code
7. **Low Priority**: Clarify `severity` vs `violation` directive relationship

---

*Generated: 2026-04-28*
