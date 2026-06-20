# Halo Limux Handoff

Author/runtime/date: halo / Codex GPT-5 / 2026-06-20 EDT

## Immediate Next Action

1. Keep `LIFO_HANDOFF.md` and `archive/` untouched unless lifo transfers
   ownership.
2. Pull lifo's read-only findings from the sibling worktree lane for:
   - manual workspace unread marking from a right-click context menu;
   - a three-second blue pane/session attention outline after hover.
3. Implement the smallest Limux-native change that satisfies one request at a
   time, starting with whichever path lifo's report shows is least invasive.
4. Verify with targeted Rust checks first, then the Xvfb smoke harness if the
   GTK bridge/UI path changed.

## Current Baseline

| Item | Value |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Branch | `lifo/reboot-handoff-20260619` |
| HEAD | `1df7621 fix(host): isolate concurrent runtime launches` |
| Installed `limux` | `/home/riche/.local/limux-reviewed/multi-runtime-isolation-20260620/bin/limux` |
| PR intake | No open PRs on `RichelynScott/limux` at intake. |
| Runtime health | `limux --json surface-health` reported two healthy terminal surfaces. |
| Build health | `cargo check -p limux-host-linux` passed. |

## Active Coordination

- Lifo confirmed via hcom `#113038` that he does not own root `HANDOFF.md`.
- Lifo's active lane is read-only investigation in
  `/home/riche/MCPs/limux-workspaces-sidebar-notifications`, branch
  `lifo/workspaces-sidebar-notifications-20260620`.
- His investigation scope is workspace context-menu/unread helpers, pane widget
  classes/focus/hover controllers, and terminal notification target routing.
- Halo owns this root handoff rewrite and should continue implementation only
  after checking lifo's final report or directly re-reading the same code paths.

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
```

## Do Not Resume

Do not resume the old Project Isolation Lab / VM goal from this Limux session.
That lane belongs to `/home/riche/Proj/SUPPLY_CHAIN_SECURITY`; this repo is now
focused on Limux product/runtime improvement.
