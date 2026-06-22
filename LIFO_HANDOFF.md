# Limux Lifo Reboot Handoff

Author/runtime/date: lifo / Codex GPT-5 / 2026-06-19 20:15 EDT.

## Immediate Next Action

1. After DARTH-PC restarts, resume in `/home/riche/MCPs/limux`.
2. Run:
   ```bash
   git status --short --branch
   git log --oneline -3
   readlink -f ~/.local/bin/limux
   tail -n 200 ~/.local/state/limux/logs/limux-host.log 2>/dev/null || true
   ```
3. Fully quit any old Limux windows, then launch fresh with `limux`.
4. Validate the two user-reported behaviors manually:
   - Open a new workspace. Expected: no GSettings crash.
   - Try mouse selection in one pane, drag/release/cancel, move into other panes. Expected: no stuck left-click selection state.
5. If either issue reproduces, preserve `~/.local/state/limux/logs/limux-host.log` and the exact launch environment (`env | sort | rg '^(GSETTINGS|GTK|GDK|XDG|LIMUX)_'`).

### HUMAN NOTE/ADD: THIS SECTION AND REQUEST WAS DIRECTLY ADDED BY HUMAN AFTER ALL SESSIONS FINISHED COMPACTION AND I CLOSED THEM OUT AND I WAS GOING TO CLOSE DOWN THE LIMUX PROCESS BUT SAW THESE ERRORS I WANTED TO MAKE SURE WE DOCUMENTED SO YOU DOUBLE CHECK THAT THESE ARE GETTING ADDRESSED OR GOT ADDRESSED: 
"""
➜  ~ limux

(limux-host:99589): GLib-GIO-CRITICAL **: 14:52:13.844: g_settings_schema_source_lookup: assertion 'source != NULL' failed

(limux-host:99589): Gtk-WARNING **: 14:52:13.902: While adding page: duplicate child name in GtkStack: terminal-0

(limux-host:99589): Gtk-CRITICAL **: 14:52:13.902: gtk_box_append: assertion 'gtk_widget_get_parent (child) == NULL' failed
limux: control socket at /run/user/1000/limux/limux.sock

(limux-host:99589): Gtk-WARNING **: 14:52:13.913: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-horizontal-symbolic.svg: Unrecognized image file format

(limux-host:99589): Gtk-WARNING **: 14:52:13.913: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-vertical-symbolic.svg: Unrecognized image file format
libEGL warning: failed to get driver name for fd -1

libEGL warning: MESA-LOADER: failed to retrieve device information

libEGL warning: failed to get driver name for fd -1

MESA: error: ZINK: vkCreateInstance failed (VK_ERROR_INCOMPATIBLE_DRIVER)
libEGL warning: egl: failed to create dri2 screen

(limux-host:99589): Gdk-WARNING **: 14:56:20.383: Compositor doesn't support moving popups, relying on remapping

(limux-host:99589): Gtk-WARNING **: 14:56:54.573: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-horizontal-symbolic.svg: Unrecognized image file format

(limux-host:99589): Gtk-WARNING **: 14:56:54.573: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-vertical-symbolic.svg: Unrecognized image file format

(limux-host:99589): Gtk-CRITICAL **: 15:30:48.417: gtk_scrolled_window_get_child: assertion 'GTK_IS_SCROLLED_WINDOW (scrolled_window)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:48.417: gtk_viewport_get_child: assertion 'GTK_IS_VIEWPORT (viewport)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:48.418: gtk_stack_set_visible_child_name: assertion 'GTK_IS_STACK (stack)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:50.334: gtk_scrolled_window_get_child: assertion 'GTK_IS_SCROLLED_WINDOW (scrolled_window)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:50.334: gtk_viewport_get_child: assertion 'GTK_IS_VIEWPORT (viewport)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:50.334: gtk_stack_set_visible_child_name: assertion 'GTK_IS_STACK (stack)' failed
"""

## Completed This Session

