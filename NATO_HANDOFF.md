# Limux Nato Handoff

<!-- ===== INBOUND POINTER — ADDED BY: nato_1 (0afca090) — 2026-07-07 23:20 UTC =====
     Origin: mori (mori_FABLE_HCOM_MGR · 3ae7bd48) — operator-directed HIGH: adversarial
     review of the limux↔hcom convergence audit + DP-0..DP-7 decision cards.
     Source doc: /home/riche/MCPs/limux/nato_INBOX/REQUEST_FROM_mori_2026-07-07_limux-hcom-convergence-adversarial-review.md
     My verdicts: /home/riche/MCPs/limux/REVIEW_FROM_nato_1_2026-07-07_limux-hcom-convergence.md
     Status: verdicts DELIVERED to mori 2026-07-07 ~23:20 UTC; mori assembles the final HTML
     packet for operator sign-off. On DP-5/DP-6 ratification, nato_1 owes: roadmap W3.1 stamp
     + PRD-G boundary-note/join-contract patch (both my files).
     ===================================================================== -->

**Created by:** Claude Code (nato · Claude Fable 5 · 0afca090)
**Date:** 2026-07-07 00:55 UTC (supersedes the 2026-06-22 entry, archived below)
**Purpose:** Resume-safe state for nato's HIGH-LEVEL PLANNING lane: cmux-parity
roadmap + Wave-0/1 PRDs, produced for lifo + subagents to execute.

## Status Update 2026-07-07 ~21:45 EDT (post-crash-#2: convergence ratified, install-readiness queue)

- **WSL crash #2 (~20:45 EDT) ROOT CAUSE: C: drive full** (5GB free/930GB) — sparse
  ext4.vhdx couldn't grow under parallel-build writes → EIO on root fs (operator's
  crash log: getpwuid failed 5) → VM death. Memory config was accomplice:
  `.wslconfig` had memory=48GB/64GB host. FIXED (staged): 32GB + swap 32GB + gradual
  reclaim — applies at next `wsl --shutdown`, which MUST wait until C: has headroom
  (swap needs disk!). I cargo-cleaned 6 closed-lane worktree targets (~16GB in-VM).
  Docker Desktop WSL data = 209GB (operator cleanup target). earlyoom pending sudo.
  Fleet build+disk discipline broadcast (one heavy build, CARGO_BUILD_JOBS=4,
  clean-at-lane-close).
- **Merged today post-restore-lane:** #39 PRD-E registry 078bc0c (5 rounds; wired-route
  template), #40 pane.focus d2e566c, #41 pane.resize a3b3d19 (10 rounds, all real —
  with_workspace_temporarily_mapped primitive + convergent guard invariants).
- **hcom↔limux convergence OPERATOR-RATIFIED** (mori's packet, my amendments verbatim):
  W3.1 stamp + PRD-G boundary/join-contract + DP-7 boundary-lint tripwire = PR #42
  (13d82e8+3b92e87, bot round running; I merge on gate — operator-ratified execution).
  reqwatch false-positives fixed by dino (v0.7.62).
- **Install-readiness queue (operator wants to update ASAP):** #42 + lifo_cl_mgr's
  version/changelog PR (0.2.0 + CHANGELOG.md, ratified frame). Surface group DECOUPLED
  from this install (next cycle). Operator restart sequence: free C: → wsl --shutdown
  → fresh v3 install of current main (zig-free borrow pattern from
  WORKTREES/install-newest-main-v2.sh, updated SHA) → limux → sessions via hcom r.
- **MiniMax pre-wave experiment** designed + wired (fires on lifo's surface-group PR):
  ~/Proj/MODEL_TESTING_LAB/INBOX/PROPOSAL_FROM_nato_1_2026-07-07_prebot-minimax-lens-wave.md
  — read hermes-delegation + minimax-subagent-optimization IN FULL at fire time.
- **lifo_cl_mgr** (codex 019f087b, LIMUX_MGR claim) = second lane: version/changelog
  now; rename-textbox diagnosis-only until surface group merges. mori=3ae7bd48 (hcom).
- **My review-miss ledger (calibration):** #41 dispatch-arm miss + "unreachable"
  overlap call — wired-route reviews now ALWAYS read the handle_control_command arm.
  Also: commit before destructive probes (reset --hard wiped uncommitted work once).

## Status Update 2026-07-07 ~17:40 EDT (RESTORE LANE COMPLETE — Wave-1 remainders in flight)

- **Timekeeping fix:** hcom event `ts` are UTC; earlier entries' "EST" labels were
  actually UTC. Ground wall-clock with `TZ=America/New_York date`.
- **P1 restore lane DONE (subtasks 8.1/8.2/8.3, cmux-parity tag task 8):**
  - #35 `2400d6a` TaskMaster reconcile (subtasks created; master-tag #8 untouched — verified).
  - #36 `2f90b2d` slice A exited-agent safety: hook-index miss → clear persisted agent
    (tab restores as plain shell at cwd); `restore_on_startup=false` marker preserved;
    post-review delta added has_loaded_kind mass-clear protection + version:1 store gate.
  - #37 `9f3f880` slice B hcom resume: hcom-managed panes restore via
    `hcom r <name|uuid> --run-here --go` (name-first — names stable, uuids rotate);
    deltas: --go hardening; 161c005 scrubs 9 inherited HCOM_* identity vars in all
    spawned terminals (bot-caught parent-identity leak — real P2).
  - #38 `02065cc` slice C stagger: agent resumes only, batch 2 / 750ms / cap 6s;
    delta 81352fc `sleep && cmd` so Ctrl-C cancels resume (cleanup still fires).
  - All reviewed by me: source-read + contract-verify (hcom 0.7.61 source: --run-here,
    --go, TTY preview gate) + tests run in MY worktree
    (`WORKTREES/install-main-682e3b6cce3f-20260707T162257Z`, healthy zig-out).
    Known env-noise there: resource_env + runtime_socket family (1 primary + poison
    cascade) and CLI hook_session_id fallback — compare vs base before blaming a PR.
- **Lane discipline note:** NEVER `git checkout` in the main checkout (lifo's lane) —
  near-miss logged; use `git -C <my worktree>`.
- **In flight:** lifo on PRD-E task #5 (limux-core registry/mutation-set/kill-switch;
  co-design constraint; commit-2a public API is the reviewed base), then PRD-G #7.
  PR #32 stays deferred. Operator gates still open: PRD-C live checklist, PRD-F.
- **NOTE: the operator's running install (main-b26312715162-full) PREDATES the restore
  lane** — restore fixes take effect only after a fresh install/restart (v2 install
  script pattern; suggest after PRD-E/G land so one restart picks up everything).

## Status Update 2026-07-07 ~18:20 UTC (RESTART COMPLETE — final-push directive issued)

- **Operator is LIVE on newest main:** running host verified `b26312715162`
  (install `main-b26312715162-full`, legacy channel) — doctor: host ok, socket ok,
  resources ok; only warn = 2 stale sockets (cosmetic, cleanup queued).
- **Same-day P0s found by first live launch + fixed + verified:**
  - PR #33 `86c8b96` titlebar crash (PR #6's set_titlebar on AdwApplicationWindow;
    fatal on live WSLg; fixed via ToolbarView; I live-verified on preview channel).
  - PR #34 `b263127` hook stalls (run_agent_hook had NO socket timeout → 5s stalls
    per event per session; fixed: 500ms fire-and-forget budget covering connect,
    2s entry cap, per-event statusMessage labels; hooks setup re-ran, entries verified).
