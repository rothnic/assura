from __future__ import annotations

import json
import os
from unittest import mock

from agent_delivery_contract_support import (
    FakeGithub,
    fixture_repo,
    git,
    task_cli,
    write_fake_github,
)
from common import delivery_cli
from common.delivery import load_delivery_intent
from common.delivery_cli import project_audit

class ProjectionBranchInventoryCases:
    def test_exact_local_ref_tracks_follow_up_after_merged_remote_head(self) -> None:
        branch = "goal/assura-delivery-closure-core-v2"
        recovery_alias = "goal/assura-delivery-recovery-guard"
        task_path = ".trellis/tasks/01-01-candidate"
        repo, base_oid, reviewed_tip, directory = fixture_repo()
        try:
            git(repo, "branch", "-m", branch)
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            branch_ref = f"refs/heads/{branch}"
            task["branch"] = branch_ref
            task["meta"]["delivery"]["branch_ref"] = branch_ref
            task_json.write_text(json.dumps(task), encoding="utf-8")
            delivery_cli.ensure_receipt(
                repo, load_delivery_intent(task_json), task
            )
            git(repo, "add", f"{task_path}/task.json")
            git(repo, "commit", "-m", "bind exact candidate branch")

            git(repo, "checkout", "master")
            git(repo, "merge", "--squash", reviewed_tip)
            git(repo, "commit", "-m", "squash reviewed candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "checkout", branch)
            for index in range(2):
                follow_up = repo / f"follow-up-{index}.txt"
                follow_up.write_text(f"follow-up {index}\n", encoding="utf-8")
                git(repo, "add", follow_up.name)
                git(repo, "commit", "-m", f"follow-up {index}")
            candidate_tip = git(repo, "rev-parse", "HEAD")
            git(repo, "branch", recovery_alias, candidate_tip)
            git(repo, "update-ref", f"refs/remotes/origin/{branch}", reviewed_tip)

            pull_request = {
                "number": 338,
                "state": "MERGED",
                "title": "Reviewed candidate",
                "url": "https://github.com/rothnic/assura/pull/338",
                "headRefName": branch,
                "headRefOid": reviewed_tip,
                "baseRefName": "master",
                "baseRefOid": base_oid,
                "mergeCommit": {"oid": merge_oid},
                "mergedAt": "2026-09-14T00:00:00Z",
            }
            fake_gh = repo / ".test-bin" / ("gh.py" if os.name == "nt" else "gh")
            write_fake_github(fake_gh, [pull_request])

            inspected = task_cli(
                repo, "delivery", "inspect", task_path, "--format", "json"
            )
            self.assertEqual(inspected.returncode, 0, inspected.stderr)
            status = json.loads(inspected.stdout)
            self.assertEqual(status["tip"], candidate_tip)
            self.assertEqual(status["integration"], "post_review_commits")
            issue_codes = {issue["code"] for issue in status["issues"]}
            self.assertNotIn("CANDIDATE_REF_AMBIGUOUS", issue_codes)
            self.assertIn("INTEGRATION_UNVERIFIED", issue_codes)
            self.assertIn("RECEIPT_TIP_CHANGED", issue_codes)

            advertised = sorted(
                [
                    {
                        "name": branch_ref,
                        "oid": reviewed_tip,
                        "kind": "heads",
                        "source": "remote-advertisement",
                    },
                    {
                        "name": "refs/heads/master",
                        "oid": merge_oid,
                        "kind": "heads",
                        "source": "remote-advertisement",
                    },
                ],
                key=lambda ref: ref["name"],
            )
            with mock.patch(
                "common.delivery._read_remote_refs",
                return_value=(advertised, []),
            ):
                report, _ = project_audit(
                    repo,
                    owner="tester",
                    github=FakeGithub([pull_request]),
                    refresh_remote=True,
                )
            audited = next(
                item
                for item in report["candidates"]
                if item["candidate_id"] == "candidate-1"
            )
            self.assertEqual(audited["tip"], candidate_tip)
            self.assertNotIn(
                "CANDIDATE_REF_AMBIGUOUS",
                {issue["code"] for issue in audited["issues"]},
            )
            refs = {ref["name"]: ref["oid"] for ref in report["inventory"]["refs"]}
            self.assertEqual(refs[branch_ref], candidate_tip)
            self.assertEqual(
                refs[f"refs/heads/{recovery_alias}"], candidate_tip
            )
            self.assertEqual(
                refs[f"refs/remotes/origin/{branch}"], reviewed_tip
            )
            self.assertEqual(report["inventory"]["remote_refs"], advertised)
        finally:
            directory.cleanup()

    def test_ahead_remote_advertisement_keeps_candidate_ref_ambiguous(self) -> None:
        repo, base_oid, candidate_tip, directory = fixture_repo()
        try:
            git(repo, "update-ref", "refs/remotes/origin/candidate", candidate_tip)
            git(repo, "branch", "remote-ahead", candidate_tip)
            git(repo, "checkout", "remote-ahead")
            ahead_file = repo / "remote-ahead.txt"
            ahead_file.write_text("remote-only follow-up\n", encoding="utf-8")
            git(repo, "add", ahead_file.name)
            git(repo, "commit", "-m", "remote-only follow-up")
            ahead_tip = git(repo, "rev-parse", "HEAD")
            git(repo, "checkout", "candidate")

            advertised = sorted(
                [
                    {
                        "name": "refs/heads/candidate",
                        "oid": ahead_tip,
                        "kind": "heads",
                        "source": "remote-advertisement",
                    },
                    {
                        "name": "refs/heads/master",
                        "oid": base_oid,
                        "kind": "heads",
                        "source": "remote-advertisement",
                    },
                ],
                key=lambda ref: ref["name"],
            )
            with mock.patch(
                "common.delivery._read_remote_refs",
                return_value=(advertised, []),
            ):
                report, _ = project_audit(
                    repo,
                    owner="tester",
                    github=FakeGithub([]),
                    refresh_remote=True,
                )

            audited = next(
                item
                for item in report["candidates"]
                if item["candidate_id"] == "candidate-1"
            )
            self.assertIsNone(audited["tip"])
            self.assertIn(
                "CANDIDATE_REF_AMBIGUOUS",
                {issue["code"] for issue in audited["issues"]},
            )
            self.assertIsNone(audited["outcome"])
            advertised_rows = [
                row
                for row in report["unowned_refs"]
                if row["name"] == "refs/heads/candidate" and row["oid"] == ahead_tip
            ]
            self.assertEqual(len(advertised_rows), 1)
            self.assertEqual(advertised_rows[0]["name"], "refs/heads/candidate")
            self.assertEqual(advertised_rows[0]["oid"], ahead_tip)
            self.assertEqual(advertised_rows[0]["disposition"], "unresolved")
            self.assertEqual(report["inventory"]["remote_refs"], advertised)
        finally:
            directory.cleanup()
