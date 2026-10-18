# NU10 configured local-link validation

## Outcome and source

The existing policy now enables links for exactly the two selected maintained
documents while preserving their frontmatter/trailing-space bundle and the
surrounding naming/line/lifecycle/threshold/exclusion rules. No selected-doc
repair was needed. This is local slice evidence; independent implementation
review and applicable final hosted/platform/performance gates remain open.

- Runtime/source baseline: `eee4769fed3c5ecb9a4ab1688b83fcd5a26e0cf7` plus the
  deliberate config delta below. The disposable tree is a full tracked archive
  at that commit, with only the actual candidate config substituted before the
  policy baseline. Task reporting metadata was added afterward; source docs and
  all checker inputs remain unchanged.
- Tested and final policy config SHA-256:
  `ae5b025a44914f570594492a2f2648f07ff80b3f008fc9977daa093c7fc04b26`.
- Absolute companion:
  `/Users/nroth/.codex/worktrees/assura-w02-release-story/assura/target/debug/assura-full`.
- Before/after SHA-256:
  `980bf04aae00b9196c1d9ea8d84b4680bf3ddb6ac7210282a9b7a450848867de`;
  actual before/after version `assura 0.4.1`.
- Reviewed build source: `1f8085ceb251b11098e3a9ca6ed3c731e9692b24`.
  All 397 recorded checker inputs match current candidate bytes, including
  whole src/local crates, Cargo manifests/lock, .cargo and embedded inputs.
  No build or new toolchain proof is claimed; the reviewed source binding was
  reused. The unbound regular W02 executable and PATH fallback were not used.
- Scratch/TMPDIR/raw directories are caller-owned 0700 on APFS under the
  external run below. Admission snapshots showed 67% free memory, load below
  1.7 on 16 CPUs and 100 GiB available disk; concurrency was one CLI child.

## Actual checks and controls

Every check used the same absolute companion with `check --format json .`
from the disposable archive and inherited its private TMPDIR. There were no
warn/severity/cache overrides or network/build/dependency/browser operations.
The original-config and policy-config baselines both exit 0 with identical
16 low max_lines advisories, 2,040 files and 432 directories checked. No broken
local-link debt was found in the selected originals. Findings were compared
by stable path/rule/message/severity/blocking tuples, not output bytes/timing.

Thirteen full-check control cases ran once each. Both selected docs accepted
valid file, cross-file heading and line-range links. Their body-only missing
file/heading controls preserved exact original valid frontmatter; missing-file
messages were matched exactly, including path, target and line. The existing
frontmatter failure also produced its expected docs-lifecycle finding. Blank
line trailing whitespace retained low advisory severity and therefore exit 0.

| Case | Actual exit | Added findings | Observed rule / authored line |
| --- | ---: | ---: | --- |
| support-valid | 0 | 0 | No finding delta; line 162 |
| support-missing-file | 1 | 1 | markdown_link_target; line 162 |
| support-missing-heading | 1 | 1 | markdown_link_heading_anchor; line 162 |
| compatibility-valid | 0 | 0 | No finding delta; line 287 |
| compatibility-missing-file | 1 | 1 | markdown_link_target; line 287 |
| compatibility-missing-heading | 1 | 1 | markdown_link_heading_anchor; line 287 |
| retained-frontmatter | 1 | 2 | markdown_frontmatter, docs_lifecycle:active_goal_docs |
| retained-trailing-spaces | 0 | 1 | markdown_trailing_spaces; line 162 |
| unselected-maintained | 0 | 0 | No finding delta; line 156 |
| historical-archive | 0 | 0 | No finding delta; line 252 |
| excluded-generated | 0 | 0 | No finding delta |
| examples-external | 0 | 0 | No finding delta; line 162 |
| final-restored | 0 | 0 | No finding delta |

All cases retain the baseline findings and checked scope. Each fixture was
restored between cases. The final-restored full check exits 0 with exactly the
policy baseline's findings and 2,040/432 checked scope. Config and all four
mutated documentation fixtures match saved policy-baseline hashes, and the
owned generated control file is absent. Both candidate selected docs also
match their original Git bytes; no plant was made in primary or candidate docs.

There are **19 captured CLI invocations**: 15 full checks (5 intended exit-1
negatives, 10 exit-0 checks), 2 explanations and 2 version checks. These are
controlled CLI cases, not a Rust/full-PR test-suite claim. Receipt validation
required complete state, exact argv/cwd/TMPDIR/config+doc hashes as applicable,
integer observed exit, PID and ordered start/end timestamps, with no launch,
capture or signal failure marker. Normal complete receipts legitimately omit
child_live; it was not synthesized for missing or unknown observations.

## Preserved harness failure and bounded continuation

The initial supporting-evidence snippet attempted to import the standalone
controls script, whose top-level argv dispatch raised IndexError before any
supporting CLI child. Outer chunk `2911bd` exited 1. The script and error were
preserved; no correction or repeated control batch followed. Root reviewed
three read-only triages and admitted only direct captures for two selected-doc
explanations and version-after. Those three commands exited 0 with strict
receipts; no product-source or harness fix was made.

