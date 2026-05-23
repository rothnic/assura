# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-20

### Added
- **Version 2 Configuration Format** - Structure-first hierarchical configuration system
- **Hierarchical Inheritance** - Parent rules automatically inherited by children with override capability
- **FileValidationBundle** - Bundled file-level validations (naming, size, line count, documentation)
- **MarkdownValidationBundle** - Markdown-specific validations (frontmatter, heading depth, link checking)
- **LS-Lint Compatibility Layer** - Automatic conversion of LS-Lint configs to V2 format
- **StructureNode Configuration** - Tree-based project structure definition
- **RuleResolver** - Efficient rule resolution with specificity-based precedence
- **Rule Inheritance Control** - `inherit: true/false` for fine-grained control
- **Path Specificity Scoring** - Automatic precedence for more specific paths
- **V2 Config Detection** - Automatic detection of V1 vs V2 configs
- **Unified Config Loading** - Support for both V1 and V2 configs transparently

### Configuration Features
- 12 built-in naming conventions: snake_case, kebab-case, camelCase, PascalCase, SCREAMING_SNAKE_CASE, dot.case, flatcase, FLATCASE, COBOL-CASE, Train-Case, lowercase, UPPERCASE
- Custom regex pattern support via `regex:` prefix
- Maximum file size validation (B, KB, MB, GB, TB units)
- Maximum line count validation (1-100000)
- Documentation requirement enforcement
- Frontmatter field validation for Markdown
- Heading depth limits (1-6)
- Dead link detection
- Required section validation
- Extension-based filtering
- Severity levels per bundle (critical, high, medium, low)

### Breaking Changes
- **Config Version Field**: V2 configs must specify `version: "2.0"` at the top level
- **Structure-First Syntax**: Rules are now defined within `structure:` hierarchy instead of flat `rules:` array
- **Path Syntax**: Directory paths must end with `/` (e.g., `src/` not `src`)
- **Bundle Location**: Validation options moved into `files:` and `markdown:` bundles within structure nodes
- **Inheritance Behavior**: Default is `inherit: true`, parent settings automatically apply to children

### Migration from V1
- V1 configs continue to work without changes
- Use `assura migrate --from v1 --to v2` for automatic conversion
- See [Migration Guide](docs/migration-guide.md) for detailed instructions
- Both V1 and V2 configs can coexist during transition period

### Performance Improvements
- Configuration loading: ~50% faster for typical projects
- Rule resolution: O(log n) lookup with specificity sorting
- Memory usage: ~30% reduction through shared parent bundles
- Validation runtime: Unchanged (V1 and V2 have same runtime performance)

### Documentation
- Comprehensive V2 configuration guide: [docs/config-v2.md](docs/config-v2.md)
- Step-by-step migration guide: [docs/migration-guide.md](docs/migration-guide.md)
- Updated examples for V2 format
- Troubleshooting section for common issues

## [0.1.0] - 2026-03-19

### Added
- Initial release of Assura
- Dependency-aware file system validation engine
- Core validation engine with dependency graph analysis
- Rule-based validation with configurable severity levels (Critical, High, Medium, Low)
- File system watching for continuous validation during development
- Parallel execution for large-scale validation performance
- Extensible plugin architecture for custom validators
- LS-Lint compatible naming convention validation (12 case conventions)
- Markdown validation with frontmatter, heading hierarchy, and template support
- Maturity detection based on git history, filesystem structure, and CI/CD configuration
- CLI interface with comprehensive commands (check, watch, init, config, hooks)
- Git hooks integration for pre-commit validation
- Configuration via `.assura/config.yml`
- Self-validation capability - Assura validates itself
- Comprehensive test suite with 150+ tests
- Performance benchmarks
- OpenCode plugin for IDE integration
- Documentation website with Starlight

### Validation Rules
- **Naming Conventions**: kebab-case, snake_case, camelCase, PascalCase, flatcase, FLATCASE, COBOL-CASE, Train-Case, dot.case, SCREAMING_SNAKE_CASE
- **File Organization**: Directory structure validation, path-specific rules
- **Markdown**: Frontmatter schema validation, heading hierarchy, link checking, template enforcement
- **Dependencies**: Circular dependency detection, import/require analysis
- **Extensions**: Multi-part extension support (.d.ts, .test.js, .min.css)

### Technical Features
- Async runtime with Tokio
- Graph algorithms with petgraph
- YAML/JSON configuration parsing
- Regex-based pattern matching
- File system event watching with notify
- Concurrent validation with Rayon
- Binary serialization with bincode
- Git integration with git2

### Documentation
- Comprehensive README with quickstart guide
- API documentation with rustdoc
- Configuration reference
- Rule documentation
- Integration guides for CI/CD
- Best practices guide

[0.2.0]: https://github.com/rothnic/assura/releases/tag/v0.2.0
[0.1.0]: https://github.com/rothnic/assura/releases/tag/v0.1.0
