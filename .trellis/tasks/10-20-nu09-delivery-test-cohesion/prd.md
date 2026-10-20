# NU09: Cohesive delivery contract tests

## Goal

Bead ae-whr.5 owns the live queue. Reduce navigation and agent context cost in
`tests/agent_delivery_contract_tests.py` by extracting cohesive behavior groups
without changing delivery behavior, coverage, fixtures or command contracts.

## Authority

Nick authorized ready Beads implementation, independent GPT-6 Luna review,
corrections and root-owned gated merge. Root owns native/provider/Beads writes,
push, PR, merge and closure. The implementation worker owns only the test files
and committed task activation. No release, deployment, policy, API, timeout,
source delivery protocol or infrastructure change.

## Contract

Use the independently reviewed proposal and structure/spec review in external
nu09/trellis-test-partition-{proposal,design-review}-b5-20261020.md. Flat sibling
modules fit existing tests/ policy; no new directory or .assura/config.yml edit.
Keep the existing runner and nine concrete TestCase names/selectors, using plain
behavior mixins and one explicit shared-fixture module. Bootstrap sys.path before
imports and import each moved method's globals explicitly. Do not duplicate
fixtures, change assertions, use wildcard imports, introduce test discovery of
mixins, or split solely to meet a count. Keep the original direct entrypoint,
NonEmptyTestRunner and self-referential empty-discovery negative control.

## Acceptance

Preserve the exact original nonempty set of 113 (concrete class, method) test
identities, original method bodies/assertions and full fixture semantics, with
one execution each. Record baseline membership and compare the moved source and
runtime loader identities; duplicate/omitted test identities must fail the
verification. Run the full original Python entrypoint and representative class
and method selectors; retain the existing broken-discovery negative and prove
its expected rejection. Verify each new module fits existing structure/line
policy without changing ceilings or suppressing failures. Preserve provider
errors, no-write assertions, shared-fixture ownership, direct task CLI boundaries
and Windows-specific cases. New modules need the reviewed cohesive boundaries,
not arbitrary equal-sized chunks. Independent Luna review/corrections, affected
local and three-OS hosted gates, root current-base merge, actual postmerge result,
native closure and clean owned worktree/ref disposition are required.

## Owned files

`tests/agent_delivery_contract_tests.py`, new flat
`tests/agent_delivery_contract_support.py` and
`tests/agent_delivery_contract_*_cases.py`, plus this task's activation/context.
No production source, Cargo/locks, CI/workflow, specs, generic test framework or
unrelated code edits. Request root judgment on a demonstrated scope exception.
