# NU06: Missing configuration recovery guidance

## Outcome

When `assura check --format json`, `assura doctor --format json`, or
`assura review --format json` finds no project config, stderr names the
applicable project directory and gives a usable `assura init` next step.
The diagnostic remains read-only: exit status stays 4, stdout stays empty, and
no project files or hooks are written.

## Authority and ownership

This implementation is authorized by the NU06 delegation for the
`fix/nu06-missing-config` candidate from `refs/remotes/origin/master`.
The root coordinator owns Beads state, independent review, push, PR, merge, and
native acceptance closure. This task owns the focused implementation, tests,
and documentation only. No new JSON error schema, public Rust API, module, or
dependency is authorized.

## Contract

- The existing shared `CheckError::NoConfig` path is the source of the
  recovery text for fast check, full check, doctor, and review.
- Missing configuration keeps the existing exit status 4 and empty stdout for
  JSON output; the actionable diagnostic is on stderr.
- The path identifies a directory suitable for `assura init`, including when
  the requested check path is a file. Recovery text must not present a
  platform-unsafe shell-quoted path as an executable command.
- Explicit invalid or malformed config, a nonexistent requested path, and
  other discovery failures retain their current error classification and do
  not show missing-config recovery instructions.
- Diagnostic commands must not create config, onboarding, or hook files.
- Validate a project path containing spaces and quote characters.

## Acceptance

1. Focused CLI tests cover lightweight and full check, doctor, and review in
   JSON mode: status 4, empty stdout, stderr path and recovery action, and no
   filesystem mutation.
2. Negative controls cover malformed explicit config and nonexistent target;
   neither is described as missing config. A file target with missing config
   identifies its parent directory for initialization.
3. Docs state that failed JSON diagnostics use stderr while stdout remains
   empty, and show how to initialize the reported project directory.
4. Run formatting and the focused integration test with bounded resource use.
   Preserve command, cwd, source SHA, binary/toolchain identity, exits, and
   stdout/stderr evidence outside the repository when available.
