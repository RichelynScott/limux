# BARI_HANDOFF — bari (LIMUX_MGR) — 2026-07-31 hygiene resume

**Created by:** OMP (`bari` / LIMUX_MGR; session label may appear as `kero` in the GUI)
**Date:** 2026-07-31 (UTC)
**Purpose:** Current per-session resume surface (see `HANDOFF.md` §7). Read after `HANDOFF.md`.

## Identity
- **`bari`** = `LIMUX_MGR` (this OMP session; docs/TaskMaster/hygiene owner for this packet).
- **`limu`** = possible `LIMUX_CODEX_MGR` when live (stale at hygiene time; may co-claim the Codex lane when active).
- **`kero`** appears only as the historical OMP session label in the 2026-07-30 plan-review screenshot evidence — not a second manager identity.

## Repo state snapshot (re-measured 2026-07-31)
- Shared checkout: `/home/riche/MCPs/limux` on `main`, clean, synced with `origin/main`.
- Tip at hygiene start: `ef4d376` (`docs(handoff): record the mechanism-vs-semantics failure class`).
- Open PRs: **none** (`gh pr list --state open` → `[]`).
- Installed runtime (unchanged by this packet): `limux-cli 0.2.3 (c757056d2539, release)` install-id `main-c757056d2539-adv-remediated-20260721` — **~50+ commits behind main** including #105–#108. Reinstall is **operator-gated**, not part of this packet.
- Doctor after hygiene: launchers / processes / socket / stale_sockets / ghostty_resources all `[ok]`.
- Last known full gate green from fire's closeout lane (docs-only packet; `./scripts/check.sh` not re-run here).

## Open product backlog (do **not** implement in the hygiene packet)
1. Reject unknown CLI flags before socket contact (`TUTU_HANDOFF` item-2).
2. Limux-local `CLAUDE.md` checklist card lines (item-3).
3. H1 residual CRITICAL — explicit foreign `workspace_id` (fast-follow §7 / `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md`) — operator-gated option (b).
4. Display-loss exit diagnostic (fast-follow §8).
5. Successor-rebind control path (fast-follow §9).
6. Prune `--keep` cap + prune TOCTOU (fast-follow §1–§2) — limu lane.
7. Live GUI verify of OMP scroll + #84 resize (still never operator-observed).
8. OMP plan-review / ask waiting must visibly identify the background workspace in the left sidebar; track as cmux-parity task **7.3**, ordered after native PRD-G live wiring. Source screenshot: `/mnt/c/Users/riche/Downloads/SCREENSHOTS/Screenshot 2026-07-30 205145.png`; ratified decision: `LIMU_INBOX/RESPONSE_FROM_limu_2026-07-30_omp-ask-waiting-abc-decision.md`.

## Hygiene this packet closed
- Docs refresh: `HANDOFF.md`, this file, `TUTU_HANDOFF.md` successor banner, fast-follow §3–§5 CLOSED banners.
- TaskMaster reconcile of shipped defects + add OMP waiting-visibility subtask under cmux-parity task 7.
- Stale sockets archived (same-tmpfs; cross-fs `mv` of unix sockets fails) → doctor `stale_sockets` ok.
- `/tmp` limux test debris archived under `~/.archive/limux/tmp-debris-*` (archive-not-delete).

## Discipline
- Ephemeral worktree for main commits (never branch-switch the shared checkout while peers may be live).
- Set `export CLAUDE_SESSION_NAME=BARI_LIMUX_MGR CLAUDE_AGENT=claude` on every commit shell (papa-git still falls back to global `FIRE` when unset — see `FIRE_HANDOFF.md`; hook redeploy stays with PAPA_GIT/`rizu`, do not "fix" hooks from limux).
- No live H1 disclosure probes — static-trace only.
- Never hand-edit `.taskmaster/tasks/tasks.json` — use `task-master-reviewed` only.
- Archive-not-delete for sockets, runtimes, and operator logs.
