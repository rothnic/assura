#!/usr/bin/env python3
"""Cheap contract control for distinct candidate binary and shim identities."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("validate-a07-packet.py")
SPEC = importlib.util.spec_from_file_location("validate_a07_packet", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {SCRIPT}")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class DistinctIdentityContractTests(unittest.TestCase):
    """The validator must preserve binary/shim distinction in both outcomes."""

    def test_distinct_binary_and_shim_hashes_have_valid_and_invalid_outcomes(self) -> None:
        expected = {
            "source_sha": "source",
            "source_tree_sha256": "tree",
            "binary_sha256": "binary",
            "shim_sha256": "shim",
            "public_contract_sha256": "contract",
            "evaluator_contract_sha256": "evaluator",
            "fixed_prompt_sha256": "prompt",
            "toolchain": "toolchain",
        }
        valid = expected.copy()
        invalid = expected | {"shim_sha256": expected["binary_sha256"]}

        self.assertNotEqual(expected["binary_sha256"], expected["shim_sha256"])
        self.assertTrue(MODULE.identity_matches(valid, expected))
        self.assertFalse(MODULE.identity_matches(invalid, expected))


if __name__ == "__main__":
    unittest.main()
