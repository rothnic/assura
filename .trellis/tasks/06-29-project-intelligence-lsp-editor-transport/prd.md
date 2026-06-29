# Project Intelligence LSP Editor Transport

## Goal

Make Project Intelligence usable inside local editor workflows by adding a
bounded editor protocol surface that reuses the existing validation,
context-pack, session, and safe-fix preview contracts.

## Requirements

- Add a local `assura editor session` command for editor integrations.
- Use JSON-line request/response transport for testability and local wrapper
  simplicity.
- Shape requests and responses around LSP concepts: text documents,
  diagnostics, code actions, and object context.
- Reuse existing Project Intelligence query/session behavior where practical.
- Expose diagnostics for a file URI/path without introducing a separate
  validation engine.
- Expose bounded object/file context and safe-fix previews without implicit
  writes.
- Document how editor wrappers should relate this surface to `assura agent`,
  `assura content session`, hooks, and future full LSP packaging.

## Acceptance Criteria

- [ ] `assura editor --help` and `assura editor session --help` expose the
      local editor protocol.
- [ ] `textDocument/diagnostics` requests return LSP-shaped diagnostics from
      shared Project Intelligence facts.
- [ ] `textDocument/context` requests return a bounded context pack for the
      file/object under edit.
- [ ] `textDocument/codeAction` requests return safe-fix preview actions with
      no writes and an explicit apply command.
- [ ] Protocol tests prove representative editor output agrees with CLI or
      context-pack output on Assura/Beacon fixtures.
- [ ] Docs and support surfaces classify editor behavior accurately and do not
      claim editor plugin packaging.

## Definition of Done

- Focused tests cover diagnostics, context, safe-fix code actions, invalid
  requests, and conservative reload metadata.
- Assura self-check, docs build, and evidence gates pass.
- Independent review checks that the editor protocol wraps shared contracts and
  does not fork validation behavior.

## Technical Approach

Implement a local JSON-line editor session command that keeps the same loaded
project facts and conservative fingerprinting model as `assura content
session`. The first protocol slice will be LSP-shaped rather than a full
language-server process: it will accept methods such as
`textDocument/diagnostics`, `textDocument/context`, and
`textDocument/codeAction` and return typed JSON envelopes suitable for editor
wrappers.

## Out of Scope

- Full LSP server framing with `Content-Length` headers.
- Editor marketplace package or plugin install.
- Hosted language server or remote provider.
- Semantic content generation.
- Automatic repair or writes without explicit user approval.

## Technical Notes

- Existing `assura content session` is the closest implementation model.
- Existing `assura agent` and context-pack outputs are the schema source for
  agent/editor handoff semantics.