| Time | Item | Evidence |
|---|---|---|
| 2026-06-19 | Diagnosed startup warnings and pane mouse-selection issue from the user-provided logs. | Used `$systematic-debugging`; inspected `layout_state.rs`, `terminal.rs`, `window.rs`, installer wrapper. |
| 2026-06-19 | Fixed duplicate `terminal-0` GTK stack child names from cloned tab IDs in normalized session state. | Commit `e79a1ac fix(host): stabilize startup and mouse release`, pushed to `main`. |
| 2026-06-19 | Fixed the stuck mouse-left-selection symptom by tracking active mouse button state and synthesizing releases on cancel/stop/motion-with-button-up. | Commit `e79a1ac`, host tests passed. |
| 2026-06-19 | Fixed installed icon theme lookup path for split icons. | Commit `e79a1ac`; installed icon SVG files are valid SVGs. |
| 2026-06-19 | Found likely new-workspace crash root cause: old Limux terminal inherited `XDG_DATA_DIRS=/home/riche/.local/limux-reviewed/7e3693cc7053/share`, hiding `/usr/share` GSettings schemas. | `gsettings list-schemas` failed under private-only `XDG_DATA_DIRS` and succeeded with `/usr/local/share:/usr/share`. |
| 2026-06-19 | Added automatic host stderr logging. | Commit `596bc69 fix(host): add startup logging and schema env repair`, pushed to `main`; default log path is `~/.local/state/limux/logs/limux-host.log`. |
| 2026-06-19 | Repaired host and installer `XDG_DATA_DIRS` handling so inherited private-only values keep system schema dirs. | Commit `596bc69`; targeted bad-env Xvfb smoke had no `g_settings_schema_source_lookup` critical. |
| 2026-06-19 | Installed patched user-local build. | Active symlink: `/home/riche/.local/bin/limux -> /home/riche/.local/limux-reviewed/runtime-logs-xdg-20260619/bin/limux`. |
| 2026-06-19 | Ran verification. | `cargo fmt --check`; targeted host tests; `cargo check -p limux-host-linux`; release builds; Xvfb bad-env smoke; installer no-delete static scan; installer syntax check; SHA256 install manifest; `./scripts/check.sh`. |
| 2026-06-19 | Checked GitHub PR directive. | `gh pr list --repo RichelynScott/limux --state open` returned `[]`; informed `reko`. |

## Key Files For Context

| Path | Purpose |
|---|---|
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/layout_state.rs` | Session/layout normalization; fixed duplicate cloned tab IDs in `e79a1ac`. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/terminal.rs` | Ghostty surface event handling; fixed stuck mouse-selection release path in `e79a1ac`. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/main.rs` | Host process startup; now repairs `XDG_DATA_DIRS` and installs stderr log redirection. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/window.rs` | GTK window/dialog code and color-scheme schema lookup; now avoids calling lookup on a null default schema source. |
| `/home/riche/MCPs/limux/scripts/user-local-install/install-user-local.sh` | User-local installer wrapper; now appends system XDG data dirs even when inheriting a bad private-only value. |
| `/home/riche/.local/state/limux/logs/limux-host.log` | Automatic host stderr log for new patched launches. |
| `/home/riche/.local/limux-reviewed/runtime-logs-xdg-20260619/` | Currently installed patched user-local build. |

## Current Git State And Branching

- `main` was pushed to `origin/main` at `596bc69` before the reboot directive.
- This reboot handoff is on branch `lifo/reboot-handoff-20260619` to satisfy the directive not to push new WIP directly to `main`.
- Pre-existing dirty state not owned by lifo:
  - `HANDOFF.md` has an existing dirty diff from a prior Halo/Limux/SCS handoff update.
  - `archive/` is untracked and contains smoke-test cleanup artifacts created under the repo archive path because delete-style cleanup is disallowed.
- Do not stage or rewrite the dirty shared `HANDOFF.md` unless explicit ownership is assigned.

## Critical Behavior Rules

- Logs are automatic only for newly launched patched Limux hosts. Existing old Limux windows still use the old binary and environment.
- If the new-workspace crash still reproduces, treat `~/.local/state/limux/logs/limux-host.log` as the first evidence source.
- Do not route unrelated work to `rumi`; current hcom directive scopes `rumi` only to `/home/riche/Proj/hermes-agent`.
- During the reboot stand-down window, do not start new long-running work and do not push new WIP directly to `main`.
- Preserve user/peer worktree changes. The dirty shared `HANDOFF.md` predates this reboot handoff and was intentionally left untouched.

## Residual Risks

- Manual live validation after a full restart/relaunch is still needed for the new-workspace dialog and stuck mouse-selection issue.
- EGL/Mesa/Zink warnings in the original log appear environment/driver related and were not treated as the crash root cause.
- GDK popup movement warning appears compositor capability related and was not treated as a Limux correctness bug.
- If icon warnings remain after a fresh patched launch, inspect runtime icon-theme registration and installed icon cache behavior; source SVG files are valid.
