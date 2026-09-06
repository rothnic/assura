---
title: Contributor and agent change contract
status: current
owner: maintainers
---

# Contributor and agent change contract

Start from current `master`, use the PR template, and run the smallest relevant
validation tier. Authors own understanding and verification: do not fabricate
tests or evidence. AI assistance may be disclosed in the PR, but private prompt
transcripts are not required.

Small documentation corrections may mark behavior, reproducer, or validation
fields as not applicable with a reason. New syntax, commands, dependencies, or
support promises require an accepted design card. Do not bundle unrelated
refactors with a focused change.

Changes that delete tests, add exclusions, reduce severity, alter performance
thresholds, or change CI scope require independent review by policy. This is
not enforceable until an administrator configures branch protection.
`CODEOWNERS` routes configuration, CI/release/install, core-check surfaces,
and itself to `@rothnic`; it does not make self-approval valid.

The CI scope classifier emits `policy_review=true` and the affected paths when
tests, configured exclusions/severity, current severity handling, performance
gate implementation, performance-report surfaces, or
CI-scope/workflow files change. This is a visible review prompt, not automated
approval or an NLP-based policy decision.

## Admin boundary

On 2026-09-06, `master` had no GitHub branch protection and `rothnic` had
admin permission. Do not enable a sole-owner approval requirement: Nick cannot
approve his own PR. Before an administrator configures protection, choose an
authorized independent reviewer/bot route, then require current CI checks and
that review. This repository record prepares that decision; it does not change
GitHub settings.
