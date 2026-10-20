#!/usr/bin/env python3
"""Focused tests for the Assura delivery ownership and closure contract."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from agent_delivery_contract_support import ROOT

from agent_delivery_contract_core_cases import (  # noqa: E402
    CommonJsonIOCases,
    DeliveryIntentCases,
    DeliveryStoreCases,
)
from agent_delivery_contract_projection_cases import ProjectionCases  # noqa: E402
from agent_delivery_contract_audit_cases import ProjectionAuditCases  # noqa: E402
from agent_delivery_contract_branch_inventory_cases import ProjectionBranchInventoryCases  # noqa: E402
from agent_delivery_contract_provider_cases import ProjectionProviderCases  # noqa: E402
from agent_delivery_contract_close_cases import LifecycleCloseCases  # noqa: E402
from agent_delivery_contract_registration_cases import LifecycleRegistrationCases  # noqa: E402
from agent_delivery_contract_task_state_cases import LifecycleTaskStateCases  # noqa: E402
from agent_delivery_contract_inspect_cases import LifecycleInspectCases  # noqa: E402
from agent_delivery_contract_archive_cases import LifecycleArchiveCases  # noqa: E402
from agent_delivery_contract_regression_cases import (
    RepairRegressionCases,
    RemoteFreshnessCases,
    StaleBaseCases,
)  # noqa: E402


class CommonJsonIOTests(CommonJsonIOCases, unittest.TestCase):
    pass


class DeliveryIntentTests(DeliveryIntentCases, unittest.TestCase):
    pass


class DeliveryStoreTests(DeliveryStoreCases, unittest.TestCase):
    pass


class DeliveryProjectionTests(
    ProjectionCases,
    ProjectionAuditCases,
    ProjectionBranchInventoryCases,
    ProjectionProviderCases,
    unittest.TestCase,
):
    pass


class DeliveryLifecycleCommandTests(
    LifecycleCloseCases,
    LifecycleRegistrationCases,
    LifecycleTaskStateCases,
    LifecycleInspectCases,
    LifecycleArchiveCases,
    unittest.TestCase,
):
    pass


class DeliveryRepairRegressionTests(RepairRegressionCases, unittest.TestCase):
    pass


class DeliveryRemoteFreshnessTests(RemoteFreshnessCases, unittest.TestCase):
    pass


class DeliveryAuditStaleBaseTests(StaleBaseCases, unittest.TestCase):
    pass


class DeliverySuiteSelectionGuardTests(unittest.TestCase):
    def test_empty_delivery_suite_is_rejected_by_direct_entrypoint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            sitecustomize = Path(directory) / "sitecustomize.py"
            sitecustomize.write_text(
                "import unittest\n"
                "_run_tests = unittest.TestProgram.runTests\n"
                "def _load_empty_suite(self, module, pattern=None):\n"
                "    return unittest.TestSuite()\n"
                "def _legacy_run_tests(self):\n"
                "    self.exit = False\n"
                "    return _run_tests(self)\n"
                "unittest.TestLoader.loadTestsFromModule = _load_empty_suite\n"
                "unittest.TestProgram.runTests = _legacy_run_tests\n",
                encoding="utf-8",
            )
            environment = os.environ.copy()
            environment["PYTHONPATH"] = os.pathsep.join(
                value
                for value in (directory, environment.get("PYTHONPATH", ""))
                if value
            )
            environment["PYTHONDONTWRITEBYTECODE"] = "1"
            result = subprocess.run(
                [sys.executable, str(Path(__file__).resolve())],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )

        output = result.stdout + result.stderr
        self.assertNotEqual(result.returncode, 0, output)
        self.assertIn("delivery contract suite selected zero tests", output)


class NonEmptyTestRunner(unittest.TextTestRunner):
    def run(self, test):
        if test.countTestCases() == 0:
            raise SystemExit("delivery contract suite selected zero tests")
        return super().run(test)


if __name__ == "__main__":
    unittest.main(verbosity=2, testRunner=NonEmptyTestRunner)
