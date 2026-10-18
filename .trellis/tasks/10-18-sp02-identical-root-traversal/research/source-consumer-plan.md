# SP02 source and consumer preparation

## Current source shape

At base `da23fa7dab9600463cf234fceea060df2c8f5ccc`, `src/cli/check/test_relationship.rs::validate_test_relationships` calls `test_relationship_files(checked_path)` then `test_relationship_files(&self.project_root)` (lines 18-19). The returned vectors have separate semantic roles: source matching uses `source_files`; required test evidence and ignored-test scanning use `project_files`. At root scope both inputs are equal; at nested scope they are different and both must remain. Each helper walk filters excluded entries, converts to project-relative paths, sorts and deduplicates; walk/entry errors propagate.

The validator is invoked from `custom_constraints.rs` when test-relationship policies exist. The existing integration coverage is `tests/test_relationship_tests.rs`, including passing evidence/fixture coverage, missing tests plus ignored tests and fixture violations, and JSON CLI output. This is the natural adjacent test file if an admitted candidate needs coverage. Do not alter naming-only fast-path or config-selected size-policy consumers as part of SP02.

## Consumer cost to establish later

SP01's accepted source-count evidence reports for `rich-root`: two test-relationship walks, 10,472 entries / 10,256 files each, 5,000 reads / 150,000 bytes in its instrumented test-relationship counter; root fixture root/entries counters are 1/1. For `rich-nested`, it reports two walks with distinct roots: 5,287 entries / 5,178 files for the nested checked tree and 10,472 / 10,256 for the project root. These are instrumentation boundary counts, not OS syscalls or wall-time proof. The source evidence and raw provenance are in SP01's `research/p2-attribution-evidence.md`; use them as leads, verify exact identities before reuse, and leave those receipts immutable.

Candidate cost must be established with the real consumer on an admitted host and source/binary/policy/fixture identities bound together. Record ordinary `assura check --quiet` exact commands, outputs/exits and measurements. Any candidate and baseline comparison uses the same machine, fixture, configuration, build profile and execution mode, with two balanced order-reversed runs. Keep the complete raw samples and resource evidence.

## Candidate shape and meaningful controls

If the phase is later admitted and attribution says the repeated root work can affect the public row, keep the smallest local change in `test_relationship.rs`: identify the equal-root case once and reuse the already computed sorted/deduplicated collection. Do not broaden equality to canonicalized or ancestor/descendant paths without evidence and explicit review. Nested paths must still call the helper twice with their distinct scopes.

Use existing `tests/test_relationship_tests.rs` fixtures and CLI harness patterns. Prove root-scope parity on valid and missing-required-test outcomes, including exact findings/order/severity/exit; retain the nested-target control proving project-root evidence outside the subtree is still recognized. Preserve ignored-test and fixture-family findings as relevant to those roles. Keep a naming-only config as a negative control and a configured size-policy control to detect spillover. Exercise missing/unreadable/symlink cases only in the applicable focused tier, preserving the currently declared behavior and contextual errors. Avoid a helper-literal unit test that merely restates the implementation.

## Admission and stop rules

This plan does not authorize source changes, tests, build/profile/timing work, SSH, or installation. The coordinator must admit a separate next phase after fresh resource evidence; NU03 owns the compiler lane. Prior unsafe VPS load/disk evidence does not authorize remote work. Use verified disk-backed scratch, bounded jobs/processes, and verify teardown. Do not claim improvement from walk counts. Keep only if both public timing rows each clear 5% and 0.75 ms, exact quiet improves at least 3%, spillover is at most 5% or 0.25 ms, and subsequent strict no-slower/native/warm gates pass. Otherwise preserve the adverse evidence and defer.
