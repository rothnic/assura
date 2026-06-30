# Project Intelligence Release Hardening

## Goal

Prepare the completed Project Intelligence usability slice for release by making
support status, schema examples, release-readiness docs, install/adoption
evidence, and final program audit evidence consistent and checkable.

## Current Evidence

- Adoption blueprint, real-repo proof, onboarding template, context pack,
  persistent session, safe-fix workflow, `.assura/` directory organization,
  agent CLI surface, and editor session are completed locally.
- `website/src/content/docs/reference/release-readiness.md` still reflects a
  narrower pre-project-intelligence release surface.
- Support policy, compatibility matrix, release notes, and website references
  now mention many project-intelligence surfaces, but the final release
  hardening pass has not proved they agree with live output and the program DoD.

## Requirements

- Align release-readiness docs with support policy, compatibility matrix, and
  release notes for project-intelligence commands, schemas, transports, and
  unsupported surfaces.
- Add checked schema/golden coverage for stable project-intelligence JSON
  examples where current tests only assert selected fields.
- Record install/adoption smoke evidence for the supported local workflow,
  clearly labeling local source-checkout evidence if release-candidate archives
  are not available.
- Produce a final program audit mapping every
  `docs/goals/assura-project-intelligence-usability-program.md` definition of
  done item to current evidence.
- Keep MCP, hosted services, full LSP server framing, and editor marketplace
  packages out of the supported release surface.

## Acceptance Criteria

- [ ] Release-readiness website page names the supported project-intelligence
      CLI surfaces and unsupported/roadmap transport boundaries accurately.
- [ ] Support policy, compatibility matrix, release notes, and release-readiness
      docs classify the same surfaces consistently.
- [ ] Stable schemas used in docs have checked examples or snapshot-style
      assertions against live command output.
- [ ] Adoption/install smoke evidence is recorded in repo-native docs or
      analysis artifacts with exact commands.
- [ ] Final program audit exists and proves each program DoD item or leaves the
      program open with named gaps.
- [ ] Independent review checks docs/status consistency and schema evidence.

## Non-Goals

- No 1.0 compatibility guarantee.
- No hosted service, remote MCP requirement, or marketplace packaging.
- No new project-intelligence behavior unless a release proof exposes a
  release-blocking inconsistency.
