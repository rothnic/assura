# Publish Project Intelligence Docs

## Goal

Publish the completed Project Intelligence documentation branch to GitHub,
verify it through local and hosted checks, and merge it to the production
branch so Cloudflare deploys the latest documentation to the Assura site.

## What I already know

- Cloudflare production is building the remote `master` branch.
- The latest Project Intelligence docs are committed locally on
  `codex/project-intelligence-agent-surfaces`.
- The local branch is clean and ahead of `origin/master`.
- The current deployed Workers URL serves the older 30-page docs build from
  `master`; the latest local docs build contains 37 pages.
- The user explicitly asked to push, review, and merge if everything passes.

## Requirements

- Push `codex/project-intelligence-agent-surfaces` to `origin`.
- Run local validation before publishing.
- Get an independent review before creating the PR.
- Open a PR targeting `master`.
- Wait for hosted checks and inspect failures if any.
- Merge only if local validation, independent review, and hosted checks pass.

## Acceptance Criteria

- [ ] Branch is pushed to GitHub.
- [ ] Pull request exists from `codex/project-intelligence-agent-surfaces` to
      `master`.
- [ ] Required local validation is recorded.
- [ ] Independent review records no blocking findings or all blockers are fixed.
- [ ] Hosted checks pass.
- [ ] PR is merged to `master`.

## Definition of Done

- Production branch includes the Project Intelligence documentation commits.
- Cloudflare can deploy the latest docs from the production branch.
- Handoff names the PR, merge commit, validation commands, and any remaining
  domain/deploy follow-up.

## Out of Scope

- Changing Cloudflare dashboard settings.
- Adding or changing Workers/Pages configuration unless the PR checks expose a
  release-blocking need.
- Creating a new release tag.

## Technical Notes

- Repository: `rothnic/assura`.
- Production branch observed via `git ls-remote`: `master`.
- Current branch: `codex/project-intelligence-agent-surfaces`.
