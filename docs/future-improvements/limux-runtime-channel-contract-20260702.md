# Limux Runtime Channel Contract

Author/runtime/date: lifo / Codex gpt-5.5 (xhigh) / 2026-07-02
TaskMaster: #19.2

## Contract

Limux supports an explicit runtime channel layer for running the daily-driver
runtime and preview/test runtimes side by side.

## Channel Names

| Channel | Selector | Purpose |
|---|---|---|
| Legacy default | no channel | Preserves existing behavior for current installs and scripts. |
| Stable | `stable` | Named daily-driver runtime. |
| Preview | `preview[:id]` | Isolated test runtime. `id` must be ASCII alphanumeric, `_`, or `-`. |

`preview` without an id resolves to `preview:default`.

## Selector Precedence

Normal legacy resolution remains unchanged:

1. `--socket <path>`
2. `LIMUX_SOCKET`
3. `LIMUX_SOCKET_PATH`
4. `LIMUX_CHANNEL`
5. legacy runtime default

Explicit CLI channel targeting uses a stronger contract:

1. `--socket <path>`
2. `--channel stable|preview[:id]`
3. legacy/env fallback

This means `limux --channel preview:branch identify` will not accidentally use
an inherited stable `LIMUX_SOCKET` from a terminal pane.

## Socket Paths

With `$XDG_RUNTIME_DIR`:

| Channel | Socket |
|---|---|
| Stable | `$XDG_RUNTIME_DIR/limux/stable/limux.sock` |
| Preview | `$XDG_RUNTIME_DIR/limux/preview/<id>/limux.sock` |

Without `$XDG_RUNTIME_DIR`:

| Channel | Socket |
|---|---|
| Stable | `/tmp/limux-stable.sock` |
| Preview | `/tmp/limux-preview-<id>.sock` |

The legacy no-channel default remains `$XDG_RUNTIME_DIR/limux/limux.sock`, with
fallback `/tmp/limux.sock`.

## Session Persistence

With `LIMUX_CHANNEL` set and no explicit `LIMUX_SESSION_DIR`:

| Channel | Session dir |
|---|---|
| Stable | `$XDG_DATA_HOME/limux/stable/session` |
| Preview | `$XDG_DATA_HOME/limux/preview/<id>/session` |

`LIMUX_SESSION_DIR` remains the highest-precedence session-state override.

## Host Launch

`limux --channel <channel>` with no subcommand launches the host with
`LIMUX_CHANNEL=<channel>` in its environment. Existing `limux` with no channel
continues to launch the legacy default runtime.

## Implementation Pointers

- Channel parsing and socket paths: `rust/limux-control/src/socket_path.rs`
- Host session namespace: `rust/limux-host-linux/src/layout_state.rs`
- CLI `--channel` targeting and host launch env: `rust/limux-cli/src/main.rs`

## Next Work

Task #19.3 should add stable/preview user-local wrappers so the operator does
not need to remember `--channel preview:<id>` manually, and so preview installs
do not rewrite the stable `~/.local/bin/limux` launcher.
