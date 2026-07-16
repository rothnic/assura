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
