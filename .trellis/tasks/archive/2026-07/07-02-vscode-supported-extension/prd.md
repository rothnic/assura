# Support VS Code Extension Surface

## Goal

Move `integrations/editors/vscode` from an experimental local adapter to a
supportable beta extension package surface over shared Assura CLI, daemon, and
editor-session contracts.

This is a child slice of
`docs/goals/assura-post-beta-capabilities-program.md`. It should advance the
parent beta increment without claiming GA, marketplace publication, or a
separate editor validation engine.

## What I Already Know

- PR #132 is merged into `master`; the parent program can continue from the
  current clean baseline.
- The current extension package shells out to `assura check`,
  `assura daemon`, `assura daemon check-path`, `assura editor session`, and
  `assura fix markdown --dry-run`.
- Existing tests cover daemon command construction, editor diagnostics,
  one-shot fallback check arguments, safe-fix preview arguments, JSON handling
  for non-zero Assura exits, daemon summary mapping, and path scoping.
- Support docs still classify `integrations/editors/vscode` as experimental.
- Package metadata still says experimental, private, and `0.0.0`.
- The goal requires install/update/remove/doctor guidance, build/test/package
  validation, compatibility policy, daemon failure visibility, and support
  classification.

## Requirements

- Keep all diagnostics and commands sourced from shared Assura CLI, daemon, and
  editor-session outputs.
- Prove the user-specific maintainer story from
  `docs/goals/assura-vscode-supported-extension.md`: a documentation-heavy
  branch can use VS Code diagnostics, visible daemon fallback, safe-fix preview,
  and support/package evidence to decide whether the branch is mergeable.
- Add or tighten VS Code package lifecycle commands for local packaging,
  install/update/remove/doctor documentation, and release smoke validation.
- Preserve explicit preview-only safe fixes; no implicit writes.
- Make daemon health and fallback behavior visible when daemon commands fail or
  report unhealthy state.
- Update support policy, compatibility matrix, release-surface metadata,
  website/docs, roadmap, and goal progress so the extension status is accurate.
- Decide and document whether marketplace publication is deferred for this
  supported beta milestone.
- Add regression tests/build-smoke coverage for any new package commands,
  lifecycle metadata, or doctor/fallback behavior.

## Acceptance Criteria

- [ ] `integrations/editors/vscode/package.json` and README describe a beta
      support-grade local package, not an untracked experiment.
- [ ] A reviewer can run package test/build/package or equivalent smoke
      commands without installing VS Code extension-host dependencies.
- [ ] Docs explain install, update, remove, doctor, daemon recovery, one-shot
      fallback, and marketplace deferral.
- [ ] Support policy and release surfaces classify the VS Code extension
      consistently.
- [ ] Tests prove the extension still uses shared contracts and does not apply
      fixes implicitly.
- [ ] Goal and roadmap progress logs identify this child as active/completed
      with validation evidence.

## Out Of Scope

- Marketplace publication.
- A full LSP server.
- Zed, JetBrains, or other editor packages.
- Private editor-side validation logic.
- Automatic fix application.

## Validation

```bash
pnpm test
pnpm run build
pnpm run doctor
pnpm run package
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
git diff --check
```

If the package does not use `pnpm run package`, replace it with the actual
package smoke command and update this PRD before implementation closes.

## Review Criteria

Independent review must block if:

- VS Code implements private validators instead of shared Assura contracts.
- Daemon failures are swallowed or hidden behind successful one-shot fallback.
- Docs imply marketplace availability without release evidence.
- Safe fixes can write without explicit user action.
- Package lifecycle commands are documented but not executable.
