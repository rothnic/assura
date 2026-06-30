# Rust Markdown Validation And Fixing

## Objective

Execute `docs/goals/assura-rust-markdown-validation-and-fixing.md` as the
second successor goal in the Project Intelligence Runtime program.

## Scope

- Refresh current upstream evidence for `rumdl`, `mkdlint`, and `comrak`.
- Compare candidate rule coverage, spans, autofix behavior, frontmatter/GFM
  handling, embeddability, binary impact, and performance.
- Record a decision artifact before adding broad Markdown lint or fix behavior.
- Integrate a Rust-native Markdown lint path into `assura check` only after
  the candidate decision is evidence-backed.
- Add a deterministic safe fix path for at least one generic Markdown issue.
- Preserve Assura-owned path scoping, modeled frontmatter validation, and
  nested heading hierarchy validation.

## Out Of Scope

- JavaScript markdownlint runtime dependency.
- Arbitrary repository-defined command execution.
- Semantic graph/search runtime.
- Duplicate typed frontmatter field validation in generic Markdown rules.
- Broad hand-rolled Markdown lint rules before maintained Rust tooling is
  evaluated.

## Acceptance Criteria

- A decision record compares `rumdl`, `mkdlint`, and `comrak` with current
  upstream and local evidence.
- `assura check` reports at least one generic Markdown lint diagnostic through
  the selected Rust-native path.
- A safe fix command or equivalent operation applies at least one deterministic
  Markdown formatting fix.
- Tests cover lint-only diagnostics, successful fix, no-op fix, frontmatter
  preservation, and modeled-frontmatter interaction.
- Benchmarks or timing evidence record overhead on representative docs.
- Docs explain Assura-owned, delegated, and unsupported Markdown rules.

## Validation

- `cargo fmt --check`
- `cargo test markdown --quiet`
- `cargo test --test content_runtime_validation --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask docs`
- `git diff --check`
- Candidate-specific benchmark or timing commands recorded in the decision
  artifact.
