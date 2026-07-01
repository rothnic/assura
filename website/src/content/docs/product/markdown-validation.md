---
title: Markdown Validation
description: Generic Markdown linting, safe fixes, and Assura-owned document hierarchy checks.
---

Markdown validation sits above structure validation. It handles generic document
shape while typed frontmatter fields stay in content models.

## Responsibility Split

| Area | Status | Owner |
| --- | --- | --- |
| `markdown.require_frontmatter` | Shipped | Generic Markdown presence rule. |
| `markdown.outline` | Shipped | Assura-owned heading hierarchy with required and optional nested headings. |
| `markdown.lint_trailing_spaces` | Experimental | Rust-native lint for blank Markdown lines containing spaces or tabs. |
| `markdown.check_links` | Experimental | Local validation for relative file links, unrendered local references, Markdown heading anchors, and GitHub-style line/range anchors. |
| `markdown.rules.<rule_id>.severity` | Experimental | Per-rule severity overrides for supported Markdown findings. |
| `assura-ignore` comments | Experimental | Reasoned suppressions for supported Markdown rule IDs. |
| `assura fix markdown` | Experimental | Safe fix command for deterministic blank-line trailing whitespace and required-section heading appends with preview and explicit `--apply` audit JSON. |
| Typed frontmatter fields | Shipped | Content runtime `models` and `collections`, not generic Markdown rules. |
| Broad markdownlint-compatible rules | Planned | Revisit when Assura has a compatible dependency or external-binary contract. |
| Remote link checking | Planned | Needs offline/network policy and normalized diagnostics before support. |

## Safe Fixes

Safe fix operations are explicit and bounded. The trailing-spaces rule removes
only spaces and tabs from otherwise blank Markdown lines in configured Markdown
scopes. The required-sections rule appends missing configured headings when the
project has declared `markdown.required_sections`. Neither rule rewrites
content-line hard breaks, frontmatter fields, or body prose.

```bash
assura fix markdown --rule trailing-spaces --dry-run --format json .
assura fix markdown --rule trailing-spaces --apply --format json .
assura fix markdown --rule required-sections --dry-run --format json .
```

The JSON report uses `assura.safe-fix.markdown.v1` for both preview and apply.
Preview records planned fix IDs without writing. Apply records changed paths,
applied fix IDs, skipped Markdown files, and VCS-first rollback guidance.

## Local Link Checks

`markdown.check_links: true` validates relative Markdown-authored links that
point inside the repository:

```markdown
[Guide](guide.md)
[Install](guide.md#install)
[Code](../src/lib.rs#L12-L34)
```

The check is local and offline. It reports missing files, missing Markdown
heading anchors, invalid line anchors, invalid line ranges, and root-absolute
internal links that should be relative for GitHub rendering in branches, forks,
and pull requests. It also reports existing local file references in prose or
inline code that should be rendered as Markdown links, such as
`../src/lib.rs:1-2`.

## Severity And Suppression

Markdown scopes can lower or raise supported rule severities:

```yaml
markdown:
  check_links: true
  rules:
    markdown_link_target:
      severity: low
```

Use `<!-- assura-ignore <markdown_rule>: <reason> -->` for intentional local
exceptions. The rule ID must be supported and the reason must be non-empty;
otherwise Assura reports `markdown_suppression`.

See [Configuration](/docs/configuration/) for Markdown fields and
[Content Models](/product/content-models/) for typed frontmatter validation.
