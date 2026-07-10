#!/usr/bin/env python3
"""Emit a read-only reconciliation snapshot as JSON."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any

PROBE_TIMEOUT_SECONDS = 10


def output_text(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def run(command: list[str], cwd: Path) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            timeout=PROBE_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        stderr = output_text(error.stderr)
        timeout_message = f"command timed out after {PROBE_TIMEOUT_SECONDS} seconds"
        return {
            "command": command,
            "exit_code": 124,
            "stdout": output_text(error.stdout),
            "stderr": f"{stderr}\n{timeout_message}".lstrip(),
        }
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


def collect_limux_topology(
    identify: dict[str, Any], cwd: Path, workspace_override: str | None
) -> dict[str, Any]:
    workspace = workspace_override
    error = ""
    if workspace is None and identify["exit_code"] == 0:
        try:
            payload = json.loads(identify["stdout"])
            workspace = payload.get("caller", {}).get("workspace_ref")
        except (json.JSONDecodeError, AttributeError) as parse_error:
            error = f"invalid limux identify JSON: {parse_error}"

    probes = []
    if workspace:
        for command in ("list-panes", "list-panels", "surface-health"):
            probes.append(
                run(["limux", "--json", command, "--workspace", workspace], cwd)
            )
    elif not error:
        error = "limux identify did not provide caller.workspace_ref; pass --workspace"

    return {
        "command": ["limux", "topology", "--workspace", workspace or "<unresolved>"],
        "exit_code": 0
        if identify["exit_code"] == 0
        and bool(workspace)
        and all(probe["exit_code"] == 0 for probe in probes)
        else 1,
        "stdout": "",
        "stderr": error,
        "workspace_ref": workspace,
        "probes": probes,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--name", required=True, help="Current hcom name")
    parser.add_argument(
        "--workspace", help="Explicit Limux workspace ref for topology probes"
    )
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
    results["limux_topology"] = collect_limux_topology(
        results["limux_identify"], repo, args.workspace
    )
    print(json.dumps({"repo": str(repo), "results": results}, indent=2, sort_keys=True))
    return 0 if all(item["exit_code"] == 0 for item in results.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
