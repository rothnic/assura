# Quality Guidelines

> Code quality standards for backend development.

---

## Overview

<!--
Document your project's quality standards here.

Questions to answer:
- What patterns are forbidden?
- What linting rules do you enforce?
- What are your testing requirements?
- What code review standards apply?
-->

(To be filled by the team)

---

## Forbidden Patterns

<!-- Patterns that should never be used and why -->

- Do not respond to a `max_lines` failure by compressing logic through terse
  naming, comment removal, branch flattening, or other readability-only cuts.
- Do not keep unrelated responsibilities in one file just because the code can
  be shortened enough to satisfy the limit.

---

## Required Patterns

<!-- Patterns that must always be used -->

### Natural Module Splits For Line-Limit Pressure

When a backend file reaches or is clearly trending toward its configured
`max_lines` limit in `.assura/config.yml`, treat that as a design signal to
split the module by responsibility.

Preferred response:

- Extract cohesive helpers, data types, validators, or execution modes into
  sibling submodules.
- Keep the entrypoint file focused on orchestration and public surface.
- Preserve descriptive names, explanatory comments, and straightforward control
  flow even after the split.

Avoid artificial shortening:

- Do not rewrite clear code into terse one-liners just to save lines.
- Do not collapse match arms or error messages only to get under the limit.
- Do not hide a second concern inside a vaguely named helper to avoid creating
  a submodule.

Good examples:

- Split a config validation file into `validation.rs` plus focused helpers such
  as `validation/naming.rs` or `validation/extensions.rs`.
- Split a CLI surface into `foo.rs` plus `foo/report.rs`, `foo/rows.rs`, or
  `foo/cache.rs` when those concerns can evolve independently.

---

## Testing Requirements

<!-- What level of testing is expected -->

(To be filled by the team)

---

## Code Review Checklist

<!-- What reviewers should check -->

- If a file hit or neared a configured line limit, did the change create a
  natural submodule boundary instead of just shortening the file?
- Do extracted modules have clear ownership by concern, not arbitrary
  line-count slicing?
- Did the split preserve readable naming, comments, and error messages?
