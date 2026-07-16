# Renderer Backend Preview Runner

`renderer-backend-preview.sh` proves renderer fallback as separate isolated
process attempts. It never switches GTK renderers inside a running process.

Every attempt receives its own socket, session copy, XDG directories, bounded
stdout/stderr capture, and retained health evidence. The host runs in an
attempt-owned process session; rejection or completion stops that session and
its terminal descendants, then waits for the tracked capture helpers. It never
installs, promotes, or mutates the daily driver. The artifacts directory must
not already exist.

Example:

```bash
scripts/renderer-backend-preview/renderer-backend-preview.sh \
  --host /absolute/path/to/limux-host \
  --cli /absolute/path/to/limux-cli \
  --session-template /absolute/path/to/session.json \
  --artifacts /tmp/limux-renderer-preview-unique \
  --start wsl-d3d12-gl
```

Supported candidates and fallback order:

1. `wsl-d3d12-gl`
2. `desktop-gl`
3. `software-gl`

The final software entry sets `LP_NUM_THREADS=2` so fallback CPU use is bounded
during preview evidence collection.

`invalid-test` is a deliberate failure-injection entry that falls back to
`wsl-d3d12-gl`. It is for isolated validation only.

Acceptance requires a captured renderer diagnostic and at least one terminal
surface that is healthy, realized, and has non-zero pixel dimensions. Three
consecutive captured unhealthy or candidate-mismatch samples reject the
candidate. D3D12 additionally requires a non-software `GskGLRenderer` and an
open `/dev/dxg` descriptor. Automatic desktop GL rejects software fallback so
the bounded `LP_NUM_THREADS=2` software entry is not skipped. Successful hosts
are also stopped after verification; `result.json` records the selected backend
and attempt order. Each CLI probe has a two-second timeout and each backend has
a 30-second wall-time limit.

Before executing a modified runner or test, apply the global no-delete static
gate to this directory.
