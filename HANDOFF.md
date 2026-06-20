# Limux Session Handoff

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
   `49fb4cf3a15262fd4d09532c0f8fdc38ab8fdc45` as the recommended integration
   lane. Lifo and halo agreed not to merge halo's branch wholesale and not to
   cherry-pick halo docs into lifo's branch right now.
3. Resolve the reported Limux crash with the operator by choosing one numbered
   path:
   - Option 1, recommended: operator-approved controlled cleanup/restart of the
     currently live Limux hosts/windows, then reopen `limux`.
   - Option 2: watch-only; if it recurs, capture the exact click/action and
     rerun the crash evidence commands below.
   - Option 3: rollback `~/.local/bin/limux` to the previous reviewed build.
4. Do not mutate the integration branch while crash evidence is inconclusive.
   Lifo is holding mutation on the integration lane.
5. Continue the separate GTK-critical investigation only after the crash/restart
   choice is settled. The recurring log lines are around
   `gtk_scrolled_window_get_child`, `gtk_viewport_get_child`, and
   `gtk_stack_set_visible_child_name`.

## Current State

| Area | State |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Current checkout | `halo/limux-ui-improvements-20260620`, tracking `origin/halo/limux-ui-improvements-20260620` |
| Current checkout HEAD | `355ce4b docs(handoff): record limux ui improvement branch` |
| Recommended integration lane | `origin/lifo/workspaces-sidebar-notifications-20260620` at `49fb4cf3a15262fd4d09532c0f8fdc38ab8fdc45` |
| Installed Limux | `/home/riche/.local/limux-reviewed/workspaces-sidebar-notifications-20260620/bin/limux` |
| Open PRs | `gh pr list --repo RichelynScott/limux --state open ...` returned `[]` on 2026-06-20 |
| Dirty peer files | `LIFO_HANDOFF.md` modified, `archive/` untracked |
| Runtime classification | Reported crash is not yet proven to be a Limux code crash; evidence points first to display/compositor/session reset plus duplicate live hosts. |

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
