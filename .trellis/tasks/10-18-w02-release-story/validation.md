# W02 source preparation and evidence

This is an intermediate implementation packet. The child remains in progress;
production rendering, browser behavior, independent final review and PR gates
have not been completed. Root owns admission and integration.

## Source and ownership

The contract is commit `ba2c85ef1ebbeabee1e407b3d4d1f7fabe902884` on
`fix/w02-release-story`, based on `bcb94802eac9d834d2313b7a398148a272d7cc4d`.
The candidate checkout is
`/Users/nroth/.codex/worktrees/assura-w02-release-story/assura`.
Only the PRD's owned W02 surfaces changed. No new dependency, policy threshold,
historical performance dataset or unrelated product file changed.

Raw receipts are external under
`/Users/nroth/workspace/.assura-review-20261017/w02-current/`.
Each execution binding records its actual inputs, cwd, argv and exits; later
source changes are not retroactively covered by an earlier run.

## Checks completed

| Check | Observed result and qualification | Receipt |
| --- | --- | --- |
| Frozen offline website dependency preparation | Exit 0; 355 packages reused, zero downloads; lock hash unchanged. | `dependency-install.json`, `dependency-install.raw.log` |
| Published/local-version regression against unchanged validator | Expected assertion failure: one executed test rejects treating local package 99.0.0 as published capability proof. This is the purposeful behavior RED. | `validator-red.binding.json`, `validator-red.raw.log`, `validator-red.exit.json` |
| Corrected release-validator controls | Eight tests passed, zero ignored. This predates the later repair/provenance additions. | `validator-green.binding.json`, `validator-green.raw.log`, `validator-green.exit.json` |
| Initial browser discovery | JSON import attribute error; zero discovered/executed tests. This was a pre-execution failure, not behavior RED. Initial PATH/leaf identity evidence was incomplete and remains so. | `browser-list.json`, `browser-list.raw.log`, `browser-list-inputs-frozen.json` |
| Corrected pinned browser discovery | Exit 0; 152 tests discovered in one file, zero behavior tests executed. The actual Playwright leaf Node argv/cwd/executable/hash and launch PATH/NODE_OPTIONS were recorded. No server/browser started. | `browser-list-corrected.binding.json`, `browser-list-corrected.raw.log`, `browser-list-corrected.exit.json`, `browser-list-node-launches.jsonl` |
| Initial provenance/repair unit attempt | Duplicate helper E0428; compilation stopped before tests. No behavior RED or test pass credit; historical bare-cargo receipt does not identify the exact compiler version. Three read-only triages supported deleting only the later identical helper. | `source-and-repair-unit.binding.json`, `source-and-repair-unit-0.raw.log`, `source-and-repair-unit-0.exit.json`, `duplicate-helper-readonly.json` |
| Corrected provenance regression | One test passed, zero ignored: generated-data-only changes retain identity; staged/unstaged/untracked relevant inputs are rejected; committed source/config/fixture changes change identity. | `source-and-repair-corrected.binding.json`, `source-and-repair-corrected-0.raw.log`, `source-and-repair-corrected-0.exit.json` |
| Corrected repair controls | Three tests passed, zero ignored: reject changed policy/file bytes, an unperformed rename, and wrong/missing process exits. The actual generated repair has not run yet. | `source-and-repair-corrected-1.raw.log`, `source-and-repair-corrected-1.exit.json` |
| Cheap source checks | Rustfmt module check and diff whitespace check exit 0; both modified manifests parse as JSON. | `cheap-checks.json`, `cheap-check-2.raw.log`, `cheap-check-3.raw.log` |
| Direct candidate structure triage | Installed `/usr/local/bin/assura` 0.4.1 checked this candidate's absolute root/config: exit 0, 2004 files, 423 directories, 16 nonblocking max-lines findings. This is not candidate-built final proof. | `cheap-checks.json`, `cheap-check-0.raw.log`, `cheap-check-1.raw.log` |
| Label correction and scoped rereview | Seven release labels across three product docs now match v0.4.1 tag/manifest evidence; all other bytes and caveats preserved. Commit `4ecc5b06469ce7b33a5eab0f6eee3a1de33bb596` passed scoped independent rereview. | `release-label-correction.json`, `release-label-independent-rereview-4ecc5b06.md` |
| One admitted canonical generation | Exit 0 at clean `4ecc5b06`; validates 20 marketed mappings and 54 documentation YAML examples, writes the five intended CLI artifacts, preserves every protected performance byte. | `generator.binding.json`, `generator.raw.log`, `generator.exit.json` |
| Generated JSON and candidate-built structure | Five JSON artifacts parse and identify input revision `4ecc5b06`; actual repair fields agree with exits/preservation. Candidate-built `assura-full` checked the absolute candidate root: exit 0, 2005 files, 423 directories, 16 nonblocking max-lines findings. Diff check passes. | `generator-cheap-validation.json`, `generated-cheap-check-0.raw.log`, `generated-cheap-check-1.raw.log` |

