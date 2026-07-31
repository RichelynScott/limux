# BARI_HANDOFF — bari (LIMUX_MGR) — 2026-07-31 resume (yields merged; host restart gated)

**Created by:** OMP (`bari` / LIMUX_MGR; session label may appear as `kero` in the GUI)
**Date:** 2026-07-31 (UTC)
**Purpose:** Current per-session resume surface (see `HANDOFF.md` §7). Read after `HANDOFF.md`.

## Identity
- **`bari`** = `LIMUX_MGR` (this OMP session; docs/TaskMaster/hygiene owner for this packet).
- **`limu`** = possible `LIMUX_CODEX_MGR` when live (stale at hygiene time; may co-claim the Codex lane when active).
- **`kero`** appears only as the historical OMP session label in the 2026-07-30 plan-review screenshot evidence — not a second manager identity.

## Repo state snapshot (re-measured 2026-07-31)
- Shared checkout: `/home/riche/MCPs/limux` on `main`, clean aside from untracked `.taskmaster/reports/`, synced with `origin/main`.
- Shared `main` carries Merge #109 (`de6d1db`) + continuity #110–#115. Yields product symbols on `main`.
- Open PRs: **none** (yields #109 merged).
- Installed CLI: `limux-cli 0.2.3 (e8e19c9c7150, release)` install-id `main-e8e19c9c7150-helpprint-20260731`. Live host PID still on `main-46ab49ded66f-yields-20260731` — restart **gated** on OMP/peer checkpoint.
- Doctor re-check: launchers / processes / socket / `stale_sockets` / ghostty_resources all `[ok]` (exit 0). Historical leave-alone `52458` paths are gone; only live listeners are `stable/limux.sock` + `.cursor`.
- Last known full Rust gate green from fire's closeout lane (docs-only continuity; `./scripts/check.sh` not re-run here).

## Open product backlog
### Merged — PR #109 (`de6d1db`; was `bari/yield-abc` @ `7d9bfb4`)
1. ~~Reject unknown CLI flags before socket contact~~ → TaskMaster **#33** (done on `main`).
2. ~~Byte-safe `limux send` `--stdin`/`--file`~~ → TaskMaster **#25** (done on `main`). **Honest residual:** Ghostty FFI PTY write not unit-proven; E2E = `scripts/xvfb-smoke-test.sh` (ScoutBridgeDelivery).
3. ~~Display-loss exit diagnostic~~ → TaskMaster **#34** (done on `main`; 34.3 via mutation evidence). Stable reinstall complete (`main-46ab49ded66f-yields-20260731`).

### Still open
4. Limux-local `CLAUDE.md` checklist card lines (`TUTU_HANDOFF` item-3).
5. H1 residual CRITICAL — explicit foreign `workspace_id` (fast-follow §7 / `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md`) — operator-gated option (b).
6. Successor-rebind control path (fast-follow §9).
7. Prune `--keep` cap + prune TOCTOU (fast-follow §1–§2) — limu lane.
8. Live GUI verify of OMP scroll + #84 resize (still never operator-observed).
9. OMP plan-review / ask waiting must visibly identify the background workspace in the left sidebar; track as cmux-parity task **7.3**, ordered after native PRD-G live wiring. Source screenshot: `/mnt/c/Users/riche/Downloads/SCREENSHOTS/Screenshot 2026-07-30 205145.png`; ratified decision: `LIMU_INBOX/RESPONSE_FROM_limu_2026-07-30_omp-ask-waiting-abc-decision.md`.

## Hygiene this packet closed
- Docs refresh: `HANDOFF.md`, this file, `TUTU_HANDOFF.md` successor banner, fast-follow §3–§5 CLOSED banners.
- TaskMaster reconcile of shipped defects + add OMP waiting-visibility subtask under cmux-parity task 7.
- Planned 4 doctor-stale sockets archived same-tmpfs under `/run/user/1000/limux-socket-archive/20260731T013021Z/` (cross-fs `mv` fails for unix sockets). Unauthorized post-warn archive of `limux-52458.*` was **restored** then later disappeared on their own; doctor `stale_sockets` is now `[ok]` — do not recreate.
- `/tmp` limux test debris archived under `~/.archive/limux/tmp-debris-*` (archive-not-delete).

## Restart gate (open)
- New stable tree is installed and linked (`main-e8e19c9c7150-helpprint-20260731`).
- Live host remains on yields tree until **all current sessions (esp. OMP)** checkpoint and ack a Limux restart.
- After acks: SIGTERM host PID, launch `~/.local/bin/limux-stable`, confirm `limux doctor` green (CLI/host SHA match).

## Discipline
- Ephemeral worktree for main commits (never branch-switch the shared checkout while peers may be live).
- Set `export CLAUDE_SESSION_NAME=BARI_LIMUX_MGR CLAUDE_AGENT=claude` on every commit shell (papa-git still falls back to global `FIRE` when unset — see `FIRE_HANDOFF.md`; hook redeploy stays with PAPA_GIT/`rizu`, do not "fix" hooks from limux).
- No live H1 disclosure probes — static-trace only.
- Never hand-edit `.taskmaster/tasks/tasks.json` — use `task-master-reviewed` only.
- Archive-not-delete for sockets, runtimes, and operator logs.
