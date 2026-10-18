# Preserve Existing Project Policy During Onboarding

## Goal

Make `assura agent onboard` preserve deliberate policy and established `.agents`
conventions in existing projects. Generated policy must not claim completion
when an explicit project closure prevents compatible output.

## Scope

- Change only existing onboarding generation, merge/status logic, its focused
  tests, and the agent-ready onboarding contract.
- Preserve explicit project recipe values and closed policies. Familiar paths
  alone do not prove Assura ownership; use an explicit marker or proven
  generated content/registry.
- Existing unmanaged `AGENTS.md` and skill files retain their contents and
  conventions. New managed artifacts receive precise policy coverage without
  copying observed user files into a whitelist.
- New/empty managed projects retain strict generated defaults.
- Keep existing report and status shapes; no new public flag, API, schema,
  module, dependency, automatic repair, or policy weakening.

## Acceptance

- [ ] Disposable copies of the retained Python and TypeScript trials reproduce
  existing `.agents` guidance/skill conventions and any custom closures; source
  trials and hooks remain unchanged.
- [ ] Existing user policy values and unmanaged guidance bytes are preserved;
  conflicting explicit closures produce a visible actionable conflict/status
  and do not yield a false passing/completeness claim.
- [ ] A valid unmanaged project receives only precisely constrained generated
  managed artifacts and reports unresolved specialization honestly.
- [ ] A new/empty managed project keeps intended strict defaults and passes.
- [ ] Repeated onboarding is idempotent; generated policy either passes or
  reports the actionable conflict.
- [ ] Negative controls prove unrelated custom skill conventions remain
  accepted when compatible, and explicit user closures are never relaxed.

## Validation

Use focused onboarding regression coverage after root resource admission;
preserve raw command output, exit status, cwd, source SHA and tool identity
before writing derived summaries. Do not run full build/PR tiers without fresh
resource admission.

## Out of Scope

Broad refactors, inferred whitelists, automatic repair, changes to historical
A02/A03 ownership, public API/schema changes, pushes, PRs, merges, or Beads
writes.
