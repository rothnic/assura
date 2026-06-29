---
title: Project intelligence real repo proof
status: active
---

# Project Intelligence Real Repo Proof

## Assura Repository State

The live Assura repository now participates in the proof through an
`assura_goals` collection in `.assura/config.yml`. It models checked
`docs/goals/*.md` frontmatter through
`.assura/project-intelligence-goals.schema.json`.

Command:

```bash
cargo run --quiet -- check --format json .
```

Expected result: success with no violations.

Command:

```bash
cargo run --quiet -- content search "Project Intelligence Usability" . --format json
```

Expected result: matches include the
`goal-assura-project-intelligence-usability-program` model instance from
`docs/goals/assura-project-intelligence-usability-program.md`.

## Non-Assura Scenario

`tests/fixtures/project_intelligence_real_repo/beacon_crm` is the deterministic
non-Assura TypeScript workspace fixture. It models a small CRM repo with:

- a web app under `apps/web`;
- a shared UI package under `packages/ui`;
- an epic in `docs/epics/epic_checkout.md`;
- an ADR in `docs/decisions/adr_ui_boundary.json`;
- a package intelligence record in `packages/ui/package.assura.json`;
- a project-intelligence schema artifact in
  `schemas/project-intelligence.schema.json`.

The fixture is intentionally local and checked in. Ordinary tests do not fetch
or clone any external repository.

## Valid State

Command:

```bash
cargo run --quiet -- check --format json tests/fixtures/project_intelligence_real_repo/beacon_crm/valid
```

Expected result: success with 11 files, 10 directories, and no violations.

Command:

```bash
cargo run --quiet -- content search "checkout onboarding" tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --format json
```

Expected result: matches include the `epic-checkout` model instance and the
`Checkout Onboarding` Markdown section.

Command:

```bash
cargo run --quiet -- content expand epics epic-checkout tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --format json
```

Expected result: related facts include
`docs/decisions/adr_ui_boundary.json` and
`packages/ui/package.assura.json`.

Command:

```bash
cargo run --quiet -- content agent-query graph-expand tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --collection epics --id epic-checkout --format json
```

Expected result: the response uses
`assura.project-intelligence.agent-query.v1` and the `graph_queries`
capability family.

## Invalid State

Command:

```bash
cargo run --quiet -- check --format json tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid
```

Expected result: failure with:

- `content_runtime:invalid_object_shape` for the missing `owner` field on
  `docs/epics/epic_checkout.md`;
- `content_runtime:missing_reference` for
  `adr-missing-payment-risk`.

Command:

```bash
cargo run --quiet -- content missing-relations tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --format json
```

Expected result: `decision_refs` reports the missing target
`adr-missing-payment-risk`.

Command:

```bash
cargo run --quiet -- content agent-query diagnostics tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --format json
```

Expected result: the agent envelope reports both the missing `owner` field and
the missing decision reference as high-severity diagnostics.

## Safe-Fix Preview

The repository keeps checked fixtures free of trailing whitespace so
`git diff --check` remains useful. The Markdown-drift proof is therefore
materialized in a temporary copy:

```bash
tmp=$(mktemp -d)
cp -R tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid/. "$tmp/"
python3 - "$tmp/docs/epics/epic_checkout.md" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
path.write_text(text.replace('# Checkout Onboarding\n\n', '# Checkout Onboarding\n   \n'))
PY
cargo run --quiet -- fix markdown "$tmp" --dry-run --format json
rm -rf "$tmp"
```

Expected result:

```json
{
  "schema": "assura.safe-fix.markdown.v1",
  "dry_run": true,
  "files_checked": 1,
  "files_changed": 0,
  "fixes_applied": 0,
  "files_would_change": 1,
  "fixes_would_apply": 1
}
```

## Regression Coverage

`tests/project_intelligence_real_repo_proof.rs` covers Assura goal search,
Assura goal graph expansion, Beacon valid check, Beacon search, Beacon graph
expansion, Beacon graph agent-query envelope, Beacon invalid check,
missing-relations query, diagnostics agent-query envelope, and materialized
Markdown safe-fix preview.

Focused validation:

```bash
cargo test --test project_intelligence_real_repo_proof --quiet
```
