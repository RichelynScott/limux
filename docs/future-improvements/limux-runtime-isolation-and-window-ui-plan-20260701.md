# Limux Runtime Isolation And Window UI Plan

Date: 2026-07-01
Owner: lifo
TaskMaster tag: master

## Current Window Chrome Fix

TaskMaster #15 tracks the immediate window edge/titlebar issue.

The current implementation fix is commit `d94dc89` on branch
`lifo/hermes-workspace-highlight-resize-20260627`. Limux now uses its own
GTK/libadwaita client-side titlebar even when the Wayland compositor advertises
server-side decoration support. The titlebar decoration layout is
`minimize,maximize,close`, which restores standard window controls and avoids
delegating resize hit testing to the compositor path that exposed only a close
button.

Verification completed:

- `cargo fmt --check`
- `cargo check -p limux-host-linux`
- `cargo test -p limux-host-linux window::tests::window_chrome_policy_prefers_limux_controls_when_compositor_decorates`
- `cargo test -p limux-host-linux` outside the restricted sandbox
- `cargo clippy -p limux-host-linux --all-targets -- -D warnings`

The restricted sandbox run of the full host test suite failed existing runtime
socket tests with `/run/user/...` socket bind `PermissionDenied`; the same suite
passed outside the sandbox.

## High-Priority Runtime Isolation Lane

TaskMaster #19 tracks isolated stable and preview Limux runtime channels.

This should be a first-class subproject because Limux is now the primary working
environment. The goal is to let the daily-driver Limux process continue with all
active workspaces and panes while preview builds are installed, launched, tested,
restarted, or discarded without touching the primary runtime.

Required isolation surfaces:

- install prefix: stable and preview builds must live under separate directories;
- process identity: preview host should have a distinct app/session identity;
- socket path: preview CLI commands must target only the preview socket by default;
- session state: preview must not load or mutate stable workspace/session state;
- config path: preview settings should not overwrite stable settings;
- desktop/window identity: preview should be visually distinguishable;
- launch tooling: scripts should make stable vs preview targeting explicit;
- smoke tests: preview startup, socket health, workspace creation, and shutdown
  should be testable without interacting with the stable runtime.

Recommended first milestone:

1. Audit current runtime identity inputs: install prefix, host path,
   `LIMUX_SOCKET`, session dir, app id, config dir, and persistence dir.
2. Add a named runtime/channel flag such as `--runtime-channel stable|preview`
   or `LIMUX_RUNTIME_CHANNEL=preview`.
3. Derive socket, config, session, and app identity from that channel unless an
   explicit override is supplied.
4. Add a `scripts/launch-preview-runtime.sh` wrapper that builds or launches the
   preview host without killing or contacting the stable runtime.
5. Add smoke coverage that asserts stable and preview sockets/session dirs are
   distinct.

Acceptance condition: the operator can keep stable Limux open with many active
workspaces, launch a preview build, test changes there, close preview, and still
have the stable runtime unaffected.

## Future UI/Research Tasks

TaskMaster #16 tracks future opacity and always-on-top controls.

TaskMaster #17 tracks a future detachable workspace notification/sidebar system
with separate always-on-top, opacity, show/hide ribbon, click-to-raise, and
reattach behavior.

TaskMaster #18 tracks later multi-agent upstream research across Limux and
manaflow-ai/cmux PRs, issues, releases, and code so candidate features/fixes can
be sorted, rated, and promoted into PRDs or subprojects.

