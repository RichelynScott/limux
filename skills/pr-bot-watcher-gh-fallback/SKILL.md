---
name: pr-bot-watcher-gh-fallback
description: >
  TEMPORARY fallback for requesting and watching Codex GitHub PR reviews while
  the canonical pr-bot watcher cannot register a dedicated hcom identity. Uses
  gh CLI exact-head polling, treats eyes reactions as pending, notifies existing
  owner sessions without consuming hcom messages, and drives fix/review loops.
  Use for PR bot waiting, @codex review, exact-head bot review, or watcher
  failures until the canonical watcher is repaired and this skill is retired.
---

# Temporary GH PR-Bot Watcher Fallback

> [!CAUTION]
> Temporary operational fallback. The canonical `$pr-bot-watcher-fix-loop`
> dedicated-identity path is unusable with installed hcom v0.7.66. Do not use
> its watcher helper, create an improvised hcom identity, or run `hcom listen`
> under a manager identity. Retire this skill after the canonical path is fixed.

## Purpose

Run a bounded, model-less PR review watch without consuming hcom message
cursors. GitHub reads and the review request use authenticated `gh`. Waiting is
process-local. Completion is sent once through an already registered owner
identity using `hcom send`; the watcher never calls `hcom listen`.

Firecrawl monitor is not the primary path: its minimum schedule is 15 minutes,
it is unavailable in the current Limux runtime, and page-change detection does
not provide exact review commit IDs. It may be a coarse public-page wake-up
backup only. Exact-head classification must still use `gh`.

## Contract

- Freeze the PR's full `headRefOid` before requesting or watching.
- Post at most one `@codex review` request for that head. The request must name
  the full head SHA or a prefix of at least 10 characters. A host-local
  repo/PR/head lock serializes the read-recheck-post sequence, including a
  fresh head check immediately before POST.
- Poll all four surfaces: PR reviews, inline review comments, issue comments,
  and reactions on the request comment. Paginated `gh api` reads must use
  `--slurp` and flatten every page.
- Accept only exact allowlisted GitHub logins whose API `user.type` is `Bot`.
  Human accounts with `codex` or `chatgpt` in their names are never bot proof.
- Confirm that the request comment belongs to the watched repository and PR.
  A caller-supplied comment ID must equal the prepared current-PR request.
- An `eyes` reaction means acknowledged/pending only. It is never success,
  greenlight, or a terminal result.
- Stop successfully only for an exact-head bot review/comment or the locally
  documented terminal `+1` reaction. Re-read the head after all review
  surfaces and immediately before terminal return; a changed head fails closed.
- Never put bot bodies in watcher logs or hcom notifications. The owner reads
  bodies directly with `gh` after notification.
- A watcher result is evidence intake, not automatic approval. The owner
  independently classifies findings and verifies the current head.
- The watcher does not edit branches, fix code, merge, or install anything.
- Under the operator's 2026-07-15 authorization, the owning session may merge
  only after required tests pass and the bot explicitly greenlights the exact
  current head with no unresolved P1/P2 finding.

## Prepare

The script accepts either a PR URL or a PR number plus `--repo`.

```bash
python3 skills/pr-bot-watcher-gh-fallback/scripts/pr_bot_watch_fallback.py \
  prepare https://github.com/OWNER/REPO/pull/123 --request-review
```

`prepare` resolves the canonical URL and head. It reuses an existing exact-head
review request and only posts a new one when `--request-review` is explicit.
If an existing request was edited for the new head, the later edit timestamp is
the response eligibility boundary. Save the returned `head` and
`request_comment` fields.

## Watch

Run in a persistent foreground/background execution slot that survives the
launching tool call. Do not create an hcom worker merely to wait.

```bash
python3 skills/pr-bot-watcher-gh-fallback/scripts/pr_bot_watch_fallback.py \
  watch https://github.com/OWNER/REPO/pull/123 \
  --head FULL_HEAD_SHA \
  --request-comment COMMENT_ID \
  --rounds 20 --interval 60 --grace 90 \
  --sender-name OWNER_HCOM_NAME \
  --notify @OWNER_HCOM_NAME \
  --log /tmp/pr-123-bot-watch.log
```

The process exits:

- `0`: terminal exact-head bot response detected and notification delivered.
- `2`: bounded timeout after a successful final all-surface poll.
- `3`: fail-closed precondition, GitHub, head-change, or notification failure.

## Review And Fix Loop

After a terminal notification:

1. Re-read the current `headRefOid`; it must equal the watched head.
2. Inspect review bodies and inline comments directly with repo-scoped `gh api`.
3. Classify each finding as owned-fix, peer-route, ambiguous, deferred, or
   noise. Verify findings against code before changing anything.
4. For owned fixes, patch the PR branch in its owned lane, run the repository's
   required checks, commit, and push.
5. The push creates a new head. Run `prepare --request-review` and `watch` again
   for that new head. Never let an old-head result vouch for new bytes.
6. When the exact current head receives an explicit no-findings greenlight and
   required checks pass, independently verify mergeability and merge using the
   repository's approved `gh pr merge` method. Reconcile local branches after.

## Validation

```bash
python3 -m unittest discover \
  -s skills/pr-bot-watcher-gh-fallback/tests -v
python3 /home/riche/.codex/scripts/static_check_no_delete_api.py \
  --target-dir skills/pr-bot-watcher-gh-fallback \
  --out-tsv "${TMPDIR:-/tmp}/pr-bot-watcher-gh-fallback-no-delete.tsv"
```
