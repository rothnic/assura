# LS-Lint Structure Parity And Agent Integrations

## Goal

Make Assura's structure-first check enforce the project shape we expect, not
only naming and documentation rules for files that happen to exist. Unexpected
well-named files or directories must be detectable and actionable.

## Current Gap

`cargo run --quiet -- check --format json .` is clean on the current branch, but
the checker does not yet reject unexpected direct files or child directories
when they satisfy inherited naming rules. This means `.assura/config.yml` can
describe the desired structure without fully enforcing it.

## Requirements

- Add explicit direct-content policy to the structure-first config:
  - `files.allow_extra`
  - `files.allowed_patterns`
  - `files.forbidden_patterns`
  - `directories` bundle with naming, required, allowed/forbidden names and
    patterns, `allow_extra`, and severity.
- Preserve existing behavior when new fields are omitted.
- Add LS-Lint parity for basic filesystem control:
  - `.dir` rules;
  - wildcard extension rules such as `.*` and `.*.js`;
  - `exists`, `exists:0`, `exists:1`, and `exists:N-M` for direct files and
    directories;
  - `|` rule composition, where naming rules are alternatives and non-naming
    rules are additional constraints.
- Update `.assura/config.yml` to model this repository as a closed structure
  after the enforcement behavior is covered by tests.
- Move installable downstream agent integration source to
  `integrations/agents/`.
- Move the existing OpenCode package to `integrations/agents/opencode/`.
- Add a Codex integration skeleton at `integrations/agents/codex/`, without
  claiming a complete runtime hook implementation.
- Leave project-local `.codex/` Trellis support in place.

## Out Of Scope

- Full Codex runtime integration behavior.
- Windows CI restoration.
- New workflow or task systems outside Trellis.
- Removing project-local Trellis/Codex configuration.

## Acceptance Checks

- Unexpected direct files fail when `files.allow_extra: false`.
- Unexpected direct child directories fail when `directories.allow_extra: false`.
- Allowed names, allowed patterns, configured child directories, and extensions
  pass when expected.
- Forbidden patterns override broad allowed patterns.
- `exists` count constraints pass and fail correctly for files and directories.
- `.dir` naming and existence rules work through LS-Lint compatibility.
- Wildcard extension rules match direct files as expected.
- `cargo fmt --all -- --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo test --all-targets --quiet` passes.
- `cargo run --quiet -- check --format json .` reports zero violations.
- `cargo run -- check .` reports zero violations.
