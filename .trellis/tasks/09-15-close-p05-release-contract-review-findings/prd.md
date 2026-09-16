# Close P05 release-contract review findings

## Context

This is the focused corrective task for the independent retrospective review of
PR #340. The reviewed head was `34af5e35b9690469573bb20be05c8dc2c24884e6`,
merged as `cb1dafcc198c4198e2580fc23e90f6ece2827858`. The current integrated
base is `47ca7080c84dc6349bf01f48f1c26c481bc63497` (PR #345 merge commit).

PR #340's hosted checks passed, but GitHub recorded no reviews. The
retrospective review is not a pre-merge approval. This task must produce a
narrow current-master correction that receives independent review before its
integration.

## Goal

Make the P05 release evidence builder and retry path reject inconsistent
identity and prove the important negative/orchestration paths in tests. Keep
versioned preview/install compatibility intact. This task does not create a
release tag or publish assets.

## Requirements and acceptance

1. `build_release_receipt` requires full publication `tag_oid` and
   `commit_oid` values, validates them, and rejects either identity when it
   differs from the separately supplied receipt identity. The returned receipt
   must retain normalized, mutually consistent identities.
2. Receipt tests isolate install-proof rejection: use a matching `v0.4.0` tag
   and otherwise valid workflow, archive, checksum, and publication evidence;
   prove missing and wrong-version installed proof are rejected.
3. Exercise `publish_assets`, not only `plan_asset_uploads`, with mocked
   GitHub boundary calls and real decision logic. Prove matching assets cause
   no create/upload command, missing assets upload only the missing paths,
   and conflicts or missing remote digests stop before upload. Assert exact
   command arguments and returned action; no test release may be created.
4. Exercise the standalone `assert-version` command with an individually
   wrong `assura` binary and an individually wrong `assura-full` binary. Also
   reject an archive whose companion binaries differ in version while one is
   correct.
5. Add a Windows-only CI negative contract that intentionally makes the
   PowerShell `assert-version` invocation fail and asserts the GitHub Actions
   step outcome is `failure`. The validation job itself must pass only when
   that failure is observed.
6. Test `validate_release_receipt` against a valid built receipt and a
   table-driven set of missing/mismatched required schema fields. Keep the
   fixture aligned with the shared `.trellis/spec/assura/delivery-lifecycle.md`
   contract. The nested publication-OID predicate check is owned by the
   integrated P01-P04 lane because that lane changes `delivery_status.py`; do
   not edit that file in this task. The P05 coordinator must confirm that the
   shared-lane check is integrated and tested before overall closure.
7. Preserve the version-plus-full-source-SHA preview naming and prove the
   Unix and Windows installer smoke paths consume the produced versioned
   bundle/checksum. Existing workflow evidence may satisfy this requirement;
   add no duplicate installer implementation.

## Boundaries

- Do not tag, publish, deploy, change durable installer URLs, or mutate an
  installed Assura binary.
- Do not modify the root checkout's A04 research note or
  `09-14-resolve-versioned-build-artifact-generation` task.
- Do not inspect or edit the historical core-v2 diff. Coordinate any change to
  `delivery_status.py` with its existing integrated owner; this task does not
  edit that file.
- Do not claim runtime release-receipt or explicit/latest installer proof
  without a separately authorized release run. Report those as release holds.

## Verification

- `python3 tests/release_process_contract_tests.py` must pass with a nonzero
  test count and no unexpected output.
- `cargo test -p xtask versioned_ci_artifact_contract_matches_ci_workflow`
  and `cargo xtask release-readiness --format json` must pass on the final
  current-base candidate.
- Hosted checks must pass, including the Windows negative contract job and
  the supported delivery/process contract matrix.
- An independent pre-merge review must approve the exact candidate SHA; after
  integration, verify merged reachability and post-merge checks.
