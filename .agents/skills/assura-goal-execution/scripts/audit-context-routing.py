#!/usr/bin/env python3
"""Audit the small, portable context router used by Assura goal execution.

This is a read-only process check.  It verifies that the universal AGENTS
router and the goal-execution index can reach their next disclosure layer.
It deliberately does not inspect product code, card acceptance, CI state, or
private evaluator inputs.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import Optional


MAX_AGENTS_LINES = 140
MAX_DESCRIPTION_CHARS = 180
MAX_CHECKPOINT_BYTES = 1024
MIN_CHECKPOINT_BULLETS = 3
MAX_CHECKPOINT_BULLETS = 6
SKILL_RELATIVE = Path(".agents/skills/assura-goal-execution/SKILL.md")
REQUIRED_REFERENCES = (
    "references/context-routing.md",
    "references/continuation-control.md",
    "references/execution-contract.md",
    "references/runner-isolation.md",
    "references/source-pointer-lifecycle.md",
    "references/validation-routing.md",
    "scripts/audit-ledger.sh",
)
REQUIRED_TASK_FILES = (
    "orchestration-plan.md",
    "recovery-plan.md",
    "e2e-goal-prompt.md",
    "executor-prompt.md",
)
MARKDOWN_LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
DESCRIPTION = re.compile(r"^description:\s*(.+?)\s*$", re.MULTILINE)
TERMINAL_CARD_STATES = {"done", "not_needed"}
OWNED_SCOPE_FIELDS = ("owner", "handle", "next_action", "closure")


def emit(kind: str, *fields: object) -> None:
    print("\t".join((kind, *(str(field).replace("\t", " ") for field in fields))))


def git_root(candidate: Path) -> Path:
    result = subprocess.run(
        ["git", "-C", str(candidate), "rev-parse", "--show-toplevel"],
        check=True,
        capture_output=True,
        text=True,
    )
    return Path(result.stdout.strip()).resolve()


def link_target(markdown_file: Path, raw_link: str) -> Optional[Path]:
    link = raw_link.strip()
    if link.startswith(("http://", "https://", "mailto:", "#")):
        return None
    return (markdown_file.parent / link.split("#", 1)[0]).resolve()


def check_markdown_links(markdown_file: Path) -> int:
    failures = 0
    contents = markdown_file.read_text(encoding="utf-8")
    for raw_link in MARKDOWN_LINK.findall(contents):
        target = link_target(markdown_file, raw_link)
        if target is None:
            continue
        if target.is_file():
            emit("LINK", markdown_file, raw_link, "PASS")
        else:
            emit("LINK", markdown_file, raw_link, "FAIL")
            failures += 1
    return failures


def checkpoint_errors(checkpoint: str) -> list[str]:
    """Return bounded-context violations without interpreting product state."""
    errors: list[str] = []
    if len(checkpoint.encode("utf-8")) > MAX_CHECKPOINT_BYTES:
        errors.append("checkpoint exceeds configured byte cap")
    bullets = [line for line in checkpoint.splitlines() if line.startswith("- ")]
    if not MIN_CHECKPOINT_BULLETS <= len(bullets) <= MAX_CHECKPOINT_BULLETS:
        errors.append("checkpoint must contain 3-6 bullets")
    ledger_rows = [
        line
        for line in checkpoint.splitlines()
        if line.startswith("CARD\t") or line.startswith("| CARD |")
    ]
    if len(ledger_rows) > 1:
        errors.append("checkpoint duplicates a full ledger")
    return errors


def product_terminal(cards: list[dict[str, str]]) -> bool:
    """Return whether every declared product card has a terminal outcome."""
    return all(card.get("state") in TERMINAL_CARD_STATES for card in cards)


def owned_scope_errors(topology: list[dict[str, str]]) -> list[str]:
    """Reject abandoned owned records without changing foreign/unknown data."""
    errors: list[str] = []
    for record in topology:
        if record.get("scope") != "owned":
            continue
        missing = [field for field in OWNED_SCOPE_FIELDS if not record.get(field)]
        if missing:
            errors.append(
                f"{record.get('name', 'candidate')}: missing {','.join(missing)}"
            )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository", type=Path)
    parser.add_argument(
        "task_relative_path",
        nargs="?",
        default=".trellis/tasks/09-04-maturity-portfolio-strategy",
    )
    parser.add_argument(
        "--checkpoint-file",
        type=Path,
        help="also validate one compact 3-6 bullet checkpoint",
    )
    args = parser.parse_args()

    try:
        repo_root = git_root(args.repository)
    except (OSError, subprocess.CalledProcessError) as error:
        emit("ROOT", args.repository, "FAIL")
        print(f"context-routing audit: cannot resolve Git root: {error}", file=sys.stderr)
        return 2

    emit("ROOT", repo_root, "PASS")
    failures = 0
    checks = 0

    if args.checkpoint_file is not None:
        try:
            checkpoint = args.checkpoint_file.read_text(encoding="utf-8")
        except OSError as error:
            emit("CHECKPOINT", args.checkpoint_file, "FAIL")
            print(f"context-routing audit: cannot read checkpoint: {error}", file=sys.stderr)
            failures += 1
        else:
            errors = checkpoint_errors(checkpoint)
            result = "PASS" if not errors else "FAIL"
            emit("CHECKPOINT", args.checkpoint_file, result)
            for error in errors:
                emit("CHECKPOINT_ERROR", error)
            failures += bool(errors)
        checks += 1

    agents_file = repo_root / "AGENTS.md"
    skill_file = repo_root / SKILL_RELATIVE
    if agents_file.is_file():
        agent_lines = len(agents_file.read_text(encoding="utf-8").splitlines())
        result = "PASS" if agent_lines <= MAX_AGENTS_LINES else "FAIL"
        emit("AGENTS_LINES", agent_lines, f"max={MAX_AGENTS_LINES}", result)
        failures += result == "FAIL"
        checks += 1
        if SKILL_RELATIVE.as_posix() not in agents_file.read_text(encoding="utf-8"):
            emit("AGENTS_ROUTE", SKILL_RELATIVE, "FAIL")
            failures += 1
        else:
            emit("AGENTS_ROUTE", SKILL_RELATIVE, "PASS")
        checks += 1
        failures += check_markdown_links(agents_file)
        checks += len(MARKDOWN_LINK.findall(agents_file.read_text(encoding="utf-8")))
    else:
        emit("AGENTS_FILE", agents_file, "FAIL")
        failures += 1
        checks += 1

    if not skill_file.is_file():
        emit("SKILL_FILE", skill_file, "FAIL")
        failures += 1
        checks += 1
        emit("SUMMARY", f"checks={checks}", f"failures={failures}", "FAIL")
        return 1

    emit("SKILL_FILE", SKILL_RELATIVE, "PASS")
    checks += 1
    skill_contents = skill_file.read_text(encoding="utf-8")
    description_match = DESCRIPTION.search(skill_contents)
    if description_match is None:
        emit("SKILL_DESCRIPTION", "missing", "FAIL")
        failures += 1
    else:
        description = description_match.group(1).strip().strip('"').strip("'")
        result = "PASS" if len(description) <= MAX_DESCRIPTION_CHARS else "FAIL"
        emit("SKILL_DESCRIPTION", len(description), f"max={MAX_DESCRIPTION_CHARS}", result)
        failures += result == "FAIL"
    checks += 1

    scope_result = "PASS" if ".trellis/tasks/" in skill_contents else "FAIL"
    emit("SKILL_SCOPE", ".trellis/tasks/", scope_result)
    failures += scope_result == "FAIL"
    checks += 1

    failures += check_markdown_links(skill_file)
    checks += len(MARKDOWN_LINK.findall(skill_contents))
    for reference in REQUIRED_REFERENCES:
        reference_file = skill_file.parent / reference
        result = "PASS" if reference_file.is_file() else "FAIL"
        emit("REFERENCE", reference, result)
        failures += result == "FAIL"
        checks += 1

    task_root = repo_root / args.task_relative_path
    if task_root.is_dir():
        emit("TASK", args.task_relative_path, "PASS")
        checks += 1
        for task_name in REQUIRED_TASK_FILES:
            task_file = task_root / "research" / task_name
            if not task_file.is_file():
                emit("TASK_FILE", task_name, "FAIL")
                failures += 1
                checks += 1
                continue
            emit("TASK_FILE", task_name, "PASS")
            checks += 1
            failures += check_markdown_links(task_file)
            checks += len(MARKDOWN_LINK.findall(task_file.read_text(encoding="utf-8")))
    else:
        emit("TASK", args.task_relative_path, "FAIL")
        failures += 1
        checks += 1

    result = "PASS" if failures == 0 else "FAIL"
    emit("SUMMARY", f"checks={checks}", f"failures={failures}", result)
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
