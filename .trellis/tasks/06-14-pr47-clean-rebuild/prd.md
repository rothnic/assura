# Rebuild PR #47 cleanly from current master

## Goal

Create a narrow replacement for PR #47 from current `master`, preserving current notation/spec source truth and porting only still-needed code fixes.

## Scope

- Keep: stale config fingerprint invalidation, compatible exclusion/path review fixes that still apply on current master.
- Preserve: current notation/spec docs already on master.
- Drop: stale performance stack, stale Trellis churn, and any OBE carryover from the old branch lineage.

## Acceptance

- Fresh branch from current `origin/master`
- Only current needed fixes are present
- Targeted failing test passes
- Full relevant suite passes
- No stale branch carryover added
