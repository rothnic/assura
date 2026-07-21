# Quality Guidelines

> Code quality standards for frontend development.

---

## Overview

Website changes should prove both build correctness and rendered layout quality.
For standalone landing pages, verify desktop, tablet, and mobile widths in light
and dark color schemes before claiming completion.

---

## Forbidden Patterns

- Do not turn the docs home into the product landing experience by expanding
  Starlight Markdown splash pages. Use a custom Astro page when the route needs
  standalone product polish.
- Do not commit temporary browser screenshots or verification folders at the
  repository root. Assura should reject those as structure drift.
- Do not rely on low-contrast accent colors for small labels. Light-mode
  product accents used for text should meet WCAG AA contrast for normal text.

---

## Required Patterns

- Run the website production build after route or style changes.
- Smoke-test at least one existing docs route when changing `/` or shared docs
  palette CSS.
- Keep static pages static unless an island adds clear product value.
- In syntax-highlighted configuration examples, color keys consistently whether
  they open a nested scope or assign an inline value. Apply secondary color to
  the value token, not the entire line, so paths do not imply different states.
- For tool comparisons, render checked configuration fixtures exactly and keep
  the view selector directly adjacent to the content it changes. Separate the
  shared benchmark surface from Assura-only capabilities before showing code.
- When a comparison fixture cannot express a marketed Assura guarantee, mark
  the limitation beside the relevant valid YAML as a comment and style it as an
  annotation. Do not present unsupported behavior as equivalent coverage.
- Map every displayed competitor limitation one-to-one to an executable native
  tool scenario. The test must compare the exact displayed claim list with the
  proof registry so copy cannot be added, removed, or reworded without updating
  its behavioral evidence.
- Present multi-fixture performance evidence as a visible comparison table
  before verbose fixture internals. Each row should identify the variable under
  test, workload scale, both cold timings, the cold speed ratio, and the warm
  timing; keep generated trees and source policies in optional disclosures.
- Keep marketing claims executable: if copy says a path is required, the
  checked fixture must include the corresponding required-path directive.

---

## Testing Requirements

For landing-page work, collect:

- `pnpm build` or `cargo xtask docs`
- `cargo run --quiet -- check --format json .`
- browser screenshots and DOM overflow checks at representative breakpoints
- a docs-route smoke check

---

## Code Review Checklist

- Does `/` feel like a deliberate product surface rather than a docs template?
- Do existing Starlight docs routes still build and render?
- Do text, buttons, and navigation avoid wrapping overflow on mobile?
- Do light and dark palettes preserve contrast for small text?
- Can a reader distinguish shared coverage, unsupported behavior, and the
  currently selected configuration without reading both files end to end?
