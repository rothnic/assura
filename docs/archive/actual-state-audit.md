---
title: 'Assura Codebase — Actual State Audit'
status: historical
---

# Assura Codebase — Actual State Audit

**Date**: 2026-04-29
**Auditor**: Systematic code inspection via SSH to vps-dev
**Methodology**: Direct source code reading, test execution, benchmark verification
**Scope**: All Rust source, tests, benchmarks, configs, CLI, and documentation

---

## 1. Codebase Scale

| Metric | Count |
|--------|-------|
| Rust source lines | ~22,880 |
| Source files | ~45 |
| Integration test lines | ~2,865 |
| Integration test files | 6 |
| Benchmark lines | ~2,310 |
| Benchmark suites | 5 |
| Documentation lines | ~10,779 |
| Documentation files | ~20 |
| Total tests | 292 (289 pass, 3 ignored) |

---

## 2. Configuration Formats (Two Competing Formats)

### 2.1 Legacy V1 Format (`config.yml`)
- Uses `version: "2.0"`, `structure:`, `files:`, `children:` hierarchy
- Types: `DirectoryNode`, `FileBundle`
- Parsed by: `ConfigLoader` (in `src/config/loader.rs`)
- Status: **Active** — this is the self-validation config for the Assura project itself

### 2.2 New AST Format (`config.new.yml`)
- Uses `rules:`, `policy:`, `apply:`, `contexts:`, `messages:`
- Types: `Config`, `Rule`, `PolicyNode`, `Constraint`
- Parsed by: `ConfigParser` (in `src/config/parser.rs`)
- Status: **Partially implemented** — newer but CLI does not use it

### 2.3 Critical Issue
The two formats are **incompatible** and parsed by entirely different parsers. There is **no migration path** between them. The `ConfigLoader` and `ConfigParser` do not share code.

---

## 3. What Is Actually Implemented (Working Code)

### 3.1 Validation Engine (`src/validation/mod.rs`)
| Function | Status | Notes |
|----------|--------|-------|
| `ValidationEngine::validate_file()` | ✅ Working | Resolves rules, validates constraints, attaches severity prefix |
| `ValidationEngine::should_block()` | ⚠️ Working but fragile | String-matches `[level]` in message instead of using structured data |
| `ValidationEngine::get_violation_level()` | ✅ Working | Resolves context-specific violation level |
| Directory walking | ❌ **NOT IMPLEMENTED** | `Cli::check()` has `TODO: Walk directory tree and validate files` |

### 3.2 Constraint Validation (`src/validation/constraints.rs`)
| Constraint | Status | Notes |
|------------|--------|-------|
| `Naming` | ✅ Working | 7 conventions in AST validator (12 in separate `CaseConvention` enum) |
| `Lines` | ✅ Working | `Range::Exact` and `Range::RangeString` |
| `Size` | ✅ Working | Reads file metadata, parses KB/MB/GB |
| `ConstraintsArray` | ✅ Working | Validates multiple constraints |
| `Exists` | ❌ **STUBBED** | Always returns `ValidationResult::pass("exists")` |

### 3.3 Rule Resolution (`src/validation/resolver.rs`)
- `RuleResolver::resolve()` traverses policy tree, matches file patterns
- Supports `${name}` variable patterns, glob patterns (`*`), exact matches
- Resolves `apply` directives, inline constraints, violation entries, messages
- **Messages are collected but NOT used by ValidationEngine**

### 3.4 Context Matching (`src/validation/context.rs`)
- `ContextMatcher::match_context()` matches by hook, branch pattern, version range, env vars
- `ExecutionContext::from_env()` reads `ASSURA_HOOK`, `ASSURA_BRANCH`, `GIT_BRANCH`, `ASSURA_VERSION`
- `ViolationLevel` enum: Info, Warn, Block, Notify

### 3.5 Pairing Validation (`src/validation/pairing.rs`)
- `PairingValidator::find_requirements()` scans for variable patterns
- `PairingValidator::validate_pairings()` checks paired files
- **NOT integrated into ValidationEngine** — standalone module
- Tests are `#[ignore]` with note "Phase 4: File pairing validation"

### 3.6 Config Parsing (`src/config/parser.rs`, `src/config/preprocessor.rs`)
- `ConfigParser::parse()` preprocesses YAML and parses to AST
- `YamlPreprocessor::process()` quotes extension/glob/variable keys, normalizes ranges
- **BUG**: Preprocessor regex `NEEDS_QUOTING` uses `\$\w+` which matches `$name` but NOT `${name}`

