# Limux Nato Handoff

**Created by:** Claude Code (nato · Claude Fable 5 · 0afca090)
**Date:** 2026-07-07 00:55 UTC (supersedes the 2026-06-22 entry, archived below)
**Purpose:** Resume-safe state for nato's HIGH-LEVEL PLANNING lane: cmux-parity
roadmap + Wave-0/1 PRDs, produced for lifo + subagents to execute.

## Status Update 2026-07-06 evening (pre-compaction checkpoint)

PR #16 is in lifo's Codex-bot loop. Lifo has pushed two mechanical P2 fix
commits onto this branch with nato's standing endorsement: `98ebccb` (roadmap
W1.1 wording resync + PRD-G 30-min decay consistency) and `fcdf1be` (PRD-A
dirty-state rerun inputs + PRD-F exact method names `browser.url.get` /
`browser.wait` / `browser.console.list` / `browser.errors.list`). nato is
hands-off on the branch unless a finding needs PRD-author judgment. Nothing is
blocked on nato; next nato involvement = PRD-author judgment calls during
review/import, or Wave-2 PRD cutting after Wave 1 lands.

## Immediate Next Action

1. **lifo (when his Cursor PR-bot loop clears):** review PR #16
   (`nato/cmux-parity-roadmap-20260706`), then import PRDs into TaskMaster via
   `task-master-reviewed parse-prd` (or `add-task` per PRD) — handoff shape
   agreed on hcom 2026-07-06 (#302026/#302520).
2. **Execution order:** Wave 0 first (PRD-A/B/C/D — runtime trust), PRD-C's
   first checklist run clears the needs-verification backlog with the
   operator; Wave 1 (PRD-E/F/G/H) after; PRD-E commit 2a (limux-core public
   API) is reviewed before anything builds on it; PRD-E registry co-design
   happens AFTER lifo's PR #15 merges (agreed — don't churn his PR).
3. **FYI.md entry**: deliberately NOT appended on this branch — lifo has
   uncommitted FYI.md edits in the main checkout and an append here would
   conflict. Add the journal entry at merge/import time (content: this
   handoff's "What Happened" table).

## What Happened This Session (2026-07-06)

| Item | Detail |
|---|---|
| Model/role change | Operator brought nato back on Claude Fable 5 for high-level planning; lifo continues execution lanes. |
| Research fan-out | 3 subagents: cmux upstream (fresh, 2026-07-06), limux origin/main inventory, defect+WSLg catalog. |
| Strategic finding | Upstream cmux landed `mux` — a Rust, Linux-capable backend (cmux #7180/#7346/#7347, CDP browser panes #7325). Roadmap aligns contracts, no pivot. |
| Roadmap | `docs/cmux-parity-roadmap-20260706.md` — W0 runtime-trust → W1 parity core → W2 fidelity → W3 reach; Cursor lane committed-parallel. |
| PRDs | 8 PRDs at `.taskmaster/docs/limux-prd-{a..h}-*.md` (runtime-trust, ghostty-packaging, verify-loop, pane-attention, bridge-parity, browser-live, agent-sidebar, restore-pack). |
| Adversarial review | 8 parallel reviewer subagents (one per PRD), all findings code-verified; every CRITICAL/HIGH/MEDIUM folded with `(Codex-revised/required)` annotations. Biggest catches: PRD-B terminfo layout contradicted runtime contract; no tic-consumable terminfo source in vendored tree; WebKitGTK browser pane ALREADY SHIPS (default feature) — PRD-F reframed; Claude hook installer maps Notification→stop (inverts needs-input); PRD-H cwd design rebuilt on the shipped `term_cwd` mechanism (no `/proc`, no pid handle exists); limux-core lacks the pub API PRD-E needs (named commit 2a). |
| PR | #16 open on `nato/cmux-parity-roadmap-20260706` (roadmap + 8 PRDs). Codex PR bot = the cross-family review pass. |
| Operator decisions | Full roadmap stability-first; Cursor lane continues in parallel; all 8 PRDs approved for authoring; PR authorized. |

## Key Facts / Paths

- Branch/worktree: `nato/cmux-parity-roadmap-20260706` at
  `.worktrees/nato-parity-roadmap-20260706` (gitignored dir, repo convention).
- Roadmap: `docs/cmux-parity-roadmap-20260706.md` (PRD table §10, sequencing §9).
- PRDs: `.taskmaster/docs/limux-prd-[a-h]-*-20260706.md`.
- Research db to refresh at each wave boundary: `docs/research/cmux-upstream/`.
- Lane truth at handoff: TaskMaster #14 done (unconfirmed live — PRD-C
  closes), #15 review, Cursor tasks #6–#13 = lifo's active lane (PR #15 open).
- Operator's live install is STALE (`resize-live-sync-ae26e0a`, 2026-07-01,
  behind main) — PRD-C's first run installs+verifies fresh main.

## Critical Behavior Rules (this lane)

- nato does NOT execute implementation — planning/PRD lane only; lifo +
  subagents execute.
- Main checkout `/home/riche/MCPs/limux` is lifo's (spent PR-6 branch + his
  dirty files) — intake-only; all nato work happens in the worktree above.
- Do NOT touch `HANDOFF.md` (halo), `LIFO_HANDOFF.md`/`HALO_HANDOFF.md`
  (peers), lifo's dirty `FYI.md`/`.gitignore`/TaskMaster files.
- PRDs carry `(Codex-revised/required)` annotations — preserve them; they are
  the review trail.

---

## Archived: 2026-06-22 session record (GTK/GLib stale-build triage)

Lane accepted from lifo; root cause = operator running stale `29fd2ff` build;
current-main reinstall `main-20260622-2fcfc55`; post-restart verified all 10
workspaces restored, zero human-NOTE GTK/GLib errors; rollback archived at
`~/.local/limux-reviewed/archive/20260622T203030Z/`. The 9× `terminal-0`
session.json ids are pane-local and VALID (now codified in PRD-H's
contributor-docs item). Full detail in git history of this file (`619c67d`).
