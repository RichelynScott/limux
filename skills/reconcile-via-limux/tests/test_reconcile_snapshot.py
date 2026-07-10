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

    def test_worktree_statuses_include_each_porcelain_path(self) -> None:
        listing = {
            "exit_code": 0,
            "stdout": (
                "worktree /tmp/project\nHEAD abc\nbranch refs/heads/main\n\n"
                "worktree /tmp/project linked\nHEAD def\ndetached\n"
            ),
        }
        success = {
            "command": ["git", "status", "--short", "--branch"],
            "exit_code": 0,
            "stdout": "## main\n",
            "stderr": "",
        }

        with mock.patch.object(MODULE, "run", return_value=success) as run_mock:
            result = MODULE.collect_worktree_statuses(listing)

        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(
            [record["path"] for record in result["worktrees"]],
            ["/tmp/project", "/tmp/project linked"],
        )
        self.assertEqual(
            [call.args[1] for call in run_mock.call_args_list],
            [Path("/tmp/project"), Path("/tmp/project linked")],
        )

    def test_timeout_returns_structured_partial_evidence(self) -> None:
        with mock.patch.object(
            subprocess,
            "run",
            side_effect=subprocess.TimeoutExpired(
                ["limux", "identify"],
                MODULE.PROBE_TIMEOUT_SECONDS,
                output=b"partial",
                stderr=b"stalled",
            ),
        ):
            result = MODULE.run(["limux", "identify"], Path("/tmp"))

        self.assertEqual(result["exit_code"], 124)
        self.assertEqual(result["stdout"], "partial")
        self.assertIn("stalled", result["stderr"])
        self.assertIn("timed out", result["stderr"])

    def test_limux_topology_uses_explicit_workspace_for_all_probes(self) -> None:
        identify = {"exit_code": 0, "stdout": "{}"}
        success = {"exit_code": 0, "stdout": "", "stderr": ""}

        with mock.patch.object(MODULE, "run", return_value=success) as run_mock:
            result = MODULE.collect_limux_topology(
                identify, Path("/tmp/project"), "workspace:expected"
            )

        self.assertEqual(result["exit_code"], 0)
        self.assertEqual(result["workspace_ref"], "workspace:expected")
        self.assertEqual(len(result["probes"]), 3)
        self.assertEqual(
            [call.args[0][2] for call in run_mock.call_args_list],
            ["list-panes", "list-panels", "surface-health"],
        )


if __name__ == "__main__":
    unittest.main()
