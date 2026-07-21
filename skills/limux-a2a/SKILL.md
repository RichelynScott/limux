---
name: limux-a2a
description: Use inside Limux to identify the current pane/surface/workspace, spawn terminal panes or workspaces, launch agent tasks, coordinate same-workspace surfaces, recover a named hcom agent in its existing pane, optionally send callbacks, read peer output, and notify the human.
---

# Limux A2A

Use Limux itself as the live registry. Do not rely on generated files or persistent rosters. Prefer the installed `limux` command; use `./target/.../limux-cli` only when testing this repo build.

## Fast Launch

When asked to launch a peer from inside Limux, split first using the already-exported `LIMUX_*` values. Do not run `identify`, `list-workspaces`, or `read-screen` before the first spawn unless an env value is missing.

```bash
created="$(limux --json new-pane \
  --workspace "$LIMUX_WORKSPACE_ID" \
  --pane "$LIMUX_PANE_ID" \
  --surface "$LIMUX_SURFACE_ID" \
  --direction right \
  --command 'claude "Task prompt here. Parent surface if you need it: '"$LIMUX_SURFACE_ID"'."')"

child_surface="$(printf '%s\n' "$created" | jq -r '.surface_ref // .surface_id' | sed 's/^surface://')"
child_pane="$(printf '%s\n' "$created" | jq -r '.pane_ref // .pane_id' | sed 's/^pane://')"
```

If the fast launch fails with `failed to connect to socket`, retry that exact `new-pane` command with approved/escalated socket access. If it fails with `Unknown option --json`, switch to the repo-built `./target/debug/limux-cli` or `./target/release/limux-cli` and retry the same command. Only after those retries should you inspect socket/listing state.

## Identity

Use **Surface/Pane** in human-facing instructions. Technically, a pane can host
multiple tab surfaces; the surface ID identifies the active tab inside the
pane. Preserve `surface:`, `pane:`, and `tab:` refs in commands and copied
context so the distinction remains machine-safe.

Each Limux terminal should have:

```bash
printf 'workspace=%s\npane=%s\nsurface=%s\ntab=%s\nsocket=%s\n' \
  "$LIMUX_WORKSPACE_ID" "$LIMUX_PANE_ID" "$LIMUX_SURFACE_ID" "$LIMUX_TAB_ID" "$LIMUX_SOCKET"
```

Fallbacks:

```bash
limux --json identify
limux --json list-workspaces
limux --json list-panels --workspace "$LIMUX_WORKSPACE_ID"
```

The terminal right-click menu's **Workspace & Surface/Pane Info** submenu can
copy the same canonical context or a complete `read-screen` command. Prefer
that copied context when another human/session supplies the target.

Do not trust bare `limux identify` for a background or remotely controlled
session when its `LIMUX_*` variables are missing or stale: it can report the
currently focused pane instead. Resolve the workspace explicitly, list its
surfaces, read the candidate, and compare persisted/hcom identity before
mutation.

Target exact peers with `--surface <surface-id>`. Add `--workspace <id-or-name>` when the peer is outside the current workspace.

## Spawn Panes

Launch tools directly with `new-pane --command`; do not create an empty shell and later inject a long escaped `codex "..."` line. Long injected launch lines can wrap or corrupt the shell input.

```bash
created="$(limux --json new-pane \
  --workspace "$LIMUX_WORKSPACE_ID" \
  --pane "$LIMUX_PANE_ID" \
  --surface "$LIMUX_SURFACE_ID" \
  --direction right \
  --command 'codex "Task: inspect the diff. Parent surface available if needed: '"$LIMUX_SURFACE_ID"'."')"

child_surface="$(printf '%s\n' "$created" | jq -r '.surface_ref // .surface_id' | sed 's/^surface://')"
child_pane="$(printf '%s\n' "$created" | jq -r '.pane_ref // .pane_id' | sed 's/^pane://')"
```

`new-pane` returns the child workspace, pane, and surface IDs. Capture them immediately. Live GTK pane creation supports terminal panes. For a launch request, this is the first command to run; use identity/listing commands only as fallback diagnostics.

Interactive and non-interactive examples:

