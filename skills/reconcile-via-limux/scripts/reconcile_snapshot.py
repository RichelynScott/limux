#!/usr/bin/env python3
"""Emit a read-only reconciliation snapshot as JSON."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


def run(command: list[str], cwd: Path) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
    except OSError as error:
        return {
            "command": command,
            "exit_code": 127,
            "stdout": "",
            "stderr": f"{type(error).__name__}: {error}",
        }
    return {
        "command": command,
        "exit_code": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def worktree_paths(porcelain: str) -> list[Path]:
    return [
        Path(line.removeprefix("worktree "))
        for line in porcelain.splitlines()
        if line.startswith("worktree ")
    ]


def collect_worktree_statuses(worktree_listing: dict[str, Any]) -> dict[str, Any]:
    records = []
    if worktree_listing["exit_code"] == 0:
        for path in worktree_paths(worktree_listing["stdout"]):
            records.append(
                {
                    "path": str(path),
                    **run(["git", "status", "--short", "--branch"], path),
                }
            )
    return {
        "command": ["git", "worktree", "status", "--all"],
        "exit_code": 0
        if worktree_listing["exit_code"] == 0
        and all(record["exit_code"] == 0 for record in records)
        else 1,
        "stdout": "",
        "stderr": "",
        "worktrees": records,
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
    results["git_worktree_statuses"] = collect_worktree_statuses(results["git_worktrees"])
    print(json.dumps({"repo": str(repo), "results": results}, indent=2, sort_keys=True))
    return 0 if all(item["exit_code"] == 0 for item in results.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
