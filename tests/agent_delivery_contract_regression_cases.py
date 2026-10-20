from __future__ import annotations

import json
import tempfile
from pathlib import Path
from unittest import mock

from agent_delivery_contract_support import (
    FakeGithub,
    fixture_repo,
    git,
    task_cli,
)
from common import delivery_cli, delivery_status
from common.delivery import DeliveryValidationError, collect_inventory, load_delivery_intent
from common.delivery_cli import project_audit
from common.delivery_store import DeliveryStore, ReceiptBusy

class RepairRegressionCases:
    def test_register_rejects_raw_non_branch_base_refs_without_mutation(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        for index, raw_base in enumerate(
            ("HEAD", "candidate~1", "missing-branch", "refs/tags/v0.4.0")
        ):
            with self.subTest(base_ref=raw_base):
                repo, _, _, directory = fixture_repo()
                try:
                    task_json = repo / task_path / "task.json"
                    task = json.loads(task_json.read_text(encoding="utf-8"))
                    task["meta"].pop("delivery")
                    task_json.write_text(json.dumps(task), encoding="utf-8")
                    before_task = task_json.read_bytes()
                    candidate_id = f"invalid-base-{index}"

                    result = task_cli(
                        repo,
                        "delivery",
                        "register",
                        task_path,
                        "--candidate",
                        candidate_id,
                        "--owner",
                        "tester",
                        "--base-ref",
                        raw_base,
                    )

                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn("BASE_REF_INVALID", result.stdout + result.stderr)
                    self.assertEqual(task_json.read_bytes(), before_task)
                    self.assertFalse(
                        (repo / ".git/assura/delivery-v1" / f"{candidate_id}.json").exists()
                    )
                finally:
                    directory.cleanup()

    def test_legacy_registration_checks_repository_before_receipt_repair(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"]["repository"] = "elsewhere/assura"
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git/assura/delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "elsewhere/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )
            receipt_path = store.path_for("candidate-1")
            before_receipt = receipt_path.read_bytes()

            result = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
                "--authority-ref",
                "test:delivery-authority",
            )

            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIn("REPOSITORY_IDENTITY_MISMATCH", result.stdout + result.stderr)
            self.assertEqual(task_json.read_bytes(), before_task)
            self.assertEqual(receipt_path.read_bytes(), before_receipt)
            self.assertEqual(store.read("candidate-1")["generation"], 0)
        finally:
            directory.cleanup()

    def test_detached_fresh_registration_requires_exact_declared_branch_tip(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        for label, detached_oid, should_register in (
            ("different-tip", "base", False),
            ("matching-tip", "candidate", True),
        ):
            with self.subTest(binding=label):
                repo, base_oid, tip, directory = fixture_repo()
                try:
                    task_json = repo / task_path / "task.json"
                    task = json.loads(task_json.read_text(encoding="utf-8"))
                    task["meta"].pop("delivery")
                    task_json.write_text(json.dumps(task), encoding="utf-8")
                    before_task = task_json.read_bytes()
                    actual_oid = base_oid if detached_oid == "base" else tip
                    git(repo, "checkout", "--detach", actual_oid)
                    candidate_id = f"detached-{label}"

                    result = task_cli(
                        repo,
                        "delivery",
                        "register",
                        task_path,
                        "--candidate",
                        candidate_id,
                        "--owner",
                        "tester",
                    )

                    receipt_path = repo / ".git/assura/delivery-v1" / f"{candidate_id}.json"
                    if should_register:
                        self.assertEqual(result.returncode, 0, result.stderr)
                        receipt = DeliveryStore(receipt_path.parent).read(candidate_id)
                        self.assertEqual(receipt["attached_branch_ref"], "refs/heads/candidate")
                        self.assertEqual(receipt["registered_tip"], tip)
                        self.assertEqual(git(repo, "rev-parse", "HEAD"), tip)
                    else:
                        self.assertNotEqual(result.returncode, 0, result.stdout)
                        self.assertIn("BRANCH_BINDING_UNPROVEN", result.stdout + result.stderr)
                        self.assertEqual(task_json.read_bytes(), before_task)
                        self.assertFalse(receipt_path.exists())
                finally:
                    directory.cleanup()

    def test_fresh_registration_requires_exact_attached_ref_or_equal_oid_alias(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        for label, declared_ref, declared_oid, should_register in (
            ("divergent-remote", "refs/remotes/upstream/candidate", "base", False),
            ("equal-origin-alias", "refs/remotes/origin/candidate", "tip", True),
        ):
            with self.subTest(binding=label):
                repo, base_oid, tip, directory = fixture_repo()
                try:
                    task_json = repo / task_path / "task.json"
                    task = json.loads(task_json.read_text(encoding="utf-8"))
                    task["meta"].pop("delivery")
                    task["branch"] = declared_ref
                    task_json.write_text(json.dumps(task), encoding="utf-8")
                    remote_oid = base_oid if declared_oid == "base" else tip
                    remote_name = declared_ref.removeprefix("refs/remotes/")
                    git(repo, "update-ref", f"refs/remotes/{remote_name}", remote_oid)
                    candidate_id = f"attached-{label}"
                    before_task = task_json.read_bytes()
                    receipt_path = (
                        repo / ".git" / "assura" / "delivery-v1" / f"{candidate_id}.json"
                    )

                    if should_register:
                        intent = delivery_cli.register_task(
                            repo, task_json, candidate_id, "tester"
                        )
                        self.assertEqual(intent.branch_ref, declared_ref)
                        self.assertTrue(receipt_path.is_file())
                    else:
                        with self.assertRaisesRegex(
                            DeliveryValidationError, "BRANCH_BINDING_UNPROVEN"
                        ):
                            delivery_cli.register_task(
                                repo, task_json, candidate_id, "tester"
                            )
                        self.assertEqual(task_json.read_bytes(), before_task)
                        self.assertFalse(receipt_path.exists())
                finally:
                    directory.cleanup()

    def test_audit_keeps_divergent_remote_refs_distinct_from_local_candidate(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            (repo / "foreign-ref.txt").write_text("foreign\n", encoding="utf-8")
            git(repo, "add", "foreign-ref.txt")
            git(repo, "commit", "-m", "foreign candidate ref")
            foreign_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "checkout", "candidate")
            git(repo, "update-ref", "refs/remotes/origin/candidate", base_oid)
            git(repo, "update-ref", "refs/remotes/upstream/candidate", foreign_oid)

            report, statuses = project_audit(repo, github=FakeGithub([]))

            unowned = {row["name"]: row for row in report["unowned_refs"]}
            self.assertEqual(unowned["refs/remotes/origin/candidate"]["oid"], base_oid)
            self.assertEqual(
                unowned["refs/remotes/upstream/candidate"]["oid"], foreign_oid
            )
            for ref_name in (
                "refs/remotes/origin/candidate",
                "refs/remotes/upstream/candidate",
            ):
                self.assertEqual(unowned[ref_name]["disposition"], "unresolved")
                self.assertTrue(unowned[ref_name]["next_action"])
            candidate = next(
                status for status in statuses if status.candidate_id == "candidate-1"
            )
            self.assertEqual(candidate.tip, tip)
            self.assertNotIn(
                "CANDIDATE_REF_AMBIGUOUS",
                {issue.code for issue in candidate.issues},
            )
            self.assertNotEqual(base_oid, tip)
            self.assertNotEqual(foreign_oid, tip)
        finally:
            directory.cleanup()

    def test_registration_serializes_distinct_candidate_ids_for_one_task(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis/tasks/01-01-candidate/task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"].pop("delivery")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            store = DeliveryStore(repo / ".git/assura/delivery-v1")
            original_ensure = delivery_cli.ensure_receipt
            attempted = False
            competing_errors: list[ReceiptBusy] = []

            def race_ensure(
                repo_root: Path,
                intent: object,
                data: dict,
                phase: str = "registered",
            ) -> dict:
                nonlocal attempted
                if not attempted:
                    attempted = True
                    try:
                        delivery_cli.register_task(
                            repo_root, task_json, "candidate-second", "tester"
                        )
                    except ReceiptBusy as error:
                        competing_errors.append(error)
                return original_ensure(repo_root, intent, data, phase)

            with mock.patch.object(delivery_cli, "ensure_receipt", side_effect=race_ensure):
                delivery_cli.register_task(repo, task_json, "candidate-first", "tester")

            stored_task = json.loads(task_json.read_text(encoding="utf-8"))
            self.assertEqual(
                stored_task["meta"]["delivery"]["candidate_id"], "candidate-first"
            )
            self.assertEqual(len(competing_errors), 1)
            self.assertTrue(store.path_for("candidate-first").is_file())
            self.assertFalse(store.path_for("candidate-second").exists())
        finally:
            directory.cleanup()

    def test_local_codex_audit_is_not_independent_review_evidence(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_oid = git(repo, "rev-parse", "HEAD")
            task_json = repo / ".trellis/tasks/01-01-candidate/task.json"
            intent = load_delivery_intent(task_json)
            inventory = collect_inventory(
                repo, github=FakeGithub([]), base_ref="refs/heads/master"
            )
            evidence = {
                "review": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "issue-comment:338-audit-fallback",
                    "result": "approved",
                    "head_oid": tip,
                    "reviewer": "codex-local-audit",
                    "reviewer_role": "audit-fallback",
                    "review_id": "local-audit-338",
                    "findings": [],
                },
                "checks": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:checks-338",
                    "result": "passed",
                    "conclusion": "success",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "run_id": "run-338",
                    "job_id": "process-contracts",
                },
                "postmerge": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:postmerge-338",
                    "result": "verified",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "merge_oid": merged_oid,
                },
                "acceptance": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "owner",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:acceptance-338",
                    "result": "accepted",
                    "head_oid": tip,
                    "acceptance_ref": "prd.md#acceptance",
                    "authority_ref": "test:delivery-authority",
                    "approved_by": "release-authority",
                    "artifact_sha256": "d" * 64,
                },
            }

            facts, issues = delivery_status._required_facts(
                intent,
                inventory,
                {"evidence": evidence},
                tip,
                "ancestry_integrated",
                False,
                None,
            )

            self.assertFalse(facts["review_resolved"])
            self.assertTrue(facts["required_checks_pass"])
            self.assertTrue(facts["postmerge_verified"])
            self.assertTrue(facts["acceptance_verified"])
            self.assertIn(
                "REVIEW_PROVENANCE_UNVERIFIED", {issue.code for issue in issues}
            )
        finally:
            directory.cleanup()

    def test_github_review_requires_matching_provider_id_and_author(self) -> None:
        provider_url = "https://github.com/rothnic/assura/pull/14#pullrequestreview-14"
        pull_request = {
            "number": 14,
            "head_oid": "a" * 40,
            "review_decision": "approved",
            "reviews": [
                {
                    "id": "review-14",
                    "url": provider_url,
                    "author": None,
                    "state": "approved",
                }
            ],
        }
        evidence = {
            "source": "github",
            "pr_number": 14,
            "head_oid": "a" * 40,
            "review_id": "review-14",
            "review_url": provider_url,
            "reviewer": "independent-reviewer",
        }

        with self.subTest(missing_provider_author=True):
            self.assertFalse(
                delivery_status._github_review_matches_pr(evidence, pull_request),
                "a missing provider author cannot corroborate an approved review",
            )

        pull_request["reviews"][0]["author"] = "independent-reviewer"
        wrong_id = {**evidence, "review_id": "stale-review-id"}
        self.assertFalse(
            delivery_status._github_review_matches_pr(wrong_id, pull_request),
            "a matching URL cannot substitute for the exact provider review ID",
        )

class RemoteFreshnessCases:
    def test_close_and_archive_reject_stale_origin_tracking_base(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        for action, terminal in (("close", False), ("archive", True)):
            with self.subTest(action=action):
                repo, directory, task_json, store = self._stale_tracking_candidate(terminal)
                try:
                    self.assertEqual(
                        git(repo, "rev-parse", "refs/remotes/origin/master"),
                        git(repo, "rev-parse", "refs/heads/candidate"),
                    )
                    self.assertEqual(
                        git(repo, "ls-remote", "origin", "refs/heads/master").split()[0],
                        git(repo, "rev-parse", "refs/heads/master"),
                    )
                    before_task = task_json.read_bytes()
                    receipt_path = store.path_for("candidate-1")
                    before_receipt = receipt_path.read_bytes()
                    if action == "close":
                        result = task_cli(
                            repo,
                            "delivery",
                            "close",
                            task_path,
                            "--outcome",
                            "delivered",
                        )
                    else:
                        result = task_cli(repo, "archive", task_path, "--no-commit")

                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn(
                        "REMOTE_BASE_REF_STALE", result.stdout + result.stderr
                    )
                    self.assertEqual(task_json.read_bytes(), before_task)
                    self.assertEqual(receipt_path.read_bytes(), before_receipt)
                    self.assertTrue(task_json.is_file())
                finally:
                    directory.cleanup()

class StaleBaseCases:
    def test_stale_master_worktree_is_unresolved_until_oid_matches_configured_base(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temp_root = Path(directory)
            repo = temp_root / "repo"
            repo.mkdir()
            git(repo, "init", "-b", "master")
            git(repo, "config", "user.email", "delivery@example.test")
            git(repo, "config", "user.name", "Delivery Test")
            (repo / "README.md").write_text("local master\n", encoding="utf-8")
            git(repo, "add", "README.md")
            git(repo, "commit", "-m", "local master")
            stale_master_oid = git(repo, "rev-parse", "HEAD")

            git(repo, "checkout", "-b", "integration-base")
            (repo / "README.md").write_text("configured base\n", encoding="utf-8")
            git(repo, "commit", "-am", "configured base")
            configured_base_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "remote", "add", "origin", "https://github.com/test/repo.git")
            git(repo, "update-ref", "refs/remotes/origin/master", configured_base_oid)

            stale_worktree = temp_root / "stale-master"
            git(repo, "worktree", "add", str(stale_worktree), "master")

            inventory = collect_inventory(repo, github=FakeGithub([]))
            self.assertEqual(inventory.base_ref, "origin/master")
            self.assertEqual(inventory.base_oid, configured_base_oid)
            worktree = next(
                item
                for item in inventory.worktrees
                if Path(item["path"]).resolve() == stale_worktree.resolve()
            )
            self.assertEqual(worktree["branch_ref"], "refs/heads/master")
            self.assertFalse(worktree["dirty"])
            self.assertEqual(worktree["head_oid"], stale_master_oid)
            self.assertNotEqual(worktree["head_oid"], inventory.base_oid)

            report, _ = project_audit(repo, github=FakeGithub([]))
            classified = next(
                item
                for item in [*report["unowned_worktrees"], *report["base_worktrees"]]
                if Path(item["path"]).resolve() == stale_worktree.resolve()
            )
            self.assertEqual(classified["scope"], "candidate")
            self.assertEqual(classified["classification"], "stale_base")
            self.assertEqual(classified["disposition"], "unresolved")
            self.assertEqual(classified["head_oid"], stale_master_oid)
            self.assertEqual(classified["base_oid"], inventory.base_oid)
            self.assertIn(stale_master_oid, classified["next_action"])
            self.assertIn(inventory.base_oid, classified["next_action"])

            git(repo, "update-ref", "refs/remotes/origin/master", stale_master_oid)
            matching_report, _ = project_audit(repo, github=FakeGithub([]))
            matching_base = next(
                item
                for item in matching_report["base_worktrees"]
                if Path(item["path"]).resolve() == stale_worktree.resolve()
            )
            self.assertEqual(matching_base["base_oid"], stale_master_oid)
            self.assertEqual(matching_base["head_oid"], stale_master_oid)
            self.assertEqual(matching_base["head_oid"], matching_base["base_oid"])
            self.assertEqual(matching_base["classification"], "retained_base")
            self.assertEqual(matching_base["disposition"], "retained_base")
