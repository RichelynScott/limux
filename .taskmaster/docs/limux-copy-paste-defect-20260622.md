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

This branch has a broken/uninitialized `.taskmaster/` skeleton, not a verified
empty TaskMaster store. `task-master-reviewed list` and `next` reported missing
configuration and fell back to defaults, which means `.taskmaster/config.json`
is absent and the output must not be interpreted as "zero valid tasks".

Per the reviewed TaskMaster rules, this document records the task surface
without hand-creating `.taskmaster/tasks/tasks.json` or inventing task IDs. If
long-lived tracking is needed, repair the store with operator-approved
`task-master-reviewed init`, then populate tasks through the reviewed
TaskMaster wrapper path and SCRIM-backed PRD parsing.

2026-06-22 follow-up: Soho relayed operator approval for an in-place
`task-master-reviewed init` repair in this worktree. The first recipe failed
because `task-master init --version <value>` collided with the CLI global
version flag and exited after printing `0.43.1`. The corrected command omitted
`--version`, created `.taskmaster/config.json` and
`.taskmaster/tasks/tasks.json`, preserved this seed doc, and `list` now runs
without missing-configuration warnings.

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
  unknown button.
- The first fix over-corrected stuck selection by treating motion/enter/leave
  events with missing `BUTTON1_MASK` as a release. Under WSLg this can happen
  while the physical left mouse button is still held, so Ghostty completed and
  copied a partial selection before the user released the mouse.
- Follow-up reproduction showed the same symptom after motion/enter/leave cleanup.
  The remaining premature-release path was `GtkGestureClick` `cancel`/`stopped`:
  those are gesture-recognition lifecycle signals and can fire when a pointer
  movement becomes a drag, so treating them as physical button release ends the
  Ghostty selection too early.
- A Claude plugin rescue review found one more contributor that matches the
  user's "Copied to clipboard" toast during drag: Ghostty's Linux
  `copy-on-select` writes the PRIMARY/selection clipboard as the selection grows,
  and Limux showed the visible copy toast for selection-clipboard writes as well
  as explicit standard-clipboard writes. Limux now keeps PRIMARY selection writes
  but only shows the toast for standard clipboard writes, so drag-selecting text
  should not produce the premature "Copied to clipboard" signal.
- Operator follow-up confirmed the drag bug was fixed but asked to restore the
  prior automatic regular-clipboard copy and bottom notification. Limux now
  caches PRIMARY/selection writes during drag and promotes the latest non-empty
  selection to the standard clipboard only after left-button release, then shows
  the bottom copy toast. This restores auto-copy notification behavior without
  reintroducing mid-drag copy/toast interruptions.

## Verification

- `cargo fmt --check`
- `git diff --check`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib cargo test -p limux-host-linux terminal::tests -- --nocapture`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib cargo test -p limux-host-linux -- --nocapture`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib cargo clippy -p limux-host-linux --all-targets -- -D warnings`

2026-06-22 follow-up for commit `4bfae87`:

- `cargo fmt --check`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux-copy-paste-fix-20260622/ghostty/zig-out/lib cargo test -p limux-host-linux terminal::tests -- --nocapture`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux-copy-paste-fix-20260622/ghostty/zig-out/lib cargo clippy -p limux-host-linux --all-targets -- -D warnings`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux-copy-paste-fix-20260622/ghostty/zig-out/lib cargo build -p limux-host-linux --bin limux -p limux-cli --bin limux-cli`
- `scripts/user-local-install/install-user-local.sh --apply --profile debug --install-id copy-paste-toast-fix-20260622-4bfae87`

Installed reviewed runtime:

- `/home/riche/.local/limux-reviewed/copy-paste-toast-fix-20260622-4bfae87`
- `~/.local/bin/limux` and `~/.local/bin/limux-cli` point at this install.
- Any already-running Limux process must be restarted before the fix is live in
  that window.

2026-06-22 follow-up for restored release-time auto-copy:

- `cargo fmt --check`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux-copy-paste-fix-20260622/ghostty/zig-out/lib cargo test -p limux-host-linux terminal::tests -- --nocapture`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux-copy-paste-fix-20260622/ghostty/zig-out/lib cargo clippy -p limux-host-linux --all-targets -- -D warnings`
- `LD_LIBRARY_PATH=/home/riche/MCPs/limux-copy-paste-fix-20260622/ghostty/zig-out/lib cargo build -p limux-host-linux --bin limux -p limux-cli --bin limux-cli`