Actual outer sessions: original baseline `71042` terminal 0; policy baseline
`28031` terminal 0; selected controls `48790` terminal 0; scope/restoration
controls `93287` terminal 0. The admitted explanations and version completed
without ongoing sessions, chunks `21d2a6`, `931510`, `e089fe`, each exit 0.
Original version-before chunk `e018b7` also exited 0. Full raw child stdout,
stderr, exact argv, times and direct-child exits remain in per-run receipts.

## Supporting explain gap and held scope

Both actual explain outputs show directory frontmatter/trailing-space fields
and naming/line-limit context, but omit check_links and the file-specific
Markdown bundle. Their exit-0 results are supporting evidence, not proof of
this selector. Actual positive/negative controls establish the policy.

Separate future backlog candidate: expose the winning file-specific Markdown
bundle and check_links in explain. User impact: a maintainer cannot verify the
active link setting from the inspection surface. Acceptance: selected paths
show the winning pattern and complete effective Markdown fields; an unselected
path shows its unchanged policy, with CLI regression coverage. No explain
schema/parser/engine change belongs to this config-only slice.

No broad reference-candidate promotion, archive/generated repair, external URL
check, frontmatter-path policy, public plugin or global setting was introduced.
Root remains sole tracker/native/push/PR/merge writer. NU11 performance remains
unresolved; passing local controls do not authorize integration or gate bypass.

## Evidence and cleanup bounds

- [Raw invocation receipts](/Users/nroth/workspace/.assura-review-20261017/nu10/run-20261018T144327Z/raw).
- [Checker/runtime/resource binding before](/Users/nroth/workspace/.assura-review-20261017/nu10/run-20261018T144327Z/binding-before.json).
- [Strict summary and final binding](/Users/nroth/workspace/.assura-review-20261017/nu10/run-20261018T144327Z/validation-summary.json).
- [Case assertion rows](/Users/nroth/workspace/.assura-review-20261017/nu10/run-20261018T144327Z/case-results.jsonl).
- [Saved harness failure](/Users/nroth/workspace/.assura-review-20261017/nu10/run-20261018T144327Z/supporting-harness-failure.json).
- [Root continuation disposition](/Users/nroth/workspace/.assura-review-20261017/nu10/supporting-harness-root-disposition.json).

All 19 recorded direct CLI PIDs were independently observed absent after their
completed receipts. This establishes direct-child observations only; no general
descendant cleanup claim is made. Scratch, original archive, fixtures, raw
outputs and any TMPDIR entries are retained. No W02 runtime/global binary was
modified or archived. Its final matching hash is recorded above; this worker
has finished its trial runtime use and returns custody to root.

## Current-base refresh: 775eb057

Root admitted a normal merge of `origin/master` at
`775eb0577dd6f5ed33d36219096df71749b52993` into candidate 341d before final
review. The merge was retained intact without conflicts, reset or replay.
W02's inherited xtask/website changes are from that reviewed base, not authored
NU10 source changes. Actual tree/byte comparison confirms all 397 recorded
checker/build inputs, the tested config and both selected document bytes still
match the original trial. The existing 19 captured invocations therefore remain
applicable; the 13 controls were not repeated. The original 2,040/432 counts
remain historical trial observations, not counts claimed for the refreshed tree.

The new source-bound full check used the same absolute frozen companion and
`check --format json .`, with fresh private APFS TMPDIR/raw directories. It
exited 0, checking **2,046 files and 434 directories** with exactly the same
16 low advisory findings as the original trial. Strict receipt validation
passed. Actual outer session `53028`, terminal exit 0, completion chunk
`d521b9`; direct PID `44536`. No extra explanation command was run and no
visibility claim is inferred from prior explain output.

A refresh preflight first failed an unsupported hardcoded expectation of five
execution-parent children. Root's contract requires the four preserved NU10,
DX02, DX03 and NU16 entries; W02 belongs to the separate maturity parent. The
ordinary merge was correct. The original exit-1 assertion, raw three parent
blobs and unknown/not-launched product outcome of that preflight are preserved.
It is **not** an intentional negative or product test result. Root read three
independent diagnostic triages and admitted exact union plus uniqueness,
required-four IDs and nonchildren parity. The corrected assertions passed on
the retained merge without adding a fake child. The triages receive diagnosis
credit only; the new actual full-check receipt supplies the product observation.

- [Current-base byte/parent reuse proof](/Users/nroth/workspace/.assura-review-20261017/nu10/refresh-775-20261018T150147Z/proof-reuse.json).
- [Fresh full-check summary and raw receipt](/Users/nroth/workspace/.assura-review-20261017/nu10/refresh-775-20261018T150147Z/structure-summary.json).
- [Original failed assertion](/Users/nroth/workspace/.assura-review-20261017/nu10/refresh-775-20261018T150147Z/refresh-assertion-failure.json).
- [Root four-child continuation disposition](/Users/nroth/workspace/.assura-review-20261017/nu10/refresh-parent-root-disposition.json).

Root's historical gen0 registration at preparation tip/base was not rebound.
All tracker/native/GitHub writes and final review/integration remain root
ownership. No Cargo/build/install/browser work or control rerun occurred.
