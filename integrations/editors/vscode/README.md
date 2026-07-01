# Assura VS Code Extension

This package is the first experimental VS Code adapter for Assura. It is local
only and shells out to the shared Assura CLI contracts:

```bash
assura daemon status --format json .
assura daemon doctor --format json .
assura daemon check-path . --changed docs/guide.md --format json
assura editor session .
assura check --format json .
assura fix markdown --dry-run --format json .
```

The extension shows daemon health in the status bar, reports Assura check
findings through VS Code diagnostics, refreshes changed documents with daemon
changed-path and editor-session diagnostics, and exposes command-palette actions
for daemon lifecycle and safe-fix preview commands. It does not start a remote
service, publish marketplace packaging, or apply fixes automatically.

## Development

```bash
pnpm test
pnpm run build
```

The current tests use Node's built-in test runner and do not require installing
VS Code extension host dependencies.
