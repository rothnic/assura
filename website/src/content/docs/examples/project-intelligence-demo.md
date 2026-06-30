---
title: Project Intelligence Demo
description: A visual walkthrough from modeled content to search, graph context, agent envelopes, and safe-fix previews.
---

Assura project intelligence starts with ordinary repository files. A
maintainer checks in typed content models, Markdown scopes, and relations, then
uses the local CLI to answer four practical questions:

- Is this content valid?
- What matches this text?
- What other content or code is related?
- What safe Markdown fixes are available?

This page shows the current commands directly. The lower-level commands are
usable today, but the product still needs one simpler repo-wide command for
searching code and content together.

<section class="pi-demo-flow" aria-label="Project intelligence workflow">
  <div class="pi-demo-step">
    <strong>1. Model</strong>
    <span>Goals and specs stay in Markdown and JSON.</span>
  </div>
  <div class="pi-demo-step">
    <strong>2. Validate</strong>
    <span><code>assura check</code> catches invalid frontmatter, records, and relations.</span>
  </div>
  <div class="pi-demo-step">
    <strong>3. Search</strong>
    <span><code>assura content search</code> returns scored local keyword matches.</span>
  </div>
  <div class="pi-demo-step">
    <strong>4. Expand</strong>
    <span><code>assura content expand</code> follows modeled relations.</span>
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
    <p>Broken fields and references become diagnostics.</p>
  </div>
  <div class="pi-demo-lane">
    <h2>Simple Target</h2>
    <p>One command should search code and content.</p>
    <p>Results should explain score, source, and related context.</p>
    <p>That is a follow-up goal, not a current claim.</p>
  </div>
</section>

## What Works Today

| Need | Current command | Notes |
| --- | --- | --- |
| Validate modeled content and Markdown rules | `assura check --format json .` | Deterministic validation truth. |
| Search modeled facts, Markdown sections, and diagnostics | `assura content search "text" .` | Local keyword score, not semantic truth. |
| Get scored local semantic candidates | `assura content semantic-search "text" . --enable-local` | Optional local baseline, disabled by default. |
| Follow related modeled objects | `assura content expand <collection> <id> .` | Follows configured relations and diagnostics. |
| Connect modeled content to code symbols | `assura content symbols <collection> <id> .` and `assura content symbol-refs <symbol> .` | Uses modeled symbol references, not full repo search. |
| Hand a bounded packet to an agent | `assura agent context-pack . --text "text"` | Local CLI wrapper over the same facts. |

## What Is Still Missing

The current commands do not yet provide one obvious repo-wide search command.
They also do not search every code file by default. The next product slice
should make this low ceremony:

```bash
assura find "checkout timeout"
```

Expected future behavior: return scored code and content matches, show why each
match ranked, and let the user expand from content to code or code to content
without choosing a collection or writing complex flags.

## Start From A Template

For a fresh repo, create the starter content model, collections, and example
records in one command:

```bash
assura init --project-intelligence --no-git-hooks .
```

The starter writes `.assura/config.yml`,
`.assura/models/project-intelligence/starter.schema.json`, a modeled goal, a
spec, an ADR, and a broken-state example under `docs/examples/`. Model
artifacts stored under `.assura/` stay grouped under `.assura/models/**`, and
you can add domain-specific hierarchy inside that directory as the project
grows.

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

Use the agent CLI when a local coding agent needs one bounded packet for an
editing task. It combines diagnostics, missing relations, optional keyword
search, safe-fix preview metadata, and object context when an instance is named:

```bash
assura content context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --text checkout --limit 5 --format json
assura agent context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --text checkout --limit 5
```

The response uses `assura.project-intelligence.context-pack.v1`, reports
`bounds.limit`, lists omitted fields such as object context when no
`--collection` and `--id` are provided, and includes the same missing owner and
missing ADR diagnostics that lower-level commands expose.

For object-oriented work, include the modeled object:

```bash
assura content context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --text "Project Intelligence Usability" --limit 5 --format json
assura agent context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --text "Project Intelligence Usability" --limit 5
```

Use lower-level commands such as `assura content search`, `assura content
expand`, and `assura content missing-relations` when inspecting one capability.
Use `assura agent context-pack` when preparing a bounded handoff for an agent,
editor integration, or reviewer.

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

## Show Diagnostics In An Editor

Use an editor session when a local editor wrapper wants LSP-shaped diagnostics,
object context, and safe-fix code-action previews over the same project facts:

```bash
assura editor session tests/fixtures/content_runtime/missing_reference
```

Send one request per line:

