---
name: limux-use-guide
description: Use when driving Limux workspaces or panes, recovering a named hcom agent in its existing pane, diagnosing Limux runtime issues, using the Limux CLI/socket/doctor/hooks/agent-team surfaces, or deciding Limux-vs-hcom routing.
---

# Limux Use Guide

Use Limux as the local terminal workspace and GUI control layer. Use hcom as the
cross-agent/session messaging and coordination layer.

## Two Binary Model

Installed packages expose `limux` as the user-facing CLI. Running `limux` with
no arguments launches the GTK app through the private host binary.

Local debug builds produce two binaries:

```bash
cargo build -p limux-cli --bin limux-cli
./target/debug/limux-cli --help
./target/debug/limux-cli --version
```

- `target/debug/limux-cli` is the CLI for subcommands such as `doctor`,
  `target-info`, `notify`, `read-screen`, `send`, `agent-team`, and hooks.
- `target/debug/limux` is the GTK host binary from `limux-host-linux`; it only
  accepts GTK/GApplication flags.

Use the top-level CLI help as the command-surface source of truth. Some
subcommands do not implement `--help`; do not probe an unknown subcommand help
path against a live runtime unless the command is known to be help-only.

## First Diagnostics

```bash
limux --version
limux target-info
limux doctor --json
limux doctor --log-triage --lines 200
```

`--version` reports CLI version and build identity. Installed builds may also
include fields from `install-info.json`, including install id and channel.

`target-info` / `socket-info` resolves the socket and channel without connecting
to a host. Use it to confirm whether a shell targets the default, stable, or
preview runtime.

JSON flag placement is a parser gotcha. Most commands use the global flag
before the subcommand, such as `limux --json identify`; `doctor --json` is a
subcommand-local exception.

`doctor` checks launchers, running processes, socket reachability, stale
sockets, Ghostty resources, and optional log triage.

Exit codes:

- `0`: all checks passed.
- `1`: at least one check failed.
- `2`: warnings were found but no check failed.

## Socket And Environment Contract

Every terminal spawned by Limux should inherit:

```bash
printf 'workspace=%s\npane=%s\nsurface=%s\ntab=%s\nsocket=%s\n' \
  "$LIMUX_WORKSPACE_ID" "$LIMUX_PANE_ID" "$LIMUX_SURFACE_ID" \
  "$LIMUX_TAB_ID" "$LIMUX_SOCKET"
```

Most CLI commands accept explicit flags first, then fall back to `LIMUX_*`
environment values:

```bash
limux --json identify
limux --json list-workspaces
limux --json list-panels --workspace "$LIMUX_WORKSPACE_ID"
limux --json surface-health --workspace "$LIMUX_WORKSPACE_ID"
```

`surface-health` is workspace-scoped today; do not pass `--surface` unless the
CLI grows real surface-target support for that command.

Useful global flags:

- `--socket <path>` targets an explicit control socket.
- `--channel stable|preview[:id]` targets a user-local runtime lane.
- `--json` emits machine-readable output where supported.
- `--id-format refs|both|uuids` controls handle shape for automation.

## Day-To-Day Pane Commands

Observe current context:

```bash
limux --json identify
limux list-panels --workspace "$LIMUX_WORKSPACE_ID"
limux read-screen --surface "$LIMUX_SURFACE_ID" --lines 80
limux capture-pane --surface "$LIMUX_SURFACE_ID" --scrollback --lines 200
```

Create panes and workspaces:

```bash
limux --json new-pane --direction right --command 'codex "Task prompt here."'
limux --json new-pane --direction down --command 'claude "Task prompt here."'
limux --json new-workspace --cwd "$PWD" --command 'codex "Task prompt here."'
```

Send text and keys:

```bash
limux send --surface "<surface-id>" "short message"
limux send-key --surface "<surface-id>" enter
```

`send` is for printable text plus tab, LF, and CR. Use `send-key` for control
keys. For long prompts, write a file and send the file path instead of injecting
a very long shell line.

Mark attention:

```bash
limux notify --workspace "$LIMUX_WORKSPACE_ID" \
  --subtitle "input needed" \
  --body "A pane is blocked and needs a decision" \
  "Limux task needs attention"

limux pane-action --action set_flag_color --color orange
limux pane-action --action clear_flag_color
```

Pane flag colors are `orange`, `red`, `purple`, `pink`, `green`, `yellow`,
`teal`, and `cyan`. Pane-originated attention should show a blue pane border
without hiding any manual flag color. `limux notify` creates user-visible
workspace/sidebar attention; it is not by itself proof that the pane border
overlay path works.

## Limux And hcom Interplay

Limux and hcom are complementary, not interchangeable:

- Limux socket: local GUI control bus. Use it for panes, workspaces, terminal
  text, screen reads, pane flags, toasts, sidebar badges, and host diagnostics.
- hcom: cross-agent/session messaging bus. Use it for named agents, durable
  messages, transcripts, resume/fork, manager routing, and multi-project
  coordination.

Choose based on intent:

- Same Limux pane or same workspace terminal operation: use `limux send`,
  `limux send-key`, `limux read-screen`, or `limux notify`.
- Durable agent-to-agent message, named recipient, or cross-project routing:
  use `hcom send`.
- Human-visible GUI attention: use `limux notify`.
- Persistent session registration and transcript-aware coordination: use hcom.

hcom workers launched inside Limux inherit the `LIMUX_*` environment values.
Include `LIMUX_SURFACE_ID` in task prompts or metadata when a worker may need to
call back into its parent pane.

