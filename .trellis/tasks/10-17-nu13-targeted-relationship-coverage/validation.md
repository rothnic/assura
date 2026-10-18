# NU13 validation checkpoint

## Candidate and outcome

Validated source: `6e49d8c1a9143058802ab8beab6321eb1897037e`.
Integration base: `f3540d4a11d36afa43f69c96fd7f1c233599822a`, freshly
fetched from `origin/master`. Branch: `fix/nu13-targeted-relationships`.

Selected captured file and directory producers now participate in ordinary
scoped relationship checks. Unrelated producers stay outside the requested
scope; external providers, exclusions, explicit LS-Lint semantics and prepared
full fallback retain their contract. Default non-Git caches use the actual
policy root. Existing cache-only markers no longer shadow ancestor policy;
invalid or inaccessible nearest policies still fail instead of borrowing a
parent policy. No cache deletion, API/schema/dependency change or speedup claim.

## Behavior and gates

| Proof | Result |
| --- | --- |
| Selected-producer tests-only baseline `13ea0fba` | 2 pass, 5 expected failures; exit101 |
| Focused notation suite after producer fix `43657628` | 24 pass; exit0 |
| Cache/discovery baseline `f433511e` | discovery4/2, cache1/1, CLI0/1 pass/fail; exit101 each |
| Cache/discovery fix `de8bc53c` | discovery6, cache2, CLI4 pass; no ignored tests |
| Invalid-nearest-policy baseline `47ba22fd` | 1 expected failure; exit101 |
| Invalid-nearest-policy fix `822f87e0` | discovery7 pass; exit0 |
| Final `cargo +stable xtask pr` | exit0; 1,450 passed, 0 failed, 1 intentional manual-performance ignore |
| Included target-state, evidence, format, Clippy and docs | pass; Clippy all-targets/all-features with warnings denied; website48 pages |
| Public launcher CLI journey, final source | 11 calls pass; exits1,1,0,0,0,0,1,0,1,1,0; exact project/config identity retained |
| Existing poisoned cache directory retained | scoped producer exits1 with relationship finding; cache still exists |
| Inaccessible nearest policy, non-root Linux user | exit4; no permissive-parent fallback |
| Release markers independently removed | exit1 each for missing candidate release link, live verification, beta status; restored input exits0 |
| Candidate self-check | exit0; 1,912 files, 15 low advisories, zero blocking findings |

The one ignored fixture is the explicitly manual performance audit in
`tests/ls_lint_parity_regression_tests.rs`. Empty filtered test invocations
earn no coverage credit. Rich relationship policies bypass the naming-only
cache: the CLI journey proves wrapper correctness, not incremental coverage.

Final PR gate elapsed2:47.12, peak RSS1,117,564KiB. This is validation timing,
not a benchmark. Two new line-limit advisories are disclosed: config discovery
with its regression tests (530/500 lines), and the native-comparator test module
(1,002/1,000). Thirteen other advisories are inherited. No limits were changed.

## Findings retained and resolved

- Native comparator initially failed22 tests because npm climbed outside its
  scratch install directory. Explicit `--prefix` restores the same pinned
  native LS-Lint2.3.1 comparator; 52 native tests pass. Parent manifest/lock
  hashes stay unchanged across the corrected run; pre-existing parent files
  have no earlier ownership baseline and were preserved.
- The real CLI policy-relax/restore journey originally exited4 on its ninth
  call after creating a nested cache marker. Ordinary/config-explicit controls
  isolated discovery; the actual default-cache path and discovery were fixed.
- Independent review rejected `Path::exists` as the absence test: dangling
  links and metadata errors must retain nearest-policy authority. A red/green
  regression and live permission control substantiate the correction.
- A full test run failed one Unix-socket test with a110-byte socket path.
  Three Luna reviewers confirmed the Linux pathname limit. The same source
  and assertions pass under short private disk-backed runtime scratch.
- The next full run passed all tests but failed two stale release literals.
  History and Luna review confirmed `c5cd79c6` had correctly documented actual
  publication. The checker now requires the stronger current publication,
  verification and beta facts; negative controls fail when each is removed.

## Environment, evidence and review

Remote source: `/home/ubuntu/data/projects/assura-nu13-20261017.TLlZ9a/source`
on `vps-9cb01956` through SSH alias `vps`. Ubuntu, Rust stable1.93.0,
Node22.22.1; frozen website install selected pinned pnpm10.25.0 (shell10.29.3).
CI uses Node24. One compiler lane, jobs2, test threads2, incremental/debug info
disabled; private runtime `TMPDIR=/var/tmp/n13.wQHgci` on ext4. Resource and
ownership checks admitted each batch; no unrelated process/cache cleanup.

Raw commands, exits, failures, final binary hashes and source identities are
preserved in `/Users/nroth/workspace/.assura-review-20261017/nu13-vps-logs/`.
The adjacent `evidence-manifest.json` binds the retained files by SHA-256.
Final CLI binary identities also live in the raw `cli-probe.json` there.

Independent `/root/nu13_luna_final_review` (GPT-6 Luna) found no blocking source
findings at `822f87e0` and accepted the marker delta through `6e49d8c1`.
Vendor/environment/adversarial reviews remain advisory evidence; coordinator
`/root` owns finding dispositions and integration judgment. Later task-only
metadata is reviewed separately and does not substitute for source proof.

## Integration boundary

Engineering behavior and local gates are proven. Required final-candidate
hosted OS/features/install/performance checks, integration, post-merge outcome
and native delivery closure remain separate obligations. The task stays
in_progress; neither an agent review nor this record asserts Nick's human
acceptance, release authority or a delivered outcome. P05 and its receipt remain
unchanged. The reviewed backlog remains prepared for Beads, without import.
