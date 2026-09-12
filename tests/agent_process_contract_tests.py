#!/usr/bin/env python3
"""Focused regression fixtures for the Assura execution-process contract."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
LEDGER_AUDIT = ROOT / ".agents/skills/assura-goal-execution/scripts/audit-ledger.sh"
CONTEXT_AUDIT = ROOT / ".agents/skills/assura-goal-execution/scripts/audit-context-routing.py"


def command(*args: str) -> str:
    return subprocess.run(
        list(args),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def ledger_snapshot() -> str:
    return command("bash", str(LEDGER_AUDIT), str(ROOT))


def records(snapshot: str, kind: str) -> list[list[str]]:
    return [line.split("\t") for line in snapshot.splitlines() if line.startswith(f"{kind}\t")]


def fields(snapshot: str, prefix: str) -> dict[str, str]:
    summary = next(line for line in snapshot.splitlines() if line.startswith(prefix))
    return dict(field.split("=", 1) for field in summary.split("\t")[1:])


def import_context_audit() -> Any:
    spec = importlib.util.spec_from_file_location("context_audit", CONTEXT_AUDIT)
    if spec is None or spec.loader is None:
        raise AssertionError("context-routing audit is not importable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_idempotent_reconciliation_and_outcome_accounting() -> None:
    before_head = command("git", "rev-parse", "HEAD")
    before_status = command("git", "status", "--porcelain")
    first = ledger_snapshot()
    second = ledger_snapshot()
    assert first == second, "unchanged reconciliation produced a different snapshot"
    assert command("git", "rev-parse", "HEAD") == before_head
    assert command("git", "status", "--porcelain") == before_status

    cards = records(first, "CARD")
    assert len(cards) == 32, "the fixture ledger must include every card"
    states = [record[2] for record in cards]
    assert states.count("pending") == 6
    assert states.count("active") == 1
    assert states.count("verified") == 1
    assert states.count("blocked") == 2
    summary = fields(first, "SUMMARY\t")
    assert summary == {
        "items": "32",
        "ready_pending": "0",
        "nonterminal": "10",
        "done": "21",
        "not_needed": "1",
        "held": "2",
    }
    assert "unfinished" not in first


def test_process_observations_do_not_promote_product_acceptance() -> None:
    cards = [{"id": record[1], "state": record[2]} for record in records(ledger_snapshot(), "CARD")]
    audit = import_context_audit()
    assert not audit.product_terminal(cards)
    for observation in ("process-only-merge", "external-hold", "reviewer-restriction"):
        checkpoint = {"observation": observation, "cards": cards}
        assert checkpoint["cards"] == cards
        assert not audit.product_terminal(checkpoint["cards"])

    parent = {"id": "CF01", "state": "pending", "reviewer_restriction": "review-only"}
    assert parent["state"] == "pending"
    assert not audit.product_terminal([parent])


def test_context_contract_and_owned_scope_preserve_unknown_topology() -> None:
    audit = import_context_audit()
    compact = "\n".join(
        [
            "- base: origin/master@abc",
            "- card: CF01 / verify",
            "- proof: focused tests pass",
            "- next: review the frozen candidate",
        ]
    )
    assert audit.checkpoint_errors(compact) == []
    oversized = "\n".join(["- item"] * 7) + "\n" + ("x" * 1024)
    assert audit.checkpoint_errors(oversized)
    duplicated_ledger = "\n".join(
        [
            "- base: origin/master@abc",
            "- card: CF01 / verify",
            "CARD\tCF01\tpending",
            "CARD\tCF02\tpending",
        ]
    )
    assert any("ledger" in error for error in audit.checkpoint_errors(duplicated_ledger))

    topology = [
        {
            "scope": "owned",
            "name": "goal/abandoned",
            "owner": "",
            "handle": "none",
            "next_action": "",
            "closure": "active",
        },
        {"scope": "foreign", "name": "foreign/dirty", "owner": "other"},
        {"scope": "unknown", "name": "/private/tmp/unreadable", "owner": "unknown"},
    ]
    errors = audit.owned_scope_errors(topology)
    assert errors and "goal/abandoned" in errors[0]
    # The production audit is read-only; the input fixture remains intact and
    # foreign/unknown records are still present for the caller to preserve.
    assert topology[1]["scope"] == "foreign"
    assert topology[2]["scope"] == "unknown"


def main() -> int:
    tests = [
        test_idempotent_reconciliation_and_outcome_accounting,
        test_process_observations_do_not_promote_product_acceptance,
        test_context_contract_and_owned_scope_preserve_unknown_topology,
    ]
    for test in tests:
        test()
        print(f"PASS {test.__name__}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, subprocess.CalledProcessError) as error:
        print(f"FAIL agent process contract: {error}", file=sys.stderr)
        raise SystemExit(1)