## Agent Teams

Dry-run first when checking generated files or command shape:

```bash
tmp="$(mktemp -d -t limux-agent-team.XXXXXX)"
limux agent-team --dry-run --no-launch \
  --agents codex,claude \
  --cwd "$tmp" \
  --protocol-path "$tmp/LIMUX_AGENTS.md" \
  --roster-path "$tmp/LIMUX_TEAM_ROSTER.md" \
  --ledger-path "$tmp/LIMUX_REVIEW_LEDGER.md"
```

Launch agents directly inside Limux panes:

```bash
limux agent-team --agents codex,claude --cwd "$PWD"
```

Launch hcom-managed agents inside Limux panes:

```bash
limux agent-team --agents codex,claude,hermes --launch-mode hcom --cwd "$PWD"
```

`--launch-mode hcom` creates normal Limux panes, but pane commands use
`hcom <agent> --run-here`; Hermes uses `hcom hermes --run-here`.

## Hooks And Sidebar Lifecycle

Install hooks from the launcher you want hooks to call later:

```bash
limux hooks setup
limux hooks setup codex
limux hooks setup claude
limux hooks setup gemini
limux hooks setup opencode
```

Checked-in templates live under `hooks/`. Limux also exposes receiver commands
for hook payloads:

```bash
limux hooks claude stop
limux hooks gemini finished
limux hooks hermes pre_approval_request
limux claude-hook --event stop
limux gemini-hook --event finished
limux hermes-hook --event pre_approval_request
```

Hermes lifecycle plugin installation is owned externally by the Hermes/hcom
side. Limux handles receiver events once they are delivered.

Default hook setup covers Codex, Claude Code, and Gemini. OpenCode is opt-in
with `limux hooks setup opencode`; Hermes is receiver-only from the Limux side.

PRD-G agent lifecycle work is partial. The direction is hook events feeding
sidebar/session lifecycle states, but do not claim every family has complete
sidebar/socket/CLI lifecycle parity until the PRD-G task is closed.

## Runtime Channels And Verification

Use user-local preview lanes when testing a new Limux without replacing the
daily-driver runtime:

```bash
scripts/user-local-install/install-user-local.sh --apply --channel preview --profile release
~/.local/bin/limux-preview --version
~/.local/bin/limux-preview target-info
~/.local/bin/limux-preview doctor --json
```

Stable and named lanes:

```bash
scripts/user-local-install/install-user-local.sh --apply --channel stable --profile release
scripts/user-local-install/install-user-local.sh --apply --channel preview:lab --profile release
~/.local/bin/limux-preview-lab doctor --log-triage --lines 200
```

The installer writes `install-info.json` with install id, channel, source SHA,
and creation time. Channel-aware launchers pass `--channel` so stable and
preview runtimes resolve separate sockets and state.

Run the Ghostty resource packaging check before trusting an install:

```bash
bash scripts/tests/validate-ghostty-resources.sh
```

For promotion, follow:

- `docs/verification/post-install-checklist-v1.md`
- `docs/verification/run-template.md`

Stable promotion should wait for a full PASS in the preview runtime.

## Review Workflow

Prepare a review request without launching an agent:

```bash
limux review prepare \
  --artifact rust/limux-cli/src/main.rs \
  --reviewer claude \
  --lens maintainability \
  --summary "Review the CLI change for blockers"
```

Valid built-in lenses include `security`, `correctness`, `maintainability`,
`ux`, and `release`.

Launch a prepared review:

```bash
limux review spawn --review-id <review-id>
limux review spawn --review-id <review-id> --launch-mode hcom
```

## Current Bridge Limits

The production GTK bridge supports workspace, pane, surface, terminal
send/key/read/health, notification, and terminal pane-create commands.

PRD-E mirror API parity is partial as of this staged guide:

- Native bridge registry covers the current supported GTK methods.
- Read-only state-mirror fallthrough currently includes `window.list` and
  `window.current`.
- Wider read-only parity, mutation routing, browser-pane bridge behavior, and
  kill-switch behavior remain open PRD-E/PRD-F work.

When in doubt, verify the production GTK bridge path, not only the standalone
`limux-control-server` dispatcher.

## Existing-Pane hcom Resume

For an operator request such as "find the TaskMaster pane, `/exit`, then run
`hcom r sage`", use the exact-surface workflow in `skills/limux-a2a/SKILL.md`:

1. Resolve the workspace and raw surface ID; never inject into the currently
   focused pane as a fallback, and do not restart the Limux host for this
   single-pane operation.
2. If the background workspace is unrealized, identify the agent from
   `$HOME/.local/share/limux/session.json`, then select the workspace through
   the typed `workspace.select` socket method.
3. Read the pane, inject `/exit` plus `Return`, wait for the shell prompt, then
   inject `hcom r <name>` plus `Return` in the same surface.
4. Verify the authoritative session ID, cwd, process/live/terminal/transcript
   bindings, a real hcom round trip, one client for the UUID, and process
   ancestry reaching `limux-host`. Re-read the pane to catch a historical
   session or a duplicate Windows Terminal attachment.
5. If the wrong session resumes, exit only that duplicate and follow the
   evidence-gated recovery in `limux-a2a`; do not blindly retry or routinely
   reset hcom.

If the visible pane and persisted session registry disagree, stop without
injecting or editing `session.json`; another live agent may own the pane.
