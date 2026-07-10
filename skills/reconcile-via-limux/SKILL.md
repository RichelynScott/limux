---
name: reconcile-via-limux
description: Reconcile a dirty project checkout and its Git worktrees by verifying Limux topology, mapping file ownership, resuming only required owners through hcom in explicit healthy Limux surfaces, landing unique work through reviewed branches and PRs, and closing stale worktrees without loss. Use for dirty worktree cleanup, post-crash repo reconciliation, merged-but-dirty branches, stale hcom-owned files, or requests to reopen owners in panes to clean a project.
---

# Reconcile Via Limux

Reconcile repository state without losing user or peer work and without typing
instructions into unverified terminal panes. This skill composes
`dirty-worktree-owner-cleanup`, `limux-use-guide`, `hcom`, `github-cli`, and
`taskmaster-reviewed`; those skills remain canonical for their mechanics.

## Preconditions

1. Confirm the operator authorized reconciliation. Cleanup authorization does
   not bypass secret, production, force-push, or unrelated destructive gates.
2. Load the composed skills above. Load `global-config-change-control` only when
   the result is a proposed global skill or policy.
3. Verify the project manager identity and track meaningful work in the
   project's existing TaskMaster store.
4. Never use blanket `git add`, reset, clean, checkout, restore, worktree force
   removal, or session termination before the exact owner map exists.

## Workflow

### 1. Snapshot Repository And Runtime State

Capture the primary checkout, every worktree, remote base, open PRs,
TaskMaster, hcom managers and sessions, and exact Limux workspace topology.
Use `scripts/reconcile_snapshot.py --name <hcom-name>` for a compact read-only
snapshot. See `references/command-surface.md` for exact command shapes.

### 2. Build The Exact Owner Map

Classify every dirty path and worktree as current-session-owned,
active-peer-owned, stale-hcom-owned, unsafe-worker-owned, generated-artifact,
user-owned, or unknown. Evidence priority is operator instruction, current
session changes, live hcom metadata, branch/PR ownership, durable repo records,
then the static roster.

Distinguish apparent dirt caused by an old merged checkout from genuinely
local-only content. Compare each path against current remote base before
deciding it needs a commit or owner.

### 3. Decide Which Sessions Are Required

Resume a session only when its direct ownership or judgment is necessary. Do
not reopen reviewers merely because inbound review files remain, and do not
reopen an owner whose completion record already classifies remaining dirt.

Before resume, verify a real transcript identifier, safe session state, an
idle healthy right-side surface, and matching recovered cwd/branch. If hcom's
stored UUID is missing, search lifecycle and transcript indexes for the last
real transcript. Retry once with the verified UUID; never cycle through guesses.

### 4. Resume Through An Explicit Limux Surface

1. Read the exact target surface immediately before injection.
2. Send `hcom r <verified-id> --run-here --go` to that explicit workspace and
   surface, then send Enter separately.
3. If an unambiguous resume-summary confirmation appears, verify it before
   sending the confirmation key.
4. Wait for hcom to show expected identity, process binding, cwd, live
   delivery, and terminal control.
5. Send the assignment through hcom, never by pasting it into the pane.

Use `assets/cleanup-brief-template.md` for the bounded assignment.

### 5. Reconcile Branches And Files

- Stop adding commits to merged or spent branches.
- Preserve unique work on a fresh branch from current remote base.
- Stage exact owned paths only and use a PR when content should survive.
- Treat current-main-identical paths as checkout reconciliation, not new work.
- Route peer-owned paths to their owner and park unknown paths.
- If `.git` is read-only, publish through a clean temporary clone of the
  already-pushed branch/base, applying only the verified delta.

### 6. Close Worktrees Through The No-Loss Gate

Before removal, prove the branch is merged or pushed, no unique TaskMaster or
evidence state remains, generated artifacts have an authorized disposition,
and the lane is inactive. If any gate fails, keep the worktree and report the
blocker. Never use force removal merely to improve the inventory count.

### 7. Verify Closure

Report remote base SHA and PRs, primary checkout state, all remaining
worktrees, TaskMaster status, resumed sessions, committed versus untouched
paths, and runtime install/restart status. A merged branch alone is not clean.

## Global Promotion

This project skill is the mechanics canonical. Promotion requires source
classification and commit SHA, matching Codex and Claude manager-inbox
requests, a cross-runtime handshake, adversarial review, and real dogfood.
Each runtime owner installs only its owned mirror. Do not self-install this
draft globally.
