#!/usr/bin/env python3
"""Emit a read-only reconciliation snapshot as JSON."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


def run(command: list[str], cwd: Path) -> dict[str, Any]:
    result = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    return {
        "command": command,
        "exit_code": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--name", required=True, help="Current hcom name")
    args = parser.parse_args()
    repo = args.repo.resolve()

    commands = {
        "git_status": ["git", "status", "--short", "--branch"],
        "git_worktrees": ["git", "worktree", "list", "--porcelain"],
        "git_log": ["git", "log", "--oneline", "--decorate", "-12"],
        "hcom_roster": ["hcom", "list", "-v", "--json", "--name", args.name],
        "hcom_managers": ["hcom", "list", "mgrs", "--json", "--name", args.name],
        "limux_identify": ["limux", "identify", "--json"],
    }
    results = {key: run(command, repo) for key, command in commands.items()}
    print(json.dumps({"repo": str(repo), "results": results}, indent=2, sort_keys=True))
    return 0 if all(item["exit_code"] == 0 for item in results.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
