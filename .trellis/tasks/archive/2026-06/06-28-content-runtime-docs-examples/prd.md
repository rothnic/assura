# Content Runtime Documentation And Examples

## Objective

Implement increment 10 from
`docs/goals/assura-repo-native-content-runtime-implementation.md`: publish the
user-facing documentation, website examples, generated examples, and fixture
checks that make repo-native content runtime adoption clear for projects that
need portable structure plus Markdown/frontmatter constraints.

## Scope

- Make the primary user docs explain what project authors write, what Assura
  loads or compiles, and what agents call for create/update workflows.
- Add or update website docs/examples so the feature is discoverable outside
  repository-only analysis files.
- Ensure checked examples cover Markdown frontmatter, JSON, YAML, JSONL,
  references, create/update write paths, and validation commands.
- Include generated-runtime-schema examples that connect the selected authoring
  decision to checked runtime artifacts without making authoring tools runtime
  dependencies.
- Provide high-level implementation guidance that another agent can apply to a
  real example repo independently.
- Keep the work documentation/fixture-oriented unless a small test helper is
  needed to prove the docs reference real files and commands.

## Out Of Scope

- New runtime validation semantics.
- New adapter behavior.
- New public write-operation APIs.
- Release-readiness hardening beyond docs/examples proof.
- Making LinkML, TypeSpec, CUE, Deeb, SQLite, Node, Python, Go, or a server
  required for normal validation.

## Acceptance Criteria

- User-facing docs cover author-authored config, checked runtime artifacts,
  collection adapters, references, and agent create/update calls.
- Website docs include at least one content-runtime example page linked from
  the existing docs navigation/content tree.
- Checked fixture or generated example paths demonstrate Markdown
  frontmatter, JSON, YAML, JSONL, references, and writes.
- Docs link to the authoring decision, DX inspection guide, performance
  decision, and runtime fixtures.
- A focused docs/example regression test proves the documented paths and
  commands stay real.
- `cargo xtask docs`, Assura self-check, and relevant content-runtime tests
  pass.
- Independent review finds no blocker against the goal reviewer criteria.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- Focused docs/examples test added or updated for this increment.
- Relevant content-runtime fixture tests:
  - `cargo test --test content_runtime_validation --quiet`
  - `cargo test --test content_runtime_adapters --quiet`
  - `cargo test --test content_runtime_create --quiet`
  - `cargo test --test content_runtime_update --quiet`
  - `cargo test --test content_runtime_references --quiet`
  - `cargo test --test content_runtime_dx_docs --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask docs`
- `git diff --check`

Full workspace test/clippy gates are reserved for the PR boundary or for code
changes that make focused docs/example validation insufficient.
