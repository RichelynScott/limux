from __future__ import annotations

import importlib.util
import subprocess
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).parents[1] / "scripts" / "reconcile_snapshot.py"
SPEC = importlib.util.spec_from_file_location("reconcile_snapshot", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class RunTests(unittest.TestCase):
    def test_missing_command_returns_structured_failure(self) -> None:
        with mock.patch.object(
            subprocess,
            "run",
            side_effect=FileNotFoundError(2, "No such file or directory", "hcom"),
        ):
            result = MODULE.run(["hcom", "status"], Path("/tmp"))

        self.assertEqual(result["command"], ["hcom", "status"])
        self.assertEqual(result["exit_code"], 127)
        self.assertEqual(result["stdout"], "")
        self.assertIn("FileNotFoundError", result["stderr"])
        self.assertIn("hcom", result["stderr"])


if __name__ == "__main__":
    unittest.main()
