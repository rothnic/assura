# Config notation references

## YAML Marker Behavior

- YAML 1.2.2 defines question mark followed by space as the explicit
  mapping-key indicator. This makes a bare list item such as
  `- ? Optional` parse as a map shape instead of a plain string in common YAML
  parsers.
- Product implication: do not use a single bare leading `? ` as Assura's
  optional-heading marker.
- Recommended marker: `?? ` at the beginning of a heading line inside Assura's
  `outline:` list. It parses as a normal string and avoids ambiguity with
  headings that naturally end with question marks.
- Source: https://yaml.org/spec/1.2.2/

## Nested YAML As A Familiar Structure Idiom

- MkDocs uses nested YAML lists/maps to represent navigation hierarchy. It is a
  useful precedent for expressing document-like hierarchy directly in YAML,
  rather than maintaining separate numeric depth fields.
- Product implication: Assura Markdown outlines should use nested YAML to
  represent heading hierarchy.
- Source: https://www.mkdocs.org/user-guide/writing-your-docs/

## Existing Heading Validation Precedent

- markdownlint rule MD043 validates a required heading structure from a list of
  heading strings.
- Product implication: required heading structure is a known linting use case,
  but Assura can improve readability by representing nesting directly and by
  marking optional nodes inline.
- Source: https://github.com/DavidAnson/markdownlint/blob/main/doc/md043.md

## Personal LS-Lint Fork Evidence

- `rothnic/ls-lint` PR #1 merged required file/directory validation under
  `exists` and removed a separate `required` directive.
- Product implication: Assura should use cardinality (`exists:1`,
  `exists:0-1`, `exists:0`, `exists:N-M`) as the concise common-case model
  instead of duplicating required and allow concepts.
- Source: https://github.com/rothnic/ls-lint/pull/1

- `rothnic/ls-lint` PR #4 keeps examples concise with `groups:`, `@group`
  references, composed rules, and content-rule prototypes.
- Product implication: Assura should support reusable rule fragments and
  concise references, but adapt the syntax to Assura's tree-first `structure:`
  model.
- Source: https://github.com/rothnic/ls-lint/pull/4

- The fork's TypeScript best-practices example uses reusable groups such as
  `ts-defaults`, `doc-page`, and `doc-templates`, then references them from
  file and directory scopes.
- Product implication: reusable fragments are necessary to avoid duplicating
  README/AGENTS/package rules across root and package scopes.
- Source:
  https://github.com/rothnic/ls-lint/blob/copilot/featureagent-constraints-feedback/examples/typescript_best_practices/.ls-lint.yml

- The fork's future content-rule note proposes simple inline content checks and
  named profiles for more detailed validation.
- Product implication: Assura should keep simple notation concise while leaving
  object-form attributes and validator IDs for complex cases.
- Source:
  https://github.com/rothnic/ls-lint/blob/copilot/featureagent-constraints-feedback/docs/reference/future-content-rules.md
