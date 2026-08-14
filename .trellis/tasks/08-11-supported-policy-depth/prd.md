# Promote Deterministic Policy Depth

## Goal

Align core product pages with supported deterministic structure, content,
reference, severity, suppression, and agent-guidance behavior while moving
future intelligence to roadmap surfaces.

## Requirements

- Inventory every exact claim on the eight core product pages.
- Add behavioral evidence for implemented deterministic checks required by
  those claims; implement missing bounded behavior only when it is part of the
  agreed core story.
- Keep check as the gate, review as advisory radar, explain as scoped policy
  evidence, and doctor as setup/runtime diagnosis.
- Ensure unchecked capabilities are reported inactive, never passing.
- Remove experimental/planned language from core pages by supporting the claim
  or removing it from the core story.
- Route semantic search, symbols, deeper dependency intelligence, plugin SDK,
  MCP, full LSP, marketplace, and automatic repair to the roadmap.
- Generate landing and documentation examples from executable fixtures and the
  supported CLI renderers. Visual styling may adapt the same semantics, but
  labels, hierarchy, states, values, and interactions must not invent a second
  product experience.
- Keep review visually advisory and check visibly blocking, with the same
  distinction in the real CLI output.

## Acceptance Criteria

- [ ] Home, Project Review, Agent Onboarding, Agent Guardrails, Repository
  Validation, Project Intelligence, Performance, and LS-Lint Comparison have
  zero unsupported capability claims.
- [ ] Markdown/local-link, reference, severity/suppression, agent guidance, and
  computed deterministic claims each have behavioral proof where promoted.
- [ ] Core pages do not label a marketed capability experimental or planned.
- [ ] Future capabilities are discoverable on roadmap pages without reading as current.
- [ ] Website examples remain generated from executable fixtures.
- [ ] Hero and focused-page output lines are selected from the exact supported
  renderer output; fixture-backed policy-tree values match the files checked.
- [ ] Stale generated examples fail the build, and light/dark screenshots pass
  wrap and horizontal-overflow checks at 360, 390, 430, 768, 1024, and 1440 px.

## Validation

```bash
cargo xtask website-demo-data --check
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
pnpm --dir website test:marketing
cargo run --quiet -- check --format json .
```

## Review Blocking Criteria

Block on claim-by-copy without behavioral evidence, inactive-as-passing output,
duplicate validation paths, roadmap concepts presented as current, or examples
that cannot be executed by the CLI.
