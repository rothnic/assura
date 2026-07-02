---
id: goal-assura-agent-requirements-evidence-traceability
type: goal
title: Assura agent requirements evidence traceability
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-content-activation-source-docs.md
  - ./assura-supported-document-graph.md
---

# Assura Agent Requirements Evidence Traceability

## Objective

Make requirements, claims, evidence, findings, and document coverage queryable
and enforceable for serious documentation workflows without requiring a
domain-specific proposal pack.

## Scope

- Add reusable content models and relation checks for Requirement, Claim,
  Evidence, Finding, Decision, SourceDocument, and Doc records.
- Validate high-priority requirements have coverage.
- Validate claims link to evidence and evidence links to a source.
- Validate findings carry owner/status metadata when configured.
- Expose coverage gaps through doctor, agent-query, reference graph, and
  next-actions output.
- Provide fixtures for Markdown frontmatter, YAML/JSON records, and mixed
  document repositories.

## Non-Goals

- No weighted proposal score computation.
- No domain-specific government or SBIR portal rules.
- No natural-language fact extraction from arbitrary prose.

## Definition Of Done

- Agents can ask which requirements lack coverage and receive deterministic
  query results.
- Claims without evidence and evidence without sources produce actionable
  findings.
- Traceability gaps appear in checked-versus-unchecked doctor output when the
  model is inactive or only partially configured.
- Website docs describe traceability as a reusable document-project capability.

## Validation Commands

```bash
cargo fmt --check
cargo test content_runtime --quiet
cargo test --test content_runtime_references --quiet
cargo test --test project_intelligence_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if traceability only works for one domain, if gaps are hidden behind a
green check, if relation queries require knowing exact source or target IDs
first, or if evidence/source references bypass repository reference validation.
