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

## Start From A Template

For a fresh repo, create the starter content model, collections, and example
records in one command:

```bash
assura init --project-intelligence --no-git-hooks .
```

The starter writes `.assura/config.yml`,
`schemas/project-intelligence-starter.schema.json`, a modeled goal, a spec, an
ADR, and a broken-state example under `docs/examples/`.

```bash
assura check --format json .
assura content search "Adopt Project Intelligence" . --format json
assura content expand goals goal-project-intelligence-starter . --format json
```

To see relation diagnostics, copy the broken example into the modeled goal
collection:

```bash
cp docs/examples/project-intelligence-broken-goal.md docs/goals/goal_project_intelligence_missing_context.md
assura content missing-relations . --format json
assura content agent-query diagnostics . --format json
```

Replace the starter goal, spec, and ADR with project-specific goals, specs,
ADRs, packages, or release artifacts once the first query works.

## Hand Off A Context Pack

Use a context pack when an agent or editor needs one bounded packet for an
editing task. It combines diagnostics, missing relations, optional keyword
search, safe-fix preview metadata, and object context when an instance is
named:

```bash
assura content context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --text checkout --limit 5 --format json
```

The response uses `assura.project-intelligence.context-pack.v1`, reports
`bounds.limit`, lists omitted fields such as object context when no
`--collection` and `--id` are provided, and includes the same missing owner and
missing ADR diagnostics that lower-level commands expose.

For object-oriented work, include the modeled object:

```bash
assura content context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --text "Project Intelligence Usability" --limit 5 --format json
```

Use lower-level commands such as `assura content search`, `assura content
expand`, and `assura content missing-relations` when inspecting one capability.
Use `assura content context-pack` when preparing a bounded handoff for an
agent, editor integration, or reviewer.

## Keep Context Warm

Use a content session when an agent or editor wrapper needs repeated
project-intelligence queries without restarting the CLI process:

```bash
assura content session .
```

Then send JSON-line requests on stdin:

```json
{"request_id":"ctx-1","type":"context-pack","collection":"assura_goals","id":"goal-assura-project-intelligence-usability-program","text":"Project Intelligence Usability","limit":5}
```

The session emits one JSON response per line with schema
`assura.project-intelligence.session.response.v1`. Each response reports
`reload.state` as `initial_load`, `reused`, or `reloaded`, so wrappers can tell
whether the process reused the loaded project facts or rebuilt them after a
config/content change. This is a local disposable session, not hosted
infrastructure and not `assura watch`.

Supported request types include `agent-context`, `collections`, `context-pack`,
`diagnostics`, `expand`, `missing-relations`, `safe-fixes`, and `search`.
Invalid requests return the same response envelope with `ok: false` and an
error code such as `request_failed`.

## Agent Editing Handoff

Task: repair the Beacon CRM checkout epic so project-intelligence validation can
trust its owner and decision references.

Context command:

```bash
assura content context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --collection epics --id epic-checkout --text checkout --limit 5 --format json
```

Inspect these response fields before editing:

- `diagnostics`: identify `content_runtime:invalid_object_shape` and
  `content_runtime:missing_reference`.
- `instance.data`: confirm the modeled epic fields and current relation IDs.
- `instance.sections`: find the Markdown section that describes the epic.
- `related.related`: inspect any resolved ADR or package records.
- `missing_relations`: identify the unresolved target instance ID.
- `safe_fixes`: confirm no automatic write is proposed for this semantic
  relation repair.

Edit constraints: do not change `.assura/config.yml`, do not invent a remote
provider, and do not apply safe fixes automatically. Fix the source records or
the broken reference, then verify with:

```bash
assura check --format json tests/fixtures/project_intelligence_real_repo/beacon_crm/valid
assura content context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --collection epics --id epic-checkout --text checkout --limit 5 --format json
```

Expected evidence: validation succeeds, `missing_relations` is empty, and the
context pack still includes the `epic-checkout` model instance plus related ADR
and package records.

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

1. Start with `assura init --project-intelligence` for a working starter.
2. Replace the starter records with the project knowledge agents need.
3. Run `assura check --format json .` until the model is clean.
4. Use `assura content context-pack` for a bounded agent or editor handoff.
5. Use `assura content search`, `assura content missing-relations`, and
   `assura content expand` for human inspection.
6. Use `assura content agent-context` and `assura content agent-query` for
   automation.
7. Use `assura fix markdown --dry-run --format json` before accepting safe
   Markdown repairs.

This path is local, source-control friendly, and does not require a daemon,
hosted service, remote embedding provider, or editor plugin.

## Realistic Repo Proof

The visual demo above uses a small content-runtime fixture so the data is easy
to inspect. The same workflow is also checked against the Assura repository and
a more realistic TypeScript-style workspace.

The Assura repo models its own goal docs as project-intelligence facts:

```bash
assura content search "Project Intelligence Usability" . --format json
```

The result includes
`goal-assura-project-intelligence-usability-program` from
`docs/goals/assura-project-intelligence-usability-program.md`.

```bash
assura check --format json tests/fixtures/project_intelligence_real_repo/beacon_crm/valid
assura content search "checkout onboarding" tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --format json
assura content missing-relations tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --format json
assura content agent-query diagnostics tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --format json
```

The Beacon CRM fixture models a non-Assura repo with `apps/web`, `packages/ui`,
an epic, an ADR, and a package intelligence record. The invalid state proves
missing field and missing relation diagnostics, while the regression test
materializes Markdown trailing-whitespace drift in a temporary copy for
safe-fix preview.

See `docs/analysis/2026-06-29-project-intelligence-real-repo-proof.md` and
`tests/project_intelligence_real_repo_proof.rs` for the exact evidence.
