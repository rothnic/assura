---
title: Agent initialization evaluation contract
status: active
---

# Agent initialization evaluation contract

`scripts/evaluate-agent-init.py` evaluates one already-completed initialization
run. It does not start an agent, choose a model, modify a source fixture, or
interpret instructions from the evaluated repository.

## Trusted inputs

The evaluator accepts an absolute project path, a JSON contract, an absolute
candidate Assura binary, and a private output path. `--public-output` optionally
writes a redacted aggregate result. Contract commands are fixture-owned
argument arrays with an explicit relative cwd. They are not read from project
text. The v1 contract records `schema`, `fixture_id`, `stack`, required and
forbidden paths, preservation hashes, positive and negative policy probes,
native commands, and required hook states.

The evaluator rejects a malformed contract before any probe: required v1 fields
must have the expected top-level type, command arguments must be nonempty string
arrays, and project paths/cwds must be relative and cannot contain `..`.

## Evidence model

Structural paths, preservation hashes, and declared hook paths are checked on
the completed project. Policy probes and native commands run only in a
disposable copy. Each result records the command, cwd, exit status, and captured
output. A required missing executable is `unavailable`, not a skipped pass. A
required test command that reports zero collected tests fails even when its
process exits zero. Commands have a 30-second timeout; a timeout is a failed
command with any partial stdout/stderr retained in the private result. A timed
out negative probe is not evidence that policy rejected its mutation. A missing
probe executable or cwd is `unavailable` evidence, not an evaluator crash.

A negative policy probe applies one trusted mutation at a time to a fresh
disposable copy and must be rejected by the candidate binary. A permissive configuration
that accepts the mutation is a critical failure. The source fixture is never
restored with a broad Git command because it is never mutated.

## Scope and claims

Allowed dimensions are `structure`, `policy`, `guidance`, `hooks`, `native`,
`preservation`, and `idempotence`. A selected subset is marked
`verification_scope: partial`, is `acceptance_eligible: false`, and cannot be
used for A01 or A07 acceptance. A full run with any critical failure or
unavailable required evidence is not acceptance-pass.

Every requested dimension receives a `pass`, `fail`, or `unavailable` state.
An omitted assertion is not a pass: a full fixture contract that has no
guidance, hook, or native evidence reports those dimensions as unavailable and
cannot be accepted. This evaluator's idempotence assertion proves that repeated
evaluation does not mutate the source fixture; initialization rerun/lifecycle
proof remains with the owning onboarding and hook cards.

The current hook-path assertion is only a contract input. A real lifecycle event
and preservation of existing hook ownership are A04 evidence. Likewise, this
evaluator supplies per-run contract evidence; it does not establish adoption,
agent success rates, or a release claim.

## Publication boundary

Raw command stdout and stderr remain only in the private `--output` artifact.
When `--public-output` is used, the evaluator removes fixture and command
identifiers, hashes, paths, cwd values, stdout, stderr, and named failure
identifiers; it retains only aggregate outcome fields. Publication workflows
must use that artifact rather than attempting to redact private diagnostic
evidence by hand.