```json
{"request_id":"diag-1","method":"textDocument/diagnostics","params":{"textDocument":{"uri":"docs/goals/goal_portable_structure.md"}}}
{"request_id":"ctx-1","method":"textDocument/context","params":{"uri":"docs/goals/goal_portable_structure.md","text":"portable","limit":5}}
{"request_id":"fix-1","method":"textDocument/codeAction","params":{"uri":"docs/goals/goal_portable_structure.md"}}
```

The diagnostic response uses
`assura.project-intelligence.editor.response.v1` and returns LSP-shaped ranges,
severity, source, code, message, and diagnostic data:

```json
{
  "schema": "assura.project-intelligence.editor.response.v1",
  "method": "textDocument/diagnostics",
  "reload": {
    "state": "initial_load"
  },
  "ok": true,
  "result": {
    "diagnostics": [
      {
        "source": "assura",
        "code": "content_runtime:missing_reference",
        "severity": 1,
        "data": {
          "path": "docs/goals/goal_portable_structure.md"
        }
      }
    ]
  }
}
```

`textDocument/codeAction` returns preview actions only. Applying a repair still
requires an explicit `assura fix markdown --apply --format json` command, so an
editor wrapper can keep user approval in control.

## Agent Editing Handoff

Task: repair the Beacon CRM checkout epic so project-intelligence validation can
trust its owner and decision references.

Context command:

```bash
assura agent context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --collection epics --id epic-checkout --text checkout --limit 5
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
assura agent context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --collection epics --id epic-checkout --text checkout --limit 5
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

Search is deterministic and local. It indexes modeled instances, Markdown
sections, and diagnostics. Each result has a lexical score based on query term
matches; this is a ranking signal, not proof that the content is correct.

```bash
assura content search "Portable Structure" tests/fixtures/content_runtime/valid --format json
```

The result includes the Markdown section, the spec instance, and the goal
instance. Text output shows the same scores:

```text
Search matches: 3
3.000 goal-portable-structure - id: goal-portable-structure status: active title: Portable Structure Policy
3.000 spec-portable-structure - id: spec-portable-structure status: active title: Portable structure and frontmatter
2.000 markdown_section:11e59f57145df031 - Portable Structure Policy
```

JSON output is stable for agents:

```json
{
  "query": "Portable Structure",
  "matches": [
    {
      "source_id": "instance:4e78e4a53e9087e4",
      "source_kind": "model_instance",
      "score": 3.0,
      "collection": "goals",
      "instance_id": "goal-portable-structure",
      "path": "docs/goals/goal_portable_structure.md"
    },
    {
      "source_id": "instance:d2a61adf1fa7cae4",
      "source_kind": "model_instance",
      "score": 3.0,
      "collection": "specs",
      "instance_id": "spec-portable-structure",
      "path": "specs/spec_portable_structure.json"
    },
    {
      "source_id": "markdown_section:11e59f57145df031",
      "source_kind": "markdown_section",
      "score": 2.0,
      "path": "docs/goals/goal_portable_structure.md",
      "text": "Portable Structure Policy"
    }
  ]
}
```

For scored local semantic candidates, use the separate opt-in command:

```bash
assura content semantic-search "portable structure policy" tests/fixtures/content_runtime/valid --enable-local --format json
```

Semantic candidate scores help choose context. Validation still comes from
`assura check`, missing relations, and schema diagnostics.

## Expand Related Context

Graph expansion starts from a modeled instance and follows facts Assura already
knows: configured content relations, incoming relations, code-symbol edges, and
diagnostics attached to that object. It does not run another search. It answers
"I found this thing; what should I inspect next?"

```bash
assura content expand goals goal-portable-structure tests/fixtures/content_runtime/valid --format json
```

The valid fixture goal points to one spec, so expansion returns that related
model instance:

```json
{
  "root_id": "instance:4e78e4a53e9087e4",
  "related": [
    {
      "id": "instance:d2a61adf1fa7cae4",
      "kind": "model_instance",
      "relationship": "outgoing_relation",
      "path": "specs/spec_portable_structure.json"
    }
  ]
}
```

Use this after search when an agent needs the spec, Markdown section,
diagnostics, or code-symbol references around one project object before
editing.

## Catch Broken Relations

Assura also catches incomplete or invalid content when the source file no
longer matches the intended content model. In the starter shape, Markdown
frontmatter is the modeled object:

```markdown
---
id: goal-portable-structure
title: Portable structure policy
status: planned
specs:
  - spec-portable-structure
---

