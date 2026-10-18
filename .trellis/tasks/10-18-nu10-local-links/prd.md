# NU10 maintained documentation local links

## Goal

Make the normal configured gate catch a missing relative file or cross-file
Markdown anchor in two maintained Assura policy documents, with valid
frontmatter, without turning historical/generated reference candidates into
blocking policy. Bead `ae-bku` is the root-owned live scheduling record; this
Trellis child stores the contract and evidence, not a second backlog.

## Authority

The user authorized the Assura new-user review and dependent backlog execution.
Root accepted the independently reviewed NU10 design on 2026-10-18 and authorized
this preparation commit. Root is the sole Beads, native receipt, push, PR and
merge writer. This worker owns only this candidate's scoped config, selected
docs if a real finding requires repair, and this new Trellis task.

Current authority is **preparation only**: create the branch/task, curate
portable context, commit clean, then freeze. Do not run native initialization,
`task.py start`, delivery registration, a policy edit or intentional runtime
controls until root registers/starts the task and explicitly releases
implementation. Normal repository status/structure guards triggered by Git
remain enabled; their results are not NU10 product acceptance evidence.

## Source and reviewed decisions

- Candidate base: `46e1e13b9f66012dd07ffeed0a0272b31582b374`.
- Branch: `fix/nu10-maintained-doc-links`; owner: `nu10-local-links`.
- Parent: `.trellis/tasks/10-17-assura-beads-execution`; preserve every existing
  child when adding this child.
- [Bounded proposal](/Users/nroth/workspace/.assura-review-20261017/nu10/ready-slice-proposal.md).
- [Accepted independent design review](/Users/nroth/workspace/.assura-review-20261017/nu10/design-review.md).
- Static intake found three existing rendered local file links in the selected
  documents, all with existing targets. This is not a CLI-clean result; do not
  claim a repair count until the actual configured baseline is observed.

## Scope

Use the existing quoted finite selector directly under the existing `docs/`
structure node, alongside its `files`, `markdown`, `directories` and `children`:

```yaml
"{support-policy,compatibility-and-surface}.md":
  markdown:
    require_frontmatter: true
    lint_trailing_spaces: true
    check_links: true
```

The complete per-file Markdown bundle is intentional: a matching pattern
replaces the directory Markdown bundle instead of field-merging it. Preserve
current naming, line limits, lifecycle/frontmatter rules, severity, thresholds,
exclusions and every unselected path's policy. Do not enable links on the
whole docs tree. Repair either selected document only for an observed current
finding; otherwise keep both source files byte-identical.

No engine feature, parser/schema/dependency, repository-reference promotion,
external URL checker, frontmatter-path validation, broad docs cleanup, release
wording, global config, skip/ignore broadening, severity reduction or native
receipt rebinding is in scope. The 1,116 intake reference candidates remain
unclassified broader data, not verified defects or newly blocking policy.

## Admitted implementation method after root release

Use only the absolute source-bound companion for every NU10 command:

`/Users/nroth/.codex/worktrees/assura-w02-release-story/assura/target/debug/assura-full`

Expected SHA-256:
`980bf04aae00b9196c1d9ea8d84b4680bf3ddb6ac7210282a9b7a450848867de`.

The reviewed binding is source merge
`1f8085ceb251b11098e3a9ca6ed3c731e9692b24`; its checker/build inputs match
candidate base 46e. The companion and regular single-root check route share
the structure checker. Rehash/source-compare before reuse and preserve the
actual version, cwd, argv, TMPDIR, raw stdout/stderr, elapsed time, observed
exit and outer tool session. Do not use the unbound regular W02 executable or
PATH fallback. Root preserves W02 runtime until this trial ends. No Cargo,
dependency install, full PR suite, website/browser/watch or network batch is
admitted for this config-only proof.

Use a fresh private disk-backed full tracked-tree archive of the frozen
candidate with its actual config and two real docs. Do not plant in primary or
copy/adopt live Git, Beads or native stores. Tracked provenance stays inert.
Save original fixture/header bytes, restore between cases, and retain adverse
results. Use direct `assura-full check --format json .`, without `--warn`,
minimum-severity override or cache. Compare normalized findings and exits;
`explain` is supporting scope evidence because it currently omits file-specific
Markdown/check_links settings.

## Acceptance

After root releases implementation, prove all of the following with the same
bound executable and actual config:

- Unmodified-config and policy-config full-check baselines exit 0, with no new
  link debt or unrelated findings. Existing low advisory findings stay equal.
- In **each** selected real doc, valid relative file, cross-file heading and
  line-range links pass; a body-only missing file with byte-identical valid
  frontmatter exits 1 with `markdown_link_target` at the exact path/line/target.
- A missing cross-file heading exits 1 with `markdown_link_heading_anchor` at
  the correct path/line; frontmatter/lifecycle errors cannot satisfy a link
  negative. Restore every case before the next one.
- Separate missing-header and trailing-space controls preserve their existing
  Markdown findings; the new pattern must not drop the old bundle fields.
- Missing links in existing unselected
  `docs/release-candidate-checklist.md`, historical
  `docs/archive/actual-state-audit.md`, and an excluded owned
  `target/nu10-generated.md` add no link blockers or unrelated policy changes.
  Preserve their actual frontmatter/status and existing configured checks.
- Inline-code/fenced examples and external links keep their existing behavior;
  no networking or broader link-form support is claimed.
- Restore exact fixture bytes, remove only the owned generated control, assert
  no planted token remains, and prove final full check exit 0 with findings and
  checked scope equal to the policy baseline. Rehash the executable afterward.
- Freeze a coherent clean implementation commit for independent review of the
  actual one-node config diff and raw receipts. Root owns applicable hosted
  gates, current-base integration, post-merge observation and closure.

Preserve any unexpected baseline/control failure and return for required
independent triage; do not rerun unchanged failure or weaken policy to get green.
A preparation commit, static inventory, task status or Git hook result alone
is not acceptance. Existing engine tests plus the real-config controls are
proportionate; a discovered engine defect needs separately admitted source
work and focused regression gates.

## Preparation checkpoint

Phase: planning / prepared, implementation held. Initial workflow gate exited
0 on clean base with no active task. Branch creation and task creation exited
0. The normal post-checkout hook printed read-only Assura project status
(270 configured directories, 172 file bundles, 8 Markdown bundles); preserve it
as incidental guard evidence only. Curated context validation passed (2 implement and 3 check entries); all
references are portable repo-relative specs. The pre-commit workflow gate
reported Ready: no solely for this task and its owned parent-child addition;
the prescribed resolution is the authorized preparation commit. The parent
change is exactly the append-only union of its prior three children and this
child. Config and both selected docs remain unchanged. Task remains planning;
no native task start, registration or receipt mutation was performed. Root's next action is native
registration/start and explicit implementation release after this clean commit.

## Root implementation release

Root registered and started this task through the supported helper at prepared
commit `3f41be386eead96eca79b6838b95b0eb4fddf265`; the native receipt is
generation 0, with original identities preserved. The independent design review
is accepted. This release supersedes the preparation-only hold above and admits
the exact config-only edit and disposable controls in this contract, using the
absolute source-bound companion and unchanged checker inputs. It admits no
Cargo/build, dependency, browser, website, network, native or Beads mutations by
the worker. Root retains review, integration and closure authority. Keep the
original performance failure under NU11 unresolved; every eventual candidate
requires its own applicable final-head hosted gates.
