# A04 acceptance repair plan

Spec: [initialization packet](initialization-packet.md#a04).
Reviewed baseline: `6d613f809feabd65b520c57bfd7aaf1b521a107e`.

## Global Constraints

- No user hook is lost; update/remove touch only managed content.
- Required host permission is an explicit unavailable state until granted.
- Existing hook managers are preserved; generated files do not prove activation.
- Preserve ordinary-branch advisory behavior and opt-in blocking push behavior.
- Test first; no policy weakening or performance regressions to clear a gate.
- Each repair is reviewed and gated against current master before integration.

## Task 1: Protect Git hook ownership at every mutation boundary

Own `src/cli/hooks.rs`, its cohesive helpers if necessary, existing hook tests,
and the exact Git ownership section of the hook spec. Do not edit backlog,
progress or A04 evidence; the controller owns those.

Independent-review follow-up also owns only the install/remove preservation
wording in `src/cli/full_entry.rs`: preserved drift, orphan and unsafe artifacts
must not all be labeled custom hooks. Literal Unix paths are byte strings;
Linux must prove real non-UTF-8 path invocation and reject lossy legacy ownership.
macOS filesystem rejection is not a passing or failing wrapper contract test.

Final-review correction: exact historical wrappers are ownership evidence, not
proof of a safe/current runnable wrapper. Their old double-quoted path expands
`$()` and backticks and mishandles embedded double quotes. Keep exact legacy
ownership for refresh/removal, but distinguish it from current safe content.
Default installation must transactionally upgrade a proven legacy pair, not
leave it unchanged until `--force`. Before upgrade status must not claim ready;
after upgrade prove literal invocation and no shell-substitution sentinel.
Test ordinary legacy upgrade, rerun idempotence, forced refresh, removal and
path variants without broadening ownership of custom/drifted files.

1. Run the current focused tests as the baseline; then add focused failing tests
   for direct forced install over a custom wrapper, an exact managed wrapper
   with a modified sidecar, and an orphan sidecar with arbitrary user content.
   Exercise both single and bulk mutation entrypoints.
2. Centralize ownership classification for the wrapper AND sidecar before any
   write/delete. A marker or expected filename is not ownership proof. Preserve
   drifted content, including when force is set, and report preservation honestly.
   Missing sidecars may be repaired only under proven managed ownership; an
   orphan may be removed only when its own managed content is proven.
3. Prevent static symlink escapes in hook files and Assura hook directories;
   dangling symlinks are not absent files. Preserve external targets. Bound the
   guarantee accurately; do not claim protection from arbitrary concurrent
   hostile filesystem mutation without implementing it.
4. Keep a failed two-file installation from leaving changed live hook content.
   Use the existing safe-write patterns where appropriate, handle rollback
   failures explicitly, and test a deterministic interrupted/failed install.
5. Safely quote literal filesystem paths in generated shell wrappers. Test real
   invocation with spaces and shell metacharacters using disposable paths, so a
   path cannot expand into a command. Retain existing managed-hook compatibility
   without treating arbitrary marker-bearing content as owned.
6. Preserve positive lifecycle cases: clean install, rerun, refresh, removal,
   repair of missing managed artifacts, and accurate status for sidecar drift.
   Update the old arbitrary orphan-deletion test to demand preservation because
   the higher-level A04 preservation contract overrides that incorrect test.
7. Run focused red/green, hook unit/integration tests, formatting and self-check;
   inspect and run `cargo xtask fast` then `cargo xtask pr` before PR readiness.
   Retain exact cwd/SHA/binary/commands/exits/test counts. Capture exec session IDs
   and poll those sessions; a timeout of observation is not a terminated process.
   Do not start parallel Cargo commands against one target directory.

This is a bounded A04 repair, not full A04 closure. Effective `core.hooksPath`
and managed-host permission/runtime reporting remain explicit follow-up repairs.
Do not silently declare those accepted because this ownership slice passes.

## Subsequent current-master slices

- Effective hook directory and manager handoff: resolve actual Git configuration,
  preserve custom managers, integrate through supported project configuration,
  and prove a real event through the effective path.
- Honest host state: preserve configuration verification scope and approval
  requirements in onboarding, report runtime evidence separately, and exercise
  claimed hosts or mark unavailable. Never infer runtime proof from config equality.
- Re-run full A04 acceptance before A06/A07; preserve A05's existing branch and
  correct native coverage/preservation before continuing it.

## Preflight rulings

| Intersection | Finding and ruling |
| --- | --- |
| Task 1 tests versus A04 acceptance | Existing orphan test deletes arbitrary content. Replace that assertion with preservation; retaining it would violate the spec. |
| Task 1 versus effective-path repair | Both touch hooks.rs. Serialize integrations and rebase the next slice on reviewed merged master. |
| Task 1 versus host-state repair | Separate mutation/reporting surfaces; retain separate proof and do not mark A04 done after Task 1. |

Cost if these rulings are wrong: extra repair iteration, never intentional loss
of user content or weakened acceptance.