```bash
limux --json new-pane --direction right --command 'codex "Task prompt here."'
limux --json new-pane --direction right --command 'codex exec "Task prompt here."'
limux --json new-pane --direction right --command 'claude "Task prompt here."'
limux --json new-pane --direction right --command 'claude -p "Task prompt here."'
```

## Split Layout

For multiple workers in the same workspace, choose the split source explicitly. Do not repeatedly split the parent `right`, and do not repeatedly split the newest tiny pane unless that is intentional.

Column pattern for three workers:

```bash
ws="$LIMUX_WORKSPACE_ID"
parent_pane="$LIMUX_PANE_ID"
parent_surface="$LIMUX_SURFACE_ID"

w1="$(limux --json new-pane --workspace "$ws" --pane "$parent_pane" --surface "$parent_surface" --direction right --command 'codex "Worker 1 task."')"
w1_pane="$(printf '%s\n' "$w1" | jq -r '.pane_ref // .pane_id' | sed 's/^pane://')"
w1_surface="$(printf '%s\n' "$w1" | jq -r '.surface_ref // .surface_id' | sed 's/^surface://')"

w2="$(limux --json new-pane --workspace "$ws" --pane "$w1_pane" --surface "$w1_surface" --direction down --command 'codex "Worker 2 task."')"
w2_pane="$(printf '%s\n' "$w2" | jq -r '.pane_ref // .pane_id' | sed 's/^pane://')"
w2_surface="$(printf '%s\n' "$w2" | jq -r '.surface_ref // .surface_id' | sed 's/^surface://')"

w3="$(limux --json new-pane --workspace "$ws" --pane "$w2_pane" --surface "$w2_surface" --direction down --command 'codex "Worker 3 task."')"
```

Rules:

- First worker: split the parent `right`.
- More workers: split the worker column `down`.
- Keep a worker column around 3 panes on normal screens.
- For more workers or a second column, prefer a new workspace until Limux has a balanced-grid spawn command.

## Workspaces

Use workspaces when a task needs isolation or same-workspace panes would become too small.

```bash
created="$(limux --json new-workspace --cwd "$PWD" --command 'codex "Task prompt here. Parent surface available if needed: '"$LIMUX_SURFACE_ID"'."')"
workspace="$(printf '%s\n' "$created" | jq -r '.workspace_ref // .workspace_id' | sed 's/^workspace://')"
limux --json list-panels --workspace "$workspace"
```

## Observe vs Callback

Default: the parent observes child progress with `read-screen`. Limux already
supports direct pane-content inspection; do not ask the child to manually
transcribe visible output unless `read-screen` fails.

```bash
limux --json list-workspaces
limux list-panels --workspace "<workspace-ref>"
limux read-screen --workspace "<workspace-ref>" \
  --surface "<surface-ref>" --scrollback --lines 120
limux capture-pane --workspace "<workspace-ref>" \
  --surface "<surface-ref>" --scrollback --lines 200
limux --json surface-health --workspace "<workspace-ref>"
```

`capture-pane` is an alias of `read-screen`. Always include both workspace and
surface when duplicate workspace names exist or when inspecting another
workspace. `surface-health` is workspace-scoped in the current CLI.

Give children the parent surface as an available route, not a mandatory report-back requirement:

```text
Parent surface if you need it: <parent-surface>.
Leave concise results visible in your pane. Use limux send to contact the parent only if blocked, finished with something important the parent should not miss, coordinating with siblings, or asked for status pings.
```

Callback command when needed:

```bash
limux send --surface "<parent-surface>" "short message"
limux send-key --surface "<parent-surface>" enter
```

Codex child note: if a callback fails with `failed to connect to socket`, retry the exact `limux send` / `send-key` command with approved or escalated command execution. The surface ID is still valid; socket access is the blocker.

## Send And Coordinate

Send text:

```bash
limux send --surface "<surface>" "hello"
limux send --workspace "<workspace>" --surface "<surface>" "hello"
limux send-key --surface "<surface>" enter
```

Use a small envelope for structured requests:

