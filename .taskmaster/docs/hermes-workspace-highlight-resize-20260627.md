# Limux Hermes Notifications, Workspace Highlights, And Resize Stabilization - 2026-06-27

## Scope

1. Treat Hermes / hcom Hermes sessions as first-class Limux notification
   receivers without making Limux mutate Hermes lifecycle-plugin installation.
2. Add right-click workspace highlight colors with an Off action.
3. Preserve unread precedence: unread workspaces keep the existing blue unread
   treatment, while any user-selected highlight remains visible as an outline.
4. Reduce pane-resize mangling by preventing persisted split ratios from
   collapsing panes into edge slivers.

## TaskMaster Experience

Command run from this branch:

```bash
TASKMASTER_REVIEWED_OFFLINE=1 task-master-reviewed list
```

Observed behavior:

- The reviewed wrapper executed, but reported no `.taskmaster/config.json` at
  `/work` and warned that the project is not initialized.
- It still referenced `/work/.taskmaster/tasks/tasks.json` and reported no
  matching tasks.
- No task IDs or task statuses were invented, and no TaskMaster store files
  were manually edited.

Practical result:

- The live Codex plan tracked this branch.
- This document is the TaskMaster-compatible seed record for the implemented
  work and the wrapper behavior observed during the session.
- A future TaskMaster repair should use the approved bootstrap or parse-prd
  path before assigning durable task IDs.

## Implementation Notes

- `limux hermes-hook` and `limux hooks hermes <event>` now map Hermes lifecycle
  names such as `pre_approval_request`, `pre_llm_call`, `post_llm_call`,
  `pre_tool_call`, `post_tool_call`, `on_session_start`, `on_session_end`, and
  `on_session_finalize` into the existing notification/session-state path.
- Hermes payload fields may be read from top-level JSON, `extra`, or
  `metadata`, matching hcom/Hermes lifecycle payload shapes.
- `agent-team --agents hermes --launch-mode hcom` generates
  `hcom hermes --run-here`.
- Hermes restorable-session lookup now reads `hermes-hook-sessions.json`.
- Workspace session state now persists an optional highlight color.
- Workspace context menus include Highlight color actions and Off.
- Split ratio clamping now combines an 8% static edge guard with child-pixel
  minimums, so persisted drag positions are kept away from unrecoverable edge
  states.

## Verification

- `cargo fmt --check`
- `cargo check -p limux-host-linux`
- `cargo check -p limux-cli`
- `cargo test -p limux-cli hermes -- --nocapture`
- `cargo test -p limux-cli cli_arg_tests:: -- --nocapture`
- `cargo test -p limux-cli agent_launch -- --nocapture`
- `cargo test -p limux-host-linux layout_state::tests:: -- --nocapture`
- `cargo test -p limux-host-linux window::tests:: -- --nocapture`
- `cargo test -p limux-host-linux workspace_highlight_css -- --nocapture`
- `./scripts/check.sh`
- `./scripts/xvfb-smoke-test.sh`
