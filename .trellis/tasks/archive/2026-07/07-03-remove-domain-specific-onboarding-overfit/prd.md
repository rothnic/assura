# Remove Domain-Specific Onboarding Overfit

## Goal

Remove domain-specific onboarding language and product surfaces so agent-ready
document-heavy project support stays generic and reusable.

## Requirements

- Remove every repository file/content reference to the overfit domain term,
  including lower-case, upper-case, slugs, module names, test names, docs,
  roadmap evidence, and task artifacts in the working tree.
- Keep generic `agent-project` and `document-project` onboarding behavior.
- Do not keep a special domain template that implies one specific
  document-heavy customer/project workflow.
- Update tests, docs, support policy, compatibility notes, target-state checks,
  and command-surface metadata to match the generic surface.

## Acceptance Criteria

- [ ] A case-insensitive forbidden-term content search returns no matches.
- [ ] A case-insensitive forbidden-term path search returns no matches.
- [ ] `cargo test --test project_intelligence_onboarding --quiet` passes.
- [ ] `cargo test --test computed_checks --quiet` passes if computed-check
  references are touched.
- [ ] `cargo run --quiet -- check --format json .` passes.
- [ ] `cargo xtask target-state` passes.
- [ ] `git diff --check` passes.

## Definition of Done

- Focused tests and repository self-checks pass.
- No forbidden domain-term references remain in tracked or task files.
- The workspace is clean after commit or explicitly reported dirty if the user
  asks not to commit.

## Out of Scope

- Reintroducing another one-off project-type package under a different label.
- Rewriting git history.
- Removing ordinary planning docs unless they are part of the overfit
  onboarding surface.

## Technical Notes

- Initial search found an explicit overfit content template, source modules,
  tests, website docs, command-surface metadata, support/compatibility rows,
  goal docs, and archived task artifacts.
- This is a cleanup of unpushed branch work; prefer removal or genericization
  over compatibility shims.
