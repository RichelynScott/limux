# TUTU_HANDOFF — tutu (LIMUX_MGR) — 2026-07-29 cycle-close state

> **Successor banner (2026-07-31):** the tutu LIMUX_MGR cycle is closed.
> Current resume surface is `BARI_HANDOFF.md` (`bari` = LIMUX_MGR).
> Keep the OPEN section below — item-2 / item-3 / H1 remain accurate backlog;
> do not treat this file as the live manager handoff.


**Created by:** Claude Code (tutu / LIMUX_MGR · cd1a39d7)
**Date:** 2026-07-29 (EST)
**Purpose:** Per-session resume surface (HANDOFF.md §7). State after the 2026-07-29 C:-space-crisis PR cycle + the H1 security static-trace. A successor LIMUX_MGR resumes from here.

## Co-manager arrangement (live)
- **tutu** = `LIMUX_MGR` (Claude) — repo-shared doc/coordination surfaces + Claude-on-Codex review.
- **limu** = `LIMUX_CODEX_MGR` (Codex, co-claim `mgr-76ec78924d3e7564`) — Codex-lane execution. Non-superseding co-claim (levu source-confirmed).

## DONE (this session, durable + verified)
- **Phase-2 review** of limu's keep-last-N runtime prune → CONFIRMED-GOOD (fire PASS); merged **PR #102** (`59876fa`). Cross-family eval logged to `MODEL_TESTING_LAB/INBOX/EVAL_DATA_FROM_tutu_2026-07-29_cross-family-limux-prune-review.md`.
- **Consolidation lane CLOSED**: FYI **PR #104** merged (`ab05eacd` — huno profiles + tutu compaction entries + source-verified `profile list/path/rm` fix). Cleanup COMPLETE: all six `coordsurf/huno-*` + both `coordsurf/tutu_*` branches deleted (0 left, content on main), 3 stale `.claude/worktrees/agent-*` swept, shared HANDOFF.md finalized, lifo packet preserved (`preservation/lifo-closeout-packet-20260716` + `~/.archive/limux/`). Shared checkout reconciled to main by fire.
- **item-1 / H1 security — static-trace COMPLETE** → design note on main: `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md` (`d6cd153`). Cross-workspace surface disclosure CONFIRMED live on current code. ROOT: socket auth (`auth.rs is_authorized` L66-72) is **uid-level only** (SO_PEERCRED, once at accept), no per-workspace ownership. Two divergent paths: standalone `resolve_surface_target` (L3704; `--surface`-alone = `find_workspace_for_surface` L3673 = global scan); live GTK `ReadSurfaceText` (`window.rs` L6450; `workspace_index_for_target` L1077 Active/Handle, no ownership check). `dfb5d40` = client-side CLI mitigation only. **Blast-radius crux: the only legit cross-workspace reader is the OPERATOR (same uid as agents) — entitlement cannot key on uid.** CODE fix is operator-gated (3 options in the design note).
- **HANDOFF.md stale-claim fix** (task #4): `archive/`-gitignored claims at :13 + §7 dropped (`e62d28b`).

## OPEN (for the next session)
- **item-2 read-screen unknown-flag drop** (H1's flag-rejection half): CONFIRMED — read-screen/send/send-key silently drop unrecognized flags (`parse_opt` scans, no reject-unknown step). `--xyzzy` contacts the socket + reads. `profile list` inherited it (`REPO_AUDIT_limux_2026-07-21.md:76`). FIX = reject unknown flags BEFORE socket contact (Q1/T0.1). CODE change → ephemeral worktree off `origin/main` + tests (revert-callsite discipline) + PR + @codex.
- **Per-command targeting-flag matrix** (the inconsistency that produced the `--pane` bug — 3 sessions burned on it):
  | flag | commands that ACCEPT it |
  |---|---|
  | `--surface` / `--workspace` | read-screen/capture-pane, send, send-key, close-surface, identify |
  | `--pane` | new-pane, agent-team, pane-action |
  | `--tab` | rename-tab, tab-action |
  read-screen does NOT take `--pane` → supplying it silently no-ops. Fix scope = general reject-unknown, not a `--pane` special-case.
- **item-3 CLAUDE.md checklist card lines** (limux-local remaining piece): pipe-trap rule (capture-then-filter; 3 hazards: `$?`-after-pipe / bash-zsh PIPESTATUS / volatile-under-inspection) + trap-restore + self-describing-exit backstop + lint + card lines. NOTE: several generalizations already landed in GLOBAL rules (verify-before-claiming, cross-directory-durable-delivery, archive-not-delete — kazu's lane); only the limux-local checklist addition remains.
- **H1 CODE fix**: operator-gated design decision — see design note `d6cd153`.

## Discipline lessons (this session — do NOT relearn)
- **BRANCH-VERIFY before ANY commit in the shared checkout** (`git branch --show-current`): the shared checkout switches branches between sessions (on `fire/fastfollows-3-4-20260729` at cycle close). I committed a doc onto fire's #105 branch by mistake (local-only, caught via verify-the-push, `git reset` restored fire's branch, re-landed on main via ephemeral worktree). For any main commit use an ephemeral worktree off `origin/main`.
- **Verify-the-PUSH**: "Everything up-to-date" while you hold a new local commit = you are on the wrong branch (not main).
- pushed-ref ≠ blob (verify the committed BLOB); verify-on-read (`??` in a parked checkout ≠ untracked — `git log --all`); **no live disclosure probe** (the probe IS the incident — static-trace instead).
- Shared checkout: coordination surfaces via carve-out (`commit_coordination_surface.py`) to `coordsurf/`; code + docs to main via ephemeral worktree off `origin/main`; never pile onto fire's branch.
