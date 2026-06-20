# Halo Limux Handoff

Author/runtime/date: halo / Codex GPT-5 / 2026-06-20 EDT

## Immediate Next Action

1. Keep `LIFO_HANDOFF.md` and `archive/` untouched unless lifo transfers
   ownership.
2. Keep `origin/lifo/workspaces-sidebar-notifications-20260620` at
   `49fb4cf3a15262fd4d09532c0f8fdc38ab8fdc45` as the recommended integration
   lane. Do not merge `halo/limux-ui-improvements-20260620` wholesale and do
   not cherry-pick halo docs onto lifo's branch unless the operator/lifo changes
   that decision.
3. For the reported crash, ask the operator to pick one path:
   - Option 1, recommended: controlled cleanup/restart of live Limux
     windows/hosts, then reopen `limux`.
   - Option 2: watch-only until recurrence; capture exact repro action if it
     happens again.
   - Option 3: rollback `~/.local/bin/limux` to the previous reviewed build.
4. Hold integration-branch mutation until crash evidence becomes concrete.
5. Continue GTK-critical investigation only after the restart/watch/rollback
   decision is settled.

## Current Baseline

| Item | Value |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Current checkout | `halo/limux-ui-improvements-20260620` |
| Current checkout HEAD | `355ce4b docs(handoff): record limux ui improvement branch` |
| Recommended integration lane | `origin/lifo/workspaces-sidebar-notifications-20260620` at `49fb4cf3a15262fd4d09532c0f8fdc38ab8fdc45` |
| Installed `limux` | `/home/riche/.local/limux-reviewed/workspaces-sidebar-notifications-20260620/bin/limux` |
| PR intake | No open PRs on `RichelynScott/limux` at intake. |
| Runtime health | Selected-workspace surfaces were healthy on both live sockets during crash triage. |
| Dirty peer files | `M LIFO_HANDOFF.md`, `?? archive/` |

## Active Coordination

- Lifo accepted option A on hcom thread `limux-ui-integration-20260620`:
  his branch at `49fb4cf` is the integration lane; halo docs stay separate.
- Lifo is holding mutation on the integration branch during crash triage.
- Crash thread: `limux-crash-20260620`.
- Lifo independently confirmed:
  - his integration worktree is clean and matches origin at `49fb4cf`;
  - installed Limux resolves to the expected reviewed build;
  - he sees no branch-specific panic, segfault, or unpushed-code evidence.

## Installed Integration Work

Installed from lifo branch:

- workspace sidebar hide/restore ribbon;
- compact persisted sidebar widths;
- manual workspace unread/read support;
- pane attention marker behavior;
- Codex PreToolUse `user-input-needed` hook path;
- hook delivery debug logging;
- scoped TaskMaster experience note.

Verification already run:

```bash
./scripts/check.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
cargo build --release -p limux-cli -p limux-host-linux
scripts/user-local-install/install-user-local.sh --dry-run --profile release --install-id workspaces-sidebar-notifications-20260620
scripts/user-local-install/install-user-local.sh --apply --profile release --install-id workspaces-sidebar-notifications-20260620
sha256sum -c SHA256SUMS
/home/riche/.local/bin/limux --help
/home/riche/.local/bin/limux --json hooks codex user-input-needed
/home/riche/.local/bin/limux --json surface-health
```

## Crash Triage Findings

Operator reported a crash after the integration build was installed. Evidence
so far does not prove a Limux code crash:

- Host log shows `Gdk-Message: Error reading events from display: Connection
  reset by peer` at `01:31:21`.
- No Rust panic, segfault, fatal GLib stack, or matching user-journal crash
  entry was found.
- `coredumpctl` is unavailable.
- Two installed-lane Limux hosts were alive during triage:
  - PID `23541` on `/run/user/1000/limux/limux-23541.sock`
  - PID `24840` on `/run/user/1000/limux/limux.sock`
- Both sockets list the same workspace set but have different selected
  workspaces.
- Explicit selected-workspace `surface-health` checks were healthy on both
  sockets.
- Some non-selected/tab surfaces reported unrealized. Treat that as weak
  evidence only; it is not a crash by itself.
- `/run/user/1000/limux/limux-25211.sock` was stale and failed to connect.

Current hypothesis: WSLg/display/compositor/session reset or duplicate live
host state, not a proven branch-specific crash.

## Runtime Findings To Carry Forward

- WSL EGL/Mesa/Zink/GDK compositor warnings remain.
- GTK criticals around these calls remain a separate investigation target:
  - `gtk_scrolled_window_get_child`
  - `gtk_viewport_get_child`
  - `gtk_stack_set_visible_child_name`
- Concurrent runtime launch isolation landed in `1df7621`, adding fallback
  socket behavior when the default control socket is in use.

## Do Not Resume

Do not resume the old Project Isolation Lab / VM goal from this Limux session.
That lane belongs to `/home/riche/Proj/SUPPLY_CHAIN_SECURITY`; this repo is now
focused on Limux product/runtime improvement.
