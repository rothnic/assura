# Position Document Examples As Research Authoring

## Goal

Make the document-project examples read as academic research and content
authoring workflows rather than government-contract or overfit domain work.

## Requirements

- Use a generic name such as `research-authoring project` for example wording.
- Keep the actual CLI template name `document-project`.
- Emphasize literature reviews, paper/thesis/report drafting, source custody,
  evidence, notes, drafts, and final documents.
- Avoid department-of-defense, government contract, scoring-package, portal, or
  procurement framing.

## Acceptance Criteria

- Website onboarding guide clearly shows the document-project flow as a
  research/content authoring fit.
- Generated onboarding questions use the same generic research/content
  authoring framing.
- Goal/support docs no longer lean on vague document-heavy or compliance
  examples where a research-authoring example would be clearer.
- Focused docs/target-state checks pass.

## Validation

- `cargo fmt --check`
- `cargo test --test project_intelligence_onboarding --quiet`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo run --quiet -- check --format json .`
- `git diff --check`
