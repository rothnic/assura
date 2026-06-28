---
id: analysis-2026-06-28-content-runtime-release-readiness-audit
type: analysis
title: Content runtime release readiness audit
status: active
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/content-runtime.md
  - docs/content-runtime-inspection.md
  - docs/analysis/2026-06-28-content-runtime-authoring-decision.md
  - docs/analysis/2026-06-28-content-runtime-index-performance.md
---

# Content Runtime Release Readiness Audit

## Verdict

Repo-native content runtime is release-ready for the scope defined in
`docs/goals/assura-repo-native-content-runtime-implementation.md`, subject to
the final increment 11 PR passing local, hosted Linux/macOS/Windows, coverage,
performance, docs, evidence, and installable smoke gates.

The feature remains pre-1.0 and experimental as a public Rust module, but the
goal's requested runtime path is implemented: ordinary repo files define typed
objects, `assura check` validates them natively, and Assura-owned create/update
operations provide bounded agent writes.

## Increment Evidence

| Increment | Evidence | Status |
| --- | --- | --- |
| 0. Branch and research hygiene | PR #95 merged research evidence; PR #96 started implementation from the reviewed base. | Complete |
| 1. Runtime model slice | `src/content_repository/`, `tests/content_runtime_validation.rs`, and `tests/fixtures/content_runtime/{valid,invalid_shape,missing_reference}/`. | Complete |
| 2. CLI reporting integration | `tests/content_runtime_check_cli.rs` covers JSON, YAML, text, and agent diagnostics. | Complete |
| 3. First typed write operation | `ContentRepository::create_record` and `tests/content_runtime_create.rs`. | Complete |
| 4. Update operation and write safety | `ContentRepository::update_record` and `tests/content_runtime_update.rs`. | Complete |
| 5. Additional storage adapters | `yaml_record` and `jsonl_record` support plus `tests/content_runtime_adapters.rs`. | Complete |
| 6. Reference graph completeness | `tests/content_runtime_references.rs` and `tests/fixtures/content_runtime/references/`. | Complete |
| 7. Authoring toolchain decision | `docs/analysis/2026-06-28-content-runtime-authoring-decision.md` and checked generated artifacts under `tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/`. | Complete |
| 8. DX and cross-language inspection | `docs/content-runtime-inspection.md` and `tests/content_runtime_dx_docs.rs`. | Complete |
| 9. Index and performance hardening | `benches/content_runtime.rs`, refreshed performance history, and `docs/analysis/2026-06-28-content-runtime-index-performance.md`. | Complete |
| 10. Documentation and examples | `docs/content-runtime.md`, `website/src/content/docs/examples/content-runtime.md`, website sidebar link, and docs regression coverage. | Complete |
| 11. Release readiness | This audit plus final review and PR-boundary validation. | In progress until final PR merge |

Merged implementation PRs are #96 through #105. The final increment PR should
link this audit and map each increment in its summary.

## Completion Definition Audit

| Requirement | Evidence | Result |
| --- | --- | --- |
| Project config can say which object types may live under which paths. | Fixture `.assura/config.yml` files declare `collections` with `class`, `path`, and adapter fields; `ContentRepository::from_config` builds the runtime model. | Satisfied |
| Each collection declares storage adapter, schema artifact, ID field, and reference fields. | `models.validation_artifact`, `collections.*.adapter`, `collections.*.id`, and `relations` are exercised by content-runtime fixtures. | Satisfied |
| Shape validation uses compiled or cached Rust validators in the runtime path. | `ContentRepository` caches `jsonschema::Validator` instances; `artifact_authoring_paths_proof.rs` proves no runtime authoring-tool dependency. | Satisfied |
| Reference validation resolves records across collections and reports source file plus field. | `tests/content_runtime_references.rs` and `tests/content_runtime_check_cli.rs` assert missing, ambiguous, duplicate, required, optional, many, and cycle diagnostics. | Satisfied |
| Typed mutations validate payload and affected references before writing. | `create_record` and `update_record` validate shape, placement, ID, and references before I/O; create/update tests assert failures leave trees unchanged. | Satisfied |
| Markdown frontmatter writes preserve body bytes. | `tests/content_runtime_update.rs` compares pre/post Markdown body bytes and includes CRLF delimiter coverage. | Satisfied |
| JSON, YAML, and JSONL writes are deterministic and atomic. | `tests/content_runtime_create.rs`, `tests/content_runtime_update.rs`, `tests/content_runtime_adapters.rs`, and `src/content_repository/io.rs` cover deterministic serialization and temp-file replacement/no-clobber writes. | Satisfied |
| Authoring formats are optional build-time inputs, not runtime dependencies. | Authoring decision and compile manifest record empty runtime hot-path dependencies; runtime tests use checked artifacts. | Satisfied |
| Docs and examples demonstrate the same model in Markdown frontmatter and JSON form. | `docs/content-runtime-inspection.md`, `docs/content-runtime.md`, and website content-runtime example. | Satisfied |
| DX evidence covers ordinary editor/language tooling and JSON Schema-aware tooling. | `docs/content-runtime-inspection.md` covers TypeScript, Python, Rust, `$defs`, and `x-assura` metadata. | Satisfied |
| Performance evidence remains appropriate for normal `assura check` use. | `benches/content_runtime.rs`, `docs/analysis/2026-06-28-content-runtime-index-performance.md`, and refreshed `benches/history/current.json`. | Satisfied |

