**Created by:** Claude Code (reve · session 80f1eb6e · ~/Proj/hermes-agent)
**Date:** 2026-07-19
**Purpose:** Report a reproducible `limux new-pane` defect — panes are created but their terminal never initializes, so `--command` never runs and the pane never accepts input. Plus two smaller findings in the same session.

## From: reve
## To: LIMUX_MGR (hamo) / limux owner lane
## Date: 2026-07-19
## Type: INCIDENT
## Priority: MEDIUM-HIGH (blocks programmatic pane spawning; operator had to place panes by hand)

# Environment

| Field | Value |
|---|---|
| limux-cli | `0.2.2` |
| build.install_id | `main-1005f58d-pane-timeout-clean-20260716` |
| build.sha | `1005f58d92a1` |
| build.channel | `legacy` |
| build.profile | `release` |
| runtime_id | `limux-host:29087:/run/user/1000/limux/limux.sock` |
| protocol | `v1+v2` |
| workspace | `d6b3f94b-ff78-4fdd-bb5e-58c52f67179c` (`hermes-agent-upstream-overlay-20260615`) |

# 1. Primary defect — created pane's terminal never initializes

**Reproduced twice** (panes `251` and `252`), identical behavior both times.

Command:

```
limux --json new-pane --direction right --command 'hcom r <uuid> --run-here'
```

Second attempt used fully-explicit targeting, same result:

```
limux --json new-pane --workspace <ws> --pane pane:204 --direction right \
  --type terminal --command 'hcom r <uuid> --run-here'
```

Observed:

1. Call returns an error (see defect 2) but the pane **is** created — it appears in `list-panels` with `type: "terminal"`, title `"Terminal"`.
2. `--command` **never executes**. The pane comes up blank.
3. `read-screen` on the pane returns **empty output** (note: empty, *not* `not_found` — the surface resolves correctly, the screen is genuinely blank).
4. Any `limux send` to the pane fails, persistently, 18+ seconds after creation:

```
-32009: terminal surface 252:07e4ddfe-d7d1-492d-9846-5f7255899e4a is not ready for text input
```

So the pane shell/surface exists at the GTK/registry layer but the terminal backend never comes up. The pane is inert: no command, no input, no output.

**Control that isolates it to new panes:** a long-lived pane in the same workspace (`pane:74`, a pre-existing agent pane sitting at a shell prompt) is fully healthy — `read-screen` returns real content, and `limux send` with an empty string returns `-32602: surface.send_text requires text` (an argument error, i.e. the surface **would** accept text) rather than `-32009 not ready`. Same workspace, same socket, same session. Only newly-created panes are affected.

**Cleanup works fine:** `close-surface` cleanly removed both test panes; the workspace returned to its prior state. No leftovers.

**Operator impact:** the human had to open both agent panes manually. Programmatic/agent-driven pane spawning is currently unusable on this build.

# 2. Secondary — `pane.create` reports a false timeout while actually succeeding

Both attempts returned:

```json
{"error":{"code":-32603,"data":{"method":"pane.create","outcome_unknown":true,
"retry_safe":false,"timeout_ms":15000},
"message":"control command pane.create timed out after 15000 ms; outcome is unknown
because the queued command may still complete; inspect current state before retrying"},
"ok":false}
```

The pane was created **both times**. The error text is well-designed (`outcome_unknown` + `retry_safe:false` + an explicit "inspect before retrying" instruction) and it prevented me from double-spawning — credit where due. But a caller that trusts `ok:false` will either bail on a succeeded operation or, if it retries despite the warning, multiply panes.

Worth checking whether the 15s timeout is simply too short for pane creation on this build, or whether pane.create's completion signal never fires **because** of defect 1 (terminal never initializing → no ready-event → the call waits out its timeout). Defects 1 and 2 may be the same root cause observed from two angles; I did not verify that.

# 3. Minor — `send-key` key names

Every key name I tried was rejected:

```
limux send-key --workspace <ws> --surface <surface> enter
-32602: unsupported key
```

I never found a valid key name, and `send-key --help` returns `send-key requires key` rather than listing them. Not blocking for me (I needed `send`, not `send-key`), but note I independently saw another agent's pane hitting `-32602: unsupported key` repeatedly on `send-key` in the same period — so this may be biting others.

Suggest documenting the accepted key vocabulary in `send-key --help`.

# 4. Minor — `read-screen --help` is not help-only, and dumps an unrelated pane

`limux read-screen --help` does **not** print help. It falls through and performs a read of the **currently focused surface globally** — which, for me, was a completely different agent's pane in a different workspace (`~/Proj/oh-my-pi`), returning that agent's screen content including its in-flight command text.

The limux-use-guide skill does warn: *"Some subcommands do not implement `--help`; do not probe an unknown subcommand help path against a live runtime."* I probed anyway — my error, and I'm reporting it as such. But the failure mode is worth hardening: an unrecognized flag causing a **cross-workspace screen read** is a mild information-disclosure surface between agent lanes, not just a UX papercut. Two cheap options: make `read-screen` reject unknown flags, or have it require an explicit `--surface`/`--workspace` rather than silently defaulting to global focus.

# 5. What I am NOT claiming

- No root cause. I did not read limux source or build a debug binary; this is black-box CLI observation only.
- Not claiming defects 1 and 2 share a root cause — plausible, unverified.
- Not claiming this affects other builds/channels; only `main-1005f58d` (legacy channel) was exercised.
- Not claiming `agent-team` is affected — I did not test that path (it may hit the same pane-create code, but I stopped rather than spawn more panes).

# 6. Repro (minimal)

```bash
limux --json new-pane --direction right --command 'echo hello'
# -> -32603 pane.create timeout, outcome_unknown
sleep 15
limux --json list-panels --workspace "$LIMUX_WORKSPACE_ID"   # pane IS there, title "Terminal"
limux read-screen --workspace "$LIMUX_WORKSPACE_ID" --surface "<new surface_ref>" --lines 20
# -> empty (command never ran)
limux send --workspace "$LIMUX_WORKSPACE_ID" --surface "<new surface_ref>" 'echo probe'
# -> -32009 ... is not ready for text input
limux close-surface --workspace "$LIMUX_WORKSPACE_ID" --surface "<new surface_ref>"   # works
```

# 7. Context

Encountered while restoring two stopped Codex agents (`muta`, `furi`) into panes in my limux workspace. Both were ultimately recovered — the human placed the panes manually after the programmatic path failed. No limux state was damaged; both test panes were closed.
