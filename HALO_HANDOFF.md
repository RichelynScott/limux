# Halo Limux Handoff

Author/runtime/date: halo / Codex GPT-5 / 2026-06-20 EDT

## Immediate Next Action

1. Keep `LIFO_HANDOFF.md` and `archive/` untouched unless lifo transfers
   ownership.
2. Keep `origin/lifo/workspaces-sidebar-notifications-20260620` at
   `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` as the recommended integration
   lane. This is the PR #1 merge commit after the Codex bot rereview cleared
   `8798eaa839`.
3. If the operator wants the G0 stability merge in the live app, perform a
   separate reviewed user-local install from
   `/home/riche/MCPs/limux-workspaces-sidebar-notifications`. The active
   `/home/riche/.local/bin/limux` symlink still resolves to the earlier
   `workspaces-sidebar-notifications-20260620` install.
4. If the old reported crash recurs, capture the exact repro action and use the
   crash evidence commands in `HANDOFF.md`; do not rollback or cleanup from
   weak evidence.
5. This halo session can be compacted/closed after this handoff refresh.

## Current Baseline

| Item | Value |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Current checkout | `halo/limux-ui-improvements-20260620` |
| Current checkout HEAD | `65cb302 docs(handoff): record limux crash triage` |
| Recommended integration lane | `origin/lifo/workspaces-sidebar-notifications-20260620` at `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` |
| Installed `limux` | `/home/riche/.local/limux-reviewed/workspaces-sidebar-notifications-20260620/bin/limux` |
| PR closeout | PR #1 merged at `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` after Codex clean rereview on `8798eaa839`. |
| Runtime health | Selected-workspace surfaces were healthy on both live sockets during crash triage. |
| Dirty peer files | `M LIFO_HANDOFF.md`, `?? archive/` |

## Active Coordination

- Lifo accepted option A on hcom thread `limux-ui-integration-20260620`:
  his branch at `49fb4cf` is the integration lane; halo docs stay separate.
- Lifo is holding mutation on the integration branch during crash triage.
- Crash thread: `limux-crash-20260620`.
- G0 stability thread: `limux-g0-stability-20260620`; closed out with lifo.
- Lifo independently confirmed:
  - his integration worktree is clean and matches origin at `299a8fc`;
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

## PR #1 / G0 Stability Closeout

PR #1 (`https://github.com/RichelynScott/limux/pull/1`) is merged.

| Item | Value |
|---|---|
| Base | `lifo/workspaces-sidebar-notifications-20260620` |
| Head | `lifo/g0-stability-20260620` |
| Head commit before merge | `8798eaa83963ecbe411cda7cc7d3c6345bd0f90d` |
| Merge commit | `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` |
| Merged at | `2026-06-20T07:08:13Z` |
| Codex bot result | `Didn't find any major issues` on `8798eaa839` |

G0 merge contents:

- Runtime/debug socket environment isolation.
- Hook `resolved_socket` diagnostics.
- Ghostty HiDPI physical sizing.
- Wrapped pane traversal for focus/attention.
- Split SVG validation before package/local install.
- Display-independent GTK traversal test behavior for plain `cargo test`.

Halo independently verified the Codex P2 fix before merge closeout:

```bash
env -u DISPLAY -u WAYLAND_DISPLAY CARGO_TARGET_DIR=/tmp/limux-g0-no-display-target cargo test -p limux-host-linux find_leaf_pane_descends_wrapped_workspace_root_to_pane -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY GDK_BACKEND=invalid CARGO_TARGET_DIR=/tmp/limux-g0-no-display-target cargo test -p limux-host-linux find_leaf_pane_descends_wrapped_workspace_root_to_pane -- --nocapture
cargo fmt --check
git diff --check
```

The forced-invalid backend path printed the skip message and passed, proving
the Codex bot's display-dependent test concern was addressed.

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
- G0 stability hardening is merged at `299a8fc` but has not been installed into
  the active user-local Limux by halo.

## Do Not Resume

Do not resume the old Project Isolation Lab / VM goal from this Limux session.
That lane belongs to `/home/riche/Proj/SUPPLY_CHAIN_SECURITY`; this repo is now
focused on Limux product/runtime improvement.