## Program DoD Audit

| Program item | Evidence | Result |
| --- | --- | --- |
| New implementation branch from reviewed base. | PR #95 merged; PR #96 started implementation from `origin/master`. | Satisfied |
| Each increment has PR, review notes, and validation evidence. | Goal progress log records PRs #96-#105; final increment uses this audit and review before PR. | Satisfied pending final PR |
| Production modules load runtime schema artifacts and cache validators. | `src/content_repository/model.rs` and `src/content_repository/mod.rs`. | Satisfied |
| Config binds classes, paths, adapters, ID fields, and references. | Content runtime fixture configs and docs examples. | Satisfied |
| Markdown, JSON, YAML, JSONL records validate as logical objects. | Validation, adapter, reference, and authoring-path tests. | Satisfied |
| Missing references produce diagnostics with source paths and fields. | `tests/content_runtime_validation.rs`, `tests/content_runtime_references.rs`, and CLI reporting tests. | Satisfied |
| Typed create/update operations validate payloads and references before writing. | `tests/content_runtime_create.rs` and `tests/content_runtime_update.rs`. | Satisfied |
| Safe-write tests prove failed validation leaves the tree unchanged. | Create, update, and adapter tests snapshot the tree before failure. | Satisfied |
| Docs include selected authoring profile, runtime artifacts, adapters, relations, and writes. | `docs/content-runtime.md`, `docs/content-runtime-inspection.md`, and authoring decision. | Satisfied |
| DX evidence covers TypeScript, Python, and Rust inspection. | `docs/content-runtime-inspection.md` and regression test. | Satisfied |
| Performance evidence captured before release readiness. | Increment 9 benchmark and tracked performance report. | Satisfied |
| Review evidence captured before PR. | Each increment progress log records an independent review agent; final review agent `Feynman` reviewed this audit before PR publication. | Satisfied |
| Final PR summary maps work back to every increment. | Required for the increment 11 PR body. | Pending final PR |

## Cross-Platform And CI Evidence

PR #105 passed hosted documentation, evidence, Rustfmt, Clippy, code coverage,
performance report, release bundle smoke, Linux/macOS/Windows tests, Windows
installer smoke, installable adoption smokes, GitGuardian, and scope checks
before merge.

The final increment PR must repeat the hosted Linux/macOS/Windows, coverage,
performance, docs, evidence, and installable smoke gates after this audit lands.

## Final Independent Review

Review agent `Feynman` reviewed this audit against the full long-running goal.
The review found one documentation bug: the adoption guide originally named the
cycle diagnostic `content_runtime:reference_cycle`, while the runtime and
reference tests use `content_runtime:cyclic_reference`. That was fixed in
`docs/content-runtime.md` and pinned in `tests/content_runtime_dx_docs.rs`.

After that fix, the reviewer found no unsupported requirement or audit overclaim.

## Diagnostics And Adoption

Current diagnostics cover:

- object type, source path, field, and referenced object for missing
  references;
- invalid object shape including the field where available;
- duplicate IDs without nondeterministic overwrites;
- ambiguous multi-target references;
- configured acyclic relation cycles;
- invalid collection paths and unknown collections for write operations;
- write and read errors with affected relative paths.

Adoption guidance now lives in:

- `docs/content-runtime.md` for config, adapters, operation payloads, fixture
  matrix, and existing-repo adoption;
- `docs/content-runtime-inspection.md` for TypeScript, Python, and Rust
  inspection;
- `website/src/content/docs/examples/content-runtime.md` for public website
  discovery.

## Final Review Bar

The final reviewer should block release readiness if:

- any completion-definition row above is unsupported by current files or tests;
- normal validation depends on LinkML, TypeSpec, Node, Python, Go, CUE, Deeb,
  SQLite, or a server;
- a write failure can partially mutate files after validation failure;
- Markdown body bytes can change during a frontmatter-only update;
- diagnostics lose source path, object type, field, or referenced object where
  that context is available;
- final hosted CI does not pass across Linux, macOS, and Windows.
