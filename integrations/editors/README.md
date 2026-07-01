# Assura Editor Integrations

Editor adapters live here when they wrap Assura's shared local CLI and JSON
contracts. They must not implement separate scanners or apply fixes implicitly.

## Packages

- `vscode/`: Experimental VS Code extension package over `assura daemon`,
  `assura check --format json`, and safe-fix preview contracts. Future slices
  may add a persistent `assura editor session` transport.
