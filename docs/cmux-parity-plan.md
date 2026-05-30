# cmux-parity plan (revised after architectural discovery)

## Architecture discovery

Limux has **two control servers**:

1. **Standalone `limux-control-server` binary** — uses `limux_core::Dispatcher`
   + `ControlState` and supports the **full** command vocabulary. Used for
   tests and for CLI calls when the GUI isn't running.

2. **Embedded bridge inside `limux-host-linux`** — `control_bridge.rs` only
   routes a narrow subset of methods to the GTK main loop. Supports
   `system.ping`, `system.identify`, `workspace.{current,list,create,
   select,rename,close}`, `pane.list`, `pane.surfaces`, `surface.list`,
   `pane.create` for terminal self-spawn, `surface.send_text`,
   `surface.send_key`, `surface.read_text`, `surface.health`, and
   `notification.create`. It still does **NOT** support browser commands.

When the GUI is running, the CLI targets the bridge via the runtime
socket. `list-panes` / `list-panels`, terminal `new-pane --command ...`,
text injection, key-level injection, `surface-health`, and terminal
`read-screen` now work against the running host.

## Delivery strategy (revised)

### Phase 1 — Env auto-wiring ✅ (shipped in 1295d12)

### Phase 2 — Make the bridge a full proxy (🚧 PARTIAL)

Bridge should route unknown methods to a local `Dispatcher` instance
seeded with live GTK state, OR to dedicated per-method `ControlCommand`
variants that interrogate the live state. The cleanest path:

- Maintain a `Arc<Mutex<ControlState>>` owned by the GTK app, kept in
  sync with live workspace/pane/surface state.
- Bridge falls through unknown methods to `Dispatcher::dispatch` on that
  shared state.
- Specific methods that need GTK side-effects (send_text, create_surface,
  notification.create) remain as `ControlCommand` variants.

The terminal introspection path is now bridged directly against live GTK state.
Remaining proxy work is for deferred browser surface commands and broader
dispatcher parity.

**Shipped so far (in 6b8eb1a and follow-up bridge work):**

- `surface.send_text` and `notification.create` now pass `allow_name=true`
  to `parse_optional_workspace_target` for workspace-name targets. The current
  generated team protocol uses surface IDs because `agent-team` splits peers
  inside one workspace.
- `pane.list`, `pane.surfaces`, and `surface.list` now route on the live
  GTK bridge, so agents can discover peer panes/surfaces in a running
  Limux window.
- `surface.send_key` now routes to the exact terminal surface when provided,
  so agents can send deterministic key-level control such as Ctrl-C.
- `surface.health` and `surface.read_text` now route on the live GTK bridge,
  so agents can inspect peer terminal health and visible screen text.
- `pane.create` now routes through the GTK bridge for terminal panes. From
  inside an agent terminal, `limux new-pane --direction right --command 'claude'`
  uses `LIMUX_WORKSPACE_ID`, `LIMUX_SURFACE_ID`, and `LIMUX_PANE_ID` to split
  the caller's pane, create a new terminal, and launch the command there.

**Still open (priority order):**

- Browser command bridge parity.

### Phase 3 — `limux notify` + GUI toast/sidebar integration ✅
`ControlCommand::CreateNotification` wired through the bridge into
`mark_workspace_unread_with_message` + libadwaita toast.
CLI: `limux notify [--workspace <id|name>] [--subtitle <…>] [--body <…>] <title>`.

### Phase 4 — `limux claude-hook` / `opencode-hook` / `gemini-hook` ✅
Reads hook JSON from stdin, translates the agent-specific event vocabulary
into a `notify` (and, where useful, an inline `send`). Drop-in for
`~/.claude/settings.json` hooks blocks.

### Phase 5 — `limux agent-team` + generated protocol file ✅
`limux agent-team [--agents codex,claude[,opencode,gemini]] [--cwd <path>]
[--protocol-path <path>] [--force-protocol-overwrite] [--no-launch]
[--dry-run]`:

- Splits the active workspace into one terminal pane per agent and launches
  each agent CLI unless `--no-launch` is set.
- Bridge passes `allow_name=true` to `parse_optional_workspace_target` for
  `surface.send_text` and `notification.create`; the generated team protocol
  still addresses peers by surface ID because agents share one workspace.
- Writes `LIMUX_AGENTS.md` in the shared cwd by default, or the explicit
  `--protocol-path`, documenting:
    - the generated-file marker,
    - detected instruction sources (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`)
      with path, modified time, and deterministic content hash metadata,
    - the peers table (agent → pane → surface → launch cmd),
    - the `<agent-msg from="…" to="…" id="…" reply-to="…" ts="…">` envelope,
    - the exact `limux send` invocation for sending and replying,
    - the `limux notify` escalation path for human input,
    - the `LIMUX_*` env contract every spawned terminal inherits,
    - the optional `LIMUX_AGENTS.local.md` durable policy sidecar,
    - editable Policies section (timeouts, size limits, destructive-action gating).
  Existing repo `AGENTS.md` files are not written by default. Existing unmarked
  protocol files are not overwritten unless `--force-protocol-overwrite` is
  explicitly passed. Symlink protocol paths are refused.

**Shipped in `cec067f`:**

- Default protocol output changed from `AGENTS.md` to `LIMUX_AGENTS.md`.
- `--protocol-path <path>` allows an explicit protocol file location.
- Regression coverage preserves existing repo `AGENTS.md` files and verifies
  the default sidecar path.

**Shipped after `cec067f`: Phase 5A — zero-friction protocol discovery**

- Added a generated-file marker to `LIMUX_AGENTS.md`.
- Added an `Instruction Sources` section that detects repo instruction files
  such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` and points agents to read
  them directly.
- Does not copy, merge, or reinterpret repo instruction files by default. Repo
  instruction files remain authoritative; the Limux sidecar only adds runtime
  topology and messaging protocol.
- Added no-overwrite semantics for existing unmarked `LIMUX_AGENTS.md`, with
  `--force-protocol-overwrite` if replacement is required.
- Documented a durable local extension point,
  `LIMUX_AGENTS.local.md` for team-specific policy that survives regeneration.

**Shipped after Phase 5A: GTK `surface.send_text` failure reporting**

- The live GTK bridge now checks `TerminalHandle::send_text` and returns a
  conflict error when a resolved terminal surface is not ready for text input
  instead of reporting `ok: true`.
- The existing `pane.create --command` path already retries until the new
  pane's terminal surface is writable before returning success.

**Deferred: Phase 5B — automatic bootstrap**

Caller-shell quoting for generated `new-pane --command` snippets now has
regression coverage, and the live GTK/Xvfb smoke path is runnable with the
documented host prerequisites. Full two-phase launch/bootstrap is still
deferred until arbitrary prompt text is sent after pane readiness instead of
embedded inside `--command`, and the live smoke path proves metacharacter
payloads cannot trigger shell side effects.

### Phase 6 — (deferred) `limux progress`, `limux log`, `limux markdown`
Nice polish, not blockers.

## Why phase 2 first

Without a real bridge, every subsequent feature ends up routing around
the same hole: the GUI owns the ground truth about surfaces/panes but
the CLI can't query it. Fixing this once, properly, makes phases 3–5
small.
