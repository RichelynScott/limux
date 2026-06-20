# Limux Session Handoff

Last updated: 2026-06-20 EDT
Owner/session: halo / Codex GPT-5

## Active Goal - Limux Improvement

The active goal for this repo is now **improving Limux as the tool the operator
is using**. The previous Project Isolation Lab / VM goal is not the Limux repo's
active workstream anymore. That work is handled by the SCS team in
`/home/riche/Proj/SUPPLY_CHAIN_SECURITY`.

Do not restart the old VM/isolation planning loop from this handoff. Limux work
should be product/runtime improvement work: keeping the installed user-local
Limux usable, fixing observed UI/runtime defects, and adding scoped features
that help the operator run multiple terminal/agent sessions.

## Immediate Next Actions

1. Preserve peer dirt: do not edit or stage `LIFO_HANDOFF.md` or `archive/`
   unless lifo explicitly hands them over.
2. Coordinate with lifo on the current UI requests. Lifo confirmed his active
   lane is read-only investigation in
   `/home/riche/MCPs/limux-workspaces-sidebar-notifications` on branch
   `lifo/workspaces-sidebar-notifications-20260620`; he does not own root
   `HANDOFF.md`.
3. Consume lifo's code-path report for:
   - right-clicking a workspace to mark it unread manually;
   - a blue outline around the pane/session needing attention, lingering for
     about three seconds after hover.
4. Before code edits, keep the current baseline visible:
   - `git status --short --branch`
   - `cargo check -p limux-host-linux`
   - `limux --json surface-health`
   - `tail -n 200 ~/.local/state/limux/logs/limux-host.log`
5. After implementing UI/runtime changes, run the narrow relevant checks first,
   then `./scripts/xvfb-smoke-test.sh` or `./scripts/check.sh` when the blast
   radius warrants it.

## Current State

| Area | State |
|---|---|
| Repo | `/home/riche/MCPs/limux` |
| Branch | `lifo/reboot-handoff-20260619`, tracking `origin/lifo/reboot-handoff-20260619` |
| Current HEAD | `1df7621 fix(host): isolate concurrent runtime launches` |
| Main baseline | `origin/main` at `596bc69 fix(host): add startup logging and schema env repair` |
| Installed Limux | `/home/riche/.local/limux-reviewed/multi-runtime-isolation-20260620/bin/limux` |
| Open PRs | `gh pr list --repo RichelynScott/limux --state open ...` returned `[]` on 2026-06-20 |
| Dirty peer files | `LIFO_HANDOFF.md` modified, `archive/` untracked |
| Live runtime | `limux --json surface-health` returned two healthy terminal surfaces |

Latest host log observations from
`~/.local/state/limux/logs/limux-host.log`:

- The older GSettings schema-source critical and split-icon load warnings were
  not present in the latest tail.
- WSL graphics warnings remain: EGL/Mesa/Zink/GDK compositor warnings. Treat
  those as environment warnings unless a user-visible failure ties to them.
- GTK criticals remain around `gtk_scrolled_window_get_child`,
  `gtk_viewport_get_child`, and `gtk_stack_set_visible_child_name`. Those are
  still a real Limux investigation target.
- Concurrent runtime launch isolation is now present at `1df7621`; newer hosts
  can fall back to a per-process socket when the default socket is already in
  use.

## Completed This Session

| Time | Item | Evidence |
|---|---|---|
| 2026-06-20 | Re-anchored repo goal from VM/isolation work back to Limux improvement. | Operator directive in chat; this handoff rewrite. |
| 2026-06-20 | Coordinated with lifo. | hcom `#113038`: lifo does not own root `HANDOFF.md`; his lane is read-only in the sibling worktree. |
| 2026-06-20 | Checked hcom collision alerts for `HANDOFF.md` / `FYI.md`. | Alerts referenced halo's stale 2026-06-11 `apply_patch` events; `muvi` is not active. |
| 2026-06-20 | Verified current repo/runtime intake before implementation. | `git status --short --branch`; `git log --oneline`; `gh pr list`; `readlink -f ~/.local/bin/limux`; `tail -n 200 ~/.local/state/limux/logs/limux-host.log`; `limux --json surface-health`; `cargo check -p limux-host-linux`. |

## Key Files For Context

| Path | Purpose |
|---|---|
| `/home/riche/MCPs/limux/LIFO_HANDOFF.md` | Lifo-owned reboot/runtime handoff; currently dirty from a human-added log note. Do not edit unless lifo hands it over. |
| `/home/riche/MCPs/limux/HALO_HANDOFF.md` | Halo-owned successor state for Limux improvement work. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/window.rs` | Workspace/sidebar UI, notification activation, pane/tab focus, and several likely GTK-critical code paths. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/main.rs` | Host startup, logging, socket handling, and concurrent runtime launch isolation. |
| `/home/riche/MCPs/limux/rust/limux-cli/src/main.rs` | Installed CLI entrypoint and host-launch behavior. |
| `/home/riche/.local/state/limux/logs/limux-host.log` | Current automatic host stderr log. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications` | Sibling worktree where lifo is investigating workspace unread and pane-attention outline requests read-only. |
| `/home/riche/MCPs/limux/docs/project-isolation-lab-goal.md` | Historical Limux-local VM/isolation alignment note; superseded for active Limux work. |

## Critical Behavior Rules

- Focus on Limux product/runtime improvement unless the operator explicitly
  redirects back to VM/isolation work.
- Preserve existing repo patterns and run verification before claiming fixes.
- Do not touch `LIFO_HANDOFF.md` or `archive/` while lifo owns that local dirt.
- Treat package installs, global runtime changes, host OS mutation, sudo, and
  generated installers as gated work.
- Keep vendored `ghostty/` read-only from the Limux layer.
- For user-visible CLI/control behavior, verify the production GTK bridge path
  when feasible, not only the standalone core dispatcher.

## Verification Commands Run

```bash
git status --short --branch
git branch --show-current
git remote -v
git log --oneline --decorate -8
git worktree list
gh pr list --repo RichelynScott/limux --state open --json number,title,headRefName,author,url,updatedAt --limit 50
readlink -f /home/riche/.local/bin/limux
tail -n 200 /home/riche/.local/state/limux/logs/limux-host.log
limux --json surface-health
cargo check -p limux-host-linux
hcom list -v --name halo
hcom transcript lifo --last 1 --full --name halo
hcom events --sql 'id=113041 or id=113043' --name halo
hcom events --agent muvi --last 20 --name halo
```

## Out Of Scope / Historical

The old Project Isolation Lab material formerly in this root handoff was
removed from the active resume path on 2026-06-20 by operator direction. SCS
still owns that lane. Limux may later be used as an acceptance case if the
operator explicitly asks, but that is not the current Limux repo goal.