```bash
limux send --surface "<surface>" $'<limux-msg from-surface="'"$LIMUX_SURFACE_ID"'" to-surface="<surface>" id="'"$(uuidgen)"'" ts="'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'">\n<request>Do the task.</request>\n</limux-msg>\n'
limux send-key --surface "<surface>" enter
```

For related workers, send each child the sibling surface map after all spawns complete:

```bash
limux send --surface "$w1_surface" $'Sibling surfaces: '"$w2_surface $w3_surface"$'\n'
limux send-key --surface "$w1_surface" enter
```

## Human Attention

```bash
limux notify --workspace "$LIMUX_WORKSPACE_ID" \
  --subtitle "input needed" \
  --body "A pane is blocked and needs a decision" \
  "Limux task needs attention"
```

## Resume A Named hcom Agent In Its Existing Pane

Use this when the operator names a workspace and asks to exit the agent in one
of its existing panes, then run `hcom r <name>` in that exact pane. Never use
the currently focused pane as a fallback. This is a single-pane operation: do
not restart the Limux host merely to complete it.

1. Resolve the workspace and its raw surface IDs:

   ```bash
   limux --json list-workspaces
   limux list-panels --workspace "<workspace-id>"
   ```

   Some installed versions print `surface:<id>` references. Pass the raw
   `<pane-id>:<tab-id>` value to `read-screen`, `send`, and `send-key`.

2. Identify the agent surface before injecting anything. Read each candidate:

   ```bash
   limux read-screen --workspace "<workspace-id>" \
     --surface "<raw-surface-id>" --scrollback --lines 80
   ```

   If a background workspace is unrealized and every surface reports zero
   rows/columns or an empty screen, inspect the persisted session registry at
   `$HOME/.local/share/limux/session.json`. Match the workspace, then require
   the tab's `agent.kind`, `agent.session_id`, or
   `agent.launch_command.environment.HCOM_INSTANCE_NAME` to identify the named
   agent. Do not infer identity from pane order.

   Fail closed if the visible pane, persisted `agent.session_id`, persisted
   `HCOM_INSTANCE_NAME`, and hcom identity do not agree. Do not inject `/exit`,
   rewrite `session.json`, or restart Limux in that state; preserve the evidence
   and route the restore-state corruption to the Limux/hcom owners.

3. Realize the workspace when the installed CLI has no `workspace.select`
   wrapper. Resolve the channel socket with `limux target-info` or
   `$LIMUX_SOCKET`, then send the typed request:

   ```bash
   printf '%s\n' \
     '{"id":1201,"method":"workspace.select","params":{"workspace_id":"<workspace-id>"}}' |
     nc -U -N "${LIMUX_SOCKET:-/run/user/$(id -u)/limux/limux.sock}"
   ```

   Re-run `surface-health` and `read-screen` after selection. Do not hardcode
   the legacy socket when operating a stable or preview channel.

4. Exit only the verified surface, wait for its shell prompt, then resume:

   ```bash
   limux send --workspace "<workspace-id>" --surface "<raw-surface-id>" '/exit'
   limux send-key --workspace "<workspace-id>" --surface "<raw-surface-id>" Return
   limux read-screen --workspace "<workspace-id>" --surface "<raw-surface-id>" --lines 40

   limux send --workspace "<workspace-id>" --surface "<raw-surface-id>" 'hcom r <name>'
   limux send-key --workspace "<workspace-id>" --surface "<raw-surface-id>" Return
   ```

   Use the key spelling accepted by the installed runtime. `Return` is the
   compatibility spelling when lowercase `enter` is rejected.

5. Verify identity, not just process startup:

   ```bash
   hcom list <name> -v --json --name <manager-name>
   hcom send @<name> --intent request --thread <recovery-thread> \
     --name <manager-name> -- "Reply with your hcom name and session id."
   ```

   Require the expected name, authoritative session ID, working directory,
   `process_bound`, `live_delivery_available`, `term_available`, transcript
   binding, no unexpected control warnings, and a real reply. Re-read the pane
   to ensure the expected agent, not another historical session, resumed.

   Also prove there is only one client attached to the authoritative UUID and
   that its process ancestry reaches `limux-host`. A correct screen and a
   correct hcom name are insufficient if the same native session is also
   resumed in Windows Terminal or another host. Treat `WT_SESSION` on the live
   launch context, an `/init` ancestry that bypasses `limux-host`, or multiple
   `codex resume <uuid>` clients as a placement failure.

   For a formal evidence run, follow
   `docs/verification/host-owned-surface-process-attestation.md`. It separates
   observer PID visibility, read-only Surface/Pane ancestry attestation, and
   pane-preserving teardown into independently authorized fail-closed gates.

