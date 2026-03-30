# Assura Constitution

## Version 1.1

**IMPORTANT**: This document establishes core principles only. It does not contain specific notation examples or syntax definitions. Notation is documented in SPEC.md and user-facing documentation.

This document establishes the core principles and requirements for the Assura project. All changes must be reviewed against these principles before merging.

---

## 1. Configuration Philosophy

### 1.1 User Experience First
Configuration files should be:
- **Readable**: Clear intent without excessive syntax
- **Writable**: Minimal boilerplate for common cases
- **Reviewable**: Easy to understand what rules apply where

### 1.2 Structure-First Representation
- **Nesting represents project structure**: Indentation mirrors directory hierarchy
- **Files and folders are first-class citizens** in the configuration
- **Non-structural elements use array notation**: Directives and constraints expressed as list items
- **Visual fidelity**: Config structure should resemble the actual project tree

### 1.3 YAML-First, JSON-Compatible
- Primary format is YAML-like with preprocessing for convenience
- Preprocessor adds required quotes for valid YAML parsing
- Must convert cleanly to JSON for tooling and interoperability
- Standard YAML libraries can parse preprocessed output
- Array notation ensures JSON compatibility

### 1.4 LS-Lint Compatibility
Assura must be **feature-complete with ls-lint**:
- All ls-lint directives have equivalent Assura syntax
- Performance target: 2x faster than ls-lint for equivalent workloads
- Migration path: ls-lint configs can be converted automatically

---

## 2. Rule System

### 2.1 Rule Definition
Rules define reusable constraints that can be applied across the project.

### 2.2 Rule Application
Rules are applied through directives, supporting single or multiple rule references.

### 2.3 Rule Composition
Rules compose through logical operators:
- OR logic for alternative constraints
- AND logic for combined constraints via array notation

### 2.4 Array Notation for Attributes
Non-structural constraints use array notation to:
- Clearly separate structure from configuration
- Enable natural JSON conversion
- Support extensibility

### 2.5 Rule Extension
Rules can extend other rules to build upon base constraints.

---

## 3. Pattern System

### 3.1 Glob Foundation with Regex Enhancement
- **Glob patterns for discovery**: Fast file system traversal using glob matching
- **Regex for precise validation**: Regular expressions for complex pattern matching on discovered files
- **Hybrid approach**: Glob provides performance, regex provides precision
- **Valid regex only**: String patterns must be valid regular expressions, no custom string syntax

### 3.2 Structural Patterns
Files and directories represent project structure directly in configuration.

### 3.3 Pattern Quantifiers
Quantifiers define count constraints for pattern matching.

### 3.4 Variable Substitution
Support for capturing and reusing filename and path components.

### 3.5 File Alternation
OR logic for specifying alternative file existence requirements.

---

## 4. Constraints

### 4.1 Naming Conventions
Support for standard naming conventions.

### 4.2 File Attributes
Constraints on file properties such as line count and size.

### 4.3 File Pairing
Mechanism for establishing relationships between files.

---

## 5. Extensibility

### 5.1 Plugin Architecture
Users can extend functionality through plugins for:
- Custom validators
- New constraint types
- External rule sources

### 5.2 Constraints on Extensions
All extensions must:
- Be valid YAML when preprocessed
- Not conflict with core syntax
- Be documentable in user-facing specifications

---

## 6. Performance Requirements

### 6.1 Performance Target
Assura must be **2x faster than LS-Lint** for equivalent workloads across all project sizes.

### 6.2 Critical Performance Pitfalls to Avoid

#### 6.2.1 O(n²) Directory Walking
**Problem**: Recursive directory constraints combined with external file walking create quadratic complexity.

**Example (ANTI-PATTERN)**:
```rust
// External walk: O(n) for n files
WalkDir::new(path).into_iter().for_each(|entry| {
    // DirectoryConstraint with recursive=true
    // Each directory re-walks all subdirectories: O(n) per directory
    // Total: O(n²) - CATASTROPHIC for large projects
});
```

**Solution**:
- Use `non_recursive()` directory validation when walking externally
- OR use a single recursive walk and batch all validation
- Cache directory metadata to avoid redundant filesystem calls

#### 6.2.2 Redundant Metadata Calls
**Problem**: Multiple constraints calling `fs::metadata()` on the same file.

**Solution**:
- Cache metadata in constraint context
- Share file information across constraints
- Use `PathBuf` metadata caching when available

#### 6.2.3 Sequential File Walking
**Problem**: Using `std::fs::read_dir()` or `walkdir` sequentially instead of parallel walkers.

**Solution**:
- Use `jwalk` with `Parallelism::RayonNewPool(0)` for parallel directory traversal
- Parallel validation after collecting entries
- Batch validation to reduce overhead

### 6.3 Optimization Principles

1. **Walk once, validate many**: Single filesystem walk, multiple constraint validations
2. **Parallelize I/O**: Use parallel directory walkers (jwalk)
3. **Parallelize CPU**: Use rayon for constraint validation
4. **Avoid redundant work**: Cache metadata, avoid recursive re-walking
5. **Batch operations**: Validate files in batches, not one-by-one

### 6.4 Benchmarking Requirements

All performance claims must be validated against:
- **Small projects**: 50-100 files
- **Medium projects**: 500-1,000 files  
- **Large projects**: 5,000-10,000 files
- **Realistic file types**: Not just .txt files

---

## 7. Observability

### 6.1 Compiled Rules Inspection
Users must be able to:
- View the complete set of active rules
- See how overrides and inheritance resolved
- Export compiled rules to JSON

### 6.2 Aggregation and Escalation
- Aggregate pass/fail status at any directory level
- Define thresholds for escalation
- Support configurable notification strategies

---

## 8. Change Process

### 8.1 Before Merging
All PRs must:
1. Update CONSTITUTION.md if changing core principles
2. Update SPEC.md with feature status
3. Pass performance benchmarks (2x ls-lint target)
4. Maintain ls-lint feature parity

### 8.2 Constitution Changes
- Core principles require explicit review
- Syntax changes must show migration path
- Breaking changes require version bump

---

## 9. References

- **AGENTS.md**: Project guidance (references this Constitution)
- **SPEC.md**: Feature specifications and implementation status
- **docs/**: User-facing documentation

---

*Last Updated: 2026-03-24*
*Version: 1.2 - Added Glob Foundation with Regex Enhancement principle*
*Next Review: 2026-04-24*