### 3.7 Legacy Config System (`src/config/types.rs`, `engine.rs`, `validator.rs`, `inheritance.rs`, `loader.rs`)
- Full V2 unified config format with `Rule`, `PolicyNode`, `PolicyEntry`, `InlineRule`, `Directive`, `ApplyEntry`
- `PolicyEngine::resolve()` with specificity scoring (exact > glob > extension)
- `ValidationEngine::validate_file()` for legacy format
- `RuleResolver` for hierarchical inheritance with `inherit: true/false`
- `ConfigLoader::load()/parse()/save()` with `validator::Validate` integration
- **This is a large, complete implementation with NO user-facing documentation**

### 3.8 LS-Lint Compatibility (`src/ls_compat/parser.rs`)
- `LsLintParser::parse()` — parses `.ls-lint.yml` flat format
- `LsLintParser::convert_to_assura()` — converts to AST format
- `MigrationTool::migrate()` — generates Assura YAML
- `MigrationTool::generate_report()` — counts rules
- **100% feature parity claimed in docs** for: extension rules, path rules, OR syntax, ignore patterns, exists directives, multi-part extensions

### 3.9 Constraint System (`src/constraints/`)
| Component | Status |
|-----------|--------|
| `Constraint` trait | ✅ Implemented |
| `ConstraintEngine` | ✅ Implemented |
| `NamingConstraint` | ✅ 12 case conventions |
| `FileSizeConstraint` | ✅ Configurable limits |
| `ChildrenLimitConstraint` | ⚠️ Exists but NOT integrated |
| `DirectoryConstraint` | ✅ Directory naming with exclusions |
| `MultiPartExtensionRule` | ✅ `.d.ts`, `.test.js` |
| `MultipleRuleSyntax` | ✅ OR syntax |
| `PathRule` | ✅ Path-specific rules |
| `SeverityConfig` | ✅ Maturity-based severity |
| `TriggerRegistry` | ✅ File change, maturity, manual triggers |

### 3.10 Intelligence Graph (`src/intelligence/`)
- `IntelligenceGraph` — petgraph-based directed graph
- `GraphBuilder` — walks directories with `walkdir` (sequential and parallel with rayon)
- `GraphQuery` — find by type, extension, name pattern, children, path between nodes
- `GraphPersistence` — save/load JSON/YAML/bincode

### 3.11 Markdown Validation (`src/markdown/`)
- `MarkdownParser` — frontmatter, headings, links, code blocks
- `FrontmatterSchema` — required fields, type checking
- `HeadingValidator` — H1 required, single H1, hierarchy, max depth
- `TemplateDefinition` — required sections, ordering, content patterns
- `MarkdownConstraint` — applies to `.md`, `.markdown`, `.mdown`

### 3.12 Maturity Detection (`src/maturity/`)
- `MaturityDetector` — collects signals from Git, filesystem, environment
- `MaturityDecisionEngine` — evaluates signals, produces `MaturityReport`
- `MaturityLevel`: Raw, Developing, Mature, Established
- Signal types: Git (repo age, commits, branches), Filesystem (file count, structure), Environment (CI/CD, package managers)

### 3.13 CLI (`src/cli/`)
| Command | Status | Notes |
|---------|--------|-------|
| `check` | ❌ **STUB** | Prints "not yet implemented", returns `ExitCode::Success` |
| `status` | ❌ **STUB** | Same |
| `init` | ❌ **STUB** | Same |
| `watch` | ❌ **STUB** | Same |
| `migrate` | ✅ **WORKING** | Reads LS-Lint config, generates report, converts to Assura YAML |
| `info` | ✅ **WORKING** | Shows rules, contexts, policy entries |
| `hooks` | ✅ **WORKING** | Install/uninstall/status for pre-commit, pre-push, post-checkout |

---

## 4. Test Coverage (292 tests, 289 pass, 3 ignored)

| Test File | Tests | What They Actually Test |
|-----------|-------|------------------------|
| `constraint_tests.rs` | 22 | Engine registration, file size, naming conventions, severity, extensions |
| `filesystem_integration_tests.rs` | 5 | Temp dirs, naming on real files, size validation, walkdir traversal, multi-part extensions |
| `intelligence_graph_tests.rs` | 31 | Nodes, edges, queries, path finding, persistence, parallel builder |
| `ls_lint_tests.rs` | 23 | All 12 case conventions, directory validation, exclusions, recursive, OR syntax, path rules |
| `markdown_tests.rs` | 30 | Frontmatter, headings, templates, schema, unicode |
| `maturity_tests.rs` | 24 | Git repos, config files, CI/CD, signals, pipeline, level transitions |

