# Content Model Source Of Truth

## Objective

Execute `docs/goals/assura-content-model-source-of-truth.md` as the first
successor goal in the Project Intelligence Runtime program.

## Scope

- Keep `markdown.require_frontmatter` as a generic Markdown/document-style
  presence rule.
- Remove `markdown.required_fields` from the supported Assura-authored config
  path by rejecting it with a migration diagnostic that points users to
  content runtime models and collections.
- Stop structure-first Markdown validation from checking typed frontmatter
  fields.
- Prove required typed frontmatter fields through `models`, `collections`, and
  content runtime schema validation.
- Update public docs, website docs, examples, and tests that currently teach
  `markdown.required_fields`.

## Out Of Scope

- Generic Markdown lint/fix integration.
- Graph/search implementation.
- New schema language beyond the existing content runtime model artifact.
- Compatibility shim that continues validating typed fields in both Markdown
  rules and content runtime models.

## Acceptance Criteria

- `markdown.required_fields` in `.assura/config.yml` fails config validation
  with an actionable message naming `models` and `collections`.
- `markdown.require_frontmatter` still reports missing YAML frontmatter.
- Existing Markdown heading hierarchy, required sections, and outline checks
  still run.
- Content runtime validation reports missing required fields from Markdown
  frontmatter records through modeled collections.
- Public docs explain the split between Markdown formatting/presence,
  Assura-owned heading hierarchy, and typed content models.

## Validation

- `cargo fmt --check`
- `cargo test markdown --quiet`
- `cargo test --test content_runtime_validation --quiet`
- `cargo test --test cli_check_tests --quiet`
- `cargo test --test policy_language_completeness_tests --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask docs`
- `git diff --check`
