# Limux Workspaces Sidebar And Notifications - 2026-06-20

## Scope

Improve the Limux WORKSPACES sidebar and attention indicators:

1. Hide the sidebar fully while leaving a visible restore ribbon.
2. Allow compact sidebar widths below 220px while keeping text readable down to
   8pt before ellipsis.
3. Alert when Codex is waiting for user input through the AskUserQuestion /
   request_user_input tool path.
4. Allow manual right-click Mark Unread / Mark Read on workspace rows.
5. Outline panes that need attention for pane-specific terminal notifications.

## TaskMaster Experience

This repo did not have a usable `.taskmaster/tasks/tasks.json` at session start.
The reviewed wrapper aliases were verified to resolve to `task-master-reviewed`.

Commands attempted from this feature worktree:

```bash
TASKMASTER_REVIEWED_OFFLINE=1 task-master-reviewed --help
TASKMASTER_REVIEWED_OFFLINE=1 task-master-reviewed list
TASKMASTER_REVIEWED_OFFLINE=1 task-master-reviewed init --yes --name limux --description 'Limux workspace sidebar and notification improvements' --version 0.1.0 --author lifo --skip-install --no-aliases --no-git --git-tasks
TASKMASTER_REVIEWED_OFFLINE=1 task-master-reviewed next
```

Observed behavior:

- The reviewed wrapper ran successfully inside the disposable container.
- `list` and `next` exited successfully but reported no tasks.
- `init` printed `0.43.1` and left `.taskmaster/docs/` and `.taskmaster/tasks/`
  directories, but still did not create a valid task store or config file.
- No `.taskmaster/tasks/tasks.json` was hand-created, because the current
  policy forbids manual TaskMaster store fabrication.

Practical result for this work:

- The live Codex plan remained the authoritative tracker for this session.
- This file is the TaskMaster seed/experience record for a future sanctioned
  `parse-prd` or fixed init path.
- First real task population remains blocked on a wrapper/source fix or a
  SCRIM-backed provider run of `parse-prd`.

G0 stability follow-up on 2026-06-20:

- `task-master-reviewed list` and `task-master-reviewed next` were re-run from
  the G0 worktree before subagent implementation.
- The wrapper still reported no usable tasks because this repo has only
  `.taskmaster/docs/` and `.taskmaster/tasks/`, with no checked-in
  `.taskmaster/config.json` or `.taskmaster/tasks/tasks.json`.
- No task IDs, statuses, or task-store files were invented. G0 was tracked with
  the live Codex plan and the subagent lane summaries instead.
- For future refinement, the wrapper should make this failure mode explicit:
  "TaskMaster initialized but no task store/config exists; run the approved
  bootstrap or parse-prd path" rather than presenting an empty task list that
  looks like valid project state.

## Completed Implementation Notes

- Sidebar compact-state runtime and persisted width now use
  `MIN_SIDEBAR_WIDTH` instead of snapping back to 220px.
- A restore ribbon appears when the sidebar is hidden and calls the existing
  sidebar toggle action.
- Workspace row CSS has compact and tiny classes, with text no smaller than
  8pt.
- Codex hook setup now installs a matcher-scoped `PreToolUse` hook for
  user-input tools and maps it to `user-input-needed`.
- Hook notification delivery failures are logged with workspace, surface, and
  socket context.
- Workspace context menus now include Mark Unread / Mark Read.
- Pane-specific terminal notifications mark the target pane with a blue outline
  and clear the outline three seconds after hover.

## Remaining Runtime Notes

Live-refreshing an already-running Limux process is not implemented. Installing
a new binary can affect only future launches; existing GTK host processes keep
running the old in-memory code until restart. True live update would need a
separate process-supervision/session-reparenting design and is future work.
