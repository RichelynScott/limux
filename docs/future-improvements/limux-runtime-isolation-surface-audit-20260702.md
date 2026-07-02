# Limux Runtime Isolation Surface Audit

Author/runtime/date: lifo / Codex gpt-5.5 (xhigh) / 2026-07-02
TaskMaster: #19.1

## Goal

Identify the current runtime identity and mutable-state surfaces that must be
split before Limux can run a stable daily-driver runtime and an isolated preview
runtime at the same time.

## Current Surfaces

| Surface | Current behavior | Source |
|---|---|---|
| App ID | Single GTK application id: `dev.limux.linux`. App is `NON_UNIQUE`, so multiple processes can exist but still share desktop identity. | `rust/limux-host-linux/src/main.rs` |
| Host launch | `limux` with no command resolves host through `LIMUX_HOST_BIN`, then installed `libexec/limux/limux-host`, sibling `limux-host`, or sibling dev `limux`. | `rust/limux-cli/src/main.rs` |
| Install root | User-local installer writes `~/.local/limux-reviewed/<install-id>/`, then points `~/.local/bin/limux` and `limux-cli` at that one install. | `scripts/user-local-install/install-user-local.sh` |
| Control socket | Default runtime socket is `$XDG_RUNTIME_DIR/limux/limux.sock`, falling back to `/tmp/limux.sock`. Explicit `--socket`, `LIMUX_SOCKET`, then `LIMUX_SOCKET_PATH` override it. | `rust/limux-control/src/socket_path.rs` |
| Socket collision | If the default socket accepts connections and no explicit socket env is set, host uses `limux-<pid>.sock` and derives a matching session dir. This avoids bind failure but does not create a named stable/preview contract. | `rust/limux-host-linux/src/main.rs` |
| Socket auth | Default `LIMUX_SOCKET_MODE` is same-local-user. Modes are `localUser`, `limuxOnly`, and `allowAll`; owner-only sockets are mode `0600`. | `rust/limux-control/src/auth.rs` |
| Session state | Default persistence dir is `dirs::data_dir()/limux`; explicit `LIMUX_SESSION_DIR` overrides. Session file is `session.json`; legacy file is `workspaces.json`. | `rust/limux-host-linux/src/layout_state.rs` |
| CLI targeting | CLI resolves socket once at startup from `--socket`, env, or default runtime mode; all commands then use that socket. There is no `--channel stable|preview` concept yet. | `rust/limux-cli/src/main.rs` |
| Terminal env | Spawned terminals receive `LIMUX_WORKSPACE_ID`, `LIMUX_SURFACE_ID`, `LIMUX_PANE_ID`, `LIMUX_TAB_ID`, and resolved `LIMUX_SOCKET`. | `rust/limux-host-linux/src/pane.rs` |
| Host logs | Default stderr log path is `dirs::state_dir()/limux/logs/limux-host.log`; `LIMUX_HOST_LOG_PATH` overrides and `LIMUX_HOST_LOG=off|0` disables. | `rust/limux-host-linux/src/main.rs` |
| App settings | Settings file is `dirs::config_dir()/limux/settings.json`; shortcuts are under the same config dir. | `rust/limux-host-linux/src/app_config.rs`, `shortcut_config.rs` |
| GSettings | Host reads compiled GSettings schemas from `GSETTINGS_SCHEMA_DIR`, then system data dirs. No channel-specific schema dir exists. | `rust/limux-host-linux/src/window.rs` |
| Ghostty resources | Host resolves resources relative to executable ancestors first, then common system Ghostty locations. Source-only `ghostty/src` is rejected. | `rust/limux-host-linux/src/main.rs` |
| Desktop entry/icons | Installed desktop entry uses the single source desktop id/assets and can be copied to `~/.local/share/applications/dev.limux.linux.desktop`. | `scripts/user-local-install/install-user-local.sh` |

## Isolation Gaps

1. The stable and preview runtimes need explicit channel identity. Today a
   second runtime can fall back to a PID socket, but CLI commands and visual
   identity do not know which runtime is intended.
2. The installer mutates the canonical `~/.local/bin/limux` symlink. Preview
   installs need their own wrapper name, such as `limux-preview`, and must not
   overwrite the stable launcher.
3. Session/config/log paths are still shared unless env overrides are supplied.
   A preview runtime needs explicit defaults for `LIMUX_SESSION_DIR`,
   config/settings, shortcuts, and logs.
4. The control CLI needs channel-aware targeting diagnostics so accidental
   commands to the stable runtime are obvious before mutation.
5. Desktop/window identity should distinguish preview from stable at first
   glance. Reusing only `dev.limux.linux` is not enough for operator safety.

## Recommended Channel Contract

Use two named channels:

| Channel | Launcher | Socket | Session dir | Config/state/log namespace |
|---|---|---|---|---|
| stable | `limux` | `$XDG_RUNTIME_DIR/limux/stable/limux.sock` | `$XDG_DATA_HOME/limux/stable/session` | existing `limux` paths until migrated |
| preview | `limux-preview` | `$XDG_RUNTIME_DIR/limux/preview/<id>/limux.sock` | `$XDG_DATA_HOME/limux/preview/<id>/session` | preview-specific config/state/log dirs |

The preview install should use a unique `<id>` based on branch or commit. The
CLI should support explicit `--channel stable|preview` and keep `--socket` as
the highest-precedence escape hatch.

## Acceptance Criteria For #19.2+

- Stable and preview can be launched together with distinct sockets, session
  files, logs, and visible window labels.
- `limux-preview identify --json` cannot accidentally target stable.
- Preview install does not rewrite `~/.local/bin/limux`.
- Closing preview leaves the stable process and its workspaces untouched.
- Smoke tests assert both sockets exist and point at different runtimes.
