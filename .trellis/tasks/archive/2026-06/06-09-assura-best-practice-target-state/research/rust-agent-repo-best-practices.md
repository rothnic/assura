# Rust and agent-driven repository best practices

## Sources

- Cargo package layout:
  <https://doc.rust-lang.org/cargo/guide/project-layout.html>
- Cargo workspaces:
  <https://doc.rust-lang.org/cargo/reference/workspaces.html>
- Cargo manifest format:
  <https://doc.rust-lang.org/cargo/reference/manifest.html>
- Cargo test behavior:
  <https://doc.rust-lang.org/cargo/commands/cargo-test.html>
- Rustdoc documentation tests:
  <https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html>
- Rust API Guidelines:
  <https://rust-lang.github.io/api-guidelines/>
- OpenAI Codex AGENTS.md guidance:
  <https://developers.openai.com/codex/guides/agents-md>
- AGENTS.md open format:
  <https://agents.md/>

## Findings

- Cargo defines a conventional project layout: `Cargo.toml` and `Cargo.lock` in
  the package root, source in `src/`, library entry at `src/lib.rs`, binary
  entry at `src/main.rs`, additional binaries under `src/bin/`, benchmarks under
  `benches/`, examples under `examples/`, and integration tests under `tests/`.
- Workspaces are the standard way to manage multiple packages together. They
  share a lockfile, target directory, package metadata, dependency declarations,
  lints, and root profile configuration. A workspace can use package selection
  and default members to make local and CI checks explicit.
- Cargo manifests are not just build inputs. They encode package metadata,
  versioning, MSRV, license, repository, homepage, documentation, target
  discovery, features, lints, profiles, and external tool metadata.
- `cargo test` covers unit, integration, and documentation tests by default for
  selected packages. It supports workspace/package/target selection, which is
  important for layered local checks versus full pre-merge checks.
- Rustdoc doctests keep public examples current. For a CLI/library hybrid, the
  target state is not "every public item has a long example"; it is that public
  APIs and user-visible examples compile, and unsupported/internal surfaces are
  not accidentally presented as stable documentation.
- The Rust API Guidelines are intentionally guidelines, not a mandate. They are
  still the right review checklist for public Rust APIs because they represent
  idiomatic, interoperable Rust library design concerns.
- Agent instructions should be discoverable and operational. The root
  `AGENTS.md` should contain the routing and rules that apply broadly, while
  deeper skills/specs/scripts provide progressive disclosure for detailed
  workflows and edge cases.
- Codex discovers layered project instructions and has an instruction-size
  budget. This supports keeping `AGENTS.md` concise and making scripts such as
  `workflow_gate.py` return the state and next action directly instead of
  forcing the agent to read long workflow docs on every request.

## Target-State Implications for Assura

- A clean Assura repo should treat `Cargo.toml`, workspace members, feature
  gates, and public exports as a contract that must match support docs and
  release claims.
- The root should remain clean and whitelisted, but structure policy must not be
  the only quality layer. Rust-specific gates, release gates, security gates,
  and agent-workflow gates should be scoped by changed files.
- Experimental/internal modules may exist before 1.0, but they need visible
  ownership, support classification, tests that prove current behavior, and
  deterministic stale-surface detection.
- Tests should be mapped to product surfaces and failure modes, not just files.
  Ignored/manual/performance tests should have a documented purpose and should
  not be the only coverage for a supported behavior.
- Agent workflow quality should be checked as repo state: clean workspace,
  active task state, branch ownership, review gate requirements, and quality
  commands should be deterministic outputs, not chat memory.
