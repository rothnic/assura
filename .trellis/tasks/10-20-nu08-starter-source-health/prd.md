# NU08: Explicit Python and Rust starter health

## Goal

Materialized starter health rules consistently advise on Python and Rust file
size, with honest coverage/exclusion boundaries and project-owned policy.
Bead ae-4nm is the live queue; no duplicate backlog is created.

## Authority

Nick authorized dependency-ready Beads implementation, independent GPT-6 Luna
review/corrections and root-owned gated merges. Root owns Beads, native writes,
push/PR/merge/closure. nu08-implement owns the six scoped implementation paths.
No public API break, release, host configuration or blanket exclusion decision.

## Contract

Use the refreshed external nu08/current-702-readiness-20261020.md (basis b5dbf73e)
as the implementation contract, correcting prior proposal exclusion details.
Extend only the three existing recursive advisory500-line selectors with py/rs:
init_support standalone/combined and agent_onboarding_templates. Preserve severity,
thresholds, naming/layout, exclusions, generic merge and quality commands/types.
Document the seven extensions and distinct recipe/onboarding exclusion lists;
Python env/cache/vendor/build/generated names are not universally excluded.
Generated rules guidance describes the baseline conditionally when authored
conflicts may preserve different effective rules. No inferred authored-only claim.

## Acceptance

Focused CLI integration tests exercise standalone and combined init plus fresh
onboarding:501-line py/rs/md/js-or-ts positive,500-line negative, existing target
exclusion, default generated file selected then fixture-owned opt-out excluded.
Check exact materialized coverage/exclusions and no added language naming policy.
Existing-config onboarding preserves parsed authored values and effective custom
quality plan under a distinct non-bun scope. Comments are not byte-preserved by
existing YAML materialization; do not add new merge machinery. Update user docs
and durable onboarding spec. Independent Luna review, meaningful runtime controls,
applicable local/current hosted gates, root merge/postmerge/native close and clean
owned refs/worktree required. Preserve NU03 public-field approval hold separately.

## Owned files

src/cli/init_support.rs; src/cli/agent_onboarding_templates.rs;
tests/starter_source_health.rs; website/src/content/docs/reference/configuration.md;
website/src/content/docs/guides/agent-ready-onboarding.md;
.trellis/spec/assura/agent-ready-onboarding.md. Task PRD may receive evidence updates.
No new production module/dependency or .assura/config.yml policy change.
