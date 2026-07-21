**Created by:** Claude Code (tutu · cd1a39d7)
**Date:** 2026-07-21 ~10:45 EDT
**Purpose:** Succession onboarding for limu (new LIMUX_MGR, succeeding lifo). Verified current-state map + first-actions, so limu can prime cold. Durable-in-repo on purpose (hcom is flaky right now — see §8).

## For: limu (LIMUX_MGR) — from tutu (session coordinator/pusher, ~/MCPs/limux, 2026-07-16→21)

I'm not a manager — I've been the coordinator/pusher in this dir. Everything below is independently verified this turn (git/gh/taskmaster), not relayed. Read order at the bottom.

## 🔴 §1 — DO THIS FIRST: protect lifo's UNCOMMITTED reflow work

The shared checkout is on branch `lifo/pane-reflow-task29-20260721` with **uncommitted, unpushed** work:
- `rust/limux-host-linux/src/terminal.rs` (+64 lines) — lifo's in-progress fix for **word-wrap task #29**
- `.taskmaster/state.json`, `.taskmaster/tasks/tasks.json` (modified)

lifo **exited without committing this.** It exists ONLY in this working tree. **Any branch switch / reset / checkout in this shared checkout loses it.** Before you do ANY git operation: decide with the operator whether to commit+push it (preserve the #29 WIP) or stash it. Do not let a prime/cleanup step clobber it. I stayed read-only all session specifically to protect it — hand it off deliberately, don't step on it.

## §2 — Runtime state (DONE, verified)

- **Stable Limux v0.2.3 is the active runtime.** Host PID 37671, `limux-cli 0.2.3 (1a26bda0, stable)`, `doctor ok=true` via `limux-stable-cli`.
- **Launcher fixed + merged:** PR #79 (`28e3a81` promote stable as default launcher) + PR #80 (`31a9431` close launcher task). Plain `limux` now = v0.2.3 (was legacy 0.2.2 until 2026-07-21).
- Launcher scheme: `~/.local/bin/limux` (now stable) + `limux-stable`/`limux-stable-cli`; legacy v0.2.2 install retained as rollback provenance.
- origin/main = `fec619e` + the two launcher PRs on top (`31a9431`).

## §3 — HANDOFF fragmentation you inherit (the thing to consolidate)

Six per-session handoff files exist. **Authority map:**
- `HAMO_HANDOFF.md` (**ON MAIN**) = current runtime truth (v0.2.3 release + force-restart). Read this for runtime state.
- `HANDOFF.md` (**mainline = STALE**, "Last updated 2026-06-20", old Halo content). Do NOT trust as current.
- My session-agnostic **consolidated** HANDOFF (commit `18e2082`, verified-current, with a KNOWN-ISSUES section) is **STRANDED** on branch `lifo/limux-first-hcom-tracking-20260715` (= PR #58, DIRTY). It's the pattern doni/nava's fleet HANDOFF-reconciliation remedy is adopting. You may want to adopt/rebuild it as your canonical doc.
- `LIFO_HANDOFF.md` (28KB) = deep lifo-lane history. `NATO_HANDOFF.md`, `HALO_HANDOFF.md`, `LIFO_CL_MGR_HANDOFF.md` = older lanes.
- Consolidation into ONE session-agnostic canonical doc is the recommended cleanup (ties to the fleet remedy; see `LIFO_INBOX/DESIGN_QUESTION_FROM_nava_...` §C for the writer-model tension).

## §4 — TaskMaster reconciliation (has a live hazard)

- **currentTag = `limux-resource-crash-20260716`** (5 open). But `master` has 30 tasks / 15 open, including **#29 word-wrap [in-progress]** (lifo's uncommitted work, §1).
- ⚠️ **`.taskmaster/state.json` + `tasks.json` are UNCOMMITTED** in the working tree (lifo's edits). So the git-durable task state ≠ working-tree state. Reconcile which is truth before trusting task status.
- Note: TaskMaster state is **per-branch** — tags/tasks differ between branches (this caused a false "missing tag" scare earlier; it was a branch-view difference, not data loss). Use `task-master-reviewed` (host wrapper) only.
- Earlier in this session I added word-wrap + launcher tasks as #26/#27 on a stranded branch's master tag; both are **superseded** — word-wrap is now mainline `master #29`, launcher landed. Ignore the stranded #26/#27.

## §5 — Open PRs (repo audit) — all three DIRTY (need conflict resolution)

- **#58** DIRTY — `lifo/limux-first-hcom-tracking-20260715` (carries my HANDOFF consolidation `18e2082`)
- **#67** DIRTY — `lifo/renderer-diagnostics-task2-20260716` (renderer backend diagnostics)
- **#68** DIRTY — `bulo/bounded-logging-task3-20260716` (bounded host logs + doctor reads)

## §6 — Worktrees

- `~/MCPs/limux` (main checkout, current branch `lifo/pane-reflow-task29-20260721`)
- `/tmp/limux-release-0.2.3-20260719` @ `fec619e` [hamo/force-restart-stable] — **retained per no-loss closeout; do NOT remove without the no-loss gate + owner authorization** (hamo's durable evidence lives near it).

## §7 — Untracked deliverables in the tree (don't lose, don't clobber)

- `LIFO_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md` — reve's `new-pane` defect, filed against **legacy 0.2.2**; a **v0.2.3 retest was requested** (open, unowned).
- `LIFO_INBOX/DESIGN_QUESTION_FROM_nava_2026-07-21_hcom-tui-limux-symbiosis.md` — nava's hcom-TUI×Limux design seam (exploratory, no commitment). Verified findings inside: the pane-chrome mechanism (`pane-action set_flag_color`) already ships; it's the cheapest seam. **This is now YOURS as design owner** — nava's waiting on the named Limux successor to pick it up.
- `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` — **lifo/operator-owned**; do NOT stage/modify/remove without their authorization.

## §8 — Cross-cutting fleet context (affects your coordination)

- **hcom is flaky right now** — an active hcom.db lock-contention issue is causing `database is locked` failures + stale_cleanup reaping. It queued/failed my sends repeatedly this session, and plausibly contributed to lifo showing offline mid-work. Your succession messages may queue/fail — retry, and prefer durable files (like this one) over relying on live hcom delivery. doni owns that lane (fix in progress; it's a hypothesis-stage investigation, not resolved).
- **Grok→MiniMax-M3 fallback** is fleet-wide (Grok token limit hit) — any M3-gated review work is capacity-pressured; verify M3 output, defer load-bearing judgment. Not limux-specific but affects delegated reviews.

## §9 — Is it organized enough to prime? — YES, with two must-do-first caveats

Prime is safe: main is coherent, HAMO_HANDOFF (runtime truth) is durable on main, launcher landed. **But before priming touches git/branches:** (1) protect lifo's uncommitted reflow work (§1), (2) reconcile the uncommitted TaskMaster state (§4). The handoff fragmentation (§3) + DIRTY PRs (§5) are cleanup *targets*, not prime *blockers*.

## Recommended read order
1. This doc. 2. `HAMO_HANDOFF.md` (runtime truth, on main). 3. My consolidation `git show 18e2082:HANDOFF.md` (verified current-state + known issues). 4. `LIFO_HANDOFF.md` (lifo-lane depth). 5. `LIFO_INBOX/` (reve incident + nava design question — both now yours).

I'll stay available to coordinate the handoff, but I'm the pusher, not claiming your manager role — you own the decisions now.
