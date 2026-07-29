# HANDOFF — tutu (LIMUX_MGR) — pre-WSL-restart checkpoint

**Created by:** Claude Code (tutu / LIMUX_MGR · cd1a39d7)
**Date:** 2026-07-29 (EST)
**Purpose:** Durable checkpoint of ephemeral task state before huno's operator-approved `wsl --shutdown` C:-reclaim kills all sessions. A post-restart LIMUX_MGR resumes from here. Session-specific surface per HANDOFF.md §7 (shared HANDOFF.md is tutu's but on fire's intake-only branch — not co-written).

## ★ CONSOLIDATION LANE (operator-approved + ACCELERATED, fire #599614/#600100)
**STATUS: FYI CONSOLIDATION PR IS OPEN — PR #104** `consolidation/limux-fyi-coordsurf-20260729` @ `02397a4`, off origin/main 93132ae. Carries huno's profiles entry + my compaction entry (FYI 2098→2157→2164, pure appends, blob-verified). `@codex review` requested. **Accelerated** (fire #600100): FYI lineage is independent (neither PR #102 nor #103 touches FYI.md), so it merged-path independently — DON'T wait for #102/#103; sequence after them only on an actual GitHub conflict.
**MERGE #104 on greenlight** (Codex bot review). Then the CLEANUP (still follows #102/#103 merges):
1. DELETE all six `coordsurf/huno-*` branches + my `coordsurf/tutu_limux_mgr-*` branches (once #104 merged) — `git log --all` no-loss check first.
2. Sweep the three stale `.claude/worktrees/agent-*` worktrees (`git worktree remove`, **no-loss first**).
3. Reconcile shared checkout to main (post-merge-branch-reconciliation), off fire's parked branch.
4. Delete both merged feature branches (#102 limu, #103 fire) locally.
5. **Shared HANDOFF.md final update — mine as LIMUX_MGR.**
**STILL PENDING (old task #5b):** LIFO html `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` (untracked, lifo's) — preserve + route to lifo, NOT into main.
Roles: limu owns #102; huno runs full `./scripts/check.sh` on fire's branch (real da8c108 gate — limu's green ran main-based, doesn't discharge it) + drives #103 fix loop; fire does #103 merge judgment. Do NOT act on decision-packet D1/D2/D3 (operator's picks — `docs/LIMUX_SPACE_CRISIS_PR_CYCLE_DECISION_PACKET_2026-07-29.html`).

## Co-manager arrangement (live)
- **tutu** = `LIMUX_MGR` (Claude) — repo-shared doc/coordination surfaces + Claude-on-Codex review.
- **limu** = `LIMUX_CODEX_MGR` (Codex, co-claim `mgr-76ec78924d3e7564`) — Codex-lane execution. levu source-confirmed the co-claim is non-superseding.

## DONE this session (durable, verified)
- **Phase-2 cross-family review of limu's keep-last-N prune** → CONFIRMED-GOOD; fire final judgment PASS. **Blob-verified** the pushed commit `6214fa69b16d130252da0efe88d29b1d9d4312f3` (branch `limu/keep-last-reviewed-runtimes-20260729`, parent origin/main 93132ae): 3 paths, no scanner tsv, review anchors present. **Unmerged — awaiting operator PR.** Eval logged to `MODEL_TESTING_LAB/INBOX/EVAL_DATA_FROM_tutu_2026-07-29_cross-family-limux-prune-review.md`.

## OPEN TASKS (post-restart — were tracker #1–#5)
1. **item-1 security** — reve cross-workspace disclosure. ARCHAEOLOGY DONE: `dfb5d40` is the narrowing commit (legacy `1005f58d` had no workspace default → global-focus fallback = reve's disclosure; current defaults workspace to `LIMUX_WORKSPACE_ID` → workspace-scoped WHEN set). STILL OPEN: flag-rejection half (`--xyzzy` falls through, only `--help` intercepted; = REPO_AUDIT H1, fix Q1/T0.1 unmet), and explicit `--surface` cross-workspace = UNTESTED (static-trace the server's explicit-surface resolution first; scoped consenting live test only if inconclusive).
2. **read-screen unknown-flag drop** — REPO_AUDIT `docs/REPO_AUDIT_limux_2026-07-21.md:57` (H1). Fix: reject unknown flags BEFORE socket contact. `profile list` inherited it (REPO_AUDIT:76). Post-compact via ephemeral worktree; ask before PR.
3. **Durable finding doc + flag matrix + CLAUDE.md checklist** — pipe-trap rule (capture-then-filter; 3 hazards) + trap-restore + lint + card lines (null-case / instrument-resolved / bug-that-hides-a-bug / trigger-on-closure / flattering-reading-about-self / verify-on-read / sweep-after-discovery). Item (5) ROUTE TO KAZU (global archive-not-delete + verify-before-claiming generalizations — not limux-local).
4. **HANDOFF.md:13 + :329** drop stale "`archive/` is gitignored" (61a0f36 un-ignored it; verified tracked). REPO_AUDIT:45 = DATED audit → ANNOTATE (dated correction), don't edit the snapshot.
5. **fire PHASE-1 doc reconciliation** — (a) FYI compaction entry (vhdx 347.19→196.40 GiB −150.79; C 22→176G) — append onto huno's `coordsurf/huno-20260729T114020Z` (one lineage, NOT origin/main). BLOCKED on operator's FYI→main PR decision (huno flagging). (b) LIFO html `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` (genuinely untracked, lifo's) — preserve on coordsurf/preservation + route to lifo.

## Discipline reminders (this session's traps)
- Pushed-ref ≠ blob (verify committed blob, not working file). Verify-on-read: `??` in a parked checkout ≠ untracked in git — `git log --all` first. Never run `git clean` where coordination surfaces live. Freeze directive: announce before check.sh/cargo build/test; scoped greps; one-at-a-time.
- Checkout parked on `fire/log-retention-and-cache-hygiene-20260728` (intake-only) — coordination surfaces via carve-out to coordsurf; code via ephemeral worktrees off origin/main; never pile onto fire's branch.
