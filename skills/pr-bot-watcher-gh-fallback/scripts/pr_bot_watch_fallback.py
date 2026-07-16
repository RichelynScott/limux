#!/usr/bin/env python3
"""Temporary exact-head Codex PR review watcher using gh and local waits."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable


EXIT_RESULT = 0
EXIT_TIMEOUT = 2
EXIT_INFRA = 3
TERMINAL_REACTION = "+1"
PENDING_REACTIONS = {"eyes"}
DEFAULT_BOT_LOGINS = (
    "chatgpt-codex-connector[bot]",
    "codex[bot]",
)

Runner = Callable[[list[str], int], tuple[int, str]]


def utcnow() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def run(argv: list[str], timeout: int = 60) -> tuple[int, str]:
    try:
        completed = subprocess.run(
            argv,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return 124, ""
    except OSError:
        return 127, ""
    return completed.returncode, completed.stdout


def run_json(argv: list[str], runner: Runner = run) -> tuple[bool, object]:
    rc, stdout = runner(argv, 60)
    if rc != 0 or not stdout.strip():
        return False, {"error": "command-failed", "rc": rc}
    try:
        return True, json.loads(stdout)
    except json.JSONDecodeError:
        return False, {"error": "invalid-json"}


def run_paginated_list(
    argv: list[str], runner: Runner = run
) -> tuple[bool, list]:
    ok, payload = run_json([*argv, "--paginate", "--slurp"], runner)
    if not ok or not isinstance(payload, list):
        return False, []
    if not payload:
        return True, []
    if not all(isinstance(page, list) for page in payload):
        return False, []
    return True, [item for page in payload for item in page]


def parse_target(target: str, repo: str | None) -> tuple[str, int]:
    match = re.fullmatch(
        r"https://github\.com/([^/]+/[^/]+)/pull/(\d+)(?:[/?#].*)?", target
    )
    if match:
        return match.group(1), int(match.group(2))
    if target.isdigit() and repo and re.fullmatch(r"[^/]+/[^/]+", repo):
        return repo, int(target)
    raise ValueError("target must be a GitHub PR URL or a PR number with --repo")


def resolve_pr(
    target: str,
    repo: str | None,
    expected_head: str | None = None,
    runner: Runner = run,
) -> tuple[bool, dict]:
    try:
        resolved_repo, pr = parse_target(target, repo)
    except ValueError as error:
        return False, {"error": str(error)}
    ok, payload = run_json(
        [
            "gh",
            "pr",
            "view",
            str(pr),
            "--repo",
            resolved_repo,
            "--json",
            "headRefOid,url,state,isDraft,mergeable,mergeStateStatus",
        ],
        runner,
    )
    if not ok or not isinstance(payload, dict):
        return False, {"error": "pr-unresolvable", "detail": payload}
    head = payload.get("headRefOid")
    if not isinstance(head, str) or len(head) < 10:
        return False, {"error": "invalid-head"}
    if expected_head and head != expected_head:
        return False, {
            "error": "head-changed",
            "expected_head": expected_head,
            "current_head": head,
        }
    return True, {
        "repo": resolved_repo,
        "pr": pr,
        "url": payload.get("url"),
        "head": head,
        "state": payload.get("state"),
        "is_draft": payload.get("isDraft"),
        "mergeable": payload.get("mergeable"),
        "merge_state_status": payload.get("mergeStateStatus"),
    }


def request_names_head(body: str, head: str) -> bool:
    lowered = body.lower()
    lowered_head = head.lower()
    return "@codex review" in lowered and (
        lowered_head in lowered or lowered_head[:10] in lowered
    )


def fetch_issue_comments(repo: str, pr: int, runner: Runner = run) -> tuple[bool, list]:
    return run_paginated_list(
        ["gh", "api", f"repos/{repo}/issues/{pr}/comments"], runner
    )


def find_request_comment(comments: list, head: str) -> dict | None:
    candidates = []
    for comment in comments:
        user = comment.get("user") or {}
        if user.get("type") == "Bot":
            continue
        if request_names_head(comment.get("body") or "", head):
            candidates.append(comment)
    if not candidates:
        return None
    return max(candidates, key=lambda item: item.get("updated_at") or "")


def create_request_comment(
    repo: str, pr: int, head: str, runner: Runner = run
) -> tuple[bool, dict]:
    body = f"@codex review\n\nReview exact head: {head}"
    ok, payload = run_json(
        [
            "gh",
            "api",
            "--method",
            "POST",
            f"repos/{repo}/issues/{pr}/comments",
            "-f",
            f"body={body}",
        ],
        runner,
    )
    if not ok or not isinstance(payload, dict):
        return False, {"error": "review-request-failed", "detail": payload}
    return True, payload


@contextmanager
def review_request_lock(repo: str, pr: int, head: str):
    runtime_root = Path(os.environ.get("XDG_RUNTIME_DIR") or "/tmp")
    lock_dir = runtime_root / "pr-bot-watcher-gh-fallback"
    lock_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
    digest = hashlib.sha256(f"{repo}#{pr}@{head}".encode("utf-8")).hexdigest()
    lock_path = lock_dir / f"{digest}.lock"
    with lock_path.open("a+", encoding="utf-8") as lock_file:
        fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_UN)


def prepare(
    target: str,
    repo: str | None,
    expected_head: str | None,
    request_review: bool,
    runner: Runner = run,
    lock_factory=review_request_lock,
) -> tuple[int, dict]:
    ok, info = resolve_pr(target, repo, expected_head, runner)
    if not ok:
        return EXIT_INFRA, info
    ok, comments = fetch_issue_comments(info["repo"], info["pr"], runner)
    if not ok:
        return EXIT_INFRA, {"error": "issue-comments-unavailable", **info}
    request = find_request_comment(comments, info["head"])
    created = False
    if request is None and request_review:
        with lock_factory(info["repo"], info["pr"], info["head"]):
            ok, comments = fetch_issue_comments(info["repo"], info["pr"], runner)
            if not ok:
                return EXIT_INFRA, {"error": "issue-comments-unavailable", **info}
            request = find_request_comment(comments, info["head"])
            if request is None:
                head_ok, head_info = resolve_pr(
                    str(info["pr"]),
                    info["repo"],
                    info["head"],
                    runner,
                )
                if not head_ok:
                    return EXIT_INFRA, head_info
                ok, request = create_request_comment(
                    info["repo"], info["pr"], info["head"], runner
                )
                if not ok:
                    return EXIT_INFRA, {**info, **request}
                created = True
    if request is None:
        return EXIT_INFRA, {"error": "exact-head-review-request-missing", **info}
    return EXIT_RESULT, {
        **info,
        "request_comment": request.get("id"),
        "request_url": request.get("html_url"),
        "request_created_at": request.get("created_at"),
        "request_updated_at": request.get("updated_at"),
        "request_created": created,
    }


def validate_request_comment(
    repo: str, pr: int, comment_id: int, head: str, runner: Runner = run
) -> tuple[bool, str | None, dict]:
    ok, comment = run_json(
        ["gh", "api", f"repos/{repo}/issues/comments/{comment_id}"], runner
    )
    if not ok or not isinstance(comment, dict):
        return False, None, {"error": "request-comment-unresolvable"}
    body = comment.get("body") or ""
    created_at = comment.get("created_at")
    updated_at = comment.get("updated_at")
    timestamps = [
        value for value in (created_at, updated_at) if isinstance(value, str) and value
    ]
    binding_at = max(timestamps) if timestamps else None
    issue_url = comment.get("issue_url")
    expected_issue_suffix = f"/repos/{repo}/issues/{pr}"
    belongs_to_pr = isinstance(issue_url, str) and issue_url.rstrip("/").endswith(
        expected_issue_suffix
    )
    valid = request_names_head(body, head) and binding_at is not None and belongs_to_pr
    metadata = {
        "request_comment": comment_id,
        "author": (comment.get("user") or {}).get("login"),
        "created_at": created_at,
        "updated_at": updated_at,
        "binding_at": binding_at,
        "belongs_to_pr": belongs_to_pr,
        "names_frozen_head": request_names_head(body, head),
    }
    return valid, binding_at if valid else None, metadata


def is_bot(user: dict, allowed_logins: set[str]) -> bool:
    login = (user.get("login") or "").lower()
    return user.get("type") == "Bot" and login in allowed_logins


def classify(
    reviews: list,
    inline_comments: list,
    issue_comments: list,
    reactions: list,
    frozen_head: str,
    request_time: str,
    bot_logins: set[str],
) -> dict:
    raw = []
    for review in reviews:
        user = review.get("user") or {}
        metadata = {
            "surface": "pull-request-review",
            "id": review.get("id"),
            "actor": user.get("login"),
            "actor_type": user.get("type"),
            "commit_id": review.get("commit_id"),
            "state": review.get("state"),
            "at": review.get("submitted_at"),
        }
        raw.append(metadata)
        if (
            is_bot(user, bot_logins)
            and metadata["commit_id"] == frozen_head
            and (metadata["at"] or "") >= request_time
        ):
            return {"found": True, "source": metadata["surface"], "evidence": metadata, "raw": raw}
    for comment in inline_comments:
        user = comment.get("user") or {}
        metadata = {
            "surface": "inline-comment",
            "id": comment.get("id"),
            "actor": user.get("login"),
            "actor_type": user.get("type"),
            "commit_id": comment.get("commit_id"),
            "original_commit_id": comment.get("original_commit_id"),
            "at": comment.get("created_at"),
        }
        raw.append(metadata)
        original_commit_id = metadata["original_commit_id"]
        commit_id = metadata["commit_id"]
        inline_matches_head = (
            commit_id == frozen_head
            if original_commit_id is None
            else original_commit_id == frozen_head and commit_id == frozen_head
        )
        if (
            is_bot(user, bot_logins)
            and inline_matches_head
            and (metadata["at"] or "") >= request_time
        ):
            return {"found": True, "source": metadata["surface"], "evidence": metadata, "raw": raw}
    head_prefix = frozen_head[:10]
    for comment in issue_comments:
        user = comment.get("user") or {}
        body = comment.get("body") or ""
        metadata = {
            "surface": "issue-comment",
            "id": comment.get("id"),
            "actor": user.get("login"),
            "actor_type": user.get("type"),
            "mentions_frozen_head": head_prefix in body,
            "at": comment.get("created_at"),
        }
        raw.append(metadata)
        if (
            is_bot(user, bot_logins)
            and (metadata["at"] or "") >= request_time
            and metadata["mentions_frozen_head"]
        ):
            return {"found": True, "source": metadata["surface"], "evidence": metadata, "raw": raw}
    for reaction in reactions:
        user = reaction.get("user") or {}
        content = reaction.get("content")
        metadata = {
            "surface": "request-reaction",
            "id": reaction.get("id"),
            "actor": user.get("login"),
            "actor_type": user.get("type"),
            "content": content,
            "pending": content in PENDING_REACTIONS,
            "terminal": content == TERMINAL_REACTION,
            "at": reaction.get("created_at"),
        }
        raw.append(metadata)
        if (
            is_bot(user, bot_logins)
            and (metadata["at"] or "") >= request_time
            and metadata["terminal"]
        ):
            return {"found": True, "source": metadata["surface"], "evidence": metadata, "raw": raw}
    return {"found": False, "source": None, "evidence": None, "raw": raw}


def fetch_current_head(repo: str, pr: int, runner: Runner = run) -> tuple[bool, str | None]:
    ok, payload = run_json(
        ["gh", "pr", "view", str(pr), "--repo", repo, "--json", "headRefOid"],
        runner,
    )
    if not ok or not isinstance(payload, dict):
        return False, None
    head = payload.get("headRefOid")
    return (isinstance(head, str), head if isinstance(head, str) else None)


def fetch_surfaces(repo: str, pr: int, request_comment: int, runner: Runner = run) -> tuple[bool, dict]:
    commands = {
        "reviews": ["gh", "api", f"repos/{repo}/pulls/{pr}/reviews"],
        "inline": ["gh", "api", f"repos/{repo}/pulls/{pr}/comments"],
        "issue": ["gh", "api", f"repos/{repo}/issues/{pr}/comments"],
        "reactions": [
            "gh",
            "api",
            f"repos/{repo}/issues/comments/{request_comment}/reactions",
        ],
    }
    result = {}
    for key, argv in commands.items():
        ok, payload = run_paginated_list(argv, runner)
        if not ok:
            return False, {"failed_surface": key}
        result[key] = payload
    return True, result


def local_wait(seconds: int, log: Callable[[str], None]) -> bool:
    if seconds <= 0:
        return True
    deadline = time.monotonic() + seconds
    event = threading.Event()
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return True
        log(f"wait mechanism=process-local-event remaining={remaining:.1f}s")
        event.wait(remaining)


def watch(
    config: dict,
    runner: Runner = run,
    waiter: Callable[[int, Callable[[str], None]], bool] = local_wait,
    log: Callable[[str], None] = print,
) -> tuple[int, dict]:
    valid, request_time, metadata = validate_request_comment(
        config["repo"],
        config["pr"],
        config["request_comment"],
        config["head"],
        runner,
    )
    log(f"{utcnow()} request-binding {json.dumps(metadata, sort_keys=True)}")
    if not valid or request_time is None:
        return EXIT_INFRA, {"error": "request-head-mismatch", "meta": metadata}

    any_success = False

    def poll(label: str) -> dict | None:
        nonlocal any_success
        observed = utcnow()
        ok, current_head = fetch_current_head(config["repo"], config["pr"], runner)
        if not ok:
            log(f"{observed} {label} degraded=pre-head")
            return None
        if current_head != config["head"]:
            return {
                "fatal": True,
                "error": "head-changed",
                "expected_head": config["head"],
                "current_head": current_head,
                "observed_at": observed,
            }
        ok, surfaces = fetch_surfaces(
            config["repo"], config["pr"], config["request_comment"], runner
        )
        if not ok:
            log(f"{observed} {label} degraded={surfaces.get('failed_surface')}")
            return None
        ok, current_head = fetch_current_head(config["repo"], config["pr"], runner)
        if not ok:
            log(f"{observed} {label} degraded=post-head")
            return None
        if current_head != config["head"]:
            return {
                "fatal": True,
                "error": "head-changed",
                "expected_head": config["head"],
                "current_head": current_head,
                "observed_at": observed,
            }
        any_success = True
        result = classify(
            surfaces["reviews"],
            surfaces["inline"],
            surfaces["issue"],
            surfaces["reactions"],
            config["head"],
            request_time,
            config["bot_logins"],
        )
        result["observed_at"] = observed
        result["request_time"] = request_time
        log(
            f"{observed} {label} found={result['found']} source={result['source']} "
            f"metadata={json.dumps(result['raw'], sort_keys=True)}"
        )
        if result["found"]:
            ok, terminal_head = fetch_current_head(
                config["repo"], config["pr"], runner
            )
            if not ok:
                return {
                    "fatal": True,
                    "error": "terminal-head-recheck-failed",
                    "observed_at": utcnow(),
                }
            if terminal_head != config["head"]:
                return {
                    "fatal": True,
                    "error": "head-changed-before-terminal",
                    "expected_head": config["head"],
                    "current_head": terminal_head,
                    "observed_at": utcnow(),
                }
            result["terminal_head_verified_at"] = utcnow()
        return result

    for round_number in range(1, config["rounds"] + 1):
        result = poll(f"round={round_number}/{config['rounds']}")
        if result and result.get("fatal"):
            return EXIT_INFRA, result
        if result and result.get("found"):
            return EXIT_RESULT, result
        if round_number < config["rounds"] and not waiter(config["interval"], log):
            return EXIT_INFRA, {"error": "local-wait-failed"}

    log(f"{utcnow()} nominal rounds exhausted; grace={config['grace']}s")
    if not waiter(config["grace"], log):
        return EXIT_INFRA, {"error": "local-wait-failed"}
    result = poll("grace-recheck")
    if result and result.get("fatal"):
        return EXIT_INFRA, result
    if result is None:
        return EXIT_INFRA, {
            "error": "grace-degraded-with-prior-success" if any_success else "all-degraded-watch"
        }
    if result.get("found"):
        return EXIT_RESULT, result
    return EXIT_TIMEOUT, result


def notify(
    targets: list[str],
    sender_name: str,
    intent: str,
    text: str,
    runner: Runner = run,
) -> bool:
    if not targets:
        return True
    if not sender_name:
        return False
    if shutil.which("hcom") is None and runner is run:
        return False
    rc, _ = runner(
        ["hcom", "send", *targets, "--intent", intent, "--name", sender_name, "--", text],
        60,
    )
    return rc == 0


def result_message(code: int, config: dict, result: dict) -> tuple[str, str]:
    base = f"PR {config['repo']}#{config['pr']} head={config['head']}"
    if code == EXIT_RESULT:
        return (
            "inform",
            f"{base} has a terminal exact-head Codex bot response: "
            f"source={result.get('source')} evidence={json.dumps(result.get('evidence'), sort_keys=True)}. "
            "Inspect review and inline-comment bodies with repo-scoped gh before fixing or merging.",
        )
    if code == EXIT_TIMEOUT:
        return (
            "request",
            f"{base} watcher timed out after a successful final all-surface poll. "
            "Do not duplicate the review request automatically; inspect GitHub once and decide whether to restart.",
        )
    return (
        "request",
        f"{base} watcher failed closed: {json.dumps(result, sort_keys=True)}. No bot verdict claimed.",
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    prepare_parser = subparsers.add_parser("prepare")
    prepare_parser.add_argument("target")
    prepare_parser.add_argument("--repo")
    prepare_parser.add_argument("--head")
    prepare_parser.add_argument("--request-review", action="store_true")

    watch_parser = subparsers.add_parser("watch")
    watch_parser.add_argument("target")
    watch_parser.add_argument("--repo")
    watch_parser.add_argument("--head", required=True)
    watch_parser.add_argument("--request-comment", type=int)
    watch_parser.add_argument("--request-review", action="store_true")
    watch_parser.add_argument("--rounds", type=int, default=20)
    watch_parser.add_argument("--interval", type=int, default=60)
    watch_parser.add_argument("--grace", type=int, default=90)
    watch_parser.add_argument(
        "--bot-login",
        action="append",
        help="exact allowed Bot login; repeat to override the built-in Codex bot allowlist",
    )
    watch_parser.add_argument("--sender-name")
    watch_parser.add_argument("--notify", action="append", default=[])
    watch_parser.add_argument("--log")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "prepare":
        code, result = prepare(
            args.target, args.repo, args.head, args.request_review
        )
        print(json.dumps(result, sort_keys=True))
        return code

    if args.rounds < 1 or args.interval < 1 or args.grace < 0:
        print("rounds and interval must be positive; grace cannot be negative", file=sys.stderr)
        return EXIT_INFRA
    if args.notify and not args.sender_name:
        print("--sender-name is required with --notify", file=sys.stderr)
        return EXIT_INFRA

    code, prepared = prepare(
        args.target, args.repo, args.head, args.request_review
    )
    if code != EXIT_RESULT:
        print(json.dumps(prepared, sort_keys=True))
        return code
    prepared_comment = prepared.get("request_comment")
    if args.request_comment is not None and args.request_comment != prepared_comment:
        print(
            json.dumps(
                {
                    "error": "request-comment-mismatch",
                    "prepared_request_comment": prepared_comment,
                    "supplied_request_comment": args.request_comment,
                },
                sort_keys=True,
            )
        )
        return EXIT_INFRA
    request_comment = prepared_comment
    if not isinstance(request_comment, int):
        print(json.dumps({"error": "request-comment-missing"}, sort_keys=True))
        return EXIT_INFRA

    log_file = None
    if args.log:
        log_path = Path(args.log).expanduser()
        log_file = log_path.open("a", encoding="utf-8")

    def logger(message: str) -> None:
        print(message, file=sys.stderr, flush=True)
        if log_file:
            log_file.write(message + "\n")
            log_file.flush()

    config = {
        "repo": prepared["repo"],
        "pr": prepared["pr"],
        "head": prepared["head"],
        "request_comment": request_comment,
        "rounds": args.rounds,
        "interval": args.interval,
        "grace": args.grace,
        "bot_logins": {
            login.lower() for login in (args.bot_login or DEFAULT_BOT_LOGINS)
        },
    }
    code, result = watch(config, log=logger)
    intent, message = result_message(code, config, result)
    if args.notify and not notify(args.notify, args.sender_name, intent, message):
        code = EXIT_INFRA
        result = {"error": "notification-failed", "prior_result": result}
        logger(f"{utcnow()} notification failed closed")
    if log_file:
        log_file.close()
    print(json.dumps({"code": code, "result": result}, sort_keys=True))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
