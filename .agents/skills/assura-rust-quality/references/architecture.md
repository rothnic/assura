# Architecture references

Read this for configuration, model, checker, report, CLI, or harness changes.

## Current configured-check path

`src/cli/check.rs` is the one-shot path: `run_structure_check_with_target_mode`
discovers the project, calls `ConfigLoader::load`, compiles with
`CompiledStructureConfig::new_for_check`, constructs `StructureChecker`, and
returns `StructureCheckReport`. `src/cli/commands.rs` adapts that result in
`check_command`; keep its exit and format behavior aligned with the report.

For a long-lived caller, `src/cli/check/prepared.rs` owns
`PreparedStructureCheck`: it parses and compiles once, then chooses a whole
project or explicitly safe changed-path check. `crates/assura-check-cli/src/main.rs`
is the lightweight harness entrypoint; its server reloads a prepared check when
the configuration changes instead of treating cached state as authoritative.

## Decision checks

- Current structure notation has one runtime loader: `ConfigLoader` in
  `src/config/loader.rs`. Do not add a second parser for an inspection command.
- Keep compiled configuration (`CompiledStructureConfig`) inside the check
  boundary; callers consume `StructureCheckReport`, not checker internals.
- Add a canonical path rather than a re-export ladder. The external guidance on
  this choice is [Microsoft M-SINGLE-ITEM-PATH](https://microsoft.github.io/rust-guidelines/guidelines/ai/index.html)
  and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

External sources accessed 2026-09-05; they are guidance, not a replacement for
the supported Assura contract.
