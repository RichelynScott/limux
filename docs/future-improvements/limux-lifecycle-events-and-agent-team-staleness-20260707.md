# Limux Lifecycle Events And Agent-Team Staleness

Date: 2026-07-07
Status: Follow-up intake from disconnect audit
TaskMaster tag: `cmux-parity-20260707`
Tracked tasks: `5.1`, `7.1`, `7.2`

## Problem

During the 2026-07-07 Limux/hcom disconnect audit, panes that had been manually
closed by the operator still appeared in generated agent-team context. The
runtime had no neat queryable record equivalent to:

```text
pane 119 manually closed by human at <timestamp>
pane 120 manually closed by human at <timestamp>
```

That left two separate gaps:

- Agents could not ask Limux what happened to a pane or workspace.
- `LIMUX_AGENTS.md` peer tables could point at closed surfaces. A
  protocol-following peer could then send an `agent-msg` envelope to a
  nonexistent pane, or worse, to a pane that had returned to a plain shell.

Follow-up investigation also found an immediate CLI regression: `limux
agent-team --help` was not side-effect-free. The command flowed into the normal
team creation path, which could resolve the currently focused workspace and
create unintended panes there. On 2026-07-07 this matched the operator report
that new panes appeared in the Zen-Master/PAL MCP workspace while a separate
team-regeneration flow was being investigated.

The desired behavior is informational and queryable. It must not create toast or
notification spam.

## Task 5.1: Queryable Lifecycle Events

Add a bounded lifecycle event log for pane/workspace state transitions.

Candidate event classes:

- workspace created, selected, restored, renamed, closed
- pane created, split, focused, resized, zoomed, closed
- tab/surface created, focused, renamed, closed
- best-effort close source, such as user action, command/API action, restore
  cleanup, process exit, or unknown

Candidate event fields:

- timestamp
- event type
- workspace id/ref
- pane id/ref when known
- surface/tab id/ref when known
- source/reason best effort
- human-readable summary

Expose the event log through an inquiry surface such as:

- `limux events [--workspace <id|ref>] [--pane <id|ref>] [--limit <n>] [--json]`
- a `doctor --events` or `doctor --log-triage` extension
- a control method such as `system.events` or `runtime.events`

Acceptance:

- Manual pane/workspace closes are visible through the inquiry surface.
- Event retention is bounded.
- Querying events does not mark workspaces read, clear attention, change focus,
  or show toasts.
- The event surface is safe for diagnostics and automation.

## Task 7.1: Agent-Team Peer Table Self-Heal

`agent-team` generated context should not keep teaching agents to route to dead
or repurposed panes.

Required behavior:

- Before generating, bootstrapping, or sending peer routing guidance, verify
  each listed workspace/pane/surface still exists.
- Mark missing peers as stale or regenerate the peers table.
- Fail closed before sending an envelope to a missing surface or to a pane that
  is no longer known to host the expected agent session.
- Prefer a clear diagnostic over attempting best-effort injection.
- When `--launch-mode hcom` is used, include hcom identity as correlation
  metadata but keep hcom as the messaging/session bus and Limux as the GUI
  control bus.

Acceptance:

- Closing a peer pane updates or invalidates the generated peer table before
  the next bootstrap/routing path.
- A stale destination does not receive an `agent-msg` envelope as shell input.
- `agent-team --dry-run` can show stale-peer detection without host mutation
  when enough inputs are supplied.
- Lifecycle events from task `5.1` can explain why a peer disappeared when that
  information is available.

## Task 7.2: Agent-Team Help Side-Effect Fix

Status: done

`limux agent-team --help` must be an informational command only.

Shipped behavior:

- `run_agent_team` returns help before resolving cwd sidecars, validating output
  files, contacting the Unix socket, or creating panes.
- Plain-text command output renders the help text instead of the normal
  `OK agent-team ...` launch summary.
- Regression coverage verifies that `agent-team --help --cwd <tmp>` succeeds
  with a missing socket and does not create `LIMUX_AGENTS.md`,
  `LIMUX_TEAM_ROSTER.md`, or `LIMUX_REVIEW_LEDGER.md`.

Verification:

```bash
cargo test -p limux-cli agent_team_help_is_side_effect_free -- --nocapture
./target/debug/limux-cli --socket /tmp/limux-agent-team-help-no-such.sock agent-team --help
```

Remaining product concern: live `agent-team` still falls back to the focused
workspace when invoked outside a Limux pane without `LIMUX_WORKSPACE_ID`. That is
convenient, but it is also the risky default that made this bug visible in the
wrong workspace. Decide separately whether non-interactive or peer/orchestrator
calls should require an explicit workspace target instead of using focus.

## Non-Goals

- No toast or sidebar notification spam for lifecycle events.
- No hcom runtime changes in this Limux task; hcom identity/delivery follow-up
  stays in the hcom lane.
- No unbounded transcript or terminal scrollback capture.
- No automatic relaunch of closed peers without explicit operator or
  orchestrator action.
