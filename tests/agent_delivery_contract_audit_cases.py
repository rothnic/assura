from __future__ import annotations

import json
from pathlib import Path
from unittest import mock

from agent_delivery_contract_support import (
    FakeGithub,
    fixture_repo,
    git,
    task_cli,
)
from common import delivery_status
from common.delivery import (
    CandidateStatus,
    classify_candidate,
    collect_inventory,
    load_delivery_intent,
    render_checkpoint,
)
from common.delivery_cli import project_audit
from common.delivery_store import DeliveryStore

class ProjectionAuditCases:
    def test_github_evidence_requires_provider_review_and_check_facts(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")
            provider_pr = {
                "number": 14,
                "state": "MERGED",
                "headRefName": "candidate",
                "headRefOid": tip,
                "baseRefName": "master",
                "baseRefOid": base_oid,
                "mergeCommit": {"oid": tip},
                "mergeStateStatus": "CLEAN",
                "reviewDecision": "APPROVED",
                "reviews": [
                    {
                        "id": "review-14",
                        "author": {"login": "independent-reviewer"},
                        "state": "APPROVED",
                    }
                ],
                "statusCheckRollup": [
                    {
                        "databaseId": "job-14",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/run-14/job/job-14",
                        "conclusion": "SUCCESS",
                    },
                    {
                        "databaseId": "security-job-14",
                        "name": "Security Audit",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/run-14/job/security-job-14",
                        "conclusion": "SKIPPED",
                    }
                ],
            }
            evidence = {
                "review": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "github",
                    "repository": "rothnic/assura",
                    "evidence_ref": "https://github.com/rothnic/assura/pull/14#pullrequestreview-14",
                    "result": "approved",
                    "head_oid": tip,
                    "pr_number": 14,
                    "reviewer": "independent-reviewer",
                    "reviewer_role": "architect",
                    "review_id": "review-14",
                    "findings": [],
                },
                "checks": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "github",
                    "repository": "rothnic/assura",
                    "evidence_ref": "https://github.com/rothnic/assura/actions/runs/run-14",
                    "result": "passed",
                    "conclusion": "success",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "pr_number": 14,
                    "run_id": "run-14",
                    "job_id": "job-14",
                    "check_name": "process-contracts",
                },
                "postmerge": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:postmerge-14",
                    "result": "verified",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "merge_oid": merged_oid,
                },
                "acceptance": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "owner",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:acceptance-14",
                    "result": "accepted",
                    "head_oid": tip,
                    "acceptance_ref": "prd.md#acceptance",
                    "authority_ref": "test:delivery-authority",
                    "approved_by": "release-authority",
                    "artifact_sha256": "c" * 64,
                },
            }
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
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

            status = classify_candidate(intent, inventory, store.read("candidate-1"))

            self.assertEqual(status.integration, "pr_merged")
            self.assertEqual(status.outcome, "delivered")

            changed_provider_pr = {**provider_pr, "reviewDecision": "CHANGES_REQUESTED"}
            changed_inventory = collect_inventory(
                repo,
                github=FakeGithub([changed_provider_pr]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            changed_status = classify_candidate(
                intent, changed_inventory, store.read("candidate-1")
            )

            self.assertNotEqual(changed_status.outcome, "delivered")
            self.assertTrue(
                any(
                    issue.code == "GITHUB_REVIEW_UNVERIFIED"
                    for issue in changed_status.issues
                )
            )

            changed_checks = {
                **provider_pr,
                "statusCheckRollup": [
                    {
                        "databaseId": "job-14",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/other-run/job/job-14",
                        "conclusion": "SUCCESS",
                    }
                ],
            }
            changed_checks_inventory = collect_inventory(
                repo,
                github=FakeGithub([changed_checks]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            changed_checks_status = classify_candidate(
                intent, changed_checks_inventory, store.read("candidate-1")
            )
            self.assertNotEqual(changed_checks_status.outcome, "delivered")
            self.assertTrue(
                any(
                    issue.code == "GITHUB_CHECKS_UNVERIFIED"
                    for issue in changed_checks_status.issues
                )
            )

            skipped_required = {**provider_pr, "mergeStateStatus": "BLOCKED"}
            skipped_required_inventory = collect_inventory(
                repo,
                github=FakeGithub([skipped_required]),
                base_ref="refs/heads/master",
                refresh_remote=True,
            )
            skipped_required_status = classify_candidate(
                intent, skipped_required_inventory, store.read("candidate-1")
            )
            self.assertNotEqual(skipped_required_status.outcome, "delivered")
            self.assertTrue(
                any(
                    issue.code == "GITHUB_CHECKS_UNVERIFIED"
                    for issue in skipped_required_status.issues
                )
            )

            unknown_pr = {
                "number": 14,
                "head_oid": tip,
                "merge_state_status": "unknown",
                "merged": True,
                "merge_oid": merged_oid,
                "base_oid": base_oid,
            }
            evidenced_success = {
                "run_id": "run-14",
                "job_id": "job-14",
                "name": "process-contracts",
                "conclusion": "SUCCESS",
            }
            optional_skip = {
                "run_id": "run-14",
                "job_id": "security-job-14",
                "name": "Security Audit",
                "conclusion": "SKIPPED",
            }

            def accepted(checks: list[dict], pr: dict = unknown_pr, check_evidence: dict = evidence["checks"]) -> bool:
                return delivery_status._github_checks_match_pr(
                    check_evidence, {**pr, "checks": checks}
                )

            self.assertTrue(
                accepted([evidenced_success, optional_skip]),
                "a non-evidenced conditional skip is allowed with exact merged context",
            )
            self.assertFalse(accepted([optional_skip]), "all-skipped rollups lack a successful evidenced job")
            self.assertFalse(
                accepted(
                    [optional_skip],
                    check_evidence={**evidence["checks"], "job_id": "security-job-14", "check_name": "Security Audit"},
                ),
                "the evidenced job itself must succeed",
            )
            self.assertFalse(
                accepted(
                    [evidenced_success, {**optional_skip, "conclusion": "PENDING"}],
                ),
                "pending checks remain unresolved",
            )
            self.assertFalse(accepted([evidenced_success], {**unknown_pr, "merged": False}))
            self.assertFalse(accepted([evidenced_success], {**unknown_pr, "merge_oid": "short"}))
            self.assertFalse(accepted([evidenced_success], {**unknown_pr, "base_oid": None}))
            self.assertFalse(accepted([evidenced_success], {**unknown_pr, "number": 15}))
            self.assertFalse(accepted([evidenced_success], {**unknown_pr, "head_oid": "e" * 40}))
        finally:
            directory.cleanup()

    def test_post_merge_commit_after_reviewed_head_remains_outstanding(self) -> None:
        repo, base_oid, reviewed_tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--squash", "candidate")
            git(repo, "commit", "-m", "squash candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "checkout", "candidate")
            (repo / "follow-up.txt").write_text("post-review\n", encoding="utf-8")
            git(repo, "add", "follow-up.txt")
            git(repo, "commit", "-m", "post-review change")
            current_tip = git(repo, "rev-parse", "HEAD")
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 13,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": reviewed_tip,
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

            self.assertEqual(status.tip, current_tip)
            self.assertEqual(status.integration, "post_review_commits")
            self.assertTrue(any(issue.code == "INTEGRATION_UNVERIFIED" for issue in status.issues))
        finally:
            directory.cleanup()

    def test_checkpoint_is_deterministic_and_bounded(self) -> None:
        statuses = [
            CandidateStatus(
                candidate_id=f"candidate-{index}",
                owner="tester",
                kind="integration",
                task_path=None,
                tip=None,
                phase="held",
                outcome=None,
                integration="unknown",
                dirty=False,
                closure="open",
                next_action="resolve ownership",
            )
            for index in range(20)
        ]

        first = render_checkpoint(statuses)
        second = render_checkpoint(list(reversed(statuses)))

        self.assertEqual(first, second)
        self.assertLessEqual(len(first.encode("utf-8")), 1024)
        self.assertGreaterEqual(len(first.splitlines()), 3)
        self.assertLessEqual(len(first.splitlines()), 6)

    def test_repeated_audit_is_identical_and_does_not_write(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            before = sorted(
                path.relative_to(repo).as_posix()
                for path in repo.rglob("*")
                if path.is_file()
            )
            first, _ = project_audit(repo, github=FakeGithub([]))
            second, _ = project_audit(repo, github=FakeGithub([]))
            after = sorted(
                path.relative_to(repo).as_posix()
                for path in repo.rglob("*")
                if path.is_file()
            )

            self.assertEqual(first, second)
            self.assertEqual(before, after)
        finally:
            directory.cleanup()

    def test_audit_next_checkpoint_validate_bindings_on_declared_branch_read_only(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "-b", "coordinator", "master")

            valid_audit = task_cli(
                repo, "delivery", "audit", "--owner", "tester", "--format", "json"
            )
            self.assertEqual(valid_audit.returncode, 0, valid_audit.stderr)
            valid_report = json.loads(valid_audit.stdout)
            candidate = next(
                item for item in valid_report["candidates"]
                if item["candidate_id"] == "candidate-1"
            )
            self.assertFalse(
                any(
                    "RECEIPT_BINDING_CONFLICT" in issue["message"]
                    for issue in candidate["issues"]
                )
            )

            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            task_json = repo / task_path / "task.json"
            store = DeliveryStore(receipt_path.parent)
            receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                int(receipt["generation"]),
                {"authority_ref": "test:other-authority"},
            )
            receipt_bytes = receipt_path.read_bytes()
            task_bytes = task_json.read_bytes()
            generation = store.read("candidate-1")["generation"]
            refs_before = git(repo, "show-ref", "--heads")
            worktrees_before = git(repo, "worktree", "list", "--porcelain")

            projections = (
                (
                    "audit",
                    ("delivery", "audit", "--owner", "tester", "--format", "json"),
                ),
                (
                    "next",
                    ("delivery", "next", "--owner", "tester", "--format", "json"),
                ),
                (
                    "checkpoint",
                    ("delivery", "checkpoint", "--owner", "tester"),
                ),
            )
            for name, arguments in projections:
                with self.subTest(projection=name):
                    result = task_cli(repo, *arguments)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    if name == "checkpoint":
                        self.assertIn("candidate-1: held", result.stdout)
                        self.assertNotIn("terminal: 1", result.stdout)
                    else:
                        rows = json.loads(result.stdout)
                        if name == "audit":
                            rows = rows["candidates"]
                        candidate = next(
                            item for item in rows
                            if item["candidate_id"] == "candidate-1"
                        )
                        self.assertTrue(
                            any(
                                "RECEIPT_BINDING_CONFLICT" in issue["message"]
                                for issue in candidate["issues"]
                            ),
                            result.stdout,
                        )
                    self.assertEqual(task_json.read_bytes(), task_bytes)
                    self.assertEqual(receipt_path.read_bytes(), receipt_bytes)
                    self.assertEqual(store.read("candidate-1")["generation"], generation)
                    self.assertEqual(git(repo, "show-ref", "--heads"), refs_before)
                    self.assertEqual(
                        git(repo, "worktree", "list", "--porcelain"),
                        worktrees_before,
                    )
        finally:
            directory.cleanup()

    def test_aggregate_children_reject_misbound_receipt_before_classification(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            child_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", child_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            parent_dir = repo / ".trellis" / "tasks" / "01-02-aggregate"
            parent_dir.mkdir(parents=True)
            parent_json = parent_dir / "task.json"
            parent_json.write_text(
                json.dumps(
                    {
                        "id": "aggregate-parent",
                        "name": "aggregate-parent",
                        "status": "in_progress",
                        "branch": "refs/heads/aggregate-parent",
                        "base_branch": "master",
                        "children": ["candidate"],
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "aggregate",
                                "candidate_id": "aggregate-parent",
                                "owner": "tester",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/heads/master",
                                "branch_ref": "refs/heads/aggregate-parent",
                                "authority_ref": "test:parent-authority",
                            }
                        },
                    }
                ),
                encoding="utf-8",
            )
            inventory = collect_inventory(repo, github=FakeGithub([]))
            parent_intent = load_delivery_intent(parent_json)
            delivered_child = CandidateStatus(
                candidate_id="candidate-1",
                owner="tester",
                kind="integration",
                task_path=child_path,
                tip=None,
                phase="verified",
                outcome="delivered",
                integration="pr_merged",
                dirty=False,
                closure="verified",
                next_action="",
            )

            with mock.patch.object(
                delivery_status,
                "classify_candidate",
                return_value=delivered_child,
            ) as classify:
                self.assertTrue(
                    delivery_status._aggregate_children_satisfied(
                        parent_intent, inventory
                    )
                )
                classify.assert_called_once()

            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            task_json = repo / child_path / "task.json"
            store = DeliveryStore(receipt_path.parent)
            receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                int(receipt["generation"]),
                {"authority_ref": "test:other-authority"},
            )
            receipt_bytes = receipt_path.read_bytes()
            task_bytes = task_json.read_bytes()
            generation = store.read("candidate-1")["generation"]
            refs_before = git(repo, "show-ref", "--heads")
            worktrees_before = git(repo, "worktree", "list", "--porcelain")

            with mock.patch.object(
                delivery_status,
                "classify_candidate",
                return_value=delivered_child,
            ) as classify:
                self.assertFalse(
                    delivery_status._aggregate_children_satisfied(
                        parent_intent, inventory
                    )
                )
                classify.assert_not_called()

            self.assertEqual(task_json.read_bytes(), task_bytes)
            self.assertEqual(receipt_path.read_bytes(), receipt_bytes)
            self.assertEqual(store.read("candidate-1")["generation"], generation)
            self.assertEqual(git(repo, "show-ref", "--heads"), refs_before)
            self.assertEqual(
                git(repo, "worktree", "list", "--porcelain"), worktrees_before
            )
        finally:
            directory.cleanup()

    def test_audit_reports_all_unowned_local_branches_and_orphan_receipts(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            git(repo, "branch", "unusual/topic")
            orphan_store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            orphan_store.create("orphan-receipt", {"phase": "held"})

            report, _ = project_audit(repo, github=FakeGithub([]))

            names = {item["name"] for item in report["unowned_refs"]}
            self.assertIn("refs/heads/unusual/topic", names)
            self.assertEqual(report["orphan_receipts"][0]["candidate_id"], "orphan-receipt")
            self.assertEqual(report["orphan_receipts"][0]["disposition"], "unresolved")
            self.assertTrue(all("disposition" in item for item in report["unowned_worktrees"]))
        finally:
            directory.cleanup()

    def test_audit_binds_declared_execution_branch_refs_to_the_candidate(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task.setdefault("meta", {})["execution_branches"] = ["execution-lane"]
            task_json.write_text(json.dumps(task), encoding="utf-8")
            git(repo, "branch", "execution-lane", "candidate")

            report, _ = project_audit(repo, github=FakeGithub([]))

            self.assertFalse(
                any(
                    item["name"] == "refs/heads/execution-lane"
                    for item in report["unowned_refs"]
                )
            )
            bindings = [
                item
                for item in report["execution_bindings"]
                if item["branch"] == "execution-lane"
            ]
            self.assertEqual(len(bindings), 1)
            self.assertEqual(bindings[0]["candidate_id"], "candidate-1")
            self.assertEqual(bindings[0]["owner"], "tester")
            self.assertEqual(bindings[0]["task_path"], ".trellis/tasks/01-01-candidate")
        finally:
            directory.cleanup()

    def test_audit_keeps_dirty_execution_worktree_as_an_owned_hold(self) -> None:
        repo, _, _, directory = fixture_repo()
        execution = Path(directory.name) / "execution-lane"
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task.setdefault("meta", {})["execution_branches"] = ["execution-lane"]
            task_json.write_text(json.dumps(task), encoding="utf-8")
            git(repo, "worktree", "add", "-b", "execution-lane", str(execution), "candidate")
            (execution / "preserve.txt").write_text("uncommitted\n", encoding="utf-8")

            report, statuses = project_audit(repo, github=FakeGithub([]))

            status = next(item for item in statuses if item.candidate_id == "candidate-1")
            self.assertTrue(status.dirty)
            self.assertTrue(
                any(issue.code == "OWNED_EXECUTION_WORKTREE_DIRTY" for issue in status.issues)
            )
            binding = next(
                item
                for item in report["execution_bindings"]
                if item["branch"] == "execution-lane"
            )
            self.assertEqual(
                Path(binding["worktrees"][0]["path"]).resolve(), execution.resolve()
            )
            self.assertTrue(binding["worktrees"][0]["dirty"])
            self.assertEqual(binding["worktrees"][0]["disposition"], "held")
            self.assertFalse(
                any(
                    item.get("path") == str(execution)
                    for item in report["unowned_worktrees"]
                )
            )
        finally:
            if execution.is_dir():
                git(repo, "worktree", "remove", "--force", str(execution))
            directory.cleanup()

    def test_audit_holds_duplicate_candidate_identity_across_tasks(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            duplicate_dir = repo / ".trellis" / "tasks" / "01-02-duplicate"
            duplicate_dir.mkdir(parents=True)
            duplicate_task = json.loads(
                (
                    repo
                    / ".trellis"
                    / "tasks"
                    / "01-01-candidate"
                    / "task.json"
                ).read_text(encoding="utf-8")
            )
            duplicate_task["id"] = "duplicate"
            duplicate_task["name"] = "duplicate"
            (duplicate_dir / "task.json").write_text(
                json.dumps(duplicate_task), encoding="utf-8"
            )

            report, statuses = project_audit(repo, github=FakeGithub([]))

            duplicate_statuses = [
                status
                for status in statuses
                if status.candidate_id == "candidate-1"
            ]
            self.assertEqual(len(duplicate_statuses), 2)
            self.assertTrue(
                all(
                    any(issue.code == "DUPLICATE_CANDIDATE_ID" for issue in status.issues)
                    and status.outcome is None
                    for status in duplicate_statuses
                )
            )
            self.assertEqual(report["invalid_intents"], [])
        finally:
            directory.cleanup()

    def test_audit_keeps_detached_worktree_and_invalid_intent_visible(self) -> None:
        repo, _, _, directory = fixture_repo()
        detached = Path(directory.name) / "detached-worktree"
        try:
            git(repo, "worktree", "add", "--detach", str(detached), "master")
            (detached / "detached.txt").write_text("detached\n", encoding="utf-8")
            git(detached, "add", "detached.txt")
            git(detached, "commit", "-m", "detached candidate")

            invalid_dir = repo / ".trellis" / "tasks" / "01-02-invalid"
            invalid_dir.mkdir(parents=True)
            (invalid_dir / "task.json").write_text(
                json.dumps(
                    {
                        "id": "invalid",
                        "assignee": "tester",
                        "meta": {"delivery": {"schema_version": 1, "kind": "release"}},
                    }
                ),
                encoding="utf-8",
            )

            report, _ = project_audit(repo, github=FakeGithub([]))

            detached_rows = [
                item for item in report["unowned_worktrees"] if item.get("detached")
            ]
            self.assertEqual(len(detached_rows), 1)
            self.assertEqual(detached_rows[0]["disposition"], "unresolved")
            self.assertEqual(len(report["invalid_intents"]), 1)
            self.assertEqual(
                report["invalid_intents"][0]["issues"][0]["code"],
                "DELIVERY_INTENT_INVALID",
            )
        finally:
            git(repo, "worktree", "remove", str(detached))
            directory.cleanup()

    def test_owned_detached_worktree_dirt_blocks_candidate_and_is_not_dropped(self) -> None:
        repo, _, tip, directory = fixture_repo()
        detached = Path(directory.name) / "owned-detached-worktree"
        try:
            git(repo, "worktree", "add", "--detach", str(detached), tip)
            (detached / "uncommitted.txt").write_text("preserve\n", encoding="utf-8")

            report, statuses = project_audit(repo, github=FakeGithub([]))

            status = next(item for item in statuses if item.candidate_id == "candidate-1")
            self.assertTrue(status.dirty)
            self.assertTrue(
                any(
                    item.get("path")
                    and Path(item["path"]).resolve() == detached.resolve()
                    and item.get("dirty") is True
                    for item in report["inventory"]["worktrees"]
                )
            )
            self.assertFalse(
                any(
                    item.get("path")
                    and Path(item["path"]).resolve() == detached.resolve()
                    for item in report["unowned_worktrees"]
                )
            )
        finally:
            git(repo, "worktree", "remove", "--force", str(detached))
            directory.cleanup()

    def test_audit_marks_tracking_only_remote_refs_as_incomplete(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            report, _ = project_audit(repo, github=FakeGithub([]))

            self.assertEqual(report["coverage"]["remote_refs"], "local-tracking-only")
            self.assertFalse(report["coverage"]["complete"])
        finally:
            directory.cleanup()

    def test_audit_classifies_missing_delivery_intent_as_legacy(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            legacy_dir = repo / ".trellis" / "tasks" / "01-02-legacy"
            legacy_dir.mkdir(parents=True)
            (legacy_dir / "task.json").write_text(
                json.dumps(
                    {
                        "id": "legacy",
                        "assignee": "tester",
                        "status": "completed",
                    }
                ),
                encoding="utf-8",
            )

            report, _ = project_audit(repo, github=FakeGithub([]))

            self.assertEqual(report["invalid_intents"], [])
            self.assertEqual(len(report["legacy"]), 1)
            self.assertEqual(
                report["legacy"][0]["issues"][0]["code"],
                "LEGACY_UNCLASSIFIED",
            )
        finally:
            directory.cleanup()

    def test_explicit_remote_refresh_proves_remote_ref_coverage(self) -> None:
        repo, _, _, directory = fixture_repo()
        advertised = [{"name": "refs/heads/master", "oid": "a" * 40, "kind": "heads"}]
        try:
            with mock.patch(
                "common.delivery._read_remote_refs",
                return_value=(advertised, []),
            ):
                report, _ = project_audit(
                    repo, github=FakeGithub([]), refresh_remote=True
                )

            self.assertEqual(report["coverage"]["remote_refs"], "verified")
            self.assertTrue(report["coverage"]["complete"])
            self.assertEqual(report["inventory"]["remote_refs"], advertised)
        finally:
            directory.cleanup()

    def test_remote_advertised_candidate_without_local_branch_remains_identifiable(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "branch", "-D", "candidate")
            advertised = [
                {"name": "refs/heads/master", "oid": base_oid, "kind": "heads"},
                {"name": "refs/heads/candidate", "oid": tip, "kind": "heads"},
            ]
            with mock.patch(
                "common.delivery._read_remote_refs",
                return_value=(advertised, []),
            ):
                inventory = collect_inventory(
                    repo,
                    github=FakeGithub(
                        [
                            {
                                "number": 31,
                                "state": "OPEN",
                                "headRefName": "candidate",
                                "headRefOid": tip,
                                "baseRefName": "master",
                                "baseRefOid": base_oid,
                            }
                        ]
                    ),
                    base_ref="refs/heads/master",
                    refresh_remote=True,
                )
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )

            status = classify_candidate(intent, inventory)

            self.assertEqual(status.tip, tip)
            self.assertEqual(status.integration, "pr_open")
            self.assertFalse(
                any(issue.code == "CANDIDATE_TIP_UNRESOLVED" for issue in status.issues)
            )
        finally:
            directory.cleanup()
