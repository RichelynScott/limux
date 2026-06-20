# Halo Limux Handoff

Author/runtime/date: halo / Codex GPT-5 / 2026-06-20 EDT

## Immediate Next Action

1. Keep `LIFO_HANDOFF.md` and `archive/` untouched unless lifo transfers
   ownership.
2. Review or merge `halo/limux-ui-improvements-20260620` at `b09c7f8`.
3. Manually validate in a live Limux window after launching/merging/installing
   that branch. The active installed user-local `limux` is still the
   pre-feature `multi-runtime-isolation-20260620` build.
4. Continue the separate GTK-critical investigation from the host log if the
   operator wants the next runtime fix.

## Current Baseline

| Item | Value |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Branch | `halo/limux-ui-improvements-20260620` |
| HEAD | `b09c7f8 feat(host): add workspace unread and pane attention markers` |
| Installed `limux` | `/home/riche/.local/limux-reviewed/multi-runtime-isolation-20260620/bin/limux` (pre-feature build) |
| PR intake | No open PRs on `RichelynScott/limux` at intake. |
| Runtime health | `limux --json surface-health` reported two healthy terminal surfaces. |
| Build health | `cargo test -p limux-host-linux` and debug Xvfb smoke passed. |

## Active Coordination

- Lifo confirmed via hcom `#113038` that he does not own root `HANDOFF.md`.
- Lifo's active lane is read-only investigation in
  `/home/riche/MCPs/limux-workspaces-sidebar-notifications`, branch
  `lifo/workspaces-sidebar-notifications-20260620`.
- His investigation scope is workspace context-menu/unread helpers, pane widget
  classes/focus/hover controllers, and terminal notification target routing.
- Halo consumed the same code paths and pushed the implementation branch. Lifo
  should be treated as reviewer/coordination peer for overlap, not owner of
  the implementation branch.

## Implemented In `b09c7f8`

- Added `Mark Unread` to the workspace right-click context menu.
- Reused existing sidebar unread visuals with `Marked for follow-up`.
- Added a blue `.limux-pane-attention` outline to pane widgets when
  terminal-originated notifications or bells identify an unfocused pane.
- Added hover clearing: after the pointer enters an outlined pane, the outline
  remains for three seconds, then clears.
- Added a focused unit test for pane attention target selection.

## Runtime Findings To Carry Forward

- Latest host log tail no longer shows the earlier GSettings schema-source
  critical or split-icon warnings.
- The log still shows GTK criticals around:
  - `gtk_scrolled_window_get_child`
  - `gtk_viewport_get_child`
  - `gtk_stack_set_visible_child_name`
- WSL EGL/Mesa/Zink/GDK compositor warnings remain, but currently have no direct
  confirmed Limux correctness impact.
- Concurrent runtime launch isolation landed in `1df7621`, adding fallback
  socket behavior when the default control socket is in use.

## Verification Already Run In This Resume

```bash
git status --short --branch
git log --oneline --decorate -8
git worktree list
gh pr list --repo RichelynScott/limux --state open --json number,title,headRefName,author,url,updatedAt --limit 50
readlink -f /home/riche/.local/bin/limux
tail -n 200 /home/riche/.local/state/limux/logs/limux-host.log
limux --json surface-health
cargo check -p limux-host-linux
cargo fmt --check
cargo test -p limux-host-linux
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
```

## Do Not Resume

Do not resume the old Project Isolation Lab / VM goal from this Limux session.
That lane belongs to `/home/riche/Proj/SUPPLY_CHAIN_SECURITY`; this repo is now
focused on Limux product/runtime improvement.
