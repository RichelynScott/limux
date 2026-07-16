import importlib.util
import json
import sys
import unittest
from contextlib import nullcontext
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "scripts" / "pr_bot_watch_fallback.py"
SPEC = importlib.util.spec_from_file_location("pr_bot_watch_fallback", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


HEAD = "a" * 40
OLD_HEAD = "b" * 40
REQUEST_TIME = "2026-07-15T20:00:00Z"
BOT_LOGINS = {login.lower() for login in MODULE.DEFAULT_BOT_LOGINS}


class FallbackWatcherTests(unittest.TestCase):
    def test_parse_target_accepts_url_and_number(self):
        self.assertEqual(
            MODULE.parse_target("https://github.com/owner/repo/pull/59", None),
            ("owner/repo", 59),
        )
        self.assertEqual(MODULE.parse_target("59", "owner/repo"), ("owner/repo", 59))

    def test_eyes_reaction_is_pending_not_terminal(self):
        result = MODULE.classify(
            [],
            [],
            [],
            [
                {
                    "id": 1,
                    "user": {
                        "login": "chatgpt-codex-connector[bot]",
                        "type": "Bot",
                    },
                    "content": "eyes",
                    "created_at": "2026-07-15T20:01:00Z",
                }
            ],
            HEAD,
            REQUEST_TIME,
            BOT_LOGINS,
        )
        self.assertFalse(result["found"])
        self.assertTrue(result["raw"][0]["pending"])
        self.assertFalse(result["raw"][0]["terminal"])

    def test_exact_head_review_is_terminal_and_old_head_is_ignored(self):
        reviews = [
            {
                "id": 1,
                "user": {
                    "login": "chatgpt-codex-connector[bot]",
                    "type": "Bot",
                },
                "commit_id": OLD_HEAD,
                "state": "COMMENTED",
                "submitted_at": "2026-07-15T20:02:00Z",
            },
            {
                "id": 2,
                "user": {
                    "login": "chatgpt-codex-connector[bot]",
                    "type": "Bot",
                },
                "commit_id": HEAD,
                "state": "COMMENTED",
                "submitted_at": "2026-07-15T20:03:00Z",
            },
        ]
        result = MODULE.classify(
            reviews, [], [], [], HEAD, REQUEST_TIME, BOT_LOGINS
        )
        self.assertTrue(result["found"])
        self.assertEqual(result["evidence"]["id"], 2)
        self.assertEqual(result["source"], "pull-request-review")

    def test_issue_comment_requires_frozen_head_mention(self):
        comments = [
            {
                "id": 3,
                "user": {"login": "codex[bot]", "type": "Bot"},
                "body": "No findings for another revision",
                "created_at": "2026-07-15T20:04:00Z",
            },
            {
                "id": 4,
                "user": {"login": "codex[bot]", "type": "Bot"},
                "body": f"No findings for {HEAD[:10]}",
                "created_at": "2026-07-15T20:05:00Z",
            },
        ]
        result = MODULE.classify(
            [], [], comments, [], HEAD, REQUEST_TIME, BOT_LOGINS
        )
        self.assertTrue(result["found"])
        self.assertEqual(result["evidence"]["id"], 4)

    def test_human_login_containing_codex_or_chatgpt_is_not_a_bot(self):
        reviews = [
            {
                "id": 5,
                "user": {"login": "codex-reviewer", "type": "User"},
                "commit_id": HEAD,
                "state": "COMMENTED",
                "submitted_at": "2026-07-15T20:06:00Z",
            }
        ]
        reactions = [
            {
                "id": 6,
                "user": {"login": "chatgpt-helper", "type": "User"},
                "content": "+1",
                "created_at": "2026-07-15T20:07:00Z",
            }
        ]
        result = MODULE.classify(
            reviews, [], [], reactions, HEAD, REQUEST_TIME, BOT_LOGINS
        )
        self.assertFalse(result["found"])

    def test_prepare_reuses_exact_head_request_without_posting(self):
        calls = []

        def runner(argv, timeout):
            calls.append(argv)
            if argv[:3] == ["gh", "pr", "view"]:
                return 0, json.dumps(
                    {
                        "headRefOid": HEAD,
                        "url": "https://github.com/owner/repo/pull/59",
                        "state": "OPEN",
                        "isDraft": False,
                        "mergeable": "MERGEABLE",
                        "mergeStateStatus": "CLEAN",
                    }
                )
            if f"repos/owner/repo/issues/59/comments" in argv:
                return 0, json.dumps(
                    [[
                        {
                            "id": 99,
                            "html_url": "https://example.test/comment/99",
                            "body": f"@codex review\n\nReview exact head: {HEAD}",
                            "created_at": REQUEST_TIME,
                            "updated_at": REQUEST_TIME,
                            "user": {"login": "owner", "type": "User"},
                        }
                    ]]
                )
            return 1, ""

        code, result = MODULE.prepare(
            "59", "owner/repo", HEAD, True, runner=runner
        )
        self.assertEqual(code, MODULE.EXIT_RESULT)
        self.assertEqual(result["request_comment"], 99)
        self.assertFalse(result["request_created"])
        self.assertFalse(any("POST" in call for call in calls))

    def test_request_binding_uses_later_edit_time(self):
        edited_at = "2026-07-15T20:10:00Z"

        def runner(argv, timeout):
            return 0, json.dumps(
                {
                    "id": 99,
                    "body": f"@codex review\n\nReview exact head: {HEAD.upper()}",
                    "created_at": REQUEST_TIME,
                    "updated_at": edited_at,
                    "issue_url": "https://api.github.com/repos/owner/repo/issues/59",
                    "user": {"login": "owner"},
                }
            )

        valid, binding_at, metadata = MODULE.validate_request_comment(
            "owner/repo", 59, 99, HEAD, runner
        )
        self.assertTrue(valid)
        self.assertEqual(binding_at, edited_at)
        self.assertEqual(metadata["binding_at"], edited_at)

    def test_request_comment_must_belong_to_watched_pr(self):
        def runner(argv, timeout):
            return 0, json.dumps(
                {
                    "id": 99,
                    "body": f"@codex review\n\nReview exact head: {HEAD}",
                    "created_at": REQUEST_TIME,
                    "updated_at": REQUEST_TIME,
                    "issue_url": "https://api.github.com/repos/owner/repo/issues/60",
                    "user": {"login": "owner"},
                }
            )

        valid, _, metadata = MODULE.validate_request_comment(
            "owner/repo", 59, 99, HEAD, runner
        )
        self.assertFalse(valid)
        self.assertFalse(metadata["belongs_to_pr"])

    def test_paginated_lists_are_slurped_and_flattened(self):
        calls = []

        def runner(argv, timeout):
            calls.append(argv)
            return 0, json.dumps([[{"id": 1}], [{"id": 2}]])

        ok, rows = MODULE.run_paginated_list(
            ["gh", "api", "repos/owner/repo/pulls/59/reviews"], runner
        )
        self.assertTrue(ok)
        self.assertEqual([row["id"] for row in rows], [1, 2])
        self.assertIn("--paginate", calls[0])
        self.assertIn("--slurp", calls[0])

    def test_prepare_rechecks_inside_lock_before_posting(self):
        calls = []
        comment_reads = 0
        request = {
            "id": 99,
            "html_url": "https://example.test/comment/99",
            "body": f"@codex review\n\nReview exact head: {HEAD}",
            "created_at": REQUEST_TIME,
            "updated_at": REQUEST_TIME,
            "user": {"login": "owner", "type": "User"},
        }

        def runner(argv, timeout):
            nonlocal comment_reads
            calls.append(argv)
            if argv[:3] == ["gh", "pr", "view"]:
                return 0, json.dumps(
                    {
                        "headRefOid": HEAD,
                        "url": "https://github.com/owner/repo/pull/59",
                        "state": "OPEN",
                        "isDraft": False,
                        "mergeable": "MERGEABLE",
                        "mergeStateStatus": "CLEAN",
                    }
                )
            if "repos/owner/repo/issues/59/comments" in argv:
                comment_reads += 1
                return 0, json.dumps([[]] if comment_reads == 1 else [[request]])
            return 1, ""

        code, result = MODULE.prepare(
            "59",
            "owner/repo",
            HEAD,
            True,
            runner=runner,
            lock_factory=lambda repo, pr, head: nullcontext(),
        )
        self.assertEqual(code, MODULE.EXIT_RESULT)
        self.assertEqual(result["request_comment"], 99)
        self.assertFalse(result["request_created"])
        self.assertEqual(comment_reads, 2)
        self.assertFalse(any("POST" in call for call in calls))

    def test_prepare_rechecks_head_inside_lock_before_posting(self):
        calls = []
        head_reads = 0

        def runner(argv, timeout):
            nonlocal head_reads
            calls.append(argv)
            if argv[:3] == ["gh", "pr", "view"]:
                head_reads += 1
                current = HEAD if head_reads == 1 else OLD_HEAD
                return 0, json.dumps(
                    {
                        "headRefOid": current,
                        "url": "https://github.com/owner/repo/pull/59",
                        "state": "OPEN",
                        "isDraft": False,
                        "mergeable": "MERGEABLE",
                        "mergeStateStatus": "CLEAN",
                    }
                )
            if "repos/owner/repo/issues/59/comments" in argv:
                return 0, "[[]]"
            return 1, ""

        code, result = MODULE.prepare(
            "59",
            "owner/repo",
            HEAD,
            True,
            runner=runner,
            lock_factory=lambda repo, pr, head: nullcontext(),
        )
        self.assertEqual(code, MODULE.EXIT_INFRA)
        self.assertEqual(result["error"], "head-changed")
        self.assertEqual(result["current_head"], OLD_HEAD)
        self.assertEqual(head_reads, 2)
        self.assertFalse(any("POST" in call for call in calls))

    def test_watch_fails_closed_when_head_changes(self):
        def runner(argv, timeout):
            joined = " ".join(argv)
            if "issues/comments/99" in joined:
                return 0, json.dumps(
                    {
                        "id": 99,
                        "body": f"@codex review\n\nReview exact head: {HEAD}",
                        "created_at": REQUEST_TIME,
                        "updated_at": REQUEST_TIME,
                        "issue_url": "https://api.github.com/repos/owner/repo/issues/59",
                        "user": {"login": "owner"},
                    }
                )
            if argv[:3] == ["gh", "pr", "view"]:
                return 0, json.dumps(
                    {
                        "headRefOid": OLD_HEAD,
                        "url": "https://github.com/owner/repo/pull/59",
                        "state": "OPEN",
                    }
                )
            if argv[:2] == ["gh", "api"]:
                return 0, "[[]]"
            return 1, ""

        config = {
            "repo": "owner/repo",
            "pr": 59,
            "head": HEAD,
            "request_comment": 99,
            "rounds": 1,
            "interval": 1,
            "grace": 0,
            "bot_logins": BOT_LOGINS,
        }
        code, result = MODULE.watch(
            config,
            runner=runner,
            waiter=lambda seconds, log: True,
            log=lambda message: None,
        )
        self.assertEqual(code, MODULE.EXIT_INFRA)
        self.assertEqual(result["error"], "head-changed")

    def test_terminal_result_rechecks_head_after_surface_collection(self):
        head_reads = 0

        def runner(argv, timeout):
            nonlocal head_reads
            joined = " ".join(argv)
            if "issues/comments/99" in joined and "reactions" not in joined:
                return 0, json.dumps(
                    {
                        "id": 99,
                        "body": f"@codex review\n\nReview exact head: {HEAD}",
                        "created_at": REQUEST_TIME,
                        "updated_at": REQUEST_TIME,
                        "issue_url": "https://api.github.com/repos/owner/repo/issues/59",
                        "user": {"login": "owner"},
                    }
                )
            if argv[:3] == ["gh", "pr", "view"]:
                head_reads += 1
                current = OLD_HEAD if head_reads == 3 else HEAD
                return 0, json.dumps({"headRefOid": current})
            if "pulls/59/reviews" in joined:
                return 0, json.dumps(
                    [[
                        {
                            "id": 7,
                            "user": {
                                "login": "chatgpt-codex-connector[bot]",
                                "type": "Bot",
                            },
                            "commit_id": HEAD,
                            "state": "COMMENTED",
                            "submitted_at": "2026-07-15T20:08:00Z",
                        }
                    ]]
                )
            if argv[:2] == ["gh", "api"]:
                return 0, "[[]]"
            return 1, ""

        config = {
            "repo": "owner/repo",
            "pr": 59,
            "head": HEAD,
            "request_comment": 99,
            "rounds": 1,
            "interval": 1,
            "grace": 0,
            "bot_logins": BOT_LOGINS,
        }
        code, result = MODULE.watch(
            config,
            runner=runner,
            waiter=lambda seconds, log: True,
            log=lambda message: None,
        )
        self.assertEqual(code, MODULE.EXIT_INFRA)
        self.assertEqual(result["error"], "head-changed-before-terminal")

    def test_notify_uses_send_only_and_registered_sender(self):
        calls = []

        def runner(argv, timeout):
            calls.append(argv)
            return 0, "sent"

        self.assertTrue(
            MODULE.notify(["@lifo", "@bulo"], "lifo", "inform", "done", runner)
        )
        self.assertEqual(calls[0][0:2], ["hcom", "send"])
        self.assertIn("--name", calls[0])
        self.assertIn("lifo", calls[0])
        self.assertNotIn("listen", calls[0])


if __name__ == "__main__":
    unittest.main()
