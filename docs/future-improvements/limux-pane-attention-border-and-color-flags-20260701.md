# Pane Attention Borders And Color Flags

Date: 2026-07-01
Owner: lifo
TaskMaster: #20
Priority: medium

## Reference

Screenshot:
`docs/future-improvements/screenshots/limux-pane-attention-border-layering-20260701.png`

The screenshot shows the right pane trying to show a blue attention outline, but
the border is only barely visible along the right edge and small portions of the
top-right and bottom-right corners. The likely failure mode is that the
attention border is painted behind pane/tab/content layers instead of above
them.

## Problem

When a pane needs attention, the blue outline should clearly identify the actual
pane. The current rendering is too subtle or partially hidden, so it does not
reliably guide the operator back to the pane that needs action.

The pane context menu also needs parity with workspace flagging. A user should
be able to right-click a pane and mark it unread, or assign a temporary/manual
border color to return to later.

## Desired Behavior

- Attention outline renders above pane content and tab/header layers.
- Attention outline is visible around the actual pane bounds, not only one edge.
- Right-clicking a pane offers:
  - mark unread;
  - highlight/color submenu;
  - clear pane color.
- Pane colors are independent from workspace highlight colors.
- Settings can configure when the blue notification border clears:
  - hover over pane;
  - begin typing in pane;
  - hover inside pane for configurable seconds;
  - combinations such as hover-timer plus typing.
- Workspace sidebar entries mirror pane border colors without replacing the
  workspace highlight color. Example: in a 50/50 split workspace, if the left
  pane is orange and the right pane is yellow, the workspace row border should
  show orange on the left half and yellow on the right half.

## Acceptance Notes

This should be implemented after higher-priority runtime isolation and active
stability fixes. It will likely touch pane overlay layering, pane context-menu
actions, settings storage, workspace sidebar rendering, and session
persistence.

