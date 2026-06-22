# Limux Copy/Paste Defect - 2026-06-22

## Goal

Fix the user-visible Limux copy/paste regression without disturbing active
peer-owned work in the main checkout.

## Scope

- Worktree: `/home/riche/MCPs/limux-copy-paste-fix-20260622`
- Branch: `lifo/fix-copy-paste-20260622`
- Base: `origin/halo/limux-ui-improvements-20260620`

## Reported Symptoms

- Copy/paste is very broken in Limux.
- Earlier reports mention a pane getting stuck as if left click remains held,
  causing terminal text selection to continue while the mouse moves.
- Earlier reports also mention copy/paste and pane-border issues when two
  Limux runtimes are open.

## TaskMaster Status

This branch did not have a committed `.taskmaster/` task store when the fix lane
started. Per the reviewed TaskMaster rules, this document records the task
surface without hand-creating `.taskmaster/tasks/tasks.json` or inventing task
IDs. If long-lived tracking is needed, populate tasks through the reviewed
TaskMaster wrapper path and SCRIM-backed PRD parsing.

## Done Criteria

- Root cause documented with code references.
- Focused regression coverage added where practical.
- Copy/paste or stuck-selection behavior fixed at the event-routing source.
- Relevant Limux checks pass.

## Root Cause Notes

- Paste could be skipped before reading the clipboard when WSLg/Wayland exposed
  readable text but reported sparse or nonstandard GTK clipboard formats.
- Limux mirrored Ghostty standard clipboard writes into PRIMARY selection, and
  selection writes into the standard clipboard. That made terminal selection or
  copy actions in one pane/runtime clobber paste content in another.
- Stuck terminal selection remained possible if GTK delivered a release with an
  unknown button or the pointer left a GLArea before the matching release.

## Verification

- `cargo fmt --check`
- `git diff --check`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib cargo test -p limux-host-linux terminal::tests -- --nocapture`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib cargo test -p limux-host-linux -- --nocapture`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib cargo clippy -p limux-host-linux --all-targets -- -D warnings`
