# BARI_HANDOFF — bari (LIMUX_MGR) — 2026-07-31 resume (Wave2 #121–#123 merged; #124 OPEN)

**Created by:** OMP (`bari` / LIMUX_MGR; session label may appear as `kero` in the GUI)
**Date:** 2026-07-31 (UTC)
**Purpose:** Current per-session resume surface (see `HANDOFF.md` §7). Read after `HANDOFF.md`.

## Identity
- **`bari`** = `LIMUX_MGR` (this OMP session; docs/TaskMaster/hygiene owner for this packet).
- **`limu`** = possible `LIMUX_CODEX_MGR` when live (stale at hygiene time; may co-claim the Codex lane when active).
- **`kero`** appears only as the historical OMP session label in the 2026-07-30 plan-review screenshot evidence — not a second manager identity.

## Repo state snapshot (re-measured 2026-07-31)
- Shared checkout: `/home/riche/MCPs/limux` on `main` @ `7df751a`, synced with `origin/main`; allowed dirt only untracked `AUTOPILOT_LOG.md` + research scouts (leave unstaged).
- Shared `main` carries Merge #109 (`de6d1db`) + help-print #116 (`e8e19c9`) + continuity #110–#120 + Wave2 #121 cards / #122 prune / #123 successor-rebind. Yields + Wave2 product symbols on `main`.
- Open PRs: **#124** H1 option (b) PARTIAL default-off scaffold (`2c8ad7c`) — leave open pending review / GTK follow-up; do not merge as “H1 closed”.
- Installed CLI + live host: install-id `main-15ccb28ed4a8-matched-20260731` (lineage #109 yields + #116 help-print; source SHA `15ccb28ed4a8`). Host PID **860496** under that tree. `#122/#123` are on `main` but **not** in the live install (needs fresh peer/operator restart ack before matched rebuild).
- Doctor re-check: launchers / processes / socket / `stale_sockets` / ghostty_resources all `[ok]` (exit 0). Historical leave-alone `52458` paths are gone; only live listeners are `stable/limux.sock` + `.cursor`.
- Last known full Rust gate green from fire's closeout lane (docs-only continuity; `./scripts/check.sh` not re-run here). Focused Wave2 verifies: prune retention PASS; rebind control_bridge 56 / layout_state 67 / control_registry 7.

## Open product backlog
### Merged — PR #109 (`de6d1db`; was `bari/yield-abc` @ `7d9bfb4`)
1. ~~Reject unknown CLI flags before socket contact~~ → TaskMaster **#33** (done on `main`).
2. ~~Byte-safe `limux send` `--stdin`/`--file`~~ → TaskMaster **#25** (done on `main`). **Honest residual:** Ghostty FFI PTY write not unit-proven; E2E = `scripts/xvfb-smoke-test.sh` (ScoutBridgeDelivery).
3. ~~Display-loss exit diagnostic~~ → TaskMaster **#34** (done on `main`; 34.3 via mutation evidence). Matched stable reinstall complete (`main-15ccb28ed4a8-matched-20260731`; superseded interim yields/helpprint installs).

### Merged — Wave2 (#121–#123)
4. ~~Limux-local `CLAUDE.md` checklist card lines~~ → [PR #121](https://github.com/RichelynScott/limux/pull/121) (`8143513`).
6. ~~Successor-rebind control path~~ → [PR #123](https://github.com/RichelynScott/limux/pull/123) (`7df751a`) — `surface.rebind_session`.
7. ~~Prune `--keep` cap + prune TOCTOU~~ → [PR #122](https://github.com/RichelynScott/limux/pull/122) (`a501a8f`).

### Still open
5. H1 residual CRITICAL — [#124](https://github.com/RichelynScott/limux/pull/124) lands PARTIAL core scaffold only (`LIMUX_ENTITLEMENT=off`). GTK/`window.rs` threading, `workspace.{current,list,select}` gating, and operator-vs-agent entitlement-signal decision remain (fast-follow §7 / `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md`). No live disclosure probes.
8. Live GUI verify of OMP scroll + #84 resize (still never operator-observed).
9. OMP plan-review / ask waiting must visibly identify the background workspace in the left sidebar; track as cmux-parity task **7.3**, ordered after native PRD-G live wiring. Source screenshot: `/mnt/c/Users/riche/Downloads/SCREENSHOTS/Screenshot 2026-07-30 205145.png`; ratified decision: `LIMU_INBOX/RESPONSE_FROM_limu_2026-07-30_omp-ask-waiting-abc-decision.md`.
10. Optional: matched rebuild+reinstall from `main` @ `7df751a` to pick up #122/#123 — **only after** fresh peer/operator restart ack (do not bounce live PID 860496 without it).

## Hygiene this packet closed
- Docs refresh: `HANDOFF.md`, this file, `TUTU_HANDOFF.md` successor banner, fast-follow §3–§5 CLOSED banners.
- TaskMaster reconcile of shipped defects + add OMP waiting-visibility subtask under cmux-parity task 7.
- Planned 4 doctor-stale sockets archived same-tmpfs under `/run/user/1000/limux-socket-archive/20260731T013021Z/` (cross-fs `mv` fails for unix sockets). Unauthorized post-warn archive of `limux-52458.*` was **restored** then later disappeared on their own; doctor `stale_sockets` is now `[ok]` — do not recreate.
- `/tmp` limux test debris archived under `~/.archive/limux/tmp-debris-*` (archive-not-delete).

## Restart gate (closed 2026-07-31)
- Peer ack (`ack-ready-for-limux-restart`) + operator go-ahead received.
- SIGTERM'd prior host; live host now `main-15ccb28ed4a8-matched-20260731` (PID **860496**); `~/.local/bin/limux-stable` points at that tree.
- Verified: `limux doctor: ok`; CLI/host SHA matched; `send`/`send-key --help` print usage; unknown flags rejected.

## Discipline
- Ephemeral worktree for main commits (never branch-switch the shared checkout while peers may be live).
- Set `export CLAUDE_SESSION_NAME=BARI_LIMUX_MGR CLAUDE_AGENT=claude` on every commit shell (papa-git still falls back to global `FIRE` when unset — see `FIRE_HANDOFF.md`; hook redeploy stays with PAPA_GIT/`rizu`, do not "fix" hooks from limux).
- No live H1 disclosure probes — static-trace only.
- Never hand-edit `.taskmaster/tasks/tasks.json` — use `task-master-reviewed` only.
- Archive-not-delete for sockets, runtimes, and operator logs.
