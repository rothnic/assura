#!/usr/bin/env python3
"""Validate A07 packet metadata before an isolated protocol review.

The validator deliberately reads only packet metadata and public evaluator
summaries. It does not open child event streams, private contracts, fixtures or
raw evaluator output. A non-zero result means the packet is not review-ready.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any


DIMENSIONS = {
    "guidance",
    "structure",
    "preservation",
    "hooks",
    "policy",
    "native",
    "idempotence",
}


def load(root: Path, relative: str) -> dict[str, Any]:
    path = root / relative
    try:
        value = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise ValueError(f"missing artifact: {relative}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid JSON: {relative}: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"artifact is not an object: {relative}")
    return value


def require(errors: list[str], condition: bool, message: str) -> None:
    if not condition:
        errors.append(message)


def identity_matches(candidate: dict[str, Any], expected: dict[str, str]) -> bool:
    """Return whether a packet identity contains every expected digest exactly."""

    return all(candidate.get(key) == value for key, value in expected.items())


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path, help="private packet root")
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--tree-sha", required=True)
    parser.add_argument("--binary-sha", required=True)
    parser.add_argument("--shim-sha", required=True)
    parser.add_argument("--contract-sha", required=True)
    parser.add_argument("--evaluator-contract-sha", required=True)
    parser.add_argument("--prompt-sha", required=True)
    parser.add_argument("--toolchain", required=True)
    parser.add_argument(
        "--protocol-status",
        choices=("PENDING", "PASS", "PASS_NO_CREDIT"),
        default="PENDING",
    )
    args = parser.parse_args()
    root = args.root.resolve()
    errors: list[str] = []

    files = {
        "freeze": "candidate-freeze-2026-09-11-current-dc031.json",
        "identity": "controls/dc031527-identity-final.json",
        "construction": "holdouts/construction-current-dc031-r1.json",
        "binding": "holdouts/current-binding-2026-09-11-current-dc031-r1.json",
        "second": "holdouts/rebind-second-readonly-2026-09-11-current-dc031-r1.json",
        "manifest": "screening/manifest-2026-09-11-current-dc031-r1.json",
        "mapping": "screening/mapping-2026-09-11-current-dc031-r1.json",
        "invariants": "screening/invariants-2026-09-11-current-dc031.json",
        "canary": "screening/canary-2026-09-11-current-dc031.json",
        "receipt_a": "runs/receipt-a.json",
        "receipt_b": "runs/receipt-b.json",
        "evaluation_a": "runs/evaluation-a-public.json",
        "evaluation_b": "runs/evaluation-b-public.json",
    }
    artifacts: dict[str, dict[str, Any]] = {}
    for name, relative in files.items():
        try:
            artifacts[name] = load(root, relative)
        except ValueError as exc:
            errors.append(str(exc))

    identity = {
        "source_sha": args.source_sha,
        "source_tree_sha256": args.tree_sha,
        "binary_sha256": args.binary_sha,
        "shim_sha256": args.shim_sha,
        "public_contract_sha256": args.contract_sha,
        "evaluator_contract_sha256": args.evaluator_contract_sha,
        "fixed_prompt_sha256": args.prompt_sha,
        "toolchain": args.toolchain,
    }

    for name, artifact in artifacts.items():
        for key, expected in (
            ("source_sha", args.source_sha),
            ("candidate_source_sha", args.source_sha),
            ("source_tree_sha256", args.tree_sha),
            ("candidate_source_tree_sha256", args.tree_sha),
            ("binary_sha256", args.binary_sha),
            ("candidate_binary_sha256", args.binary_sha),
            ("shim_sha256", args.shim_sha),
            ("candidate_shim_sha256", args.shim_sha),
        ):
            if key in artifact:
                require(errors, artifact[key] == expected, f"{name}.{key} is stale")

    freeze = artifacts.get("freeze", {})
    require(errors, freeze.get("allocation_credit") is False, "freeze allocates credit")
    require(errors, freeze.get("credit_eligible") is False, "freeze is credit eligible")
    require(
        errors,
        freeze.get("public_contract_sha256") == args.contract_sha,
        "freeze public contract digest is stale",
    )
    require(
        errors,
        freeze.get("identity_check") == "pass-login-shell-identity-and-negative-controls",
        "freeze identity control is not passing",
    )
    require(
        errors,
        freeze.get("identity_evidence_ref")
        == "private:controls/dc031527-identity-final.json",
        "freeze identity evidence reference is stale",
    )

    identity_control = artifacts.get("identity", {})
    require(errors, identity_control.get("candidate_sha256") == args.binary_sha, "candidate hash control mismatch")
    require(errors, identity_control.get("login_shell", {}).get("status") == "pass", "login-shell control did not pass")
    require(
        errors,
        identity_control.get("wrong_target_control", {}).get("status") == "rejected-different-target",
        "wrong-target control did not reject",
    )
    require(
        errors,
        identity_control.get("wrong_root_control", {}).get("status") == "rejected-different-root",
        "wrong-root control did not reject",
    )

    receipts: dict[str, dict[str, Any]] = {}
    for suffix, condition_id in (("a", "condition-blinded-a"), ("b", "condition-blinded-b")):
        receipt = artifacts.get(f"receipt_{suffix}", {})
        receipts[suffix] = receipt
        require(errors, receipt.get("condition_id") == condition_id, f"receipt {suffix} condition mismatch")
        require(errors, receipt.get("source_sha") == args.source_sha, f"receipt {suffix} source mismatch")
        require(errors, receipt.get("source_tree") == args.tree_sha, f"receipt {suffix} tree mismatch")
        require(errors, receipt.get("target_sha256") == args.binary_sha, f"receipt {suffix} binary mismatch")
        require(errors, receipt.get("shim_sha256") == args.shim_sha, f"receipt {suffix} shim mismatch")
        require(errors, receipt.get("toolchain") == args.toolchain, f"receipt {suffix} toolchain mismatch")
        require(errors, receipt.get("prompt_sha256") == args.prompt_sha, f"receipt {suffix} prompt mismatch")
        require(errors, receipt.get("child_exit_code") == 0, f"receipt {suffix} child did not exit zero")
        require(errors, receipt.get("identity_status") == "pass", f"receipt {suffix} identity did not pass")
        require(errors, receipt.get("credit_eligible") is False, f"receipt {suffix} is credit eligible")

    for suffix in ("a", "b"):
        evaluation = artifacts.get(f"evaluation_{suffix}", {})
        require(errors, evaluation.get("verification_scope") == "full", f"evaluation {suffix} is not full scope")
        require(errors, evaluation.get("acceptance_eligible") is True, f"evaluation {suffix} is not eligible")
        require(errors, evaluation.get("acceptance_pass") is True, f"evaluation {suffix} did not pass")
        require(errors, evaluation.get("critical_failure_count") == 0, f"evaluation {suffix} has critical failures")
        require(
            errors,
            set(evaluation.get("dimension_states", {})) == DIMENSIONS
            and all(value == "pass" for value in evaluation.get("dimension_states", {}).values()),
            f"evaluation {suffix} dimensions are incomplete or failing",
        )

    binding = artifacts.get("binding", {})
    layouts = binding.get("layouts", [])
    require(errors, len(layouts) == 6, "binding does not contain six layouts")
    handles = [layout.get("handle") for layout in layouts]
    require(errors, len(set(handles)) == 6, "binding handles are not unique")
    require(
        errors,
        sorted(layout.get("stack") for layout in layouts)
        == ["python", "python", "rust", "rust", "typescript", "typescript"],
        "binding does not contain two layouts per stack",
    )
    require(errors, binding.get("candidate", {}).get("identity_evidence_ref") == "controls/dc031527-identity-final.json", "binding identity reference is stale")
    for layout in layouts:
        bound = layout.get("candidate_binding", {})
        require(errors, identity_matches(bound, identity), f"{layout.get('handle')} candidate identity mismatch")
        handle = layout.get("handle")
        require(errors, layout.get("creation_evidence_ref") == f"holdouts/construction-current-dc031-r1.json#{handle}", f"{handle} creation reference is stale")
        require(errors, layout.get("second_readonly_confirmation", {}).get("evidence_ref") == f"holdouts/rebind-second-readonly-2026-09-11-current-dc031-r1.json#{handle}", f"{handle} second-readonly reference is stale")

    second = artifacts.get("second", {})
    records = second.get("creation_records", [])
    require(errors, second.get("result") == "pass", "second-readonly comparison did not pass")
    require(errors, second.get("credit_eligible") is False, "second-readonly comparison is credit eligible")
    require(errors, len(records) == 6 and {record.get("handle") for record in records} == set(handles), "second-readonly records do not repeat all handles")
    for record in records:
        handle = record.get("handle")
        require(errors, record.get("creation_evidence_ref") == f"holdouts/construction-current-dc031-r1.json#{handle}", f"second-readonly {handle} creation reference is stale")

    manifest = artifacts.get("manifest", {})
    conditions = manifest.get("conditions", [])
    require(errors, len(conditions) == 2, "manifest does not contain exactly two conditions")
    if len(conditions) == 2:
        require(errors, {condition.get("condition_id") for condition in conditions} == {"condition-blinded-a", "condition-blinded-b"}, "condition IDs are unstable")
        require(errors, conditions[0].get("product_input", {}).get("name") == conditions[1].get("product_input", {}).get("name"), "conditions change different variables")
        require(errors, conditions[0].get("product_input", {}).get("value") != conditions[1].get("product_input", {}).get("value"), "condition values do not differ")
        for suffix, condition in zip(("a", "b"), conditions):
            require(errors, condition.get("product_input", {}).get("supplied_input_receipt") == f"runs/receipt-{suffix}.json", f"condition {suffix} receipt reference is stale")
            require(errors, condition.get("candidate_identity") == identity, f"condition {suffix} candidate identity mismatch")
        left = conditions[0].get("invariants", {}).copy()
        right = conditions[1].get("invariants", {}).copy()
        require(errors, left == right, "condition invariants are not equivalent")
    matrix = manifest.get("matrix", [])
    require(errors, len(matrix) == 30, "manifest does not reserve 30 cells")
    cells = {(cell.get("stack"), cell.get("condition_id"), cell.get("repetition")) for cell in matrix}
    require(errors, len(cells) == 30, "manifest matrix cells are not unique")
    require(errors, all(cell.get("state") == "reserved" and cell.get("credit_eligible") is False for cell in matrix), "manifest contains allocated or credit-eligible cells")
    require(errors, manifest.get("screening_authorized") is False and manifest.get("screening_credit") is False, "manifest grants screening")
    require(errors, manifest.get("allocation", {}).get("authorized") is False and manifest.get("allocation", {}).get("allocated_cells") == 0, "manifest allocates cells")
    require(errors, manifest.get("protocol_review", {}).get("status") == args.protocol_status, "manifest protocol status differs from requested status")

    canary = artifacts.get("canary", {})
    require(errors, canary.get("allocation_credit") is False and canary.get("screening_authorized") is False, "canary grants credit or screening")
    require(errors, len(canary.get("conditions", [])) == 2, "canary condition summary is incomplete")
    for condition in canary.get("conditions", []):
        require(errors, condition.get("initializer_exit") == 0 and condition.get("evaluator_exit") == 0, f"canary {condition.get('condition_id')} exit mismatch")
        require(errors, condition.get("critical_failure_count") == 0 and condition.get("acceptance_pass") is True, f"canary {condition.get('condition_id')} did not pass")

    for key, relative in files.items():
        path = root / relative
        if path.exists():
            require(errors, path.is_file(), f"artifact path is not a file: {relative}")

    summary = {
        "schema": "assura.a07.packet-validation-result.v1",
        "packet_root": str(root),
        "source_sha": args.source_sha,
        "source_tree_sha256": args.tree_sha,
        "binary_sha256": args.binary_sha,
        "shim_sha256": args.shim_sha,
        "protocol_status": args.protocol_status,
        "handles": len(handles),
        "matrix_cells": len(matrix),
        "conditions": len(conditions),
        "screening_authorized": manifest.get("screening_authorized"),
        "allocation_authorized": manifest.get("allocation", {}).get("authorized"),
        "credit_eligible": manifest.get("allocation", {}).get("screening_credit"),
        "errors": errors,
        "valid": not errors,
    }
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
