# Task 3 bounded-logging real-preview verification

Date: 2026-07-16
Operator lane: `bulo`
Branch: `bulo/bounded-logging-task3-20260716`
Verified source head: `490101c3a7f0fd311ac2169ab13043a00a1ba44d`
Ghostty submodule: `81ab8ffa90185221782baf785e85387321e16f8d`
Result: PASS

## Scope and isolation

This was an isolated exact-head preview observation only. No user-local install,
promotion, daily-driver restart, or Task 4+ work was performed.

- Evidence root: `/tmp/limux-task3-real-preview-490101c.HrXBPf`
- Isolated XDG runtime/state/data/config directories were all rooted below the
  evidence root.
- The host used an isolated Xvfb display, isolated D-Bus session, isolated
  socket at `runtime/limux/limux.sock`, and isolated log path at
  `logs/limux-host.current.log`.
- The exact-head host and CLI both reported version `0.2.2` and source
  `490101c3a7f0` with debug profile.
- The host used the pinned Ghostty submodule built into the evidence root and
  the repository's shared Cargo/Zig caches.

## Startup and shutdown evidence

Four isolated launches returned a responsive control socket. `system.identify`
reported the exact source SHA, and X11 inspection showed the default single
`Limux v0.2.2` 1280x800 window with its inert initial terminal surface. Each
instance was closed through the application's exported GTK `quit` action, not
by a kill signal:

```text
HOST_EXIT=0
HOST2_EXIT=0
HOST3_EXIT=0
HOST4_EXIT=0
```

Representative identity:

```json
{
  "pid": 3035497,
  "version": "0.2.2",
  "build": {
    "channel": null,
    "dirty": null,
    "install_id": null,
    "profile": "debug",
    "sha": "490101c3a7f0"
  },
  "socket_path": "/tmp/limux-task3-real-preview-490101c.HrXBPf/runtime/limux/limux.sock"
}
```

## Bounded current log and retained-file behavior

The final active log was the new isolated file and remained well below the
64 MiB active-file limit:

```text
path=/tmp/limux-task3-real-preview-490101c.HrXBPf/logs/limux-host.current.log size=311 blocks=8 inode=3516125 mode=644 uid=1000 gid=1000 mtime=1784197486 ctime=1784197486
```

An isolated 68,157,751-byte active-log fixture was moved aside on the next real
preview startup. Subsequent startups created distinct retained names. Hashes of
the pre-existing retained files were unchanged after another startup, proving
that the retained directory did not clobber prior entries:

```text
2902c65162bbb4d455fb56ffa0200a68d07720a9cc78297f5caec0b037cd0e1d  limux-host.1784197396116819341.log
537b6955f04a6a57c3411d608223e1672facf9df758508c15ea7f45a0d957d68  limux-host.1784197427564881291.log
9ba937e433418bbf2228a25938d7306a02839b0fdf3d2ff3fe147d85de5e24f3  limux-host.1784197486404733468.log
```

Final isolated log inventory:

```text
limux-host.current.log                              311 bytes
retained/limux-host.1784197396116819341.log         311 bytes
retained/limux-host.1784197427564881291.log  68,157,751 bytes
retained/limux-host.1784197486404733468.log         311 bytes
```

## Doctor bounded-read evidence

`doctor --json --log-triage --lines 2` was pointed at a separate synthetic
2 MiB preview log. It read exactly the 1 MiB cap and reported truncation:

```json
{
  "bytes_read": 1048576,
  "lines_scanned": 2,
  "path": "/tmp/limux-task3-real-preview-490101c.HrXBPf/synthetic/doctor-large.log",
  "status": "ok",
  "truncated": true
}
```

The doctor process returned `2` because it also reported pre-existing stale
socket warnings outside this isolated run. The log-triage check itself was
`ok`, the exact-head preview socket check was `ok`, and the byte cap held.

## Legacy incident-log preservation

The legacy incident log was never opened, read, hashed, moved, rewritten, or
truncated. Only metadata-only `stat` calls were made before and after the
isolated run. The captured lines were identical and `cmp` returned zero:

```text
path=/home/riche/.local/state/limux/logs/limux-host.log size=27734854453 blocks=54169664 inode=1168098 mode=644 uid=1000 gid=1000 mtime=1784187864 ctime=1784187864
LEGACY_METADATA_CMP_RC=0
```

## Retained fallback residual

The prospective logger intentionally does not delete retained logs. If moving a
new active log would exceed the configured retained-count or total-byte budget,
setup falls back to inherited console stderr and preserves every existing file.
That archive-required fallback is the remaining operational residual; it is not
a failure of this Task 3 preview gate.