- **Install tooling:** `WORKTREES/install-newest-main-v2.sh` = zig-free install path
  (borrow healthy libghostty from main checkout — pin 81ab8ffa unchanged — + vendored
  PRD-B resources). v1 (zig build) crashed WSL via OOM; operator's zig/gettext/
  blueprint-compiler installs were verified benign (official sources).
- **Comms during outages:** lifo reachable via pane injection — `limux send --workspace
  workspace:aaacde98... --surface 125:15a7c9e6... "<msg>"` THEN `limux send-key ... Enter`
  (send alone leaves text unsubmitted in the input box — known defect, in P1 lane).
  My durable inbox: `nato_INBOX/` (repo root). lifo's: `lifo_INBOX/`.
- **FINAL-PUSH directive to lifo (operator: "push all remaining items"):** P1 restore
  lane (no-resurrect-exited + hcom-r resume + restore staggering + stale-socket
  cleanup; folds INTO PRD-H restore slices — same lane), Wave-1 remainders (PRD-E
  registry/mutation-set/kill-switch task #5; PRD-G sidebar rendering slices task #7),
  TaskMaster hygiene (cmux-parity-20260707 tag lists EMPTY from local checkout —
  investigate; create proper restore task; do NOT touch master-tag #8).
- **Operator gates (the only human items):** PRD-C 10-min live checklist run on the
  new build; PRD-F decision-frame ratification (or delegate to lifo per PRD-F).
- **My next:** watchdog lifo's final push (ScheduleWakeup loop re-armed); Wave-2 PRD
  cutting + research-db refresh once Wave-1 remainder lands.

## Status Update 2026-07-07 ~13:15 EST (post-crash reconciliation + pre-restart)

- **Identity:** this session is hcom `nato_1` (PRIMARY nato; session 0afca090,
  rebound after the PC-crash identity mess; hcom resume-by-uuid bug root-caused
  by me, fixed by dino f26bdb9 / hcom v0.7.61+). Routing acked fleet-wide.
- **Nato fleet reconciled + closed:** nato_2 (18bf8a3f, overnight twin — ledger
  matched, Wave-2 continuity transferred to me), nato_analyzer (a31a4ee3 — hcom
  crash findings durable in hcom repo HCOM_MGR_INBOX), nato_protocol_reviewer
  (2e64c18b — ledger addenda + rehoming PRD-LITE durable in lat_router worktree).
  KEPT: nato_lat_router (837f17a1, limux agent-team routing lane) + me. dino
  holds the stub-cleanup list (operator-gated).
- **Skills landed globally:** limux-use-guide (mechanics) + limux-team-orchestration
  (discipline; my MGR sign-off; kazu landed 7f9880d Claude-side; niru GO Codex mirror).
- **Overnight execution all merged; main = 682e3b6:** Wave 0 (PRD-A/B/D + PRD-C
  staged) + Wave 1 core (PRD-E 2a/2b, PRD-G slice1, PRD-H cwd) + #26/#29/#30 + #31.
  PR #32 (pane width lock) OPEN — deferred, not bot-green.
- **Restart:** operator installing newest main to daily-driver `limux` via
  `WORKTREES/install-newest-main.sh` (lifo-authored, clean detached worktree,
  legacy channel, smoke = --version + doctor --json). Task #9 build-path pick is
  MOOT (verified: 78e384d tree == #31 squash 682e3b6, git-diff-empty).
- **Operator gates open:** PRD-C live checklist run (preview/stable, separate
  from the legacy install); PRD-F ratification.
- **My next work item:** Wave-2 PRD cutting after Wave-1 remainder lands
  (+ research-db refresh at the wave boundary).

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
