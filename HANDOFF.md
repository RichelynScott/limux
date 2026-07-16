# Limux Session Handoff

> **Current manager handoff:** Read [`LIFO_HANDOFF.md`](LIFO_HANDOFF.md) first.
> The Halo-owned material below is retained as historical June 2026 context;
> the current Limux manager state, runtime SHA, TaskMaster gate, and next action
> are maintained in the Lifo handoff.

Last updated: 2026-06-20 EDT
Owner/session: halo / Codex GPT-5

## Active Goal - Limux Improvement

The active goal for this repo is **improving Limux as the tool the operator is
using**. The previous Project Isolation Lab / VM goal is not the Limux repo's
active workstream anymore. That work is handled by the SCS team in
`/home/riche/Proj/SUPPLY_CHAIN_SECURITY`.

Do not restart the old VM/isolation planning loop from this handoff. Limux work
should be product/runtime improvement work: keeping the installed user-local
Limux usable, fixing observed UI/runtime defects, and adding scoped features
that help the operator run multiple terminal/agent sessions.

## Immediate Next Actions

1. Preserve peer dirt: do not edit or stage `LIFO_HANDOFF.md` or `archive/`
   unless lifo explicitly hands them over.
2. Keep `origin/lifo/workspaces-sidebar-notifications-20260620` at
   `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` as the recommended integration
   lane. This includes the workspace/sidebar work plus the G0 stability merge.
3. If the operator wants the G0 stability fixes in the live app, do a separate
   reviewed user-local install from
   `/home/riche/MCPs/limux-workspaces-sidebar-notifications`; the current
   `/home/riche/.local/bin/limux` symlink still points at the earlier
   `workspaces-sidebar-notifications-20260620` install.
4. If the old reported crash recurs, capture the exact click/action and rerun
   the crash evidence commands below before rollback or cleanup.
5. Keep the old Project Isolation Lab / VM goal out of this repo unless the
   operator explicitly redirects back to it.

## Current State

| Area | State |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Current checkout | `halo/limux-ui-improvements-20260620`, tracking `origin/halo/limux-ui-improvements-20260620` |
| Current checkout HEAD | `65cb302 docs(handoff): record limux crash triage` |
| Recommended integration lane | `origin/lifo/workspaces-sidebar-notifications-20260620` at `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` |
| Installed Limux | `/home/riche/.local/limux-reviewed/workspaces-sidebar-notifications-20260620/bin/limux` |
| PR closeout | PR #1 merged into the integration lane at `299a8fc762dc5f4a168d7d37c8148c58d0aedb08` after Codex rereview cleared `8798eaa839`. |
| Dirty peer files | `LIFO_HANDOFF.md` modified, `archive/` untracked |
| Runtime classification | Earlier crash was not proven to be a Limux code crash; evidence pointed first to display/compositor/session reset plus duplicate live hosts. |
| Compact/close state | Halo goal loop is complete; no active blocker in this session. |

## Crash Triage - 2026-06-20

Operator reported a Limux crash after the user-local integration build was
installed. Halo coordinated with lifo on hcom thread `limux-crash-20260620`.

Evidence:

- `~/.local/state/limux/logs/limux-host.log` shows:
  `Gdk-Message: Error reading events from display: Connection reset by peer` at
  `01:31:21`.
- There is no observed Rust panic, segfault, fatal GLib stack, or matching
  `journalctl --user` crash entry.
- `coredumpctl` is unavailable in this environment.
- Lifo confirmed his integration worktree
  `/home/riche/MCPs/limux-workspaces-sidebar-notifications` is clean at
  `49fb4cf3a15262fd4d09532c0f8fdc38ab8fdc45` and matches origin.
- Host-namespace process/socket checks showed two live installed-lane hosts:
  - PID `23541` on `/run/user/1000/limux/limux-23541.sock`
  - PID `24840` on `/run/user/1000/limux/limux.sock`
- Explicit selected-workspace `surface-health` checks were healthy on both
  sockets. Some non-selected/tab surfaces reported unrealized, which is not by
  itself crash evidence.
- `/run/user/1000/limux/limux-25211.sock` was stale and failed to connect.

Current hypothesis:

The best-supported explanation is a WSLg/display/compositor/session reset or a
window/session restart that left duplicate Limux hosts/sockets. This is
watch-worthy but not enough evidence to rollback or patch the integration
branch.

## Completed This Session

| Time | Item | Evidence |
|---|---|---|
| 2026-06-20 | Re-anchored repo goal from VM/isolation work back to Limux improvement. | Operator directive in chat; this handoff. |
| 2026-06-20 | Coordinated with lifo on integration ownership. | hcom `#115022`, `#115918`: option A chosen, lifo branch is integration lane, halo docs separate. |
| 2026-06-20 | Verified lifo integration branch. | `./scripts/check.sh` passed; `LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh` passed after one transient first-run stage-2 failure and successful replay/rerun. |
| 2026-06-20 | Installed integration build user-local. | `scripts/user-local-install/install-user-local.sh --apply --profile release --install-id workspaces-sidebar-notifications-20260620`; install hash check passed. |
| 2026-06-20 | Coordinated with lifo on reported crash. | hcom `limux-crash-20260620`; lifo found matching evidence and no branch-specific panic/segfault. |
| 2026-06-20 | Classified crash evidence. | Host log, socket, process, selected-workspace health, and journal checks. |
| 2026-06-20 | Reviewed lifo's clipboard paste fix. | `origin/lifo/fix-clipboard-paste-20260620` at `b05af68`; no blocking findings; host tests/check passed in exported review tree. Not installed into live Limux by halo. |
| 2026-06-20 | Reviewed and closed G0 stability PR bot loop. | PR #1: Codex bot P2 fixed at `8798eaa839`, bot rereview said no major issues, PR merged at `299a8fc762dc5f4a168d7d37c8148c58d0aedb08`. |

