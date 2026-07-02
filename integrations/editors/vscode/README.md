# Assura VS Code Extension

This package is the supported beta local VS Code adapter for Assura. It shells
out to shared Assura contracts and does not implement editor-specific
validation logic:

```bash
assura check --format json .
assura daemon status --format json .
assura daemon doctor --format json .
assura daemon check-path . --changed docs/guide.md --format json
assura editor session .
assura fix markdown --dry-run --format json .
```

The package reports Assura diagnostics through VS Code, shows daemon health in
the status bar, exposes daemon lifecycle and doctor commands, refreshes saved
and active file diagnostics, and opens safe-fix previews as JSON. It never
applies fixes automatically.

## Support Level

This is a beta local package surface, not a marketplace release. Supported
means the package metadata, command construction, daemon visibility, one-shot
fallback, safe-fix preview behavior, and package smoke checks are release-gated.

Deferred until a later goal:

- Marketplace publication.
- Full LSP `Content-Length` server framing.
- Automatic repair.
- Editor-specific validation logic.
- Zed, JetBrains, or other editor packages.

## Install

Use the package from a local Assura checkout while marketplace publication is
deferred. From the extension package directory, run:

```bash
cd integrations/editors/vscode
pnpm test
pnpm run build
pnpm run doctor
pnpm run package
```

Then start a VS Code extension development host from the repository root:

```bash
code --extensionDevelopmentPath integrations/editors/vscode
```

`pnpm run package` writes `dist/assura-vscode-package-manifest.json`. This is a
deterministic package smoke artifact for release evidence; it is not a VSIX and
should not be advertised as a marketplace package.

## Update

Update by pulling or installing a newer Assura checkout, then rerun:

```bash
cd integrations/editors/vscode
pnpm test
pnpm run build
pnpm run doctor
pnpm run package
```

The extension version in `package.json` tracks the beta integration package
version. The installed Assura CLI version remains the runtime source of truth
for validation, daemon, editor-session, and safe-fix behavior.

## Remove

For local development-host usage, close the extension development host window.
No workspace files are modified by the extension itself. Assura daemon state can
be stopped separately:

```bash
assura daemon stop . --format json
```

## Doctor

Run the package doctor before treating the extension as installable:

```bash
pnpm run doctor
```

The doctor verifies package files, command registrations, shared contract
metadata, lifecycle scripts, private package status, and marketplace deferral.

Inside VS Code, use `Assura: Daemon Doctor` when the status bar reports an
unhealthy daemon. The extension surfaces the daemon recovery command instead of
hiding daemon failures.

## Diagnostics And Fallback

Saved and active documents first use:

```bash
assura daemon check-path . --changed <path> --format json
assura editor session .
```

If daemon changed-path diagnostics fail, the extension warns the user and falls
back to:

```bash
assura check --format json .
```

Fallback is visible by design. The extension must not make daemon failures look
healthy just because one-shot diagnostics still work.

## Development

```bash
cd integrations/editors/vscode
pnpm test
pnpm run build
pnpm run doctor
pnpm run package
```

The current tests use Node's built-in test runner and do not require installing
VS Code extension-host dependencies.