# Portable Structure Policy
```

The runtime schema declares which fields are required and which values are
valid. If a goal is missing a required frontmatter field, has the wrong field
shape, or points to an object that does not exist, `assura check` reports a
content-runtime diagnostic.

The invalid fixture keeps the same goal but points it at a missing spec:

```bash
assura check --format json tests/fixtures/content_runtime/missing_reference
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

Search also includes diagnostics, so a user or agent can find invalid content
without already knowing which file failed:

```bash
assura content search "missing-spec" tests/fixtures/content_runtime/missing_reference --format json
```

That returns a diagnostic match with the affected Markdown path and the
content-runtime rule. This is the path for catching incomplete or stale content
after the model changes: update the schema/config, run `assura check`, then use
diagnostics, search, and expansion to find the records that no longer conform.

## Hand Context To An Agent

Agents can request the same diagnostic through the local agent CLI:

```bash
assura content agent-query diagnostics tests/fixtures/content_runtime/missing_reference --format json
assura agent diagnostics tests/fixtures/content_runtime/missing_reference
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

## Preview And Apply Safe Fixes

Safe-fix previews are separate from validation. They report bounded writes
without changing files, and apply requires an explicit `--apply`:

```bash
assura fix markdown --rule trailing-spaces --dry-run --format json .
assura fix markdown --rule trailing-spaces --apply --format json .
```

For a Markdown file with configured blank-line trailing whitespace, the report
uses the `assura.safe-fix.markdown.v1` schema for both preview and apply. The
preview records stable fix IDs without writing:

```json
{
  "schema": "assura.safe-fix.markdown.v1",
  "mode": "dry_run",
  "dry_run": true,
  "files_changed": 0,
  "fixes_applied": 0,
  "files_would_change": 1,
  "fixes_would_apply": 1,
  "fixes": [
    {
      "id": "markdown.safe_fix.8d4c9b0d3d55c2a1",
      "path": "docs/epics/epic_checkout.md",
      "operation": "remove_blank_line_trailing_spaces",
      "status": "planned",
      "line": 18,
      "column": 1
    }
  ]
}
```

The apply audit uses the same IDs and adds changed-path evidence plus recovery
guidance:

```json
{
  "schema": "assura.safe-fix.markdown.v1",
  "mode": "apply",
  "dry_run": false,
  "changed_paths": ["docs/epics/epic_checkout.md"],
  "applied_fix_ids": ["markdown.safe_fix.8d4c9b0d3d55c2a1"],
  "skipped_fixes": [],
  "rollback": {
    "backup_created": false,
    "guidance": "Use version control to inspect or revert applied safe fixes."
  }
}
```

Agents can correlate context-pack or session previews with the CLI audit by
matching `safe_fixes[].audit_id` to `fixes[].id`:

```bash
assura agent safe-fixes tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid
```

```json
{
  "safe_fixes": [
    {
      "id": "safe_fix:0d2e6b4d4f7c9230",
      "audit_id": "markdown.safe_fix.8d4c9b0d3d55c2a1",
      "operation": "remove_blank_line_trailing_spaces",
      "path": "docs/epics/epic_checkout.md",
      "line": 18
    }
  ]
}
```

## Adoption Path

1. Start with `assura init --project-intelligence` for a working starter.
2. Replace the starter records with the project knowledge agents need.
3. Run `assura check --format json .` until the model is clean.
4. Use `assura agent context-pack` for a bounded agent or editor handoff.
5. Use `assura agent context-pack`, `assura agent diagnostics`, and
   `assura agent safe-fixes` for local coding-agent handoffs.
6. Use `assura editor session` when building a local editor wrapper for
   diagnostics, context, and safe-fix code-action previews.
7. Use `assura content search`, `assura content missing-relations`, and
   `assura content expand` for human inspection.
8. Use `assura content agent-context` and `assura content agent-query` when
   building lower-level wrappers that need the raw content contracts.
9. Use `assura fix markdown --dry-run --format json` to preview safe Markdown
   repairs, then `assura fix markdown --apply --format json` after accepting
   the planned IDs.

This path is local, source-control friendly, and does not require a daemon,
hosted service, remote embedding provider, MCP server, or editor plugin.

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
assura agent diagnostics tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid
```

The Beacon CRM fixture models a non-Assura repo with `apps/web`, `packages/ui`,
an epic, an ADR, and a package intelligence record. The invalid state proves
missing field and missing relation diagnostics, while the regression test
materializes Markdown trailing-whitespace drift in a temporary copy for
safe-fix preview.

See `docs/analysis/2026-06-29-project-intelligence-real-repo-proof.md` and
`tests/project_intelligence_real_repo_proof.rs` for the exact evidence.
