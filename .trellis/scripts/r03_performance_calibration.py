#!/usr/bin/env python3
"""Classify a pre-registered R03 hosted-performance calibration cohort.

The classifier is diagnostic-only. It cannot change the production no-slower
gate, and missing, mixed, or insufficient evidence is deliberately
``inconclusive`` rather than a passing result.
"""

from __future__ import annotations

import argparse
from collections import defaultdict
import json
import math
import re
from pathlib import Path
from statistics import median
from typing import Any


PLAN = {
    "diagnostic_only": True,
    "production_gate": "unchanged exact no-slower gate; every production report remains independently required",
    "required_observed_cpu_model": "AMD EPYC 9V74",
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
    run_fingerprints: list[str] = []
    source_shas: list[str] = []
    binary_hashes: list[str] = []
    report_hashes: list[str] = []
    exits: list[int] = []
    deltas: list[float] = []
    valid = True
    for run in runs:
        if not isinstance(run, dict):
            valid = False
            continue
        job_id = run.get("job_id")
        cpu_model = run.get("cpu_model")
        runner_image_name = run.get("runner_image_name")
        runner_image_version = run.get("runner_image_version")
        source_sha = run.get("source_sha")
        binary_sha = run.get("assura_binary_sha256")
        report_sha = run.get("report_sha256")
        report_exit = run.get("report_exit")
        gate_exit = run.get("gate_exit")
        samples = run.get("paired_deltas_ms")
        if not all(isinstance(value, str) for value in (job_id, cpu_model, runner_image_name, runner_image_version, source_sha, binary_sha, report_sha)) or not isinstance(report_exit, int) or not isinstance(gate_exit, int) or not isinstance(samples, list):
            valid = False
            continue
        if not runner_image_name or not runner_image_version or "latest" in runner_image_name.lower() or "latest" in runner_image_version.lower():
            valid = False
            continue
        if not re.fullmatch(r"[0-9a-f]{40}", source_sha) or not re.fullmatch(r"[0-9a-f]{64}", binary_sha) or not re.fullmatch(r"[0-9a-f]{64}", report_sha):
            valid = False
            continue
        if report_exit != 0 or len(samples) != PLAN["paired_samples_per_job"] or not all(isinstance(sample, (int, float)) and math.isfinite(sample) for sample in samples):
            valid = False
            continue
        job_ids.append(job_id)
        run_fingerprints.append(f"{cpu_model}|{runner_image_name}@{runner_image_version}")
        source_shas.append(source_sha)
        binary_hashes.append(binary_sha)
        report_hashes.append(report_sha)
        exits.append(gate_exit)
        deltas.extend(float(sample) for sample in samples)

    independent = len(job_ids) == PLAN["independent_jobs_per_fingerprint"] and len(set(job_ids)) == len(job_ids)
    identity_matches = len(set(run_fingerprints)) == 1 and run_fingerprints == [fingerprint] * len(run_fingerprints) and len(set(source_shas)) == 1 and len(set(binary_hashes)) == 1 and len(set(report_hashes)) == len(report_hashes)
    provenance_complete = valid and independent and identity_matches
    pooled_median = median(deltas) if deltas else None
    if not provenance_complete or pooled_median is None:
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
        "provenance_complete": provenance_complete,
    }


def collected_fingerprint(run: Any, index: int) -> str:
    if isinstance(run, dict):
        cpu_model = run.get("cpu_model")
        image_name = run.get("runner_image_name")
        image_version = run.get("runner_image_version")
        if all(isinstance(value, str) and value for value in (cpu_model, image_name, image_version)):
            return f"{cpu_model}|{image_name}@{image_version}"
    return f"unproven:artifact-{index}"


def collect_cohorts(artifact_root: Path) -> dict[str, Any]:
    grouped: dict[str, list[Any]] = defaultdict(list)
    for index, run_path in enumerate(sorted(artifact_root.rglob("run.json")), start=1):
        try:
            run = json.loads(run_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            run = {"artifact_error": str(run_path)}
        grouped[collected_fingerprint(run, index)].append(run)
    if not grouped:
        grouped["unproven:no-run-artifacts"] = []
    return {
        "cohorts": [
            {"fingerprint": fingerprint, "runs": runs}
            for fingerprint, runs in sorted(grouped.items())
        ]
    }


def classify_payload(payload: dict[str, Any]) -> dict[str, Any]:
    cohorts = payload.get("cohorts")
    if not isinstance(cohorts, list):
        raise ValueError("COHORTS_JSON must contain a cohorts array")
    classified = [classify_cohort(cohort) for cohort in cohorts]
    required_cpu = PLAN["required_observed_cpu_model"]
    required_present = any(item["provenance_complete"] and item["fingerprint"].split("|", 1)[0] == required_cpu for item in classified)
    return {
        "diagnostic_only": True,
        "required_observed_cpu_model": {
            "value": required_cpu,
            "status": "present" if required_present else "unproven",
        },
        "cohorts": classified,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--plan", action="store_true", help="print the pre-registered cohort contract")
    group.add_argument("--classify", type=Path, metavar="COHORTS_JSON", help="classify collected cohort JSON")
    group.add_argument("--collect", type=Path, metavar="ARTIFACT_ROOT", help="assemble and classify downloaded run artifacts")
    arguments = parser.parse_args()

    if arguments.plan:
        print(json.dumps(PLAN, indent=2, sort_keys=True))
        return 0

    try:
        payload = (
            collect_cohorts(arguments.collect)
            if arguments.collect is not None
            else json.loads(arguments.classify.read_text(encoding="utf-8"))
        )
        result = classify_payload(payload)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        parser.error(str(error))
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
