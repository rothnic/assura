# W02 source preparation and evidence

This is an intermediate source packet. The child remains in progress; generated
CLI data, production rendering, browser behavior, independent final review and
PR gates have not been completed. Root owns admission and integration.

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

Corrected Rust tests explicitly pin Cargo and rustc 1.94.1 from the existing
stable toolchain, their hashes, offline/locked arguments, jobs=2,
candidate-local target, private disk-backed TMPDIR and current source hashes.
The installed structure binary's hash and absolute candidate binding are in
its receipt. The shared precommit launcher path alone does not establish which
checkout it checks; no wrong-checkout bug is claimed.

## Pending generation and final gates

The current checked-in demo JSON predates this adaptation. It lacks the new
repair and source revision fields that the prepared UI expects. It has not
been replaced by hand, and no production build or behavior pass is claimed.

After source is committed and root admits the run, request the existing
canonical `cargo xtask website-demo-data` command from this checkout, with
the pinned Rust/Node wrappers, jobs=2, offline dependencies, candidate-local
target and a private short disk-backed TMPDIR. It builds the current
`assura-full` and generates exactly these CLI artifacts:

- `website/src/data/review-demo.json`
- `website/src/data/check-demo.json`
- `website/src/data/onboarding-demo.json`
- `website/src/data/intelligence-demo.json`
- `website/src/data/policy-demo.json`

The existing command also writes `performance-summary.json`; its producer is
unchanged, so require its pre/post bytes to match. Preserve the raw/history
performance datasets. Record actual generating HEAD/toolchain/binary and raw
outputs externally. Per-artifact `source_revision` identifies the committed
CLI/generator/fixture inputs, including configuration examples and relevant
build/marketing inputs; it does not assert a literal generating HEAD after
data-only commits, nor prove published-binary behavior.

Then run the generator-dependent actual repair test and applicable current-head
docs/release checks, production build, browser behavior and responsive/theme
proof, applicable Rust and hosted gates, independent Luna review/rereview,
and root-owned integration/outcome verification. Historical 158-test evidence
remains case rationale, not current proof.
