#!/usr/bin/env python3
"""Expose the reviewed, non-acceptance R03 Callgrind collection contract.

This command deliberately has no remote-execution mode. It is a stable dry-run
surface for reviewing the finite collection contract before a separately
reviewed runner is permitted to invoke any retained executable.
"""

from __future__ import annotations

import argparse
import json


PLAN = {
    "diagnostic_only": True,
    "timeout_seconds": 30,
    "timeout_policy": "timeout --signal=KILL 30s; retain failure and reject partial output",
    "collection_sequence": [
        "baseline",
        "candidate",
        "candidate",
        "baseline",
        "baseline",
        "candidate",
    ],
    "collection_samples": 6,
    "health_controls": [
        "verify isolated launcher and Callgrind engine hashes",
        "valgrind none true exits zero",
        "callgrind true emits a numeric summary",
        "shell-loop-1000-no-io summary is strictly greater than true",
        "verify retained static-PIE binaries and valid fixture manifest",
        "probe each retained static-PIE executable before collection",
    ],
    "positive_control": "shell-loop-1000-no-io",
    "collection_requirements": [
        "fixed absolute executable, argv, cwd, and scrubbed environment",
        "unique Callgrind output and log path per invocation",
        "expected normalized diagnostic digest and numeric summary",
        "record executable, fixture, tool, and output identities",
    ],
    "inconclusive_if": [
        "any health control, timeout, identity, exit, digest, or summary check fails",
        "candidate counts do not all exceed baseline counts",
    ],
    "non_acceptance_uses": "Cannot pass or alter the hosted no-slower gate, relax a tolerance, or restore a rejected candidate.",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plan",
        action="store_true",
        help="print the reviewed collection contract as JSON",
    )
    arguments = parser.parse_args()
    if not arguments.plan:
        parser.error("only --plan is available; collection requires a separately reviewed runner")
    print(json.dumps(PLAN, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
