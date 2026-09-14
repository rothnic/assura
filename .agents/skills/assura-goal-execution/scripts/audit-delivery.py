#!/usr/bin/env python3
"""Canonical read-only delivery audit entry point for Assura.

The script intentionally delegates to the Trellis common implementation so
shell helpers and lifecycle commands cannot drift into separate inventories.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(REPO_ROOT / ".trellis" / "scripts"))

from common.delivery_cli import project_audit, strict_audit_pass  # noqa: E402


def main() -> int:
    """Run the repository delivery projection without mutating state."""
    parser = argparse.ArgumentParser(description="Audit Assura delivery ownership and closure")
    parser.add_argument("--format", choices=("json", "text"), default="text")
    parser.add_argument("--strict", action="store_true")
    parser.add_argument("--owner")
    parser.add_argument(
        "--refresh",
        action="store_true",
        help="read advertised origin heads/tags before assessing complete coverage",
    )
    args = parser.parse_args()

    repo_root = Path.cwd().resolve()
    report, statuses = project_audit(
        repo_root, owner=args.owner, refresh_remote=args.refresh
    )
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"repository: {report['repository']}")
        print(f"base: {report['base_ref']}@{report['base_oid'] or 'unknown'}")
        print(f"coverage: {report['coverage']}")
        for status in statuses:
            print(f"{status.candidate_id}: {status.outcome or status.phase}; next: {status.next_action}")
        print(f"unowned refs: {len(report['unowned_refs'])}")
        print(f"unowned worktrees: {len(report['unowned_worktrees'])}")
    if args.strict and not strict_audit_pass(report, statuses, args.owner):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