## Key Files For Context

| Path | Purpose |
|---|---|
| `/home/riche/MCPs/limux/LIFO_HANDOFF.md` | Lifo-owned handoff; currently dirty. Do not edit unless lifo hands it over. |
| `/home/riche/MCPs/limux/HALO_HANDOFF.md` | Halo-owned successor state for Limux improvement work. |
| `/home/riche/MCPs/limux/FYI.md` | Append-only decision journal; currently large and should be condensed later under a separate approved cleanup. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/window.rs` | Workspace/sidebar UI, notification activation, pane/tab focus, and several likely GTK-critical code paths. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/pane.rs` | Pane registry, pane CSS, tab UI, and pane attention outline/hover-clear behavior. |
| `/home/riche/MCPs/limux/rust/limux-cli/src/main.rs` | Installed CLI entrypoint, hook commands, agent-team, and host-launch behavior. |
| `/home/riche/.local/state/limux/logs/limux-host.log` | Current automatic host stderr log. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications` | Lifo sibling worktree for the recommended integration lane. |
| `/home/riche/MCPs/limux/docs/project-isolation-lab-goal.md` | Historical Limux-local VM/isolation alignment note; superseded for active Limux work. |

## PR #1 / G0 Stability Closeout - 2026-06-20

PR #1 (`https://github.com/RichelynScott/limux/pull/1`) was a stacked PR from
`lifo/g0-stability-20260620` into
`lifo/workspaces-sidebar-notifications-20260620`.

Closeout facts:

- Head before merge: `8798eaa83963ecbe411cda7cc7d3c6345bd0f90d`
  (`test(host): skip gtk traversal test without display`).
- Merge commit: `299a8fc762dc5f4a168d7d37c8148c58d0aedb08`
  (`fix(host): harden G0 runtime stability`).
- GitHub state: `MERGED` at `2026-06-20T07:08:13Z`.
- Codex rereview on current head reported:
  `Codex Review: Didn't find any major issues.`
- Local sibling worktree
  `/home/riche/MCPs/limux-workspaces-sidebar-notifications` is clean on
  `lifo/workspaces-sidebar-notifications-20260620` and matches origin at
  `299a8fc`.

What the merge added:

- Runtime/debug socket environment isolation.
- Hook `resolved_socket` diagnostics.
- Ghostty HiDPI physical sizing.
- Wrapped pane traversal for focus/attention paths.
- Split SVG validation before package/user-local install.
- Display-independent GTK traversal test behavior for plain `cargo test`.

Halo verification before merge closeout:

```bash
gh pr view 1 --repo RichelynScott/limux --json headRefOid,reviews,comments,mergeStateStatus,state
gh api repos/RichelynScott/limux/pulls/1/comments
env -u DISPLAY -u WAYLAND_DISPLAY CARGO_TARGET_DIR=/tmp/limux-g0-no-display-target cargo test -p limux-host-linux find_leaf_pane_descends_wrapped_workspace_root_to_pane -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY GDK_BACKEND=invalid CARGO_TARGET_DIR=/tmp/limux-g0-no-display-target cargo test -p limux-host-linux find_leaf_pane_descends_wrapped_workspace_root_to_pane -- --nocapture
cargo fmt --check
git diff --check
```

## Crash Evidence Commands

```bash
git status --short --branch
readlink -f /home/riche/.local/bin/limux
tail -n 240 /home/riche/.local/state/limux/logs/limux-host.log
/home/riche/.local/bin/limux --json surface-health
/home/riche/.local/bin/limux --socket /run/user/1000/limux/limux.sock --json list-workspaces
/home/riche/.local/bin/limux --socket /run/user/1000/limux/limux.sock --json identify
/home/riche/.local/bin/limux --socket /run/user/1000/limux/limux-23541.sock --json list-workspaces
/home/riche/.local/bin/limux --socket /run/user/1000/limux/limux-23541.sock --json identify
ps -eo pid,ppid,stat,lstart,comm,args | rg -i 'limux|ghostty'
ss -xlpn | rg 'limux|ghostty|/run/user/1000/limux'
journalctl --user --since '2026-06-20 01:20:00' --no-pager | rg -i 'limux|ghostty|gtk|gdk|segfault|crash|killed|connection reset'
```

Run the `ps`, `ss`, and `journalctl` commands outside the Codex PID sandbox or
with approved escalation when exact host-process evidence is needed.

## Critical Behavior Rules

- Focus on Limux product/runtime improvement unless the operator explicitly
  redirects back to VM/isolation work.
- Preserve existing repo patterns and run verification before claiming fixes.
- Do not touch `LIFO_HANDOFF.md` or `archive/` while lifo owns that local dirt.
- Do not mutate the integration branch during crash triage without fresh
  evidence or operator approval.
- Treat package installs, global runtime changes, host OS mutation, sudo, and
  generated installers as gated work.
- Keep vendored `ghostty/` read-only from the Limux layer.
- For user-visible CLI/control behavior, verify the production GTK bridge path
  when feasible, not only the standalone core dispatcher.

## Out Of Scope / Historical

The old Project Isolation Lab material formerly in this root handoff was
removed from the active resume path on 2026-06-20 by operator direction. SCS
still owns that lane. Limux may later be used as an acceptance case if the
operator explicitly asks, but that is not the current Limux repo goal.
