# Directory Structure

> How frontend code is organized in this project.

---

## Overview

The public website lives in `website/` and combines two route surfaces:

- standalone Astro product pages in `website/src/pages/`
- Starlight documentation content in `website/src/content/docs/`

Keep marketing/product landing work independent from Starlight docs content
unless the task explicitly asks to redesign documentation pages.

---

## Directory Layout

```
website/
├── src/
│   ├── assets/       # Optimized project-bound images imported by Astro
│   ├── components/   # Astro components used by docs or standalone pages
│   ├── content/docs/ # Starlight documentation routes
│   ├── pages/        # Standalone Astro pages, including the product landing page
│   └── styles/       # Starlight custom CSS and shared docs palette variables
└── public/           # Static files that must keep public URLs
```

---

## Module Organization

Use `src/pages/` for pages that need a custom product experience and should not
inherit Starlight page chrome. Use `src/content/docs/` only for documentation
pages that should appear in the Starlight sidebar/search flow.

Generated website assets that are imported by Astro should live in
`src/assets/`, not in a root-level verification folder. Keep local screenshots
and browser proof artifacts outside the repository, for example under `/tmp/`.

---

## Naming Conventions

Use kebab-case for frontend asset and page filenames, except for framework
entrypoints that already have a fixed name such as `index.astro`.

---

## Examples

- `website/src/pages/index.astro` is the standalone product landing page.
- `website/src/content/docs/guides/quickstart.md` is a Starlight docs page.
