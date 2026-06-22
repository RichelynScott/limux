# Limux Nato Handoff

**Created by:** Claude Code (nato_LIMUX_MGR[CLAUDE] · 0afca090)
**Date:** 2026-06-22 16:33 EDT / 20:33 UTC
**Purpose:** Resume-safe state for the Limux lane taken over from lifo (Codex limit-reached), in case the pending operator Limux restart kills this session's terminal.

## Immediate Next Action

**Pending: operator full Limux restart** (their gated step) to load the freshly-installed
current-main build. After restart, verify the human-NOTE GTK/GLib launch errors are GONE:

```bash
tail -n 60 /home/riche/.local/state/limux/logs/limux-host.log
# expect: NO 'duplicate child name in GtkStack', no gtk_box_append, no GTK_IS_*,
#         no g_settings_schema_source_lookup, no 'Unrecognized image file format'
readlink -f /home/riche/.local/bin/limux   # expect: .../main-20260622-2fcfc55/bin/limux
```

If any of those errors RECUR on the new build → it's a live-only bug (not reproduced
headless). Do NOT patch preemptively. Capture per lifo Q3: exact action+time, the host
log, a `session.json` snapshot, process/socket list (`ps`/`ss`), build-id/symlink target,
and whether multiple limux runtimes are active. Then investigate the live-interaction path.

## What Happened This Session

| Item | Detail |
|---|---|
| Lane accepted | From lifo (Codex usage-limit). Read LIFO_HANDOFF.md / HANDOFF.md / HALO_HANDOFF.md. |
| Repo state | `main` @ `2fcfc55`, local==origin, source tree clean (no patch made). |
| Root cause | Operator was running STALE installed build `29fd2ff` (v0.1.19, behind main). Human-NOTE GTK/GLib errors are from that stale build. |
| Reproduction | Built current main; restored the EXACT corrupted live `session.json` (9× `terminal-0`) headless/xvfb, full XDG isolation → all 10 workspaces restored, ZERO GTK criticals. Cluster already fixed by post-29fd2ff G0+#2/#3 work. |
| Remedy applied | Reviewed user-local install from current main → install-id `main-20260622-2fcfc55`. Symlink repointed; SHA256SUMS all OK; new CLI functional. |
| lifo sanity check | Confirmed (hcom #146559): stale-build behavior, reinstall is correct, no preemptive patch; real-display-only risk remains for the GTK_IS_ on-interaction path. |
| Fleet notice | Broadcast restart warning to 20 sessions (hcom). |

## Key Facts / Paths

- Installed (NEW): `~/.local/limux-reviewed/main-20260622-2fcfc55/` → symlink `~/.local/bin/limux`.
- **Rollback** (if new build misbehaves): old `29fd2ff` symlinks archived at
  `~/.local/limux-reviewed/archive/20260622T203030Z/`. Revert:
  `ln -sf /home/riche/.local/limux-reviewed/copy-paste-release-autocopy-20260622-29fd2ff/bin/limux /home/riche/.local/bin/limux`
  (and same for `-cli`).
- Pre-restart running host: PID 20131 (OLD build) on `/run/user/1000/limux/limux.sock` —
  single host, no stale/duplicate sockets this time. Restart = `kill 20131` then `limux`.
- `session.json` (`~/.local/share/limux/session.json`): the 9× `terminal-0` ids are
  pane-local tab ids (each pane owns its own `content_stack`) → VALID, not corruption.
  Restored cleanly on current main. Do NOT "fix" it.
- The two benign env errors: `g_settings_schema_source_lookup` = GNOME dark-mode probe
  (`window.rs:2432`); split SVG load = valid SVGs + librsvg2 present (human saw a
  source-tree dev-build path). Neither reproduced on current main.

## Critical Behavior Rules (this lane)

- Do NOT patch source for the GTK/GLib errors unless they REPRODUCE on the current-main
  live runtime after restart (lifo Q3).
- Do NOT touch `HANDOFF.md` (halo-owned), `LIFO_HANDOFF.md` / `HALO_HANDOFF.md` (peer-owned),
  or `archive/` — own state lives in THIS file.
- Source tree must stay clean; only `target/` build artifacts + the new install dir changed.
- Do NOT start Cursor integration / TaskMaster tasking (operator hasn't opened that lane;
  future ideas only in `docs/future-improvements/limux-cursor-integration-options-after-pr-greenlight.md`).

## Task Tracker (session tasks)

G0 accept lane ✅ · G1 triage ✅ · G2 build+reproduce ✅ (zero errors on main) ·
G3 code fix = REFRAMED not-needed ✅ · G4 reviewed reinstall ✅ (install done; operator
restart + live confirm remaining).
