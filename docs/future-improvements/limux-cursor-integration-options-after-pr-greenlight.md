# Limux Cursor Integration Options

Date: 2026-06-22
Source: lifo / Codex / Limux PR cleanup lane
Status: idea/options only
Gate: Do not start implementation or TaskMaster tasking until the operator
explicitly opens a post-merge Cursor/Limux integration lane.

Update 2026-06-30: the operator opened a planning/research lane, not an
implementation lane. The current draft plan is
`docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`; use
that file for the latest v1/v2 shape and keep this original options note as
historical context.

Update 2026-06-30 after GLM/MiniMax review: the current v1 plan explicitly
does not include terminal text/key injection from Cursor. Treat the "Send text"
idea below as historical only; any future terminal input/attach path belongs in
a separate v2 PRD with server-side trust-boundary review.

## Context

The operator likes Limux for running terminal and agent sessions, but misses
Cursor's left-side file explorer while working inside Limux. The June 22 Limux
PR stack has merged, so this note preserves future improvement options without
starting implementation.

Merged PR context:

- PR #2: `halo/limux-ui-improvements-20260620` -> `main`
- PR #3: `lifo/workspaces-sidebar-notifications-20260620` -> `main`
- PR #4: `lifo/fix-copy-paste-20260622` -> `main`

No implementation should begin until the operator explicitly opens the next
improvement lane.

## Options

### Option A - Open Workspace In Cursor

Add an `Open in Cursor` action to Limux workspace rows/context menus. It would
launch `cursor <workspace-folder>` using the workspace `folder_path` or `cwd`.

Pros:

- Smallest useful integration.
- Keeps Cursor as the file explorer/editor.
- Avoids trying to embed a GTK app inside Cursor.
- Low risk after the PR stack lands.

Cons:

- Cursor and Limux remain separate windows.
- Does not show Limux panes inside Cursor.

Suggested future acceptance:

- Workspace context menu has `Open in Cursor`.
- Action is disabled or hidden when no workspace folder/cwd is known.
- Launch errors show a non-intrusive Limux notification/log message.

### Option B - Cursor Extension For Limux Control

Build a Cursor/VS Code extension that talks to the Limux Unix socket and shows
Limux workspaces/panes in a Cursor side panel.

Possible commands:

- Focus Limux workspace.
- Create Limux pane.
- Send text to selected pane.
- Mark workspace unread.
- Open the selected Cursor folder as a Limux workspace.

Pros:

- Gives a real Cursor-side navigation surface.
- Keeps Limux as the terminal runtime.
- Could make Cursor and Limux feel like one workflow.

Cons:

- Requires extension development, packaging, and socket-auth design.
- More moving parts than a simple launcher action.
- Needs careful security review around socket access and command sends.

### Option C - Limux Native File Explorer Panel

Add a file explorer panel directly inside Limux, likely near or replacing part
of the workspace sidebar.

Pros:

- Keeps everything in Limux.
- Could be tuned for agent workflows and workspace/pane context.

Cons:

- Larger UI project.
- Duplicates a mature Cursor feature.
- Needs file-tree performance, filtering, keyboard navigation, and editing/open
  behavior decisions.

### Option D - Side-By-Side Layout Runbook

Document a preferred operator workflow using Cursor for files and Limux for
terminal/agent panes, with launch commands and window placement conventions.

Pros:

- No code.
- Can be adopted immediately after PR merge without technical risk.

Cons:

- Does not remove window-management friction.
- Not a product feature.

## Recommended Future Order

1. Start with Option A (`Open in Cursor`) when the operator opens the next
   improvement lane.
2. If the workflow proves useful but still awkward, evaluate Option B as a
   dedicated extension project.
3. Consider Option C only if Cursor-side integration is not enough or if Limux
   should become a broader workbench.

## Non-Goals For This Note

- Do not add Cursor UI or commands here.
- Do not initialize TaskMaster tasks for this yet.
- Do not open a Cursor-integration implementation branch or PR yet.
- Do not modify the installed runtime for this idea before the operator
  authorizes the next improvement lane.
