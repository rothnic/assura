# Contributing to Assura

Thank you for your interest in contributing to Assura! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

This project and everyone participating in it is governed by our commitment to provide a welcoming and inspiring experience for everyone.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to see if the problem has already been reported. When you are creating a bug report, please include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples to demonstrate the steps**
- **Describe the behavior you observed and what behavior you expected**
- **Include code samples and/or configuration files**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use a clear and descriptive title**
- **Provide a step-by-step description of the suggested enhancement**
- **Provide specific examples to demonstrate the enhancement**
- **Explain why this enhancement would be useful**

### Pull Requests

1. Fork the repository
2. Create a new branch from `main` for your feature or bug fix
3. Make your changes
4. Run the test suite and ensure all tests pass
5. Update documentation as needed
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.86.0 or later
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/rothnic/assura.git
cd assura

# Build the project
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## Style Guidelines

### Rust Code Style

We follow the standard Rust style guidelines:

- Use `rustfmt` for formatting (run `cargo fmt`)
- Follow `clippy` recommendations (run `cargo clippy`)
- All public items must have documentation comments
- Use meaningful variable and function names

### Commit Messages

We use conventional commits. Format your commit messages as follows:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that don't affect code meaning (formatting, etc.)
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Code change that improves performance
- `test`: Adding or correcting tests
- `chore`: Changes to build process or auxiliary tools

Example:
```
feat(rules): add support for custom rule severity

Implements the ability to define custom severity levels
for validation rules beyond the default Critical, High, Medium, Low.

Closes #123
```

### Documentation

- All public APIs must be documented with rustdoc comments
- Include examples in documentation where appropriate
- Update README.md if changing user-facing functionality

## Testing Guidelines

- Write unit tests for all new functionality
- Ensure tests cover edge cases
- Integration tests should be in the `tests/` directory
- Run the full test suite before submitting PR:
  ```bash
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check
  ```

## Project Structure

```
assura/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── core/            # Core engine
│   ├── rules/           # Rule definitions
│   ├── fs/              # File system operations
│   ├── config/          # Configuration parsing
│   └── report/          # Report generation
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── docs/                # Documentation
└── Cargo.toml          # Package manifest
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag: `git tag -a v0.1.0 -m "Release version 0.1.0"`
4. Push the tag: `git push origin v0.1.0`
5. GitHub Actions will automatically create or update the GitHub release and
   upload the prebuilt archives used by the install scripts

## Getting Help

- Check the [documentation](https://github.com/rothnic/assura/tree/master/docs)
- Open an issue for questions or problems
- Join our discussions

## License

By contributing to Assura, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