Corrected Rust tests explicitly pin Cargo and rustc 1.94.1 from the existing
stable toolchain, their hashes, offline/locked arguments, jobs=2,
candidate-local target, private disk-backed TMPDIR and current source hashes.
The installed structure binary's hash and absolute candidate binding are in
its receipt. The shared precommit launcher path alone does not establish which
checkout it checks; no wrong-checkout bug is claimed.

## Generated source evidence and remaining gates

The single admitted canonical generation ran at clean commit
`4ecc5b06469ce7b33a5eab0f6eee3a1de33bb596`, using absolute Cargo/rustc 1.94.1,
pinned inherited child PATH, offline dependencies, jobs=2, candidate-local
target and private disk-backed `/Users/nroth/workspace/.w02-gen.1atphaxs`.
The Darwin socket bind/close budget probe passed; the managed daemon default
is TCP. The generator completed, was reaped, and left no owned process-group
or candidate-binary rows; its scratch directory was empty at teardown.

The command built source `assura-full` SHA-256
`111dc66cbd69ac87f58f055d16c3457aa53828bd1ee3111441e652690363f929`.
Actual generating HEAD and tracked-input revision are recorded separately;
both were `4ecc5b06` for this run. No released-artifact behavior is inferred
from that binary's Cargo version. The five generated CLI artifacts are:

- `website/src/data/review-demo.json`
- `website/src/data/check-demo.json`
- `website/src/data/onboarding-demo.json`
- `website/src/data/intelligence-demo.json`
- `website/src/data/policy-demo.json`

The entire generated delta is 44 insertions and one deletion across those five
files: five input-revision fields and actual recorded naming repair. The repair
has before exit 1, an unrepaired exit 1, then after exit 0 with no violations;
configuration and file bytes are preserved by the generator's snapshot check.
`performance-summary.json` was rewritten byte-identically; all protected
raw/history files also retain their original hashes. Before/after files and
hashes are preserved externally. Per-artifact `source_revision` identifies
the committed CLI/generator/fixture inputs, including configuration examples
and relevant build/marketing inputs; it does not assert a literal generating
HEAD after data-only commits, nor prove published-binary behavior.

The generator-dependent unit test has not yet executed. Remaining admitted
work must include applicable current-head generator/docs/released checks,
production build, browser behavior and responsive/theme proof, applicable
Rust and hosted gates, independent Luna review/rereview, and root-owned
integration/outcome verification. Historical 158-test evidence remains case
rationale, not current proof.

## Responsive qualification correction

At `1e8e48f0`, all 13 focused generator tests, strict `--check --released`,
and the production build passed. The first production browser matrix stopped
at a real 320px vertical-order failure: 6 passed, 1 failed, 1 interrupted
(768px), 144 unrun, zero screenshots. The outside-test error was the configured
maximum-failure stop. Complete traces, exact production/server/tool binding
and process/port teardown remain under external `production-checkpoint/`.
That attempt establishes neither a full browser pass nor visual acceptance.

Three read-only triages and root adjudication supported removing only the two
responsive homepage terminal `order: -1` overrides. Source commit
`37601803375c758396224b09c835d7caf7596bfb` removes those otherwise-empty rules;
all other declarations, qualification text and tests remain unchanged.
One admitted canonical generation at that clean source passed. Only the five
CLI JSON `source_revision` values advanced to `37601803`; all CLI output/repair
payloads and protected performance/history bytes are unchanged. Its exact
receipts and before/after copies are under external `mobile-order-corrected/`.
Changed-source focused/strict/build checks and the complete production browser
matrix remain pending at this data/evidence commit.

