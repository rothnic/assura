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


MAX_AGENTS_LINES = 140
MAX_DESCRIPTION_CHARS = 180
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
MARKDOWN_LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
DESCRIPTION = re.compile(r"^description:\s*(.+?)\s*$", re.MULTILINE)


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


def link_target(markdown_file: Path, raw_link: str) -> Path | None:
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository", type=Path)
    parser.add_argument(
        "task_relative_path",
        nargs="?",
        default=".trellis/tasks/09-04-maturity-portfolio-strategy",
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
        for task_name in (
            "orchestration-plan.md",
            "recovery-plan.md",
            "e2e-goal-prompt.md",
            "executor-prompt.md",
        ):
            task_file = task_root / "research" / task_name
            if not task_file.is_file():
                continue
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
