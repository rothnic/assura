# Goal 09 first-time configuration authoring

## Goal

Execute `docs/goals/assura-goal-09-first-time-configuration-authoring.md` as
the next Iteration 02 slice.

## Requirements

- Revalidate the goal against current repo state before implementation.
- Prove a new user can author a useful `.assura/config.yml` from public docs
  without reading Rust source or historical planning notes.
- Build a notation use-case matrix that starts with LS-Lint-equivalent naming,
  extension, closed-world, ignore, and direct-child presence cases.
- Extend the matrix into Assura-native `exists`, captures, relationships,
  Markdown outlines, and reusable `rules:`.
- Update public examples, website examples, generated examples, fixtures, and
  test-case `.assura/config.yml` files affected by first-path notation.
- Keep removed alpha notation rejected; do not add compatibility shims unless a
  support-policy exception and removal plan exists.
- Keep detailed `files:` and `directories:` reference material available while
  making the first path tree-shaped `structure:` notation.
- Record a durable first-run review artifact with confusing steps, fixes, and
  any explicitly deferred product gaps.
- Use independent review before PR publication and address PR comments before
  merge.
- End each PR with a clean worktree.

## Acceptance Criteria

- [ ] Goal 09 has a current-state revalidation record.
- [ ] First-run docs and examples guide a new user through minimal useful
      config for a small Rust CLI/library project and a package-style project.
- [ ] The notation matrix covers LS-Lint-equivalent and Assura-native cases.
- [ ] Affected public/generated examples, fixtures, and test-case configs are
      migrated consistently.
- [ ] Review evidence records first-time-user findings and fixes.
- [ ] Required validation from the goal passes.
- [ ] PR review comments are addressed before merge.
- [ ] Local worktree is clean after commit/PR integration.

## Out Of Scope

- Implementing unrelated watch-mode, marketplace, or dependency graph features.
- Redesigning the already-merged canonical relationship notation.
- Adding broad runtime semantics unless a first-run example exposes a blocking
  bug that cannot be fixed in docs or diagnostics.
