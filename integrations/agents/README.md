# Assura Agent Integrations

This directory contains source code for installable Assura integrations that
run inside downstream users' agent environments.

Project-local agent configuration, such as this repository's `.codex/` Trellis
support, remains in the platform-specific configuration directories at the repo
root. Installable integration packages live here so OpenCode, Codex, and future
agent adapters are developed under one source tree.

## Packages

- `opencode/`: OpenCode plugin package for Assura validation feedback.
- `codex/`: Codex integration package with advisory nudge and optional native
  `UserPromptSubmit` hook feedback commands.
