# Limux Dual Runtime Runbook

Author/runtime/date: lifo / Codex gpt-5.5 (xhigh) / 2026-07-02
TaskMaster: #19.5

## Purpose

Let the daily-driver Limux runtime keep running while a separate preview build
is installed, launched, tested, and replaced from the Limux source tree.

## Current Contract

- `limux` remains the legacy/default launcher.
- `limux-stable` targets the named stable channel.
- `limux-preview` targets `preview:default`.
- `limux-preview-<id>` targets `preview:<id>`.
- Preview launchers use a separate install root, socket namespace, and session
  persistence namespace.
- The installer archives replaced launcher paths; it does not delete them.

## Build And Install Preview

From the Limux repo:

```bash
cargo build -p limux-cli --bin limux-cli
cargo build -p limux-host-linux --bin limux
scripts/user-local-install/install-user-local.sh --apply --profile debug --channel preview --install-id preview-$(git rev-parse --short=12 HEAD)
```

This writes a preview install under:

```text
~/.local/limux-reviewed/preview/default/<install-id>/
```

and updates:

```text
~/.local/bin/limux-preview
~/.local/bin/limux-preview-cli
```

It does not rewrite `~/.local/bin/limux`.

## Verify Targeting Before Launch

```bash
limux target-info
limux-preview target-info
LIMUX_SOCKET=/tmp/pretend-stable.sock limux-preview target-info
```

Expected preview output:

```text
explicit_channel=preview:default
connects=false
```

The resolved socket should contain either:

```text
/limux/preview/default/limux.sock
```

or the no-`XDG_RUNTIME_DIR` fallback:

```text
/tmp/limux-preview-default.sock
```

## Launch Preview

```bash
limux-preview
```

The preview host will use:

```text
LIMUX_CHANNEL=preview:default
```

and will keep its socket/session namespace separate from the legacy/default
daily-driver runtime.

## Smoke Test

Run the maintained smoke from the repo:

```bash
bash scripts/tests/runtime-isolation-smoke.sh
```

The smoke builds debug artifacts, installs legacy, stable, and preview lanes
into a unique `/tmp` prefix, verifies launcher symlinks, and confirms
`limux-preview target-info` ignores an inherited stable `LIMUX_SOCKET`.

Set `LIMUX_SMOKE_PROFILE=release` to smoke release artifacts.

## Operator Safety Notes

- Do not run the preview install with `--channel legacy` unless intentionally
  replacing the current default `limux` launcher.
- Test preview with `limux-preview` first; install over the daily-driver lane
  only after PR review and operator approval.
- Keep the daily-driver Limux running while preview is tested. The preview
  channel is designed to avoid touching the existing control socket and session
  store.
