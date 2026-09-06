#!/usr/bin/env python3
"""Evaluate one completed Assura initialization run against a trusted contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project", required=True, type=Path)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--assura-bin", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--public-output",
        type=Path,
        help="optional redacted result suitable for publication",
    )
    parser.add_argument("--dimensions")
    return parser.parse_args()


def captured_text(output: str | bytes | None) -> str:
    """Normalize subprocess output retained by a timeout exception."""
    if isinstance(output, bytes):
        return output.decode(errors="replace")
    return output or ""


def run_command(
    binary: Path, command: list[str], cwd: Path, timeout_seconds: float = 30
) -> dict[str, object]:
    try:
        completed = subprocess.run(
            [str(binary), *command], cwd=cwd, capture_output=True, text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        return {
            "command": command,
            "cwd": str(cwd),
            "exit": None,
            "state": "fail",
            "timed_out": True,
            "stdout": captured_text(error.stdout),
            "stderr": captured_text(error.stderr),
        }
    except FileNotFoundError:
        return {
            "command": command,
            "cwd": str(cwd),
            "exit": None,
            "state": "unavailable",
            "reason": "command_or_cwd_not_found",
            "stdout": "",
            "stderr": "",
        }
    return {
        "command": command,
        "cwd": str(cwd),
        "exit": completed.returncode,
        "state": "pass" if completed.returncode == 0 else "fail",
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def run_native_command(
    command: list[str], cwd: Path, timeout_seconds: float = 30
) -> dict[str, object]:
    try:
        completed = subprocess.run(
            command, cwd=cwd, capture_output=True, text=True, timeout=timeout_seconds
        )
    except FileNotFoundError:
        return {"command": command, "cwd": str(cwd), "state": "unavailable"}
    except subprocess.TimeoutExpired as error:
        return {
            "command": command,
            "cwd": str(cwd),
            "exit": None,
            "state": "fail",
            "timed_out": True,
            "stdout": captured_text(error.stdout),
            "stderr": captured_text(error.stderr),
        }
    return {
        "command": command,
        "cwd": str(cwd),
        "exit": completed.returncode,
        "state": "pass" if completed.returncode == 0 else "fail",
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def directory_hash(root: Path) -> str:
    """Hash the source fixture tree without following or modifying it."""
    digest = hashlib.sha256()
    for item in sorted(root.rglob("*")):
        relative_path = item.relative_to(root).as_posix()
        digest.update(relative_path.encode())
        if item.is_file():
            digest.update(item.read_bytes())
    return digest.hexdigest()


def contract_validation_errors(contract: object) -> list[str]:
    """Return missing or malformed top-level v1 contract field names."""
    if not isinstance(contract, dict):
        return ["root"]
    required_types: dict[str, type[object] | tuple[type[object], ...]] = {
        "fixture_id": str,
        "stack": str,
        "prompt_hash": str,
        "required_paths": list,
        "forbidden_paths": list,
        "preserve_hashes": dict,
        "positive_probes": list,
        "negative_probes": list,
        "native_commands": list,
        "required_hook_states": list,
    }
    errors = [
        field for field, expected_type in required_types.items()
        if not isinstance(contract.get(field), expected_type)
    ]
    if errors:
        return errors

    if not re.fullmatch(r"[0-9a-f]{64}", contract["prompt_hash"]):
        errors.append("prompt_hash")

    def safe_relative_path(value: object) -> bool:
        return (
            isinstance(value, str)
            and not Path(value).is_absolute()
            and ".." not in Path(value).parts
        )

    for field in ("required_paths", "forbidden_paths"):
        for index, value in enumerate(contract[field]):
            if not safe_relative_path(value):
                errors.append(f"{field}[{index}]")
    for relative_path, expected_hash in contract["preserve_hashes"].items():
        if not safe_relative_path(relative_path) or not isinstance(expected_hash, str):
            errors.append(f"preserve_hashes[{relative_path!r}]")
    for field in ("positive_probes", "negative_probes", "native_commands"):
        for index, entry in enumerate(contract[field]):
            label = f"{field}[{index}]"
            if not isinstance(entry, dict):
                errors.append(label)
                continue
            if not isinstance(entry.get("id"), str):
                errors.append(f"{label}.id")
            command = entry.get("command")
            if not isinstance(command, list) or not command or not all(
                isinstance(argument, str) for argument in command
            ):
                errors.append(f"{label}.command")
            if "cwd" in entry and not safe_relative_path(entry["cwd"]):
                errors.append(f"{label}.cwd")
            if field == "negative_probes":
                mutation = entry.get("mutation")
                if not isinstance(mutation, dict):
                    errors.append(f"{label}.mutation")
                elif not safe_relative_path(mutation.get("path")) or not isinstance(
                    mutation.get("contents"), str
                ):
                    errors.append(f"{label}.mutation")
                if not isinstance(entry.get("expected_rule"), str) or not entry["expected_rule"]:
                    errors.append(f"{label}.expected_rule")
    for index, hook_state in enumerate(contract["required_hook_states"]):
        label = f"required_hook_states[{index}]"
        if not isinstance(hook_state, dict):
            errors.append(label)
        elif not isinstance(hook_state.get("id"), str) or not safe_relative_path(
            hook_state.get("path")
        ) or ("exists" in hook_state and not isinstance(hook_state["exists"], bool)):
            errors.append(label)
    return errors


def evaluate(arguments: argparse.Namespace) -> tuple[int, dict[str, object]]:
    contract = json.loads(arguments.contract.read_text())
    if not isinstance(contract, dict) or contract.get("schema") != "assura.agent-init-evaluator.v1":
        return 1, {
            "schema": "assura.agent-init-evaluation-result.v1",
            "fixture_id": contract.get("fixture_id") if isinstance(contract, dict) else None,
            "verification_scope": "full",
            "acceptance_eligible": False,
            "acceptance_pass": False,
            "critical_failures": ["schema"],
            "command_evidence": [],
        }
    validation_errors = contract_validation_errors(contract)
    if validation_errors:
        return 1, {
            "schema": "assura.agent-init-evaluation-result.v1",
            "fixture_id": contract.get("fixture_id"),
            "verification_scope": "full",
            "acceptance_eligible": False,
            "acceptance_pass": False,
            "critical_failures": [f"contract:{field}" for field in validation_errors],
            "dimension_states": {},
            "command_evidence": [],
        }
    if not arguments.assura_bin.is_absolute():
        return 1, {
            "schema": "assura.agent-init-evaluation-result.v1",
            "fixture_id": contract.get("fixture_id"),
            "verification_scope": "full",
            "acceptance_eligible": False,
            "acceptance_pass": False,
            "critical_failures": ["assura_bin:absolute_path"],
            "dimension_states": {},
            "command_evidence": [],
        }
    allowed_dimensions = {
        "structure", "policy", "guidance", "hooks", "native", "preservation", "idempotence"
    }
    requested_dimensions = (
        {item for item in arguments.dimensions.split(",") if item}
        if arguments.dimensions
        else allowed_dimensions
    )
    unknown_dimensions = requested_dimensions - allowed_dimensions
    if unknown_dimensions:
        raise ValueError(f"unknown dimensions: {', '.join(sorted(unknown_dimensions))}")
    partial = requested_dimensions != allowed_dimensions
    critical_failures: list[str] = []
    evidence: list[dict[str, object]] = []
    if "policy" in requested_dimensions and not contract["negative_probes"]:
        critical_failures.append("contract:negative_probes")
    source_hash_before = (
        directory_hash(arguments.project) if "idempotence" in requested_dimensions else None
    )
    for relative_path in contract.get("required_paths", []) if "structure" in requested_dimensions else []:
        exists = (arguments.project / relative_path).exists()
        evidence.append({"kind": "required_path", "path": relative_path, "exists": exists})
        if not exists:
            critical_failures.append(f"required_path:{relative_path}")
    for relative_path in contract.get("forbidden_paths", []) if "structure" in requested_dimensions else []:
        exists = (arguments.project / relative_path).exists()
        evidence.append({"kind": "forbidden_path", "path": relative_path, "exists": exists})
        if exists:
            critical_failures.append(f"forbidden_path:{relative_path}")
    for hook_state in contract.get("required_hook_states", []) if "hooks" in requested_dimensions else []:
        exists = (arguments.project / hook_state["path"]).exists()
        expected = hook_state.get("exists", True)
        evidence.append(
            {"kind": "hook", "id": hook_state["id"], "path": hook_state["path"], "exists": exists}
        )
        if exists != expected:
            critical_failures.append(f"hook:{hook_state['id']}")
    preservation_hashes = contract.get("preserve_hashes", {}) if "preservation" in requested_dimensions else {}
    for relative_path, expected_hash in preservation_hashes.items():
        preserved_path = arguments.project / relative_path
        actual_hash = (
            hashlib.sha256(preserved_path.read_bytes()).hexdigest()
            if preserved_path.is_file()
            else None
        )
        evidence.append(
            {
                "kind": "preservation",
                "path": relative_path,
                "expected_hash": expected_hash,
                "actual_hash": actual_hash,
            }
        )
        if actual_hash != expected_hash:
            critical_failures.append(f"preservation:{relative_path}")
    with tempfile.TemporaryDirectory(prefix="assura-agent-init-evaluator-") as directory:
        disposable_project = Path(directory) / "project"
        shutil.copytree(arguments.project, disposable_project)
        for probe in contract.get("positive_probes", []) if "policy" in requested_dimensions else []:
            command_evidence = run_command(
                arguments.assura_bin,
                probe["command"],
                disposable_project / probe.get("cwd", "."),
            )
            command_evidence["probe_id"] = probe["id"]
            command_evidence["probe_kind"] = "positive"
            evidence.append(command_evidence)
            if command_evidence["exit"] != 0:
                critical_failures.append(f"positive:{probe['id']}")
        for index, probe in enumerate(
            contract.get("negative_probes", []) if "policy" in requested_dimensions else []
        ):
            negative_project = Path(directory) / f"negative-probe-{index}"
            shutil.copytree(arguments.project, negative_project)
            mutation = probe["mutation"]
            target = negative_project / mutation["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(mutation["contents"])
            command_evidence = run_command(
                arguments.assura_bin,
                probe["command"],
                negative_project / probe.get("cwd", "."),
            )
            command_evidence["probe_id"] = probe["id"]
            command_evidence["probe_kind"] = "negative"
            command_evidence["expected_rule"] = probe["expected_rule"]
            command_evidence["matched_expected_rule"] = probe["expected_rule"] in (
                command_evidence.get("stdout", "") + command_evidence.get("stderr", "")
            )
            evidence.append(command_evidence)
            if (
                command_evidence["state"] == "unavailable"
                or command_evidence.get("timed_out")
                or command_evidence["exit"] == 0
                or not command_evidence["matched_expected_rule"]
            ):
                critical_failures.append(probe["id"])
        for native_command in contract.get("native_commands", []) if "native" in requested_dimensions else []:
            command_evidence = run_native_command(
                native_command["command"], disposable_project / native_command.get("cwd", ".")
            )
            command_evidence["native_id"] = native_command["id"]
            if (
                native_command.get("require_collected_tests")
                and command_evidence["state"] == "pass"
                and re.search(
                    r"\b(?:0\s+tests?\s+(?:collected|run)|collected\s+0\s+items|running\s+0\s+tests)\b",
                    command_evidence.get("stdout", "") + command_evidence.get("stderr", ""),
                )
            ):
                command_evidence["state"] = "fail"
                command_evidence["reason"] = "zero_collected_tests"
            evidence.append(command_evidence)
            if command_evidence["state"] != "pass":
                critical_failures.append(f"native:{native_command['id']}")
    if "idempotence" in requested_dimensions:
        source_hash_after = directory_hash(arguments.project)
        idempotent = source_hash_before == source_hash_after
        evidence.append(
            {
                "kind": "idempotence",
                "source_hash_before": source_hash_before,
                "source_hash_after": source_hash_after,
                "state": "pass" if idempotent else "fail",
            }
        )
        if not idempotent:
            critical_failures.append("idempotence:source_fixture_mutated")
    evidenced_dimensions: set[str] = set()
    for entry in evidence:
        if entry.get("kind") in {"required_path", "forbidden_path"}:
            evidenced_dimensions.add("structure")
        elif entry.get("kind") == "hook":
            evidenced_dimensions.add("hooks")
        elif entry.get("kind") == "preservation":
            evidenced_dimensions.add("preservation")
        elif entry.get("kind") == "idempotence":
            evidenced_dimensions.add("idempotence")
        elif "probe_id" in entry:
            evidenced_dimensions.add("policy")
        elif "native_id" in entry:
            evidenced_dimensions.add("native")
    dimension_states = {dimension: "pass" for dimension in requested_dimensions}
    for dimension in requested_dimensions - evidenced_dimensions:
        dimension_states[dimension] = "unavailable"
        critical_failures.append(f"dimension:{dimension}:unavailable")
    for failure in critical_failures:
        probe_evidence = next(
            (
                entry for entry in evidence
                if entry.get("probe_id") == failure
                or f"positive:{entry.get('probe_id')}" == failure
            ),
            None,
        )
        if probe_evidence:
            dimension_states["policy"] = (
                "unavailable" if probe_evidence["state"] == "unavailable" else "fail"
            )
        elif failure.startswith(("required_path:", "forbidden_path:")):
            dimension_states["structure"] = "fail"
        elif failure.startswith("hook:"):
            dimension_states["hooks"] = "fail"
        elif failure.startswith("preservation:"):
            dimension_states["preservation"] = "fail"
        elif failure.startswith(("positive:", "must-")):
            dimension_states["policy"] = "fail"
        elif failure == "contract:negative_probes":
            dimension_states["policy"] = "fail"
        elif failure.startswith("native:"):
            native_id = failure.split(":", maxsplit=1)[1]
            native_evidence = next(
                entry for entry in evidence if entry.get("native_id") == native_id
            )
            dimension_states["native"] = (
                "unavailable" if native_evidence["state"] == "unavailable" else "fail"
            )
        elif failure.startswith("idempotence:"):
            dimension_states["idempotence"] = "fail"
    result = {
        "schema": "assura.agent-init-evaluation-result.v1",
        "fixture_id": contract["fixture_id"],
        "prompt_hash": contract.get("prompt_hash"),
        "verification_scope": "partial" if partial else "full",
        "acceptance_eligible": not partial,
        "acceptance_pass": not partial and not critical_failures,
        "critical_failures": critical_failures,
        "dimension_states": dimension_states,
        "command_evidence": evidence,
    }
    return (0 if result["acceptance_pass"] else 1), result


def publication_view(result: dict[str, object]) -> dict[str, object]:
    """Retain aggregate outcome evidence without private paths or identifiers."""
    publication = {
        key: value
        for key, value in result.items()
        if key
        not in {
            "fixture_id", "prompt_hash", "contract_hash", "assura_binary_hash",
            "critical_failures",
        }
    }
    publication["critical_failure_count"] = len(result["critical_failures"])
    publication["command_evidence"] = []
    for entry in result["command_evidence"]:
        if "native_id" in entry:
            evidence_kind = "native_command"
        elif "probe_id" in entry:
            evidence_kind = f"{entry.get('probe_kind', 'negative')}_probe"
        else:
            evidence_kind = str(entry["kind"])
        public_entry = {"kind": evidence_kind}
        for key in ("exists", "exit", "state", "reason"):
            if key in entry:
                public_entry[key] = entry[key]
        publication["command_evidence"].append(public_entry)
    return publication


def main() -> int:
    arguments = parse_arguments()
    exit_code, result = evaluate(arguments)
    result["contract_hash"] = hashlib.sha256(arguments.contract.read_bytes()).hexdigest()
    result["assura_binary_hash"] = hashlib.sha256(arguments.assura_bin.read_bytes()).hexdigest()
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(json.dumps(result, indent=2) + "\n")
    if arguments.public_output:
        arguments.public_output.parent.mkdir(parents=True, exist_ok=True)
        arguments.public_output.write_text(json.dumps(publication_view(result), indent=2) + "\n")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
