# Assura Project Memories

This file contains important project context and conventions that should be preserved across sessions.

## Project Overview

**Assura** is a dependency-aware file system validation engine written in Rust. Version 0.1.0 is production ready with 155+ tests passing.

### Key Dependencies
- `tokio` - Async runtime for file watching and parallel operations
- `petgraph` - Graph algorithms for dependency analysis
- `serde`/`serde_yaml` - Configuration file parsing
- `clap` - CLI interface
- `regex`/`glob` - Pattern matching for file discovery
- `notify` - File system event watching

## Architecture

### Module Structure
```
src/
  main.rs           # CLI entry point
  lib.rs            # Library exports
  cli/              # CLI module
    args.rs         # Command line arguments
    commands.rs     # Command implementations
    config.rs       # Configuration management
    hooks.rs        # Git hooks
    output.rs       # Output formatting
  config/v2/        # V2 configuration
    structure.rs    # Structure-first config
    inheritance.rs  # Rule inheritance
    loader.rs       # Config loading
    ls_compat.rs    # LS-Lint compatibility (testing only)
  constraints/      # Validation constraints
    naming.rs       # Naming conventions
    ls_lint/        # LS-Lint parity features
  intelligence/     # Dependency graph
  markdown/         # Markdown validation
  maturity/         # Maturity detection
```

## Backwards Compatibility Policy

**No internal backwards compatibility until 1.0 release.**

- Configuration formats, APIs, and internal structures may change without migration paths
- The LS-Lint compatibility layer (`ls_compat.rs`) is maintained for testing purposes only
- External users should expect breaking changes in pre-1.0 versions
- Once 1.0 is released, standard semantic versioning will be followed

## Completed Phases

### Phase 5: LS-Lint Parity
- 4 new case conventions (flatcase, FLATCASE, COBOL-CASE, Train-Case)
- Directory validation with exclusions
- Complex extensions (.d.ts, .test.js)
- Multiple rules syntax (OR operator)
- Path-specific rules with glob patterns
- Comprehensive benchmarks

### Phase 7: OpenCode Plugin
- TypeScript plugin for OpenCode integration
- npm package ready for publication

### Phase 8: CLI, Git Hooks & Website
- Full CLI with check, status, init, watch commands
- Git hooks for pre-commit, pre-push, post-checkout
- Astro website with Starlight (20 pages)

### Phase 9: Dogfooding & Release
- Self-validation configured
- All validation errors fixed
- crates.io and npm releases prepared
- Version 0.1.0 production ready

## Coding Standards

### Rust Standards
- **Edition**: 2021
- **Formatting**: `cargo fmt` (default configuration)
- **Linting**: `cargo clippy` with no warnings
- **Documentation**: All public items must have rustdoc

### Error Handling
- Use `thiserror` for structured error types
- Use `anyhow` for application-level error handling
- All I/O operations must use proper error propagation
- Validation errors must include context (file, line, rule)

### Testing
- Unit tests in `tests/` directory
- Benchmarks in `benches/` directory
- Use `pretty_assertions` for readable test failures
- Use `mockall` for mocking in tests
- Use `tempfile` for test fixtures

### Async Patterns
- Prefer `tokio::spawn` for parallel validation
- Use `rayon` for CPU-intensive graph operations
- Handle cancellation gracefully with `tokio::select!`
- All async functions should be `Send + 'static`

## Version Control

- Use conventional commits: `type(scope): description`
- Types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`
- Scope examples: `core`, `rules`, `config`, `fs`, `report`

## Common Commands

```bash
# Build
cargo build --release

# Test
cargo test
cargo test --lib
cargo test --integration

# Lint
cargo fmt
cargo clippy -- -D warnings

# Benchmark
cargo bench

# Documentation
cargo doc --open
```

## Project Constraints

- **Minimum Rust version**: 1.70.0
- **Supported platforms**: Linux, macOS, Windows
- **Concurrency**: Thread-safe by design
- **Memory**: No unbounded allocations in hot paths

## Recovery Process

If machine restart occurs during long-running orchestration:
1. Check git history to identify which commits were successfully made
2. Examine source files to verify what code exists vs. what was in-progress
3. Cross-reference with implementation plan to determine exact restart point
4. Resume from last completed phase, not from beginning

Git serves as the state checkpoint system. Each phase commit acts as a recovery point.
