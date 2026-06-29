---
title: Project Intelligence Demo
description: A visual walkthrough from modeled content to search, graph context, agent envelopes, and safe-fix previews.
---

Assura project intelligence starts with ordinary repository files. A maintainer
checks in typed content models, Markdown scopes, and relations, then agents and
humans query the same local facts through the CLI.

<section class="pi-demo-flow" aria-label="Project intelligence workflow">
  <div class="pi-demo-step">
    <strong>1. Model</strong>
    <span>Goals and specs stay in Markdown and JSON.</span>
  </div>
  <div class="pi-demo-step">
    <strong>2. Validate</strong>
    <span><code>assura check</code> proves typed fields and relations.</span>
  </div>
  <div class="pi-demo-step">
    <strong>3. Query</strong>
    <span>Search and graph expansion expose related project context.</span>
  </div>
  <div class="pi-demo-step">
    <strong>4. Assist</strong>
    <span>Agent envelopes and safe-fix previews reuse the same facts.</span>
  </div>
</section>

<section class="pi-demo-board" aria-label="Project intelligence demo board">
  <div class="pi-demo-lane">
    <h2>Repository Files</h2>
    <p><code>docs/goals/goal_portable_structure.md</code></p>
    <p><code>specs/spec_portable_structure.json</code></p>
    <p><code>.assura/config.yml</code></p>
  </div>
  <div class="pi-demo-lane">
    <h2>Assura Facts</h2>
    <p><strong>goal-portable-structure</strong> references <strong>spec-portable-structure</strong>.</p>
    <p>Markdown headings become searchable sections.</p>
    <p>Broken references become diagnostics.</p>
  </div>
  <div class="pi-demo-lane">
    <h2>Usable Surfaces</h2>
    <p><code>assura content search</code></p>
    <p><code>assura content expand</code></p>
    <p><code>assura content agent-query</code></p>
  </div>
</section>

## Run The Demo

From the Assura repository, use the checked fixtures as a small project with a
goal linked to a spec:

```bash
assura check --format json tests/fixtures/content_runtime/valid
```

The valid project has no violations:

```json
{
  "success": true,
  "files_checked": 3,
  "dirs_checked": 4,
  "violations": []
}
```

## Search Project Knowledge

Search is deterministic and local. It indexes modeled instances and Markdown
sections:

```bash
assura content search "Portable Structure" tests/fixtures/content_runtime/valid --format json
```

The result includes the Markdown section, the spec instance, and the goal
instance:

```json
{
  "query": "Portable Structure",
  "matches": [
    {
      "source_kind": "markdown_section",
      "path": "docs/goals/goal_portable_structure.md",
      "text": "Portable Structure Policy"
    },
    {
      "source_kind": "model_instance",
      "collection": "specs",
      "instance_id": "spec-portable-structure"
    },
    {
      "source_kind": "model_instance",
      "collection": "goals",
      "instance_id": "goal-portable-structure"
    }
  ]
}
```

## Expand Related Context

Graph expansion starts from a modeled instance and returns related facts:

```bash
assura content expand goals goal-portable-structure tests/fixtures/content_runtime/valid --format json
```

Use this when an agent needs the spec, Markdown section, and related diagnostics
around one project object before editing.

## Catch Broken Relations

The invalid fixture keeps the same goal but points it at a missing spec:

```bash
assura content missing-relations tests/fixtures/content_runtime/missing_reference --format json
```

The missing target stays machine-readable:

```json
{
  "missing_relations": [
    {
      "field": "specs",
      "target_instance_id": "missing-spec",
      "target_collections": ["specs"],
      "missing": true
    }
  ]
}
```

## Hand Context To An Agent

Agents can request the same diagnostic through the shared query envelope:

```bash
assura content agent-query diagnostics tests/fixtures/content_runtime/missing_reference --format json
```

The envelope identifies the schema, requested capability, and diagnostic:

```json
{
  "schema": "assura.project-intelligence.agent-query.v1",
  "request": {
    "capability": "diagnostics",
    "cli": "assura content agent-query diagnostics"
  },
  "response": {
    "diagnostics": [
      {
        "path": "docs/goals/goal_portable_structure.md",
        "rule": "content_runtime:missing_reference",
        "severity": "high"
      }
    ]
  }
}
```

## Preview Safe Fixes

Safe-fix previews are separate from validation. They report bounded writes
without changing files:

```bash
assura fix markdown --rule trailing-spaces --dry-run --format json .
```

For a Markdown file with configured blank-line trailing whitespace, the report
uses the `assura.safe-fix.markdown.v1` schema and separates proposed changes
from applied changes:

```json
{
  "schema": "assura.safe-fix.markdown.v1",
  "dry_run": true,
  "files_changed": 0,
  "fixes_applied": 0,
  "files_would_change": 1,
  "fixes_would_apply": 1
}
```

## Adoption Path

1. Start with `assura init` and a structure policy.
2. Add typed collections for the project knowledge agents need.
3. Run `assura check --format json .` until the model is clean.
4. Use `assura content search`, `assura content missing-relations`, and
   `assura content expand` for human inspection.
5. Use `assura content agent-context` and `assura content agent-query` for
   automation.
6. Use `assura fix markdown --dry-run --format json` before accepting safe
   Markdown repairs.

This path is local, source-control friendly, and does not require a daemon,
hosted service, remote embedding provider, or editor plugin.