**Inline tests**: 45 source files have `#[cfg(test)]` modules.

---

## 5. Performance Benchmarks

| Benchmark Suite | Status | What It Measures |
|-----------------|--------|------------------|
| `graph_benchmarks.rs` | ✅ Runnable | Graph construction (sequential vs parallel), queries |
| `constraint_validation.rs` | ✅ Runnable | Naming (all conventions), file size, cold vs warm |
| `ls_lint_benchmarks.rs` | ✅ Runnable | 20 benchmarks: conventions, directory, extensions, OR, path rules |
| `ls_lint_comparison.rs` | ⚠️ Partial | Head-to-head with LS-Lint binary (requires `npm install`) |
| `profiling.rs` | ✅ Runnable | Directory walking (jwalk vs walkdir), constraint overhead, context creation |

**Performance claims in docs**: 6.8x speedup over LS-Lint, 650K files/sec throughput.
**Verification status**: **CANNOT VERIFY** — comparison benchmarks require LS-Lint binary installation.

---

## 6. What Is Documented But NOT Implemented

1. **CLI `check` command** — documented in README as `assura check`, but is a stub
2. **CLI `init` command** — documented but stubbed
3. **CLI `watch` command** — documented but stubbed
4. **CLI `status` command** — documented but stubbed
5. **File system walking in validation** — `ValidationEngine` validates single files but never walks directories
6. **`exists` constraint validation** — parsed but stubbed in `ConstraintValidator`
7. **File pairing validation** — `PairingValidator` exists but not integrated, tests ignored
8. **`children_limit` constraint** — exists as type but not integrated into validation engines
9. **Cross-directory pairing** — documented in spec but not implemented
10. **`group` attribute for explicit pairing** — documented in spec but not implemented
11. **Message directive in validation output** — messages are resolved but never attached to results
12. **Context inheritance in resolver** — directory-level violation defaults not inherited

---

## 7. What Is Implemented But NOT Documented

1. **Legacy V1 config format** (`src/config/types.rs`, `engine.rs`, `validator.rs`, `inheritance.rs`, `loader.rs`) — extensive implementation, no user docs
2. **Two separate config parsers** — `ConfigParser` (AST) and `ConfigLoader` (legacy) — undocumented duality
3. **`ConfigExt` trait** — stub implementation in `parser.rs`
4. **Multiple `ValidationEngine` implementations** — one in `src/validation/mod.rs`, one in `src/config/validator.rs` — undocumented
5. **OpenSpec skills and prompts** in `.github/skills/` and `.github/prompts/` — not mentioned in main docs

---

## 8. Critical Architecture Issues

| Issue | Severity | Description |
|-------|----------|-------------|
| No file scanning | **CRITICAL** | Core value proposition (walk directory, validate files) is not implemented |
| Two incompatible config formats | **HIGH** | No migration path between legacy V1 and new AST format |
| `exists` is a no-op | **HIGH** | Always passes; breaks correctness guarantees |
| All 4 main CLI commands are stubs | **HIGH** | `check`, `status`, `init`, `watch` do nothing |
| `should_block()` uses string matching | **MEDIUM** | Matches `[level]` in message instead of structured data |
| Preprocessor regex bug | **MEDIUM** | `\$\w+` does not match `${name}` patterns |
| `ViolationLevel::should_block()` ignores `gate` | **MEDIUM** | Parameter is unused |
| Messages resolved but unused | **LOW** | `RuleResolver` collects messages; `ValidationEngine` ignores them |

---

## 9. Summary

**What works**:
- LS-Lint migration (`assura migrate`)
- Config info display (`assura info`)
- Git hooks management
- All constraint logic (naming, size, lines, conventions)
- Intelligence graph building and querying
- Markdown validation
- Maturity detection
- All 289 passing tests
- All benchmark suites (where dependencies are available)

**What does NOT work**:
- The primary CLI commands (`check`, `status`, `init`, `watch`)
- Directory scanning for validation
- `exists` constraint (always passes)
- File pairing validation
- `children_limit` constraint integration
- Message output in validation results

**Documentation accuracy**: Significant gaps between documented features and actual implementation. The README and user-facing docs describe a working tool, but the CLI is largely non-functional for its primary use case.

---

*End of Audit*