## Narrow performance metadata correction

At `fb2fa94f`, focused 13/13, strict released check and production build passed;
the four prior qualification-order cases also passed. The browser matrix then
stopped at a distinct 320px third-cell overflow of 5px: 50 passed, 1 failed,
1 interrupted, 100 unrun. Its 52 captures, selected visual inspection and full
teardown remain under external `mobile-order-corrected/production-checkpoint/`.
Three read-only diagnoses supported wrapping the metadata children at the
existing narrow breakpoint. Source commit
`0a1fdbac72f6ea50259d614019929617b884960f` adds only `flex-wrap: wrap` to that
rule, preserving all content, assertions and desktop declarations.

One canonical generation at clean `0a1fdbac` passed; only the five CLI JSON
source revisions changed. Their CLI/repair payloads and all protected
performance/history bytes remain identical. Receipts are under external
`performance-wrap-corrected/`. Changed-source gates and the complete unchanged
152-case matrix are pending at this data/evidence commit. Root admitted
`maxFailures=0` with workers=2 and retries=0 for the next run, to collect the
remaining failure frontier without converting a nonzero result into success.

## Complete frontier correction

At `bb8ab119`, all 152 tests executed once: 143 passed, 9 failed, zero
skipped/interrupted/flaky/global errors. Six desktop proof-cell cases reported
44px overflow; three mobile cases reported title/scope row misalignment.
All 144 captures, nine traces and full process/port teardown remain under
external `performance-wrap-corrected/production-checkpoint/`.

Three grouped read-only diagnoses supported the combined CSS correction in
`a1f1a125852c8c74e021e6e7b7b3c0b9798778ab`: bound the desktop metadata track
and allow wrapping; explicitly top-align mobile title/scope columns with
full-width links/caveat and compact band spacing. All content and assertions
remain intact. One canonical generation passed; only five source revisions
changed, with identical CLI/repair payloads and protected performance bytes.
New receipts are under external `full-frontier-corrected/`. Changed-source
focused/strict/build and complete 152-case production proof remain pending at
this data/evidence commit; the previous 143/9 result is not a passing gate.

## Remaining mobile height correction

At `7db4f51a`, the complete matrix ran 152 cases: 151 passed, one failed,
zero skipped/interrupted/flaky/global errors. The sole failure measured the
320px proof height at 238.265625px against the unchanged strict `<230` cap;
all desktop containment and mobile alignment cases passed. Its 144 captures,
one trace and teardown remain under external `full-frontier-corrected/`.
After three read-only height diagnoses, source commit
`0dbd1b9a1c543ea70b76b9de476c4d3e4786c77b` changed only existing mobile metric
cell vertical padding to 0.5rem, preserving the later metadata override and all
text/type/assertions. One canonical generation passed; five source revisions
changed with identical CLI/repair and protected performance bytes. Receipts
are under external `height-padding-corrected/`; post-fix height and complete
changed-source validation are pending at this data/evidence commit.

## Passing production checkpoint and current main

At clean `01ba6a107a81fe993b39b42b54f675893a521aa4`, the changed-source
checkpoint passed 13 focused tests, strict released-data validation, the pinned
production build and all 152 unchanged browser cases without retries, skips,
interruption, flakes or global errors. Eight actual captures were inspected;
both metrics, all metadata and full caveats remain visible. All 144 captures
were hashed, protected performance/history bytes stayed unchanged, all ten
observed owned processes exited and the preview port was released. The strict
height assertion passed; no exact post-fix height measurement is claimed.
Independent review found no blocking source/correction/proof findings at that
head. Prior failed checkpoints remain preserved.

Standard nonforce merge `12bdbb84e1d88315d9d582f29650953baa4d792c` integrated
main `74d9917dd39e599bbd43c3a5760b71644b2cf91b` without conflicts, including
NU06 recovery documentation, NU07 count correctness and DX01 local preflight.
One canonical generation at the clean merged source passed. Only five generated
source-revision fields changed; their payloads and protected performance/history
bytes remain identical. Evidence is external under
`w02-current/current-base-74d9917d/`. Current-base full PR and affected production
validation, final independent review and hosted gates remain pending.
