from __future__ import annotations

import json
import subprocess
import sys
from dataclasses import replace
from unittest import mock

from agent_delivery_contract_support import ROOT, fixture_repo
from common import delivery_status
from common.delivery import (
    DeliveryValidationError,
    GithubUnavailable,
    SubprocessGithubReader,
    collect_candidate_inventory,
    load_delivery_intent,
)
from common.delivery_github import GithubIdentityMismatch

class ProjectionProviderCases:
    def test_github_reader_rejects_a_full_page_as_incomplete_coverage(self) -> None:
        payload = json.dumps([{"number": index} for index in range(1000)])
        completed = subprocess.CompletedProcess(
            ["gh"],
            0,
            stdout=payload,
            stderr="",
        )
        with mock.patch("common.delivery_github.subprocess.run", return_value=completed):
            with self.assertRaises(GithubUnavailable):
                SubprocessGithubReader().list_pull_requests("rothnic/assura")

    def test_github_reader_merges_bounded_provider_field_queries(self) -> None:
        responses = [
            json.dumps(
                [
                    {
                        "number": 14,
                        "state": "MERGED",
                        "headRefName": "candidate",
                        "headRefOid": "a" * 40,
                    }
                ]
            ),
            json.dumps(
                [
                    {
                        "number": 14,
                        "reviews": [{"id": "review-14", "state": "APPROVED"}],
                    }
                ]
            ),
            json.dumps(
                [
                    {
                        "number": 14,
                        "statusCheckRollup": [{"databaseId": "job-14"}],
                    }
                ]
            ),
        ]
        completed = [
            subprocess.CompletedProcess(["gh"], 0, stdout=payload, stderr="")
            for payload in responses
        ]
        with mock.patch("common.delivery_github.subprocess.run", side_effect=completed) as run:
            rows = SubprocessGithubReader().list_pull_requests("rothnic/assura")

        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["headRefOid"], "a" * 40)
        self.assertIn("reviews", rows[0])
        self.assertIn("statusCheckRollup", rows[0])
        self.assertEqual(run.call_count, 3)

    def test_candidate_reader_queries_complete_identity_then_exact_pr_details(self) -> None:
        identity = {
            "number": 14,
            "state": "MERGED",
            "headRefName": "candidate",
            "headRefOid": "a" * 40,
            "baseRefName": "master",
            "baseRefOid": "b" * 40,
            "mergeCommit": {"oid": "c" * 40},
            "mergedAt": "2026-10-17T12:34:56Z",
        }
        details = {
            **identity,
            "title": "Candidate PR",
            "url": "https://github.com/rothnic/assura/pull/14",
            "reviews": [{"id": "review-14", "author": {"login": "reviewer"}, "state": "APPROVED"}],
            "statusCheckRollup": [{"__typename": "CheckRun", "databaseId": "job-14", "name": "build", "status": "COMPLETED", "conclusion": "SUCCESS"}],
        }
        completed = [
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps([identity]), stderr=""),
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps(details), stderr=""),
        ]
        with mock.patch("common.delivery_github.subprocess.run", side_effect=completed) as run:
            cohort, current = SubprocessGithubReader().candidate_pull_request(
                "rothnic/assura", "candidate", "master", "a" * 40
            )

        self.assertEqual([row["number"] for row in cohort], [14])
        self.assertEqual(current["number"], 14)
        self.assertEqual(current["mergedAt"], "2026-10-17T12:34:56Z")
        self.assertIn("reviews", current)
        self.assertIn("statusCheckRollup", current)
        self.assertEqual(run.call_count, 2)
        commands = [call.args[0] for call in run.call_args_list]
        self.assertEqual(commands[0][1:3], ["pr", "list"])
        self.assertNotIn("reviews", commands[0][-1])
        self.assertNotIn("statusCheckRollup", commands[0][-1])
        self.assertEqual(commands[1][1:4], ["pr", "view", "14"])

    def test_candidate_reader_rejects_incoherent_merge_snapshots(self) -> None:
        open_snapshot = {
            "number": 14,
            "state": "OPEN",
            "headRefName": "candidate",
            "headRefOid": "a" * 40,
            "baseRefName": "master",
            "baseRefOid": "b" * 40,
            "mergeCommit": None,
            "mergedAt": None,
        }
        merged_snapshot = {
            **open_snapshot,
            "state": "MERGED",
            "mergeCommit": {"oid": "c" * 40},
            "mergedAt": "2026-10-17T12:34:56Z",
        }
        invalid_cases = (
            ("open object timestamp", open_snapshot, {"mergedAt": {"value": "x"}}),
            ("open timestamp", open_snapshot, {"mergedAt": "2026-10-17T12:34:56Z"}),
            ("closed timestamp", {**open_snapshot, "state": "CLOSED"}, {"mergedAt": "2026-10-17T12:34:56Z"}),
            ("merged missing timestamp", merged_snapshot, {"mergedAt": None}),
            ("merged invalid timestamp", merged_snapshot, {"mergedAt": "2026-02-30T12:34:56Z"}),
            ("merged missing OID", merged_snapshot, {"mergeCommit": None}),
            ("merged invalid OID", merged_snapshot, {"mergeCommit": {"oid": "short"}}),
            ("merged arbitrary timestamp", merged_snapshot, {"mergedAt": "yesterday"}),
        )
        for label, valid_snapshot, invalid_fields in invalid_cases:
            for invalid_stage in ("identity", "details", "both"):
                with self.subTest(case=label, invalid_stage=invalid_stage):
                    invalid_snapshot = {**valid_snapshot, **invalid_fields}
                    identity = (
                        valid_snapshot if invalid_stage == "details" else invalid_snapshot
                    )
                    details = (
                        valid_snapshot if invalid_stage == "identity" else invalid_snapshot
                    )
                    details = {
                        **details,
                        "title": "Candidate PR",
                        "url": "https://github.com/rothnic/assura/pull/14",
                        "reviews": [],
                        "statusCheckRollup": [],
                    }
                    responses = [
                        subprocess.CompletedProcess(
                            ["gh"], 0, stdout=json.dumps([identity]), stderr=""
                        ),
                        subprocess.CompletedProcess(
                            ["gh"], 0, stdout=json.dumps(details), stderr=""
                        ),
                    ]
                    with mock.patch(
                        "common.delivery_github.subprocess.run", side_effect=responses
                    ):
                        with self.assertRaisesRegex(
                            GithubIdentityMismatch, "mergedAt|merge OID|mergeCommit|merge state"
                        ):
                            SubprocessGithubReader().candidate_pull_request(
                                "rothnic/assura", "candidate", "master", "a" * 40
                            )

    def test_candidate_reader_accepts_status_context_rollup_member(self) -> None:
        identity = {
            "number": 14, "state": "OPEN", "headRefName": "candidate",
            "headRefOid": "a" * 40, "baseRefName": "master",
            "baseRefOid": "b" * 40, "mergeCommit": None, "mergedAt": None,
        }
        details = {
            **identity, "title": "Candidate PR",
            "url": "https://github.com/rothnic/assura/pull/14",
            "reviews": [], "statusCheckRollup": [{
                "__typename": "StatusContext", "context": "legacy-ci",
                "state": "SUCCESS", "targetUrl": "https://ci.example/run/14",
            }],
        }
        responses = [
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps([identity]), stderr=""),
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps(details), stderr=""),
        ]
        with mock.patch("common.delivery_github.subprocess.run", side_effect=responses):
            _, current = SubprocessGithubReader().candidate_pull_request(
                "rothnic/assura", "candidate", "master", "a" * 40
            )
        self.assertEqual(current["statusCheckRollup"][0]["__typename"], "StatusContext")

    def test_candidate_reader_accepts_pending_check_without_conclusion(self) -> None:
        identity = {
            "number": 14, "state": "OPEN", "headRefName": "candidate",
            "headRefOid": "a" * 40, "baseRefName": "master",
            "baseRefOid": "b" * 40, "mergeCommit": None, "mergedAt": None,
        }
        details = {
            **identity, "title": "Candidate PR",
            "url": "https://github.com/rothnic/assura/pull/14",
            "reviews": [], "statusCheckRollup": [{
                "__typename": "CheckRun", "databaseId": "job-14",
                "name": "build", "status": "IN_PROGRESS", "conclusion": None,
                "detailsUrl": "https://github.com/rothnic/assura/actions/runs/101/job/202",
            }],
        }
        responses = [
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps([identity]), stderr=""),
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps(details), stderr=""),
        ]
        with mock.patch("common.delivery_github.subprocess.run", side_effect=responses):
            _, current = SubprocessGithubReader().candidate_pull_request(
                "rothnic/assura", "candidate", "master", "a" * 40
            )
        self.assertIsNone(current["statusCheckRollup"][0]["conclusion"])

    def test_candidate_reader_rejects_malformed_rollup_union_members(self) -> None:
        identity = {
            "number": 14, "state": "OPEN", "headRefName": "candidate",
            "headRefOid": "a" * 40, "baseRefName": "master",
            "baseRefOid": "b" * 40, "mergeCommit": None, "mergedAt": None,
        }
        malformed_rows = [
            {"__typename": "CheckRun", "name": "build", "conclusion": "SUCCESS"},
            {"__typename": "CheckRun", "name": "build", "status": "IN_PROGRESS", "conclusion": "SUCCESS"},
            {"__typename": "OtherType", "name": "build", "status": "COMPLETED", "conclusion": "SUCCESS"},
            {"__typename": "StatusContext", "context": "", "state": "SUCCESS"},
        ]
        for row in malformed_rows:
            with self.subTest(row=row):
                details = {
                    **identity, "title": "Candidate PR",
                    "url": "https://github.com/rothnic/assura/pull/14",
                    "reviews": [], "statusCheckRollup": [row],
                }
                responses = [
                    subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps([identity]), stderr=""),
                    subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps(details), stderr=""),
                ]
                with mock.patch("common.delivery_github.subprocess.run", side_effect=responses):
                    with self.assertRaisesRegex(GithubIdentityMismatch, "malformed check identity"):
                        SubprocessGithubReader().candidate_pull_request(
                            "rothnic/assura", "candidate", "master", "a" * 40
                        )

    def test_candidate_reader_rejects_identity_races_and_sanitizes_provider_errors(self) -> None:
        identity = {
            "number": 14, "state": "MERGED", "headRefName": "candidate",
            "headRefOid": "a" * 40, "baseRefName": "master",
            "baseRefOid": "b" * 40, "mergeCommit": {"oid": "c" * 40},
            "mergedAt": "2026-10-17T12:34:56Z",
        }
        raced = {**identity, "headRefOid": "d" * 40, "reviews": [], "statusCheckRollup": []}
        responses = [
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps([identity]), stderr=""),
            subprocess.CompletedProcess(["gh"], 0, stdout=json.dumps(raced), stderr=""),
        ]
        with mock.patch("common.delivery_github.subprocess.run", side_effect=responses):
            with self.assertRaisesRegex(GithubIdentityMismatch, "changed headRefOid"):
                SubprocessGithubReader().candidate_pull_request(
                    "rothnic/assura", "candidate", "master", "a" * 40
                )

        failed = subprocess.CompletedProcess(
            ["gh"], 1, stdout="", stderr="token=secret\ncontrol\x01detail"
        )
        with mock.patch("common.delivery_github.subprocess.run", return_value=failed):
            with self.assertRaises(GithubUnavailable) as caught:
                SubprocessGithubReader().candidate_pull_request(
                    "rothnic/assura", "candidate", "master", "a" * 40
                )
        self.assertIn("identity cohort unavailable", str(caught.exception))
        self.assertNotIn("secret", str(caught.exception))
        self.assertNotIn("\x01", str(caught.exception))

    def test_candidate_inventory_marks_only_candidate_scoped_github_coverage(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            intent = load_delivery_intent(task_json)
            pr = {
                "number": 18, "state": "OPEN", "headRefName": "candidate",
                "headRefOid": tip, "baseRefName": "master", "baseRefOid": "b" * 40,
                "reviews": [], "statusCheckRollup": [],
            }

            class CandidateReader:
                def candidate_pull_request(self, repository, head_ref, base_ref, head_oid):
                    self.args = (repository, head_ref, base_ref, head_oid)
                    return [pr], pr

            reader = CandidateReader()
            inventory = collect_candidate_inventory(repo, intent, github=reader)
            self.assertEqual(reader.args[-1], tip)
            self.assertEqual(inventory.coverage["github"], "unavailable")
            self.assertEqual(inventory.coverage["github_candidate"], intent.candidate_id)
            self.assertEqual(len(inventory.pull_requests), 1)
            self.assertTrue(delivery_status._candidate_github_coverage(intent, inventory))
            self.assertFalse(
                delivery_status._candidate_github_coverage(
                    replace(intent, candidate_id="other"), inventory
                )
            )

            class MismatchReader:
                def candidate_pull_request(self, repository, head_ref, base_ref, head_oid):
                    raise GithubIdentityMismatch("PR identity mismatch: changed head")

            with self.assertRaisesRegex(DeliveryValidationError, "PR identity mismatch: changed head"):
                collect_candidate_inventory(repo, intent, github=MismatchReader())
        finally:
            directory.cleanup()

    def test_delivery_provider_and_evidence_import_orders_in_fresh_processes(self) -> None:
        import_orders = (
            "import common.delivery_github; import common.delivery; import common.delivery_status",
            "import common.delivery_evidence; import common.delivery; import common.delivery_status",
            "import common.delivery; import common.delivery_github; import common.delivery_evidence",
        )
        for imports in import_orders:
            with self.subTest(imports=imports):
                code = (
                    "import sys; "
                    f"sys.path.insert(0, {str(ROOT / '.trellis' / 'scripts')!r}); "
                    f"{imports}; "
                    "from common.delivery import GithubUnavailable; "
                    "from common.delivery_github import GithubUnavailable as ProviderUnavailable; "
                    "assert GithubUnavailable is ProviderUnavailable"
                )
                result = subprocess.run(
                    [sys.executable, "-c", code],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_github_reader_resolves_cli_through_path(self) -> None:
        completed = [
            subprocess.CompletedProcess(
                ["gh"],
                0,
                stdout=json.dumps([{"number": 14}]),
                stderr="",
            )
            for _ in range(3)
        ]
        with mock.patch("common.delivery_github.shutil.which", return_value="/tmp/gh.cmd") as which:
            with mock.patch("common.delivery_github.subprocess.run", side_effect=completed) as run:
                SubprocessGithubReader().list_pull_requests("rothnic/assura")

        which.assert_called_once_with("gh")
        self.assertTrue(all(call.args[0][0] == "/tmp/gh.cmd" for call in run.call_args_list))
