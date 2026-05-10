# Assura Project - Agent Guidelines

## Project Overview

**Assura** is a dependency-aware file system validation engine written in Rust. It provides:

- **Dependency graph analysis** for detecting circular dependencies and validation ordering
- **Rule-based validation** with configurable severity levels (Critical, High, Medium, Low)
- **File system watching** for continuous validation during development
- **Parallel execution** for large-scale validation performance
- **Extensible plugin architecture** for custom validators

### Key Dependencies
- `tokio` - Async runtime for file watching and parallel operations
- `petgraph` - Graph algorithms for dependency analysis
- `serde`/`serde_yaml` - Configuration file parsing
- `clap` - CLI interface
- `regex`/`glob` - Pattern matching for file discovery
- `notify` - File system event watching

## Agent Coordination Guidelines

### Task Types and Agent Roles

**General Agent** (default)
- Feature implementation and bug fixes
- Refactoring and optimization
- Documentation updates

**Architect Agent**
- Core engine design decisions
- API design and module boundaries
- Performance critical code review

**Test Agent**
- Test suite development
- Edge case identification
- Benchmark creation

### Workflow Guidelines

1. **Always start by reading AGENTS.md** - Check for project-specific context
2. **Read SKILL.md files** before using built-in skills
3. **Check existing code patterns** before introducing new ones
4. **Use concurrency safely** - Assura uses multi-threading (Rayon, Tokio)
5. **Document public APIs** - All public structs/functions need rustdoc
6. **Write tests for validation logic** - All validators need unit tests

### Decision Authority

- **Code changes within existing modules**: Any agent can implement
- **New module creation**: Consult project documentation first
- **Breaking API changes**: Requires explicit approval
- **Dependency additions**: Must be justified in code comments

## Available Skills

### Built-in Skills

Located in `/workspace/repos/research/assura/skills/built-in/`:

| Skill | Description | Use When |
|-------|-------------|----------|
| `find-skills` | Search vercel-labs/skills registry for reusable skills | Need to discover existing skills instead of writing from scratch |

**Usage**: Read `skills/built-in/<skill-name>/SKILL.md` for instructions.

### Recommended External Skills

Install these via `npx skills add <owner/repo> --skill <skill-name>`:

**High Priority** (reference for validation patterns):
- `vercel-labs/agent-skills:web-design-guidelines` - 100+ rule audit patterns
- `vercel-labs/agent-skills:react-best-practices` - Rule prioritization methodology
- `supercent-io/skills-template:file-organization` - File structure validation

**Medium Priority** (reference for workflows):
- `supercent-io/skills-template:git-workflow` - Pre-commit validation integration
- `supercent-io/skills-template:security-best-practices` - File permission validation
- `squirrelscan/skills:audit-website` - Audit report formatting

**Reference** (techniques and patterns):
- `supercent-io/skills-template:codebase-search` - Pattern matching for file discovery
- `supercent-io/skills-template:performance-optimization` - Validation engine tuning
- `supercent-io/skills-template:technical-writing` - Error message formatting

See `docs/archive/skills-research.md` for detailed analysis.

## Coding Standards and Conventions

### Rust Standards

- **Edition**: 2021
- **Formatting**: `cargo fmt` (default configuration)
- **Linting**: `cargo clippy` with no warnings
- **Documentation**: All public items must have rustdoc

### Code Organization

```
src/
  main.rs           # CLI entry point
  lib.rs            # Library exports
  core/
    mod.rs          # Core engine module
    graph.rs        # Dependency graph
    validator.rs    # Validation engine
  rules/
    mod.rs          # Rule definitions
    severity.rs     # Severity levels (Critical, High, Medium, Low)
  fs/
    mod.rs          # File system operations
    watcher.rs      # File watching
  config/
    mod.rs          # Configuration parsing
    loader.rs       # Config file loading
  report/
    mod.rs          # Report generation
    formatter.rs    # Output formatting
```

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

## Communication Protocols

### File System Communication

Agents communicate through:

1. **This file** (`AGENTS.md`) - Project-wide guidelines
2. **Code comments** - Inline rationale for decisions
3. **Rustdoc** - Public API documentation
4. **Git commits** - Conventional commit format

### Status Reporting

When completing work:

1. **Report what was changed** - File paths and key modifications
2. **Reference skill usage** - If external skills were applied
3. **Note breaking changes** - API changes or behavioral differences
4. **Suggest next steps** - If continuation work is needed

### Error Reporting

When encountering issues:

1. **State the symptom clearly** - What is not working
2. **Provide context** - Relevant file paths and code sections
3. **Mention attempted solutions** - What has already been tried
4. **Tag appropriate agent** - If specialized knowledge is needed

### Version Control

- Use conventional commits: `type(scope): description`
- Types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`
- Scope examples: `core`, `rules`, `config`, `fs`, `report`
- Reference issues/PRs in commit messages when applicable

## Quick Reference

### Common Commands

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

# Watch mode (for development)
cargo watch -x test
```

### Project Constraints

- **Minimum Rust version**: 1.70.0
- **Supported platforms**: Linux, macOS, Windows
- **Concurrency**: Thread-safe by design
- **Memory**: No unbounded allocations in hot paths

## Backwards Compatibility Policy

**No internal backwards compatibility until 1.0 release.**

- Configuration formats, APIs, and internal structures may change without migration paths
- The LS-Lint compatibility layer (`ls_compat.rs`) is maintained for testing purposes only
- External users should expect breaking changes in pre-1.0 versions
- Once 1.0 is released, standard semantic versioning will be followed

## References

- Skills Research: `docs/archive/skills-research.md`
- Cargo Configuration: `Cargo.toml`
- Skill Examples: `skills/built-in/*/SKILL.md`

---

*Last updated: 2026-03-19*
*For questions about agent capabilities, ask: "What can you help me with?"*
<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

Trellis is the canonical workflow, task, and spec system for Assura.

When starting work:
- Read `.trellis/workflow.md` for the active development process.
- Check `.trellis/tasks/` for active and archived work.
- Check `.trellis/spec/` for durable project specs and constraints.
- Check `.trellis/workspace/` for session/developer continuity.
- Use `.assura/config.yml` and `assura check .` for project structure validation.

For Codex, Trellis context injection depends on user-level
`features.hooks = true` and one-time `/hooks` approval. If hooks are not active,
read `.agents/skills/trellis-start/SKILL.md` manually before starting Trellis
workflow work.

OpenSpec and `specs-bak/` are historical unless a newer ADR says otherwise.
See `docs/analysis/2026-05-09-trellis-governance-adr.md` and
`docs/analysis/2026-05-09-documentation-cleanup-register.md`.

Keep this managed block so `trellis update` can refresh the instructions.

<!-- TRELLIS:END -->
