# File Collection And Adapter Options

## Keystatic

Keystatic is the most relevant product reference for editable content in a
codebase. It supports local filesystem storage and Markdown, JSON, YAML,
Markdoc, and MDX formats. Its collection/singleton model maps well to "typed
objects live at configured paths."

Sources:
* https://keystatic.com/
* https://keystatic.com/docs/local-mode
* https://keystatic.com/docs/collections
* https://keystatic.com/docs/content-organisation
* https://keystatic.com/docs/format-options

Assura takeaway:
* Borrow the collection/path/field/storage mental model.
* Do not depend on Keystatic for core validation because it is TypeScript/UI/CMS
  oriented and not repo-wide policy validation.

## Astro Content Collections

Astro validates files within content collections and uses Zod schemas to create
typed query APIs. It is a strong read-side reference for developer ergonomics.

Sources:
* https://docs.astro.build/en/guides/content-collections/
* https://docs.astro.build/en/reference/modules/astro-content/

Assura takeaway:
* Borrow the collection schema + typed query ergonomics.
* Extend the concept with writes, multiple data formats, path placement rules,
  and cross-repository references.

## Adapter Implications

Assura likely needs its own adapter boundary:

* JSON: parse/update/reformat canonical records.
* JSONL: append/update individual lines only when identity can be stable.
* YAML: parse/update while deciding how much formatting/comment preservation is
  required.
* CSV: table schema validation, row identity, dialect handling.
* Markdown plus frontmatter: split metadata from body, validate outline, and
  write frontmatter without damaging body content.
* MDX/Markdoc: likely out of MVP except as future adapter targets.

## Prototype Recommendation

Start with two adapters: Markdown plus frontmatter and JSON per record. Add CSV
or JSONL only if the first prototype proves the adapter boundary cleanly.
