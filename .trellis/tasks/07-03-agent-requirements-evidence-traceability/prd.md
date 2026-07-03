# Agent Requirements Evidence Traceability

## Objective

Implement `docs/goals/assura-agent-requirements-evidence-traceability.md`
as the next child of the agent-ready onboarding program.

## Revalidation

Status: valid.

Current live state already has broad content runtime collections for
requirements, evidence, findings, docs, decisions, and document-project source
documents. It also has graph queries, doctor content-runtime gaps, and
repository-reference validation. The missing capability is a reusable
traceability policy that converts those modeled facts into deterministic
checks and agent-facing gaps:

- high-priority requirements without coverage;
- claims without evidence;
- evidence without modeled source-document links;
- findings without configured owner/status metadata;
- doctor and agent-query visibility for the same gaps.

## Scope

- Add an experimental first-party `extensions.requirements_traceability`
  policy family.
- Add traceability validation over the existing content-runtime snapshot.
- Extend generated document-project onboarding config and files with generic
  claims, evidence-to-source links, and traceability policy.
- Surface traceability diagnostics through `assura check`, doctor content gaps,
  and `assura content agent-query diagnostics/gaps/next-actions`.
- Update website/reference/support-policy docs for the experimental surface.

## Non-Goals

- No weighted proposal scoring.
- No proposal, government, or SBIR-specific fields or checks.
- No natural-language fact extraction from arbitrary prose.

## Proof Gates

- `cargo fmt --check`
- `cargo test --test requirements_traceability --quiet`
- `cargo test --test content_query_cli --quiet`
- `cargo test --test content_runtime_references --quiet`
- `cargo test --test project_intelligence_onboarding --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

## Review Criteria

Block if the checks only work for one domain, if unconfigured models produce a
green completeness signal, if agent-query cannot expose the gaps, if source
links bypass repository-reference/source-document validation, or if generated
document-project content contains proposal/SBIR-specific behavior.
