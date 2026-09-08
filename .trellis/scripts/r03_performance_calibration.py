#!/usr/bin/env python3
"""Classify a pre-registered R03 hosted-performance calibration cohort.

The classifier is diagnostic-only. It cannot change the production no-slower
gate, and missing, mixed, or insufficient evidence is deliberately
``inconclusive`` rather than a passing result.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from statistics import median
from typing import Any


PLAN = {
    "diagnostic_only": True,
    "production_gate": "unchanged exact no-slower gate; every production report remains independently required",
    "required_observed_fingerprint": "EPYC 9V74 plus its immutable runner-image fingerprint",
    "independent_jobs_per_fingerprint": 3,
    "paired_samples_per_job": 16,
    "classifications": {
        "stable-slow": "three unique jobs, every strict gate fails, and pooled paired median is greater than zero",
        "stable-no-slower": "three unique jobs, every strict gate passes, and pooled paired median is at most zero",
        "inconclusive": "any missing, mixed, malformed, or insufficient cohort; never a passing result",
    },
    "non_acceptance_uses": "Cannot relax, widen, drop, relabel, or bypass the production performance gate.",
}


def classify_cohort(cohort: dict[str, Any]) -> dict[str, Any]:
    fingerprint = cohort.get("fingerprint")
    runs = cohort.get("runs")
    if not isinstance(fingerprint, str) or not isinstance(runs, list):
        raise ValueError("each cohort requires string fingerprint and runs list")

    job_ids: list[str] = []
    exits: list[int] = []
    deltas: list[float] = []
    valid = True
    for run in runs:
        if not isinstance(run, dict):
            valid = False
            continue
        job_id = run.get("job_id")
        gate_exit = run.get("gate_exit")
        samples = run.get("paired_deltas_ms")
        if not isinstance(job_id, str) or not isinstance(gate_exit, int) or not isinstance(samples, list):
            valid = False
            continue
        if len(samples) != PLAN["paired_samples_per_job"] or not all(isinstance(sample, (int, float)) for sample in samples):
            valid = False
            continue
        job_ids.append(job_id)
        exits.append(gate_exit)
        deltas.extend(float(sample) for sample in samples)

    independent = len(job_ids) == PLAN["independent_jobs_per_fingerprint"] and len(set(job_ids)) == len(job_ids)
    pooled_median = median(deltas) if deltas else None
    if not valid or not independent or pooled_median is None:
        classification = "inconclusive"
    elif all(exit_code == 0 for exit_code in exits) and pooled_median <= 0:
        classification = "stable-no-slower"
    elif all(exit_code != 0 for exit_code in exits) and pooled_median > 0:
        classification = "stable-slow"
    else:
        classification = "inconclusive"

    return {
        "fingerprint": fingerprint,
        "classification": classification,
        "independent_job_count": len(set(job_ids)),
        "pooled_paired_median_ms": pooled_median,
        "strict_gate_exits": exits,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--plan", action="store_true", help="print the pre-registered cohort contract")
    group.add_argument("--classify", type=Path, metavar="COHORTS_JSON", help="classify collected cohort JSON")
    arguments = parser.parse_args()

    if arguments.plan:
        print(json.dumps(PLAN, indent=2, sort_keys=True))
        return 0

    payload = json.loads(arguments.classify.read_text(encoding="utf-8"))
    cohorts = payload.get("cohorts")
    if not isinstance(cohorts, list):
        parser.error("COHORTS_JSON must contain a cohorts array")
    print(json.dumps({"diagnostic_only": True, "cohorts": [classify_cohort(cohort) for cohort in cohorts]}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