### Wrong-Session Guard

If `hcom r <name>` resumes a different session, inject `/exit` into that exact
duplicate surface immediately and preserve the mismatch evidence. Do not keep
retrying by name or UUID: stale stopped-snapshot selection can resolve the same
wrong session repeatedly.

If the authoritative session is concurrently attached in Limux and another
terminal, exit the non-Limux attachment through its exact terminal/control
endpoint first. Then run the literal `/exit` followed by `hcom r <name>` in the
verified Limux surface so hcom's PTY, process, transcript, and surface bindings
all originate from the Limux-hosted process.

`hcom forget <name> --go` is a last-resort repair only after all of these are
true: the duplicate is stopped, the authoritative native transcript UUID is
known and still exists, the mismatch is proven, and the operator authorized
the mutation of hcom event history. Then adopt the authoritative UUID in the
same pane and require it to bind the intended name:

```bash
hcom r <authoritative-session-uuid> --run-here --go \
  --hcom-prompt 'Recovery action: immediately run hcom start --as <name>, then report your bound name and session id.'
```

Re-run the full identity and nonce round-trip verification afterward.

When constructing injected text in a shell command, do not place backticks
inside a double-quoted shell string; command substitution can execute them.
Use single-quoted payloads, a safely quoted heredoc, or hcom file/base64 input.

## Approved Peer-Assisted Sandbox Relaunch

Use this only when the operator explicitly approves `danger-full-access` for a
named target session and the target's current sandbox makes its real task
impossible, such as a read-only Git metadata directory. It is never a default
Limux or hcom launch mode.

The target session cannot safely replace itself. A separate authorized
controller must:

1. Capture the target's hcom name, authoritative session ID, workspace ref,
   Surface/Pane ref, cwd, and current screen. Require the target to checkpoint
   and stop new work.
2. Send `/exit` only to that exact verified surface, press the runtime-compatible
   return key, and wait until `read-screen` shows the shell prompt.
3. In the same surface, type the exact approved command:

   ```bash
   hcom r <name> --run-here --go --sandbox danger-full-access
   ```

4. Verify one native client, the expected session/name/cwd, `limux-host`
   ancestry, complete hcom bindings, the expected `LIMUX_*` target, and a nonce
   ACK. Have the resumed Codex session run `/status` and verify the effective
   sandbox and approval policy. `--sandbox danger-full-access` changes the
   filesystem sandbox; it does not by itself disable approval prompts. Confirm
   the original blocker with a non-mutating check such as `test -w "$(git
   rev-parse --git-dir)"` before continuing.

Fail closed if the pane is not at a shell, another agent appears, identity is
ambiguous, a duplicate client exists, or copied context disagrees with live
state. Do not use focused-pane fallback, do not elevate multiple sessions at
once. The override is not global, but installed hcom `0.7.66` can retain
effective per-target launch arguments for later resumes. Record the exact prior
sandbox and approval policy before relaunch, then explicitly restore both when
the elevated task ends. If launch or identity verification fails, exit the
replacement and resume with those exact prior settings or stop for operator
direction. Newer hcom versions may strip historical policy arguments, but do
not assume that behavior without checking the installed runtime.

## Failure Handling

- `failed to connect to socket`: check `LIMUX_SOCKET` and whether the host is running; Codex children may need approved/escalated socket commands.
- `workspace not found`: run `limux --json list-workspaces`.
- `terminal surface not found`: run `limux --json list-panels --workspace ...`; surfaces change when panes/tabs are recreated.
- Text appears but does not run: send `limux send-key ... enter`.
- Target is silent: use `surface-health`, then `read-screen`, then a short follow-up message if needed.
