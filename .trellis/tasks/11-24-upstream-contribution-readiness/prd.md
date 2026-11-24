# Upstream contribution readiness pilot

Beads `ae-whr.14` owns execution. Integration base is Assura `7ab92cb61b78d714d264227535e1191fe612aad6` on `master` at intake. Pilot repository is `pypa/pipx` targeting `main`, with the generated Codex integration. Use the published work for [pipx #2501](https://github.com/pypa/pipx/issues/2501) and [#2502](https://github.com/pypa/pipx/pull/2502) as an unpublished replay.

## Acceptance

- Capture documented contribution, issue and draft rules and observed, source-linked review sizes and local code/test patterns. Check a real duplicate issue/PR before applying those criteria to a corrected draft.
- Replay the baseline and proposed workflow on the same commits using installed Assura Git and Codex hooks. Count interrupted commits, Codex context messages and UTF-8 bytes. Keep the existing advisory development default and explicit staged enforcement. Leave automatic PR-size feedback opt-in unless the replay proves a benefit.
- Inspect the committed patch from the unique merge base of the intended upstream target and exact head. Distinguish overall additions/deletions and affected files from resulting file length. Exercise a shrinking rewrite, edit/revert across commits, real growth, push-tracking mismatch, unavailable and ambiguous base, and unknown coverage.
- Review the candidate and refresh both identities before ready. A changed base or head invalidates old review/validation evidence. Record actual checks for the bound candidate, and preserve any evidence gaps.
- Keep contribution planning and review in a short guide over existing Assura and Git surfaces. Add no engine rule unless a measured gap requires `ae-whr.10`. Keep external issues and PRs read-only. Review the delivery independently.
