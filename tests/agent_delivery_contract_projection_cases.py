from __future__ import annotations

import copy
import json
import os
import tempfile
from dataclasses import replace
from pathlib import Path
from unittest import mock

from agent_delivery_contract_support import (
    FakeGithub,
    RELEASE_ARCHIVES,
    UnavailableGithub,
    fixture_repo,
    git,
    local_integration_evidence,
    task_cli,
    write_fake_github,
)
from common import delivery_status
from common.delivery import (
    DeliveryIssue,
    _read_worktrees,
    classify_candidate,
    collect_inventory,
    load_delivery_intent,
)
from common.delivery_store import DeliveryStore
from common.tasks import children_progress, get_all_statuses

class ProjectionCases:
    def test_cli_initial_close_uses_exact_provider_merge_and_historical_checks(self) -> None:
        repo, original_base, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")
            check_url = "https://github.com/rothnic/assura/actions/runs/run-21/job/job-21"
            provider_pr = {
                "number": 21,
                "state": "MERGED",
                "title": "Candidate",
                "url": "https://github.com/rothnic/assura/pull/21",
                "headRefName": "candidate",
                "headRefOid": tip,
                "baseRefName": "master",
                "baseRefOid": original_base,
                "mergeCommit": {"oid": merge_oid},
                "mergedAt": "2026-10-17T00:00:00Z",
                "mergeStateStatus": "UNKNOWN",
                "reviewDecision": "APPROVED",
                "reviews": [{"id": "review-21", "author": {"login": "reviewer"}, "state": "APPROVED"}],
                "statusCheckRollup": [
                    {"__typename": "CheckRun", "databaseId": "job-21", "name": "process-contracts", "status": "COMPLETED", "detailsUrl": check_url, "conclusion": "SUCCESS"},
                    {"__typename": "CheckRun", "databaseId": "security-job-21", "name": "Security Audit", "status": "COMPLETED", "conclusion": "SKIPPED"},
                ],
            }
            fake_gh = repo / ".test-bin" / ("gh.py" if os.name == "nt" else "gh")
            write_fake_github(fake_gh, [provider_pr])

            evidence = local_integration_evidence(tip, merge_oid, "initial-close")
            evidence["review"].update(
                {"source": "github", "pr_number": 21, "reviewer": "reviewer", "review_id": "review-21", "evidence_ref": "https://github.com/rothnic/assura/pull/21#pullrequestreview-21"}
            )
            evidence["checks"].update(
                {"source": "github", "base_oid": original_base, "pr_number": 21, "run_id": "run-21", "job_id": "job-21", "check_name": "process-contracts", "evidence_ref": "https://github.com/rothnic/assura/actions/runs/run-21"}
            )
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "attached_branch_ref": "refs/heads/candidate",
                    "authority_ref": "test:delivery-authority",
                    "outcome": None,
                    "closure": "open",
                    "phase": "active",
                    "observed_tip": tip,
                    "registered_tip": tip,
                    "registered_base_oid": original_base,
                    "evidence": evidence,
                },
            )

            initial_receipt = store.read("candidate-1")
            bad_evidence = copy.deepcopy(initial_receipt["evidence"])
            bad_evidence["checks"]["base_oid"] = "b" * 40
            store.update(
                "candidate-1", initial_receipt["generation"], {"evidence": bad_evidence}
            )
            wrong_base_receipt = store.read("candidate-1")
            wrong_base_bytes = store.path_for("candidate-1").read_bytes()
            rejected = task_cli(repo, "delivery", "close", "01-01-candidate", "--outcome", "delivered")
            self.assertNotEqual(rejected.returncode, 0)
            self.assertEqual(store.path_for("candidate-1").read_bytes(), wrong_base_bytes)
            self.assertEqual(store.read("candidate-1")["generation"], wrong_base_receipt["generation"])
            store.update(
                "candidate-1",
                wrong_base_receipt["generation"],
                {"evidence": evidence},
            )

            closed = task_cli(repo, "delivery", "close", "01-01-candidate", "--outcome", "delivered")
            self.assertEqual(closed.returncode, 0, closed.stderr)
            self.assertEqual(store.read("candidate-1")["closure"], "verified")

            (repo / "unrelated.txt").write_text("advance\n", encoding="utf-8")
            git(repo, "add", "unrelated.txt")
            git(repo, "commit", "-m", "advance base")
            git(repo, "push", "origin", "master:refs/heads/master")
            git(repo, "update-ref", "refs/remotes/origin/master", git(repo, "rev-parse", "master"))
            intent = load_delivery_intent(repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json")
            inventory = collect_inventory(
                repo,
                github=FakeGithub([provider_pr]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            projected = classify_candidate(intent, inventory, store.read("candidate-1"))
            self.assertEqual(projected.outcome, "delivered", projected.as_dict())
        finally:
            directory.cleanup()

    def test_delivered_receipt_survives_unrelated_base_advance(self) -> None:
        repo, original_base, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")
            (repo / "unrelated.txt").write_text("unrelated\n", encoding="utf-8")
            git(repo, "add", "unrelated.txt")
            git(repo, "commit", "-m", "unrelated base advance")
            current_base = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")

            provider_pr = {
                "number": 21,
                "state": "MERGED",
                "headRefName": "candidate",
                "headRefOid": tip,
                "baseRefName": "master",
                "baseRefOid": original_base,
                "mergeCommit": {"oid": merge_oid},
                "mergeStateStatus": "CLEAN",
                "statusCheckRollup": [
                    {
                        "databaseId": "job-base-advance",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/run-base-advance/job/job-base-advance",
                        "conclusion": "SUCCESS",
                    }
                ],
            }
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            evidence = local_integration_evidence(tip, merge_oid, "base-advance")
            evidence["checks"].update(
                {
                    "source": "github",
                    "pr_number": 21,
                    "run_id": "run-base-advance",
                    "job_id": "job-base-advance",
                    "check_name": "process-contracts",
                    "evidence_ref": "https://github.com/rothnic/assura/actions/runs/run-base-advance",
                }
            )
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            receipt = store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "phase": "verified",
                    "outcome": "delivered",
                    "closure": "verified",
                    "observed_tip": tip,
                    "evidence": evidence,
                },
            )
            inventory = collect_inventory(
                repo,
                github=FakeGithub([provider_pr]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )

            self.assertEqual(inventory.base_oid, current_base)
            status = classify_candidate(intent, inventory, receipt)
            self.assertEqual(status.outcome, "delivered", status.issues)

            changed_checks_pr = {
                **provider_pr,
                "statusCheckRollup": [
                    {
                        "databaseId": "other-job",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/other-run/job/other-job",
                        "conclusion": "SUCCESS",
                    }
                ],
            }
            changed_checks_inventory = collect_inventory(
                repo,
                github=FakeGithub([changed_checks_pr]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            self.assertNotEqual(
                classify_candidate(intent, changed_checks_inventory, receipt).outcome,
                "delivered",
            )

            failed_checks_pr = {
                **provider_pr,
                "statusCheckRollup": [
                    {
                        "databaseId": "job-base-advance",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/run-base-advance/job/job-base-advance",
                        "conclusion": "FAILURE",
                    }
                ],
            }
            failed_checks_inventory = collect_inventory(
                repo,
                github=FakeGithub([failed_checks_pr]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            self.assertNotEqual(
                classify_candidate(intent, failed_checks_inventory, receipt).outcome,
                "delivered",
            )

            changed_tip = {**receipt, "observed_tip": "f" * 40}
            self.assertNotEqual(
                classify_candidate(intent, inventory, changed_tip).outcome,
                "delivered",
            )

            missing_postmerge = {**receipt, "evidence": {**evidence, "postmerge": None}}
            self.assertNotEqual(
                classify_candidate(intent, inventory, missing_postmerge).outcome,
                "delivered",
            )

            misbound_merge_evidence = {
                **evidence,
                "postmerge": {
                    **evidence["postmerge"],
                    "base_oid": original_base,
                    "merge_oid": original_base,
                },
            }
            misbound_merge = {**receipt, "evidence": misbound_merge_evidence}
            self.assertNotEqual(
                classify_candidate(intent, inventory, misbound_merge).outcome,
                "delivered",
            )

            misbound_checks_evidence = {
                **evidence,
                "checks": {**evidence["checks"], "base_oid": "f" * 40},
            }
            self.assertNotEqual(
                classify_candidate(
                    intent,
                    inventory,
                    {**receipt, "evidence": misbound_checks_evidence},
                ).outcome,
                "delivered",
            )

            active_receipt = {**receipt, "outcome": None}
            active_facts, _ = delivery_status._required_facts(
                intent, inventory, active_receipt, tip, "pr_merged", False,
                inventory.pull_requests[0],
            )
            self.assertFalse(active_facts["required_checks_pass"])
            self.assertFalse(active_facts["postmerge_verified"])

            git(repo, "checkout", "candidate")
            (repo / "candidate-follow-up.txt").write_text("tip changed\n", encoding="utf-8")
            git(repo, "add", "candidate-follow-up.txt")
            git(repo, "commit", "-m", "candidate tip changed after delivery")
            changed_tip_inventory = collect_inventory(
                repo,
                github=FakeGithub([provider_pr]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            self.assertNotEqual(
                classify_candidate(intent, changed_tip_inventory, receipt).outcome,
                "delivered",
            )
        finally:
            directory.cleanup()

    def test_unrelated_unavailable_worktree_does_not_block_candidate_coverage(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            missing_path = Path(directory.name) / "unrelated-missing"
            inventory = collect_inventory(
                repo, github=FakeGithub([]), base_ref="refs/heads/master"
            )
            inventory = replace(
                inventory,
                coverage={**inventory.coverage, "git": "partial", "worktrees": "partial", "complete": False},
                worktrees=inventory.worktrees
                + (
                    {
                        "path": str(missing_path),
                        "head_oid": "b" * 40,
                        "branch_ref": None,
                        "state": "missing",
                        "dirty": None,
                    },
                ),
                issues=inventory.issues
                + (
                    DeliveryIssue(
                        "WORKTREE_COVERAGE_UNAVAILABLE",
                        "worktree path is missing",
                        evidence_ref=str(missing_path),
                    ),
                ),
            )

            self.assertFalse(
                delivery_status._candidate_coverage_unknown(intent, inventory, tip)
            )
            self.assertFalse(inventory.coverage["complete"])
        finally:
            directory.cleanup()

    def test_candidate_unavailable_worktree_keeps_candidate_coverage_unknown(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            missing_path = Path(directory.name) / "candidate-missing"
            inventory = collect_inventory(
                repo, github=FakeGithub([]), base_ref="refs/heads/master"
            )
            inventory = replace(
                inventory,
                coverage={**inventory.coverage, "git": "partial", "worktrees": "partial", "complete": False},
                worktrees=inventory.worktrees
                + (
                    {
                        "path": str(missing_path),
                        "head_oid": tip,
                        "branch_ref": "refs/heads/candidate",
                        "state": "missing",
                        "dirty": None,
                    },
                ),
                issues=inventory.issues
                + (
                    DeliveryIssue(
                        "WORKTREE_COVERAGE_UNAVAILABLE",
                        "worktree path is missing",
                        evidence_ref=str(missing_path),
                    ),
                ),
            )

            self.assertTrue(
                delivery_status._candidate_coverage_unknown(intent, inventory, tip)
            )
        finally:
            directory.cleanup()

    def test_github_coverage_failure_blocks_local_ancestry_delivery(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_oid = git(repo, "rev-parse", "HEAD")
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            receipt = store.read("candidate-1")
            evidence = {
                "review": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:review-1",
                    "result": "approved",
                    "head_oid": tip,
                    "reviewer": "independent-reviewer",
                    "reviewer_role": "architect",
                    "review_id": "review-1",
                    "findings": [],
                },
                "checks": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "evidence_ref": "test:checks-1",
                    "result": "passed",
                    "conclusion": "success",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "repository": "rothnic/assura",
                    "run_id": "run-1",
                    "job_id": "process-contracts",
                },
                "postmerge": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "evidence_ref": "test:postmerge-1",
                    "result": "verified",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "merge_oid": merged_oid,
                    "repository": "rothnic/assura",
                },
                "acceptance": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "owner",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:acceptance-1",
                    "result": "accepted",
                    "head_oid": tip,
                    "acceptance_ref": "prd.md#acceptance",
                    "authority_ref": "test:delivery-authority",
                    "approved_by": "release-authority",
                    "artifact_sha256": "a" * 64,
                },
            }
            store.update(
                "candidate-1",
                receipt["generation"],
                {
                    "evidence": evidence,
                    "observed_tip": tip,
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                },
            )

            inventory = collect_inventory(
                repo,
                github=UnavailableGithub(),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(repo / task_path / "task.json")
            status = classify_candidate(intent, inventory, store.read("candidate-1"))

            self.assertEqual(status.integration, "ancestry_integrated")
            self.assertNotEqual(status.outcome, "delivered")
            self.assertTrue(
                any(issue.code == "GITHUB_COVERAGE_UNAVAILABLE" for issue in status.issues)
            )
            self.assertTrue(
                any(issue.code == "UNSUPPORTED_DELIVERY_CLAIM" for issue in status.issues)
            )
        finally:
            directory.cleanup()

    def test_release_receipt_must_match_current_tag_source(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].update(
                {
                    "kind": "release",
                    "version": "0.4.0",
                    "required_assets": list(RELEASE_ARCHIVES),
                }
            )
            task_json.write_text(json.dumps(task), encoding="utf-8")
            git(repo, "add", str(task_json.relative_to(repo)))
            git(repo, "commit", "-m", "record release intent")
            tip = git(repo, "rev-parse", "HEAD")
            git(repo, "tag", "v0.4.0")

            assets = [
                {
                    "name": name,
                    "size": 1,
                    "sha256": "a" * 64,
                    "checksum_name": f"{name}.sha256",
                    "members": [
                        "assura.exe" if name.endswith(".zip") else "assura",
                        "assura-full.exe" if name.endswith(".zip") else "assura-full",
                    ],
                }
                for name in RELEASE_ARCHIVES
            ]
            receipt = {
                "schema_version": "assura.release-receipt.v1",
                "repository": "rothnic/assura",
                "version": "0.4.0",
                "tag": "v0.4.0",
                "tag_oid": tip,
                "commit_oid": tip,
                "workflow_runs": [{"workflow": "Release", "run_id": "run-1"}],
                "assets": assets,
                "checksums_verified": True,
                "install": {
                    "verified": True,
                    "assura": "assura 0.4.0",
                    "assura-full": "assura 0.4.0",
                },
                "publish": {
                    "action": "verified",
                    "repository": "rothnic/assura",
                    "tag": "v0.4.0",
                    "tag_oid": tip,
                    "commit_oid": tip,
                    "verified_assets": list(RELEASE_ARCHIVES),
                },
            }
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "phase": "verified",
                    "outcome": "delivered",
                    "closure": "verified",
                    "observed_tip": tip,
                    "evidence": {"release": receipt},
                },
            )

            inventory = collect_inventory(repo, github=FakeGithub([]), base_ref="refs/heads/master")
            intent = load_delivery_intent(task_json)
            status = classify_candidate(intent, inventory, store.read("candidate-1"))
            self.assertNotEqual(status.outcome, "delivered")
            self.assertTrue(
                any(issue.code == "UNSUPPORTED_DELIVERY_CLAIM" for issue in status.issues)
            )
            self.assertTrue(delivery_status.validate_release_receipt(receipt, intent, tip))
            invalid_publication_ids = {
                "tag_oid missing": lambda publish: publish.pop("tag_oid"),
                "commit_oid missing": lambda publish: publish.pop("commit_oid"),
                "tag_oid malformed": lambda publish: publish.update(tag_oid="short"),
                "commit_oid malformed": lambda publish: publish.update(commit_oid="short"),
                "tag_oid mismatched": lambda publish: publish.update(tag_oid="1" * 40),
                "commit_oid mismatched": lambda publish: publish.update(commit_oid="1" * 40),
            }
            for label, mutate in invalid_publication_ids.items():
                with self.subTest(label=label):
                    candidate = copy.deepcopy(receipt)
                    mutate(candidate["publish"])
                    self.assertFalse(
                        delivery_status.validate_release_receipt(candidate, intent, tip)
                    )

            acceptance = {
                "schema_version": "assura.delivery-evidence.v1",
                "source": "owner",
                "repository": "rothnic/assura",
                "evidence_ref": "test:release-acceptance-1",
                "result": "accepted",
                "head_oid": tip,
                "acceptance_ref": "prd.md#acceptance",
                "authority_ref": "test:delivery-authority",
                "approved_by": "release-authority",
                "artifact_sha256": "b" * 64,
            }
            current_receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                current_receipt["generation"],
                {"evidence": {"release": receipt, "acceptance": acceptance}},
            )
            status = classify_candidate(intent, inventory, store.read("candidate-1"))
            self.assertEqual(status.outcome, "delivered")

            git(repo, "tag", "-f", "v0.4.0", "master")
            changed_inventory = collect_inventory(
                repo, github=FakeGithub([]), base_ref="refs/heads/master"
            )
            changed_status = classify_candidate(
                intent, changed_inventory, store.read("candidate-1")
            )
            self.assertNotEqual(changed_status.outcome, "delivered")
            self.assertTrue(
                any(issue.code == "UNSUPPORTED_DELIVERY_CLAIM" for issue in changed_status.issues)
            )
        finally:
            directory.cleanup()

    def test_active_verified_receipt_counts_as_explicit_child_outcome(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_tip = git(repo, "rev-parse", "HEAD")
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "attached_branch_ref": "refs/heads/candidate",
                    "authority_ref": "test:delivery-authority",
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                    "observed_tip": merged_tip,
                    "registered_tip": merged_tip,
                    "registered_base_oid": git(repo, "rev-parse", "HEAD^") ,
                    "evidence": {
                        "review": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:review-parent-progress",
                            "result": "approved",
                            "head_oid": merged_tip,
                            "reviewer": "independent-reviewer",
                            "reviewer_role": "architect",
                            "review_id": "review-parent-progress",
                            "findings": [],
                        },
                        "checks": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:checks-parent-progress",
                            "result": "passed",
                            "conclusion": "success",
                            "head_oid": merged_tip,
                            "base_oid": merged_tip,
                            "run_id": "run-parent-progress",
                            "job_id": "job-parent-progress",
                        },
                        "postmerge": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:postmerge-parent-progress",
                            "result": "verified",
                            "head_oid": merged_tip,
                            "base_oid": merged_tip,
                            "merge_oid": merged_tip,
                        },
                        "acceptance": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "owner",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:acceptance-parent-progress",
                            "result": "accepted",
                            "head_oid": merged_tip,
                            "acceptance_ref": "prd.md#acceptance",
                            "authority_ref": "test:delivery-authority",
                            "approved_by": "release-authority",
                            "artifact_sha256": "d" * 64,
                        },
                    },
                },
            )

            tasks_dir = repo / ".trellis" / "tasks"
            local_only_statuses = get_all_statuses(
                tasks_dir, github=FakeGithub([])
            )
            self.assertEqual(
                local_only_statuses["01-01-candidate"], "in_progress"
            )

            git(repo, "push", "origin", "master:refs/heads/master")
            statuses = get_all_statuses(
                tasks_dir, github=FakeGithub([]), refresh_remote=True
            )

            self.assertEqual(
                statuses["01-01-candidate"],
                "outcome:delivered",
            )
            self.assertIn(
                "1/1 delivered",
                children_progress(("01-01-candidate",), statuses),
            )

            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            task_bytes = task_json.read_bytes()
            receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                int(receipt["generation"]),
                {"authority_ref": "test:other-authority"},
            )
            mismatched_receipt_bytes = receipt_path.read_bytes()
            mismatched_generation = store.read("candidate-1")["generation"]
            refs_before = git(repo, "show-ref", "--heads")
            worktrees_before = git(repo, "worktree", "list", "--porcelain")

            statuses = get_all_statuses(
                repo / ".trellis" / "tasks", github=FakeGithub([])
            )

            self.assertNotEqual(statuses["01-01-candidate"], "outcome:delivered")
            progress = children_progress(("01-01-candidate",), statuses)
            self.assertIn("0/1 delivered", progress)
            self.assertIn("unknown=1", progress)
            self.assertEqual(task_json.read_bytes(), task_bytes)
            self.assertEqual(receipt_path.read_bytes(), mismatched_receipt_bytes)
            self.assertEqual(store.read("candidate-1")["generation"], mismatched_generation)
            self.assertEqual(git(repo, "show-ref", "--heads"), refs_before)
            self.assertEqual(
                git(repo, "worktree", "list", "--porcelain"), worktrees_before
            )
        finally:
            directory.cleanup()

    def test_active_task_status_wins_over_archived_name_collision(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            archive_task_dir = (
                repo / ".trellis" / "tasks" / "archive" / "01-01-candidate"
            )
            archive_task_dir.mkdir(parents=True)
            archived_task = json.loads(
                (
                    repo
                    / ".trellis"
                    / "tasks"
                    / "01-01-candidate"
                    / "task.json"
                ).read_text(encoding="utf-8")
            )
            archived_task["status"] = "completed"
            (archive_task_dir / "task.json").write_text(
                json.dumps(archived_task), encoding="utf-8"
            )

            statuses = get_all_statuses(
                repo / ".trellis" / "tasks", github=FakeGithub([])
            )

            self.assertEqual(statuses["01-01-candidate"], "in_progress")
        finally:
            directory.cleanup()

    def test_stale_terminal_receipt_does_not_count_as_child_outcome(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                    "observed_tip": tip,
                },
            )
            git(repo, "checkout", "candidate")
            (repo / "moved.txt").write_text("moved\n", encoding="utf-8")
            git(repo, "add", "moved.txt")
            git(repo, "commit", "-m", "move candidate tip")
            git(repo, "checkout", "master")

            statuses = get_all_statuses(
                repo / ".trellis" / "tasks", github=FakeGithub([])
            )

            self.assertNotEqual(
                statuses["01-01-candidate"],
                "outcome:delivered",
            )
            self.assertIn(
                "unknown=1",
                children_progress(("01-01-candidate",), statuses),
            )
        finally:
            directory.cleanup()

    def test_missing_child_is_unknown_not_done(self) -> None:
        progress = children_progress(("missing", "delivered"), {"delivered": "outcome:delivered"})

        self.assertIn("1/2 delivered", progress)
        self.assertIn("unknown=1", progress)

    def test_completed_label_without_verified_receipt_is_unknown(self) -> None:
        progress = children_progress(
            ("legacy-completed", "verified"),
            {"legacy-completed": "completed", "verified": "outcome:delivered"},
        )

        self.assertIn("1/2 delivered", progress)
        self.assertIn("unknown=1", progress)

    def test_inventory_reports_unmerged_candidate_without_inventing_delivery(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            inventory = collect_inventory(repo, github=FakeGithub([]), base_ref="refs/heads/master")
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            intent = load_delivery_intent(task_json)
            status = classify_candidate(intent, inventory)

            self.assertEqual(status.tip, tip)
            self.assertEqual(status.integration, "unmerged")
            self.assertIsNone(status.outcome)
            self.assertTrue(any(issue.code == "INTEGRATION_UNVERIFIED" for issue in status.issues))
        finally:
            directory.cleanup()

    def test_ambiguous_pull_requests_do_not_promote_a_candidate(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 21,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "master",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": tip},
                        },
                        {
                            "number": 22,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "master",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": tip},
                        },
                    ]
                ),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            status = classify_candidate(intent, inventory)

            self.assertNotEqual(status.integration, "pr_merged")
            self.assertTrue(
                any(issue.code == "PR_IDENTITY_AMBIGUOUS" for issue in status.issues)
            )
        finally:
            directory.cleanup()

    def test_pull_request_targeting_different_base_is_not_candidate(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 23,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "release",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": tip},
                        }
                    ]
                ),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )

            status = classify_candidate(intent, inventory)

            self.assertEqual(status.integration, "unmerged")
            self.assertTrue(
                any(issue.code == "INTEGRATION_UNVERIFIED" for issue in status.issues)
            )
        finally:
            directory.cleanup()

    def test_unavailable_worktree_is_a_coverage_issue(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            missing = Path(directory) / "missing-worktree"
            output = (
                f"worktree {missing}\n"
                + "HEAD "
                + "a" * 40
                + "\nbranch refs/heads/candidate\n\n"
            )
            with mock.patch(
                "common.delivery.run_git",
                return_value=(0, output, ""),
            ):
                records, issues = _read_worktrees(Path(directory))

            self.assertEqual(records[0]["state"], "missing")
            self.assertEqual(records[0]["dirty"], None)
            self.assertTrue(
                any(issue.code == "WORKTREE_COVERAGE_UNAVAILABLE" for issue in issues)
            )

    def test_inventory_serializes_the_task_json_path(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            inventory = collect_inventory(repo, github=FakeGithub([]), base_ref="refs/heads/master")

            task = next(item for item in inventory.tasks if item["id"] == "candidate")

            self.assertEqual(
                task["intent"]["task_json"],
                ".trellis/tasks/01-01-candidate/task.json",
            )
        finally:
            directory.cleanup()

    def test_inventory_uses_merge_commit_for_squash_merge(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--squash", "candidate")
            git(repo, "commit", "-m", "squash candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 12,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "master",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": merge_oid},
                        }
                    ]
                ),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json")
            status = classify_candidate(intent, inventory)

            self.assertEqual(status.integration, "pr_merged")
        finally:
            directory.cleanup()
