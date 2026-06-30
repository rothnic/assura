# Documentation IA Project Intelligence

## Objective

Execute `docs/goals/assura-documentation-ia-project-intelligence.md` as the
third successor goal in the Project Intelligence Runtime program.

## Scope

- Rework website navigation so Assura is presented as layered validation and
  project intelligence, not only scattered examples.
- Promote content runtime documentation into a first-class product area.
- Add or reorganize pages for structure validation, Markdown validation,
  content models, collection relations, agent operations, query/search, and
  optional code intelligence.
- Add implementation-status copy that distinguishes shipped, experimental, and
  planned capabilities.
- Keep graph/search, semantic search, code intelligence, and editor/agent
  surfaces clearly marked as planned unless a current implementation proves
  otherwise.
- Update docs/navigation tests or self-check surfaces that pin command examples,
  links, support status, or docs navigation.

## Out Of Scope

- Runtime feature implementation.
- Marketing-only landing pages.
- Hosted deployment changes unless needed for local docs verification.
- Overclaiming future graph/search/code intelligence as shipped.

## Acceptance Criteria

- Website sidebar has a coherent section for structure validation, Markdown,
  content models, relations/agent operations, query/search, optional code
  intelligence, and agent/editor surfaces.
- Content runtime docs are reachable as a first-class product area.
- Docs clearly distinguish shipped, experimental, and planned capabilities.
- Markdown validation docs explain the split between generic Rust lint/fix,
  Assura-owned heading hierarchy, and content runtime frontmatter models.
- Query/search and code-intelligence docs state implementation status.
- Link/docs checks cover the new navigation and command/status claims.

## Validation

- `cargo run --quiet -- check --format json .`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

Run any focused website or docs tests found during implementation if navigation
or docs assertion surfaces change.
